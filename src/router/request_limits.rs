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

/// Buffers bounded server-function requests before session lookup or form
/// extraction. Login gets a tighter bound because its valid form is tiny.
pub async fn enforce(request: Request, next: Next) -> Response {
    let Some(limit) = limit_for(request.uri().path()) else {
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
            StatusCode::PAYLOAD_TOO_LARGE.into_response()
        }
        Err(_) => StatusCode::BAD_REQUEST.into_response(),
    }
}

fn limit_for(path: &str) -> Option<usize> {
    if path == "/api/login" {
        Some(LOGIN_BODY_LIMIT)
    } else if path == "/api" || path.starts_with("/api/") {
        Some(CONTENT_BODY_LIMIT)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limits_only_complete_server_function_route_segments() {
        assert_eq!(limit_for("/api/login"), Some(LOGIN_BODY_LIMIT));
        assert_eq!(limit_for("/api/save_project"), Some(CONTENT_BODY_LIMIT));
        assert_eq!(limit_for("/api"), Some(CONTENT_BODY_LIMIT));
        assert_eq!(limit_for("/apiary"), None);
        assert_eq!(limit_for("/login"), None);
    }
}
