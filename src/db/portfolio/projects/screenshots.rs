//! Storing, ordering, and reading the Project Screenshot rows.
//!
//! Image bytes live in SQLite beside their metadata, so an upload, its
//! ordering, and the Project it belongs to all commit or fail together, and one
//! database backup carries the whole portfolio. Nothing rewrites the bytes of
//! an existing row: an Owner who wants a different image deletes the screenshot
//! and uploads another, which is what makes the immutable cache policy on
//! `/media/project` safe. Metadata edits keep the identity, because the bytes
//! they describe have not changed.
//!
//! Positions are rewritten as one through n inside the moving Project, so the
//! order stays dense and the unique constraint holds at every row.

use super::{ScreenshotError, reorder_indices};
use crate::{
    db::DbPool,
    domain::{
        ProjectSlug, ScreenshotAltText, ScreenshotCaption, ScreenshotId, ScreenshotMediaType,
        ScreenshotMove, ScreenshotSize,
    },
};
use sqlx::{Sqlite, Transaction};
use std::fmt;

/// One screenshot's stored bytes and the media type they were validated as.
pub struct StoredScreenshot {
    pub media_type: ScreenshotMediaType,
    pub bytes: Vec<u8>,
}

/// Names the image rather than printing it, so a traced response or a failed
/// assertion cannot dump an entire screenshot into the output.
impl fmt::Debug for StoredScreenshot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("StoredScreenshot")
            .field("media_type", &self.media_type)
            .field("bytes", &self.bytes.len())
            .finish()
    }
}

/// Stores an image after the Project's current final screenshot and returns
/// the identity that addresses it.
///
/// # Errors
///
/// Returns [`ScreenshotError::UnknownProject`] when no Project holds `slug`,
/// leaving nothing stored, and [`ScreenshotError::Transaction`] when the
/// transaction fails.
#[tracing::instrument(
    name = "Store project screenshot",
    skip(pool, image, alt, caption),
    fields(slug = %slug, media_type = %media_type, bytes = image.len()),
    err,
)]
pub async fn append(
    pool: &DbPool,
    slug: &ProjectSlug,
    media_type: ScreenshotMediaType,
    image: &[u8],
    size: ScreenshotSize,
    alt: &ScreenshotAltText,
    caption: Option<&ScreenshotCaption>,
) -> Result<ScreenshotId, ScreenshotError> {
    let mut transaction = pool.begin().await?;
    let project_id = project_id(&mut transaction, slug).await?;

    let id = ScreenshotId::generate();
    let stored_id = id.as_str();
    let stored_type = media_type.as_str();
    let width = i64::from(size.width());
    let height = i64::from(size.height());
    let alt = alt.as_str();
    let caption = caption.map(ScreenshotCaption::as_str);

    sqlx::query!(
        r#"
        INSERT INTO
            project_screenshot (
                screenshot_id,
                project_id,
                media_type,
                image,
                width,
                height,
                alt_text,
                caption,
                sort_order
            )
        SELECT
            ?1,
            ?2,
            ?3,
            ?4,
            ?5,
            ?6,
            ?7,
            ?8,
            COALESCE(MAX(sort_order), 0) + 1
        FROM
            project_screenshot
        WHERE
            project_id = ?2
        "#,
        stored_id,
        project_id,
        stored_type,
        image,
        width,
        height,
        alt,
        caption,
    )
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;
    Ok(id)
}

/// Replaces one screenshot's alternative text and caption, keeping its
/// identity and its stored bytes.
///
/// # Errors
///
/// Returns [`ScreenshotError::UnknownScreenshot`] when no row holds `id`.
#[tracing::instrument(
    name = "Edit project screenshot details",
    skip(pool, alt, caption),
    fields(screenshot = %id),
    err,
)]
pub async fn set_details(
    pool: &DbPool,
    id: &ScreenshotId,
    alt: &ScreenshotAltText,
    caption: Option<&ScreenshotCaption>,
) -> Result<(), ScreenshotError> {
    let stored_id = id.as_str();
    let alt = alt.as_str();
    let caption = caption.map(ScreenshotCaption::as_str);

    let changed = sqlx::query!(
        r#"
        UPDATE
            project_screenshot
        SET
            alt_text = ?1,
            caption = ?2
        WHERE
            screenshot_id = ?3
        "#,
        alt,
        caption,
        stored_id,
    )
    .execute(pool)
    .await?
    .rows_affected();

    if changed == 0 {
        return Err(ScreenshotError::UnknownScreenshot);
    }
    Ok(())
}

