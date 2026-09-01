//! Moving one Project through the public order.
//!
//! The order every reader-facing consumer follows is `project.sort_order`, so
//! a move rewrites those values rather than any per-page ordering. The whole
//! sequence is parked on negative values first and then written back as
//! positions one through n, which keeps the unique constraint satisfied at
//! every row and leaves the order dense after any number of moves.

use super::MoveError;
use crate::{
    db::DbPool,
    domain::{ProjectMove, ProjectSlug},
};

/// Moves the Project at `slug` as `movement` asks, in one transaction.
///
/// # Errors
///
/// Returns [`MoveError::UnknownProject`] for a slug that no Project holds,
/// [`MoveError::InvalidMovement`] for a move off either end or onto the
/// Project's own place, and [`MoveError::Transaction`] when the transaction
/// fails. The stored order is unchanged in every failing case.
#[tracing::instrument(name = "Move project in the public order", skip(pool), err)]
pub async fn move_to(
    pool: &DbPool,
    slug: &ProjectSlug,
    movement: &ProjectMove,
) -> Result<(), MoveError> {
    let mut transaction = pool.begin().await?;

    let rows = sqlx::query!(
        r#"
        SELECT
            id,
            slug
        FROM
            project
        ORDER BY
            sort_order
        "#
    )
    .fetch_all(&mut *transaction)
    .await?;

    let slugs = rows.iter().map(|row| row.slug.as_str()).collect::<Vec<_>>();
    let order = reordered(&slugs, slug, movement)?;

    sqlx::query!(
        r#"
        UPDATE
            project
        SET
            sort_order = -id
    "#
    )
    .execute(&mut *transaction)
    .await?;

    for (position, index) in order.into_iter().enumerate() {
        let Some(row) = rows.get(index) else {
            return Err(MoveError::InvalidMovement);
        };
        let sort_order =
            i64::try_from(position.saturating_add(1)).map_err(|_| MoveError::InvalidMovement)?;
        sqlx::query!(
            r#"
            UPDATE
                project
            SET
                sort_order = ?1
            WHERE
                id = ?2
            "#,
            sort_order,
            row.id
        )
        .execute(&mut *transaction)
        .await?;
    }

    transaction.commit().await?;
    Ok(())
}

