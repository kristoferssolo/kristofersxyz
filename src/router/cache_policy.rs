use axum::{
    extract::Request,
    http::{HeaderValue, header},
    middleware::Next,
    response::Response,
};

/// Prevents browsers and intermediaries from retaining Owner-facing pages or
/// server-function responses. Public portfolio routes keep an independent
/// cache policy.
pub async fn add(request: Request, next: Next) -> Response {
    let sensitive = is_sensitive(request.uri().path());
    let mut response = next.run(request).await;
    if sensitive {
        response
            .headers_mut()
            .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    }
    response
}

fn is_sensitive(path: &str) -> bool {
    path == "/login"
        || path == "/admin"
        || path.starts_with("/admin/")
        || path == "/api"
        || path.starts_with("/api/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sensitive_paths_match_only_complete_route_segments() {
        for path in ["/login", "/admin", "/admin/profile", "/api", "/api/login"] {
            assert!(is_sensitive(path));
        }

        for path in ["/", "/work/traxor", "/administrator", "/login-help"] {
            assert!(!is_sensitive(path));
        }
    }
}