/// Moves one screenshot a single step through its Project's order.
///
/// # Errors
///
/// Returns [`ScreenshotError::UnknownScreenshot`] for an identity no row
/// holds and [`ScreenshotError::InvalidMovement`] for a move off either end,
/// leaving the stored order unchanged in both cases.
#[tracing::instrument(
    name = "Move project screenshot",
    skip(pool),
    fields(screenshot = %id, movement = %movement),
    err,
)]
pub async fn move_within_project(
    pool: &DbPool,
    id: &ScreenshotId,
    movement: ScreenshotMove,
) -> Result<(), ScreenshotError> {
    let stored_id = id.as_str();
    let mut transaction = pool.begin().await?;

    let project_id = sqlx::query_scalar!(
        r#"
        SELECT
            project_id
        FROM
            project_screenshot
        WHERE
            screenshot_id = ?1
        "#,
        stored_id,
    )
    .fetch_optional(&mut *transaction)
    .await?
    .ok_or(ScreenshotError::UnknownScreenshot)?;

    let ordered = ordered_ids(&mut transaction, project_id).await?;
    let from = ordered
        .iter()
        .position(|candidate| candidate == id.as_str())
        .ok_or(ScreenshotError::UnknownScreenshot)?;
    let order = stepped(ordered.len(), from, movement)?;

    write_order(&mut transaction, project_id, &order, |index| {
        ordered.get(index).map(String::as_str)
    })
    .await?;

    transaction.commit().await?;
    Ok(())
}

/// Deletes one screenshot and closes the gap it leaves in its Project's order.
///
/// # Errors
///
/// Returns [`ScreenshotError::UnknownScreenshot`] when no row holds `id`.
#[tracing::instrument(name = "Delete project screenshot", skip(pool), fields(screenshot = %id), err)]
pub async fn remove(pool: &DbPool, id: &ScreenshotId) -> Result<(), ScreenshotError> {
    let stored_id = id.as_str();
    let mut transaction = pool.begin().await?;

    let project_id = sqlx::query_scalar!(
        r#"
        DELETE FROM
            project_screenshot
        WHERE
            screenshot_id = ?1
        RETURNING
            project_id AS "project_id!"
        "#,
        stored_id,
    )
    .fetch_optional(&mut *transaction)
    .await?
    .ok_or(ScreenshotError::UnknownScreenshot)?;

    let remaining = ordered_ids(&mut transaction, project_id).await?;
    let order = (0..remaining.len()).collect::<Vec<_>>();
    write_order(&mut transaction, project_id, &order, |index| {
        remaining.get(index).map(String::as_str)
    })
    .await?;

    transaction.commit().await?;
    Ok(())
}

/// Reads the bytes the media route answers with. This is the only query that
/// loads a blob, which is why it is not part of loading the portfolio.
///
/// # Errors
///
/// Returns [`ScreenshotError::Corrupt`] when a stored media type is not one
/// the application writes, and [`ScreenshotError::Transaction`] when the query
/// fails.
#[tracing::instrument(name = "Read project screenshot bytes", skip(pool), fields(screenshot = %id), err)]
pub async fn image(
    pool: &DbPool,
    id: &ScreenshotId,
) -> Result<Option<StoredScreenshot>, ScreenshotError> {
    let stored_id = id.as_str();
    let row = sqlx::query!(
        r#"
        SELECT
            media_type,
            image
        FROM
            project_screenshot
        WHERE
            screenshot_id = ?1
        "#,
        stored_id,
    )
    .fetch_optional(pool)
    .await?;

    row.map(|row| {
        Ok(StoredScreenshot {
            media_type: row
                .media_type
                .parse::<ScreenshotMediaType>()
                .map_err(|_| ScreenshotError::Corrupt)?,
            bytes: row.image,
        })
    })
    .transpose()
}

