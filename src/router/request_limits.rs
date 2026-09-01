use crate::{
    domain::MAX_SCREENSHOT_BYTES,
    security_events::{RequestKind, SecurityEvent},
};
use axum::{
    body::{Body, to_bytes},
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use http_body_util::LengthLimitError;
use std::error::Error;

const LOGIN_BODY_LIMIT: usize = 4 * 1_024;
const CONTENT_BODY_LIMIT: usize = 256 * 1_024;
/// The screenshot upload carries one image, so it needs room the other server
/// functions do not. The bound sits just above the image limit, leaving space
/// for the multipart envelope so an oversized image is answered with the
/// Owner-facing reason rather than a bare rejection.
const SCREENSHOT_BODY_LIMIT: usize = MAX_SCREENSHOT_BYTES + 64 * 1_024;
/// The endpoint the screenshot upload posts to, named by its server function.
const SCREENSHOT_UPLOAD_PATH: &str = "/api/upload_project_screenshot";

/// Buffers bounded server-function requests before session lookup or form
/// extraction. Login gets a tighter bound because its valid form is tiny, and
/// the screenshot upload a wider one because it carries an image.
pub async fn enforce(request: Request, next: Next) -> Response {
    let Some((limit, kind)) = limit_for(request.uri().path()) else {
        return next.run(request).await;
    };
    let (parts, body) = request.into_parts();

    match to_bytes(body, limit).await {
        Ok(bytes) => {
            next.run(Request::from_parts(parts, Body::from(bytes)))
                .await
        }
        Err(error)
            if error
                .source()
                .is_some_and(<dyn Error>::is::<LengthLimitError>) =>
        {
            SecurityEvent::RequestBodyRejected {
                kind,
                limit_bytes: limit,
            }
            .record();
            StatusCode::PAYLOAD_TOO_LARGE.into_response()
        }
        Err(_) => StatusCode::BAD_REQUEST.into_response(),
    }
}

fn limit_for(path: &str) -> Option<(usize, RequestKind)> {
    if path == "/api/login" {
        Some((LOGIN_BODY_LIMIT, RequestKind::Login))
    } else if path == SCREENSHOT_UPLOAD_PATH {
        Some((SCREENSHOT_BODY_LIMIT, RequestKind::ScreenshotUpload))
    } else if path == "/api" || path.starts_with("/api/") {
        Some((CONTENT_BODY_LIMIT, RequestKind::ServerFunction))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limits_only_complete_server_function_route_segments() {
        assert_eq!(
            limit_for("/api/login"),
            Some((LOGIN_BODY_LIMIT, RequestKind::Login))
        );
        assert_eq!(
            limit_for("/api/save_project"),
            Some((CONTENT_BODY_LIMIT, RequestKind::ServerFunction))
        );
        assert_eq!(
            limit_for("/api"),
            Some((CONTENT_BODY_LIMIT, RequestKind::ServerFunction))
        );
        assert_eq!(limit_for("/apiary"), None);
        assert_eq!(limit_for("/login"), None);
    }

    /// The upload needs its own bound. Widening it for every server function
    /// would let an unrelated form carry megabytes into session lookup.
    #[test]
    fn only_the_screenshot_upload_carries_an_image_sized_body() {
        assert_eq!(
            limit_for(SCREENSHOT_UPLOAD_PATH),
            Some((SCREENSHOT_BODY_LIMIT, RequestKind::ScreenshotUpload))
        );
        const { assert!(SCREENSHOT_BODY_LIMIT > MAX_SCREENSHOT_BYTES) }
        assert_eq!(
            limit_for("/api/save_project"),
            Some((CONTENT_BODY_LIMIT, RequestKind::ServerFunction))
        );
    }
}
