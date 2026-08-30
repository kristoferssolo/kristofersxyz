use axum::{
    extract::Request,
    http::{HeaderValue, Method, StatusCode, Uri, header},
    middleware::Next,
    response::{IntoResponse, Response},
};

/// Redirects `/work/slug/` to `/work/slug`, so each page answers on one URL.
///
/// The router matches paths without a trailing slash, so the variant with one
/// would otherwise reach the missing-page handler. Only readable methods are
/// redirected; anything else is left for the route to answer.
pub async fn redirect(request: Request, next: Next) -> Response {
    match canonical_target(request.method(), request.uri()).and_then(|target| {
        HeaderValue::try_from(target).ok().map(|location| {
            (
                StatusCode::PERMANENT_REDIRECT,
                [(header::LOCATION, location)],
            )
        })
    }) {
        Some(redirect) => redirect.into_response(),
        None => next.run(request).await,
    }
}

/// The same URI without its trailing slashes, or `None` when it already is
/// canonical.
fn canonical_target(method: &Method, uri: &Uri) -> Option<String> {
    if !matches!(*method, Method::GET | Method::HEAD) {
        return None;
    }

    let path = uri.path();
    let trimmed = path.trim_end_matches('/');
    // The root is already canonical, and trimming would leave it empty.
    let canonical = if trimmed.is_empty() { "/" } else { trimmed };
    if canonical == path {
        return None;
    }

    Some(uri.query().map_or_else(
        || canonical.to_owned(),
        |query| format!("{canonical}?{query}"),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("/work/traxor/", Some("/work/traxor"))]
    #[case("/work/traxor//", Some("/work/traxor"))]
    #[case("/work/traxor/?from=list", Some("/work/traxor?from=list"))]
    #[case("/admin/", Some("/admin"))]
    #[case("//", Some("/"))]
    #[case("/", None)]
    #[case("/work/traxor", None)]
    #[case("/?from=list", None)]
    fn readable_requests_lose_their_trailing_slashes(
        #[case] uri: &str,
        #[case] expected: Option<&str>,
    ) {
        let uri = uri.parse::<Uri>().expect("the test URI is valid");
        assert_eq!(canonical_target(&Method::GET, &uri).as_deref(), expected);
    }

    #[test]
    fn writing_methods_are_left_to_the_route() {
        let uri = "/api/login/".parse::<Uri>().expect("the test URI is valid");
        assert_eq!(canonical_target(&Method::POST, &uri), None);
    }
}