/// The identity of the Project a write belongs to.
async fn project_id(
    transaction: &mut Transaction<'_, Sqlite>,
    slug: &ProjectSlug,
) -> Result<i64, ScreenshotError> {
    let slug = slug.as_str();
    sqlx::query_scalar!(
        r#"
        SELECT
            id
        FROM
            project
        WHERE
            slug = ?1
        "#,
        slug,
    )
    .fetch_optional(&mut **transaction)
    .await?
    .ok_or(ScreenshotError::UnknownProject)
}

/// One Project's screenshot identities in their stored order.
async fn ordered_ids(
    transaction: &mut Transaction<'_, Sqlite>,
    project_id: i64,
) -> Result<Vec<String>, ScreenshotError> {
    let ids = sqlx::query_scalar!(
        r#"
        SELECT
            screenshot_id
        FROM
            project_screenshot
        WHERE
            project_id = ?1
        ORDER BY
            sort_order
        "#,
        project_id,
    )
    .fetch_all(&mut **transaction)
    .await?;
    Ok(ids)
}

/// Writes `order` back as positions one through n. Every row is parked on its
/// negated position first, so no intermediate state repeats a position and the
/// unique constraint holds throughout.
async fn write_order<'a>(
    transaction: &mut Transaction<'_, Sqlite>,
    project_id: i64,
    order: &[usize],
    id_at: impl Fn(usize) -> Option<&'a str>,
) -> Result<(), ScreenshotError> {
    sqlx::query!(
        r#"
        UPDATE
            project_screenshot
        SET
            sort_order = -sort_order
        WHERE
            project_id = ?1
        "#,
        project_id,
    )
    .execute(&mut **transaction)
    .await?;

    for (position, index) in order.iter().enumerate() {
        let id = id_at(*index).ok_or(ScreenshotError::InvalidMovement)?;
        let sort_order = i64::try_from(position.saturating_add(1))
            .map_err(|_| ScreenshotError::InvalidMovement)?;
        sqlx::query!(
            r#"
            UPDATE
                project_screenshot
            SET
                sort_order = ?1
            WHERE
                screenshot_id = ?2
            "#,
            sort_order,
            id,
        )
        .execute(&mut **transaction)
        .await?;
    }

    Ok(())
}

