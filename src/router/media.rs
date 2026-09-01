//! The public route that serves Project Screenshot bytes.
//!
//! Screenshots are stored in SQLite, so this is the one handler that reads an
//! image blob. Bytes are immutable under an identity: a different image is a
//! different screenshot, which is what makes the year-long immutable cache
//! policy safe. An identity that does not parse is answered exactly like one
//! that no row holds, so the route never distinguishes a malformed guess from
//! a missing screenshot.

use crate::{
    db::{DbPool, portfolio},
    domain::ScreenshotId,
};
use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};

/// The Axum path this handler answers. It has to agree with the route the
/// domain builds for a screenshot, which the test below checks.
pub const SCREENSHOT_ROUTE: &str = "/media/project/{id}";

/// A stored screenshot never changes under its identity, so a reader and any
/// intermediary may keep it for a year.
const IMMUTABLE: HeaderValue = HeaderValue::from_static("public, max-age=31536000, immutable");

/// Answers with one screenshot's stored bytes.
#[tracing::instrument(name = "Serve project screenshot", skip(pool))]
pub async fn screenshot(State(pool): State<DbPool>, Path(id): Path<String>) -> Response {
    let Ok(id) = id.parse::<ScreenshotId>() else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Ok(Some(stored)) = portfolio::project_screenshot_image(&pool, &id).await else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let Ok(media_type) = HeaderValue::from_str(stored.media_type.as_str()) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Ok(length) = HeaderValue::from_str(&stored.bytes.len().to_string()) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    (
        [
            (header::CONTENT_TYPE, media_type),
            (header::CONTENT_LENGTH, length),
            (header::CACHE_CONTROL, IMMUTABLE),
        ],
        Body::from(stored.bytes),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::SCREENSHOT_MEDIA_PREFIX;

    /// The handler and [`ProjectScreenshot::media_path`] describe one route.
    ///
    /// [`ProjectScreenshot::media_path`]: crate::domain::ProjectScreenshot::media_path
    #[test]
    fn the_route_matches_the_path_the_domain_builds() {
        assert_eq!(
            SCREENSHOT_ROUTE,
            format!("{SCREENSHOT_MEDIA_PREFIX}/{{id}}")
        );
    }
}
