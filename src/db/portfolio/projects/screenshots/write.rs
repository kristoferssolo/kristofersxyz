use super::{
    super::ScreenshotError,
    order::{ordered_ids, write_order},
};
use crate::{
    db::DbPool,
    domain::{
        ProjectSlug, ScreenshotAltText, ScreenshotCaption, ScreenshotId, ScreenshotMediaType,
        ScreenshotSize,
    },
};
use sqlx::{Sqlite, Transaction};

/// Stores an image after the Project's current final screenshot.
///
/// # Errors
///
/// Returns [`ScreenshotError::UnknownProject`] when no Project holds `slug` and
/// [`ScreenshotError::Transaction`] when the transaction fails.
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

/// Replaces one screenshot's alternative text and caption.
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

/// Deletes one screenshot and closes the gap in its Project's order.
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