/// The positions `total` screenshots hold after the one at `from` takes a
/// single step. Deciding in positions keeps the rejected cases here rather
/// than in SQL.
fn stepped(
    total: usize,
    from: usize,
    movement: ScreenshotMove,
) -> Result<Vec<usize>, ScreenshotError> {
    let last = total.saturating_sub(1);
    let to = match movement {
        ScreenshotMove::Up => from
            .checked_sub(1)
            .ok_or(ScreenshotError::InvalidMovement)?,
        ScreenshotMove::Down => {
            let next = from.saturating_add(1);
            if next > last {
                return Err(ScreenshotError::InvalidMovement);
            }
            next
        }
    };

    Ok(reorder_indices(total, from, to))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db::{portfolio::load, seed_if_empty, test_support::migrated_pool},
        domain::{Project, ProjectScreenshot},
    };
    use claims::{assert_matches, assert_none, assert_ok, assert_some, assert_some_eq};
    use rstest::rstest;

    /// A one-pixel PNG, small enough to keep the fixtures readable and real
    /// enough to survive a round trip through SQLite.
    const PNG: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89,
    ];
    const WEBP: &[u8] = &[0x52, 0x49, 0x46, 0x46, 0x00, 0x57, 0x45, 0x42, 0x50];

    async fn seeded_pool() -> DbPool {
        let pool = migrated_pool().await;
        seed_if_empty(&pool).await.expect("seed the database");
        pool
    }

    fn slug(value: &str) -> ProjectSlug {
        crate::test_support::parse(value)
    }

    fn size() -> ScreenshotSize {
        assert_ok!(ScreenshotSize::try_from((1600, 1000)))
    }

    async fn store(pool: &DbPool, project: &str, alt: &str, bytes: &[u8]) -> ScreenshotId {
        assert_ok!(
            append(
                pool,
                &slug(project),
                ScreenshotMediaType::Png,
                bytes,
                size(),
                &crate::test_support::parse::<ScreenshotAltText>(alt),
                None,
            )
            .await
        )
    }

    async fn stored(pool: &DbPool, project: &str) -> Vec<ProjectScreenshot> {
        assert_ok!(load(pool).await)
            .projects
            .into_iter()
            .find(|candidate: &Project| candidate.slug.as_str() == project)
            .map(|found| found.screenshots.as_slice().to_vec())
            .unwrap_or_default()
    }

    async fn alt_texts(pool: &DbPool, project: &str) -> Vec<String> {
        stored(pool, project)
            .await
            .iter()
            .map(|shot| shot.alt.to_string())
            .collect()
    }

    async fn sort_orders(pool: &DbPool) -> Vec<i64> {
        sqlx::query_scalar!(
            r#"
            SELECT
                sort_order
            FROM
                project_screenshot
            ORDER BY
                sort_order
            "#
        )
        .fetch_all(pool)
        .await
        .expect("read the stored order")
    }

    #[tokio::test]
    async fn a_screenshot_lands_after_the_project_s_final_one() {
        let pool = seeded_pool().await;

        store(&pool, "traxor", "First", PNG).await;
        store(&pool, "traxor", "Second", PNG).await;

        assert_eq!(alt_texts(&pool, "traxor").await, ["First", "Second"]);
        assert_eq!(sort_orders(&pool).await, [1, 2]);
    }

    #[tokio::test]
    async fn a_screenshot_belongs_to_one_project_only() {
        let pool = seeded_pool().await;

        store(&pool, "traxor", "Queue", PNG).await;
        store(&pool, "guenther", "Chat", WEBP).await;

        assert_eq!(alt_texts(&pool, "traxor").await, ["Queue"]);
        assert_eq!(alt_texts(&pool, "guenther").await, ["Chat"]);
        assert!(stored(&pool, "cipher-workshop").await.is_empty());
    }

    #[tokio::test]
    async fn an_unknown_project_stores_nothing() {
        let pool = seeded_pool().await;

        let rejected = append(
            &pool,
            &slug("ghost"),
            ScreenshotMediaType::Png,
            PNG,
            size(),
            &crate::test_support::parse::<ScreenshotAltText>("Nothing"),
            None,
        )
        .await;

        assert_matches!(rejected, Err(ScreenshotError::UnknownProject));
        let orders: [i64; 0] = [];
        assert_eq!(sort_orders(&pool).await, orders);
    }

    #[tokio::test]
    async fn editing_the_details_keeps_the_identity_and_the_bytes() {
        let pool = seeded_pool().await;
        let id = store(&pool, "traxor", "Before", PNG).await;

        assert_ok!(
            set_details(
                &pool,
                &id,
                &crate::test_support::parse::<ScreenshotAltText>("After"),
                Some(&crate::test_support::parse::<ScreenshotCaption>(
                    "A caption"
                )),
            )
            .await
        );

        let screenshots = stored(&pool, "traxor").await;
        let screenshot = assert_some!(screenshots.first());
        assert_eq!(screenshot.id, id);
        assert_eq!(screenshot.alt.to_string(), "After");
        assert_some_eq!(
            screenshot.caption.as_ref().map(ToString::to_string),
            "A caption".to_owned()
        );
        assert_eq!(assert_some!(assert_ok!(image(&pool, &id).await)).bytes, PNG);
    }

    #[tokio::test]
    async fn a_caption_can_be_cleared() {
        let pool = seeded_pool().await;
        let id = store(&pool, "traxor", "Queue", PNG).await;
        let alt = crate::test_support::parse::<ScreenshotAltText>("Queue");
        assert_ok!(
            set_details(
                &pool,
                &id,
                &alt,
                Some(&crate::test_support::parse::<ScreenshotCaption>("Present")),
            )
            .await
        );

        assert_ok!(set_details(&pool, &id, &alt, None).await);

        let screenshots = stored(&pool, "traxor").await;
        assert_none!(assert_some!(screenshots.first()).caption.as_ref());
    }

    /// Nothing rewrites the bytes of a stored screenshot, so a different image
    /// is a different identity and no cached response can go stale.
    #[tokio::test]
    async fn replacing_the_image_takes_a_new_identity() {
        let pool = seeded_pool().await;
        let first = store(&pool, "traxor", "Queue", PNG).await;

        assert_ok!(remove(&pool, &first).await);
        let second = store(&pool, "traxor", "Queue", WEBP).await;

        assert_ne!(first, second);
        assert_none!(assert_ok!(image(&pool, &first).await));
        assert_eq!(
            assert_some!(assert_ok!(image(&pool, &second).await)).bytes,
            WEBP
        );
    }

    #[rstest]
    #[case(ScreenshotMove::Up, 1, ["Second", "First", "Third"])]
    #[case(ScreenshotMove::Down, 0, ["Second", "First", "Third"])]
    #[case(ScreenshotMove::Down, 1, ["First", "Third", "Second"])]
    #[tokio::test]
    async fn a_step_swaps_with_its_neighbour(
        #[case] movement: ScreenshotMove,
        #[case] moved: usize,
        #[case] expected: [&str; 3],
    ) {
        let pool = seeded_pool().await;
        for alt in ["First", "Second", "Third"] {
            store(&pool, "traxor", alt, PNG).await;
        }
        let screenshots = stored(&pool, "traxor").await;
        let id = assert_some!(screenshots.get(moved)).id.clone();

        assert_ok!(move_within_project(&pool, &id, movement).await);

        assert_eq!(alt_texts(&pool, "traxor").await, expected);
        assert_eq!(sort_orders(&pool).await, [1, 2, 3]);
    }

    #[tokio::test]
    async fn a_move_off_either_end_leaves_the_order_alone() {
        let pool = seeded_pool().await;
        let first = store(&pool, "traxor", "First", PNG).await;
        let last = store(&pool, "traxor", "Second", PNG).await;

        assert_matches!(
            move_within_project(&pool, &first, ScreenshotMove::Up).await,
            Err(ScreenshotError::InvalidMovement)
        );
        assert_matches!(
            move_within_project(&pool, &last, ScreenshotMove::Down).await,
            Err(ScreenshotError::InvalidMovement)
        );

        assert_eq!(alt_texts(&pool, "traxor").await, ["First", "Second"]);
        assert_eq!(sort_orders(&pool).await, [1, 2]);
    }

    /// A move only reaches the Project it belongs to, so a screenshot cannot
    /// step past the end of its own order into another Project's.
    #[tokio::test]
    async fn a_move_stays_inside_its_project() {
        let pool = seeded_pool().await;
        let only = store(&pool, "traxor", "Queue", PNG).await;
        store(&pool, "guenther", "Chat", PNG).await;

        assert_matches!(
            move_within_project(&pool, &only, ScreenshotMove::Down).await,
            Err(ScreenshotError::InvalidMovement)
        );
    }

    #[tokio::test]
    async fn deleting_removes_the_screenshot_and_closes_the_gap() {
        let pool = seeded_pool().await;
        let first = store(&pool, "traxor", "First", PNG).await;
        store(&pool, "traxor", "Second", PNG).await;
        store(&pool, "traxor", "Third", PNG).await;

        assert_ok!(remove(&pool, &first).await);

        assert_eq!(alt_texts(&pool, "traxor").await, ["Second", "Third"]);
        assert_eq!(sort_orders(&pool).await, [1, 2]);
        assert_none!(assert_ok!(image(&pool, &first).await));
    }

    #[tokio::test]
    async fn an_unknown_screenshot_is_rejected_by_every_write() {
        let pool = seeded_pool().await;
        let missing =
            crate::test_support::parse::<ScreenshotId>("9c1f4e2a-1f2b-4a3c-8d4e-5f6a7b8c9d0e");

        assert_matches!(
            remove(&pool, &missing).await,
            Err(ScreenshotError::UnknownScreenshot)
        );
        assert_matches!(
            move_within_project(&pool, &missing, ScreenshotMove::Up).await,
            Err(ScreenshotError::UnknownScreenshot)
        );
        assert_matches!(
            set_details(
                &pool,
                &missing,
                &crate::test_support::parse::<ScreenshotAltText>("Nothing"),
                None,
            )
            .await,
            Err(ScreenshotError::UnknownScreenshot)
        );
        assert_none!(assert_ok!(image(&pool, &missing).await));
    }

    /// The cascade is the database's own, so a Project cannot leave orphaned
    /// image bytes behind however it is deleted.
    #[tokio::test]
    async fn deleting_a_project_deletes_its_screenshots() {
        let pool = seeded_pool().await;
        store(&pool, "traxor", "Queue", PNG).await;
        store(&pool, "guenther", "Chat", PNG).await;

        sqlx::query!(
            r#"
            DELETE FROM
                project
            WHERE
                slug = 'traxor'
            "#
        )
        .execute(&pool)
        .await
        .expect("delete the project");

        let remaining = sqlx::query_scalar!(
            r#"
            SELECT
                alt_text
            FROM
                project_screenshot
            "#
        )
        .fetch_all(&pool)
        .await
        .expect("read the remaining screenshots");
        assert_eq!(remaining, ["Chat"]);
    }

    #[tokio::test]
    async fn stored_bytes_and_media_type_come_back_unchanged() {
        let pool = seeded_pool().await;
        let id = assert_ok!(
            append(
                &pool,
                &slug("traxor"),
                ScreenshotMediaType::Webp,
                WEBP,
                size(),
                &crate::test_support::parse::<ScreenshotAltText>("Queue"),
                None,
            )
            .await
        );

        let found = assert_some!(assert_ok!(image(&pool, &id).await));

        assert_eq!(found.media_type, ScreenshotMediaType::Webp);
        assert_eq!(found.bytes, WEBP);
    }

    /// Loading the portfolio reads metadata only. A blob in that query would
    /// put every stored image in memory on every page render.
    #[tokio::test]
    async fn loading_a_project_reads_metadata_without_the_bytes() {
        let pool = seeded_pool().await;
        store(&pool, "traxor", "Queue", PNG).await;

        let screenshots = stored(&pool, "traxor").await;

        let screenshot = assert_some!(screenshots.first());
        assert_eq!(screenshot.media_type, ScreenshotMediaType::Png);
        assert_eq!(
            (screenshot.size.width(), screenshot.size.height()),
            (1600, 1000)
        );
        assert_eq!(
            screenshot.media_path(),
            format!("/media/project/{}", screenshot.id)
        );
    }

    #[rstest]
    #[case(0, ScreenshotMove::Down, vec![1, 0, 2])]
    #[case(2, ScreenshotMove::Up, vec![0, 2, 1])]
    fn a_step_reorders_positions(
        #[case] from: usize,
        #[case] movement: ScreenshotMove,
        #[case] expected: Vec<usize>,
    ) {
        assert_eq!(assert_ok!(stepped(3, from, movement)), expected);
    }

    #[rstest]
    #[case(0, ScreenshotMove::Up)]
    #[case(2, ScreenshotMove::Down)]
    #[case(0, ScreenshotMove::Down)]
    fn a_step_past_either_end_is_rejected(#[case] from: usize, #[case] movement: ScreenshotMove) {
        let total = if from == 0 && movement == ScreenshotMove::Down {
            1
        } else {
            3
        };
        assert_matches!(
            stepped(total, from, movement),
            Err(ScreenshotError::InvalidMovement)
        );
    }
}
