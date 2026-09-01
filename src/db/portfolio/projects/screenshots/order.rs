use super::super::{ScreenshotError, reorder_indices};
use crate::{
    db::DbPool,
    domain::{ScreenshotId, ScreenshotMove},
};
use sqlx::{Sqlite, Transaction};

/// Moves one screenshot a single step through its Project's order.
///
/// # Errors
///
/// Returns [`ScreenshotError::UnknownScreenshot`] for an identity no row holds
/// and [`ScreenshotError::InvalidMovement`] for a move off either end.
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

/// One Project's screenshot identities in their stored order.
pub(super) async fn ordered_ids(
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

/// Writes positions one through n without violating the unique order constraint.
pub(super) async fn write_order<'a>(
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

/// Returns the positions after one screenshot takes a single step.
pub(super) fn stepped(
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
