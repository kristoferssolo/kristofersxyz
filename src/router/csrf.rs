use crate::{configuration::PublicOrigin, security_events::SecurityEvent};
use axum::{
    extract::{Request, State},
    http::{HeaderMap, Method, header},
    middleware::Next,
    response::{IntoResponse, Response},
};

/// Rejects unsafe browser requests that do not prove the configured source
/// origin. `SameSite=Strict` remains a separate defense in depth.
pub async fn verify_origin(
    State(public_origin): State<PublicOrigin>,
    request: Request,
    next: Next,
) -> Response {
    if is_safe(request.method()) || is_same_origin(request.headers(), &public_origin) {
        next.run(request).await
    } else {
        SecurityEvent::CsrfRejected.record();
        axum::http::StatusCode::FORBIDDEN.into_response()
    }
}

const fn is_safe(method: &Method) -> bool {
    matches!(*method, Method::GET | Method::HEAD | Method::OPTIONS)
}

fn is_same_origin(headers: &HeaderMap, public_origin: &PublicOrigin) -> bool {
    if let Some(origin) = headers.get(header::ORIGIN) {
        return origin
            .to_str()
            .is_ok_and(|origin| origin == public_origin.as_str());
    }

    headers
        .get(header::REFERER)
        .and_then(|referer| referer.to_str().ok())
        .is_some_and(|referer| public_origin.matches_referer(referer))
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::assert_ok;
    use std::str::FromStr;

    fn origin() -> PublicOrigin {
        assert_ok!(PublicOrigin::from_str("https://kristofers.xyz"))
    }

    #[test]
    fn origin_must_match_exactly() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::ORIGIN,
            "https://kristofers.xyz".parse().expect("header"),
        );
        assert!(is_same_origin(&headers, &origin()));

        headers.insert(
            header::ORIGIN,
            "https://kristofers.xyz.attacker.example"
                .parse()
                .expect("header"),
        );
        assert!(!is_same_origin(&headers, &origin()));
    }

    #[test]
    fn same_origin_referer_is_the_only_fallback() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::REFERER,
            "https://kristofers.xyz/admin/profile"
                .parse()
                .expect("header"),
        );
        assert!(is_same_origin(&headers, &origin()));

        headers.insert(
            header::REFERER,
            "https://attacker.example/".parse().expect("header"),
        );
        assert!(!is_same_origin(&headers, &origin()));
        headers.remove(header::REFERER);
        assert!(!is_same_origin(&headers, &origin()));
    }
}
