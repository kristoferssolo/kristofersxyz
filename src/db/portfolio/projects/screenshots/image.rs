use super::super::ScreenshotError;
use crate::{
    db::DbPool,
    domain::{ScreenshotId, ScreenshotMediaType},
};
use std::fmt;

/// One screenshot's stored bytes and the media type they were validated as.
pub struct StoredScreenshot {
    pub media_type: ScreenshotMediaType,
    pub bytes: Vec<u8>,
}

/// Names the image rather than printing it, so diagnostics cannot dump the blob.
impl fmt::Debug for StoredScreenshot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("StoredScreenshot")
            .field("media_type", &self.media_type)
            .field("bytes", &self.bytes.len())
            .finish()
    }
}

/// Reads the bytes the media route answers with. Portfolio loading reads metadata only.
///
/// # Errors
///
/// Returns [`ScreenshotError::Corrupt`] for an unknown stored media type and
/// [`ScreenshotError::Transaction`] when the query fails.
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
