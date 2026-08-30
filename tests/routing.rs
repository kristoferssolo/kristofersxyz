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
    router::route,
    startup::ApplicationState,
};
use tempfile::NamedTempFile;
use tower::ServiceExt;

/// A seeded application over a temporary database, so the published slugs are
/// the ones in `seeds/portfolio.sql`.
async fn app() -> (Router, NamedTempFile) {
    let database = NamedTempFile::new().expect("create a temporary database");
    let settings = Settings::new(
        format!("sqlite://{}", database.path().display()),
        DeploymentMode::Local,
        "http://localhost:3000"
            .parse::<PublicOrigin>()
            .expect("the test origin is valid"),
    );
    let state = ApplicationState::new(&settings)
        .await
        .expect("build the application");
    (route(state), database)
}

async fn get(router: &Router, uri: &str) -> axum::response::Response {
    let request = Request::builder()
        .uri(uri)
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