/// The positions of `slugs`, in the order they hold after `movement` moves
/// `slug`. Working in positions keeps the decision away from SQL, so the
/// rejected cases are the ones this function names.
fn reordered(
    slugs: &[&str],
    slug: &ProjectSlug,
    movement: &ProjectMove,
) -> Result<Vec<usize>, MoveError> {
    let place_of = |wanted: &ProjectSlug| {
        slugs
            .iter()
            .position(|candidate| *candidate == wanted.as_str())
    };
    let from = place_of(slug).ok_or(MoveError::UnknownProject)?;
    let last = slugs.len().saturating_sub(1);

    let to = match movement {
        ProjectMove::Up => from.checked_sub(1).ok_or(MoveError::InvalidMovement)?,
        ProjectMove::Down => {
            let next = from.saturating_add(1);
            if next > last {
                return Err(MoveError::InvalidMovement);
            }
            next
        }
        ProjectMove::ToPlaceOf(anchor) => {
            let target = place_of(anchor).ok_or(MoveError::UnknownProject)?;
            if target == from {
                return Err(MoveError::InvalidMovement);
            }
            target
        }
    };

    let mut order = (0..slugs.len())
        .filter(|position| *position != from)
        .collect::<Vec<_>>();
    order.insert(to.min(order.len()), from);
    Ok(order)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{
        portfolio::{load, projects},
        seed_if_empty,
        test_support::migrated_pool,
    };
    use claims::{assert_matches, assert_ok};
    use rstest::rstest;

    const SEEDED: [&str; 3] = ["guenther", "traxor", "cipher-workshop"];

    fn slug(value: &str) -> ProjectSlug {
        crate::test_support::parse(value)
    }

    async fn seeded_pool() -> DbPool {
        let pool = migrated_pool().await;
        seed_if_empty(&pool).await.expect("seed the database");
        pool
    }

    async fn public_order(pool: &DbPool) -> Vec<String> {
        assert_ok!(load(pool).await)
            .projects
            .iter()
            .map(|project| project.slug.to_string())
            .collect()
    }

    async fn sort_orders(pool: &DbPool) -> Vec<i64> {
        sqlx::query_scalar!(
            r#"
            SELECT
                sort_order
            FROM
                project
            ORDER BY
                sort_order
            "#
        )
        .fetch_all(pool)
        .await
        .expect("read the stored order")
    }

    #[rstest]
    #[case("traxor", ProjectMove::Up, ["traxor", "guenther", "cipher-workshop"])]
    #[case("traxor", ProjectMove::Down, ["guenther", "cipher-workshop", "traxor"])]
    fn a_step_swaps_with_its_neighbour(
        #[case] moved: &str,
        #[case] movement: ProjectMove,
        #[case] expected: [&str; 3],
    ) {
        let order = assert_ok!(reordered(&SEEDED, &slug(moved), &movement));

        assert_eq!(
            order
                .into_iter()
                .filter_map(|index| SEEDED.get(index).copied())
                .collect::<Vec<_>>(),
            expected
        );
    }

    #[test]
    fn an_anchored_move_takes_the_place_it_names() {
        let order = assert_ok!(reordered(
            &SEEDED,
            &slug("cipher-workshop"),
            &ProjectMove::ToPlaceOf(slug("guenther"))
        ));

        assert_eq!(
            order
                .into_iter()
                .filter_map(|index| SEEDED.get(index).copied())
                .collect::<Vec<_>>(),
            ["cipher-workshop", "guenther", "traxor"]
        );
    }

    #[rstest]
    #[case("guenther", ProjectMove::Up)]
    #[case("cipher-workshop", ProjectMove::Down)]
    #[case("traxor", ProjectMove::ToPlaceOf(slug("traxor")))]
    fn an_impossible_move_is_rejected(#[case] moved: &str, #[case] movement: ProjectMove) {
        assert_matches!(
            reordered(&SEEDED, &slug(moved), &movement),
            Err(MoveError::InvalidMovement)
        );
    }

    #[rstest]
    #[case("ghost", ProjectMove::Up)]
    #[case("traxor", ProjectMove::ToPlaceOf(slug("ghost")))]
    fn an_unknown_project_is_rejected(#[case] moved: &str, #[case] movement: ProjectMove) {
        assert_matches!(
            reordered(&SEEDED, &slug(moved), &movement),
            Err(MoveError::UnknownProject)
        );
    }

    #[tokio::test]
    async fn moving_up_swaps_the_two_stored_positions() {
        let pool = seeded_pool().await;

        assert_ok!(projects::move_to(&pool, &slug("traxor"), &ProjectMove::Up).await);

        assert_eq!(
            public_order(&pool).await,
            ["traxor", "guenther", "cipher-workshop"]
        );
    }

    #[tokio::test]
    async fn a_move_off_either_end_leaves_the_order_alone() {
        let pool = seeded_pool().await;

        assert_matches!(
            projects::move_to(&pool, &slug("guenther"), &ProjectMove::Up).await,
            Err(MoveError::InvalidMovement)
        );
        assert_matches!(
            projects::move_to(&pool, &slug("cipher-workshop"), &ProjectMove::Down).await,
            Err(MoveError::InvalidMovement)
        );

        assert_eq!(public_order(&pool).await, SEEDED);
        assert_eq!(sort_orders(&pool).await, [1, 2, 3]);
    }

    #[tokio::test]
    async fn an_unknown_or_stale_request_changes_nothing() {
        let pool = seeded_pool().await;

        assert_matches!(
            projects::move_to(&pool, &slug("ghost"), &ProjectMove::Down).await,
            Err(MoveError::UnknownProject)
        );
        assert_matches!(
            projects::move_to(
                &pool,
                &slug("traxor"),
                &ProjectMove::ToPlaceOf(slug("ghost"))
            )
            .await,
            Err(MoveError::UnknownProject)
        );

        assert_eq!(public_order(&pool).await, SEEDED);
    }

    /// Positions stay one through n after repeated moves, so the next move
    /// still has neighbours to swap with and the unique constraint holds.
    #[tokio::test]
    async fn repeated_moves_keep_the_order_dense() {
        let pool = seeded_pool().await;

        for movement in [ProjectMove::Down, ProjectMove::Down, ProjectMove::Up] {
            assert_ok!(projects::move_to(&pool, &slug("guenther"), &movement).await);
        }
        assert_ok!(
            projects::move_to(
                &pool,
                &slug("cipher-workshop"),
                &ProjectMove::ToPlaceOf(slug("traxor"))
            )
            .await
        );

        assert_eq!(sort_orders(&pool).await, [1, 2, 3]);
        assert_eq!(
            public_order(&pool).await,
            ["cipher-workshop", "traxor", "guenther"]
        );
    }
}
