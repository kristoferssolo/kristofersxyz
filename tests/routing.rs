//! Public route status codes and canonical paths.

#![cfg(feature = "ssr")]
#![allow(clippy::expect_used)]

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
};
use kristofersxyz::{
    configuration::{DeploymentMode, PublicOrigin, Settings},
    db::{self, portfolio},
    domain::{ScreenshotAltText, ScreenshotId, ScreenshotMediaType, ScreenshotSize},
    router::route,
    startup::ApplicationState,
};
use tempfile::NamedTempFile;
use tower::ServiceExt;

/// A one-pixel PNG header, standing in for a stored screenshot. The route
/// serves whatever bytes are stored without looking at them again.
const IMAGE: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x42, 0x00, 0xFF,
];

const TEST_ORIGIN: &str = "http://localhost:3000";
const TEST_HOST: &str = "localhost:3000";

/// A seeded application over a temporary database, so the published slugs are
/// the ones in `seeds/portfolio.sql`.
async fn app() -> (Router, NamedTempFile) {
    let database = NamedTempFile::new().expect("create a temporary database");
    let settings = Settings::new(
        format!("sqlite://{}", database.path().display()),
        DeploymentMode::Local,
        TEST_ORIGIN
            .parse::<PublicOrigin>()
            .expect("the test origin is valid"),
    );
    let state = ApplicationState::new(&settings)
        .await
        .expect("build the application");
    (route(state), database)
}

/// Stores one screenshot on a published Project and returns the identity the
/// media route addresses it by.
async fn store_screenshot(database: &NamedTempFile) -> ScreenshotId {
    let pool = db::connect(&format!("sqlite://{}", database.path().display()))
        .await
        .expect("connect to the test database");
    portfolio::append_project_screenshot(
        &pool,
        &"traxor".parse().expect("the test slug is valid"),
        ScreenshotMediaType::Png,
        IMAGE,
        ScreenshotSize::try_from((1600, 1000)).expect("the test size is valid"),
        &"traxor listing four transfers in a terminal."
            .parse::<ScreenshotAltText>()
            .expect("the test alternative text is valid"),
        None,
    )
    .await
    .expect("store the screenshot")
}

async fn get(router: &Router, uri: &str) -> axum::response::Response {
    let request = Request::builder()
        .uri(uri)
        .header(header::HOST, TEST_HOST)
        .body(Body::empty())
        .expect("build the request");
    router
        .clone()
        .oneshot(request)
        .await
        .expect("route the request")
}

#[tokio::test]
async fn an_unknown_project_slug_answers_404() {
    let (router, _database) = app().await;

    let response = get(&router, "/work/no-such-project").await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_published_project_answers_200() {
    let (router, _database) = app().await;

    let response = get(&router, "/work/traxor").await;

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn a_trailing_slash_redirects_to_the_canonical_path() {
    let (router, _database) = app().await;

    let response = get(&router, "/work/traxor/").await;

    assert_eq!(response.status(), StatusCode::PERMANENT_REDIRECT);
    assert_eq!(
        response
            .headers()
            .get(header::LOCATION)
            .expect("the redirect sets a location"),
        "/work/traxor"
    );
}

#[tokio::test]
async fn a_stored_screenshot_answers_with_its_exact_bytes_and_headers() {
    let (router, database) = app().await;
    let id = store_screenshot(&database).await;

    let response = get(&router, &format!("/media/project/{id}")).await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "image/png");
    assert_eq!(
        response.headers()[header::CONTENT_LENGTH],
        IMAGE.len().to_string()
    );
    assert_eq!(
        response.headers()[header::CACHE_CONTROL],
        "public, max-age=31536000, immutable"
    );

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read the image");
    assert_eq!(body.as_ref(), IMAGE);
}

/// A malformed identity is answered exactly like a missing one, so the route
/// tells a guesser nothing about what is stored.
#[tokio::test]
async fn unknown_and_malformed_screenshot_ids_are_not_found() {
    let (router, _database) = app().await;

    for uri in [
        "/media/project/9c1f4e2a-1f2b-4a3c-8d4e-5f6a7b8c9d0e",
        "/media/project/not-a-uuid",
        "/media/project/9C1F4E2A-1F2B-4A3C-8D4E-5F6A7B8C9D0E",
        "/media/project/..%2F..%2Fetc%2Fpasswd",
    ] {
        let response = get(&router, uri).await;

        assert_eq!(
            response.status(),
            StatusCode::NOT_FOUND,
            "{uri} should not be found"
        );
    }
}
