//! End-to-end auth flow over the real router: the session guard blocks a
//! signed-out visitor, wrong credentials are rejected, and a completed login
//! carries its cookie back into the admin area.

#![cfg(feature = "ssr")]

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
};
use kristofersxyz::{
    admin_cli::set_password,
    configuration::{DatabaseSettings, Settings},
    router::route,
    startup::App,
};
use secrecy::SecretString;
use tempfile::NamedTempFile;
use tower::ServiceExt;

/// A router serving a portfolio whose database already holds one owner. The
/// temp file is returned so it outlives the pool.
async fn app_with_owner() -> (Router, NamedTempFile) {
    let database = NamedTempFile::new().expect("create a temporary database");
    let settings = Settings {
        database: DatabaseSettings {
            url: format!("sqlite://{}", database.path().display()),
        },
    };
    set_password(&settings, "owner", &SecretString::from("s3cret".to_owned()))
        .await
        .expect("create the owner");
    let app = App::new(&settings).await.expect("build the application");
    (route(app), database)
}

fn login_request(username: &str, password: &str) -> Request<Body> {
    Request::post("/login")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(format!(
            "username={username}&password={password}"
        )))
        .expect("build the login request")
}

#[tokio::test]
async fn the_admin_area_redirects_a_signed_out_visitor() {
    let (router, _database) = app_with_owner().await;

    let response = router
        .oneshot(
            Request::get("/admin")
                .body(Body::empty())
                .expect("build request"),
        )
        .await
        .expect("send the request");

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(response.headers()[header::LOCATION], "/login");
}

#[tokio::test]
async fn a_wrong_password_is_rejected() {
    let (router, _database) = app_with_owner().await;

    let response = router
        .oneshot(login_request("owner", "wrong"))
        .await
        .expect("send the request");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn a_completed_login_reaches_the_admin_area() {
    let (router, _database) = app_with_owner().await;

    let login = router
        .clone()
        .oneshot(login_request("owner", "s3cret"))
        .await
        .expect("send the login");
    assert_eq!(login.status(), StatusCode::SEE_OTHER);
    assert_eq!(login.headers()[header::LOCATION], "/admin");

    let cookie = login.headers()[header::SET_COOKIE]
        .to_str()
        .expect("cookie is text")
        .split(';')
        .next()
        .expect("cookie has a value")
        .to_owned();

    let admin = router
        .oneshot(
            Request::get("/admin")
                .header(header::COOKIE, cookie)
                .body(Body::empty())
                .expect("build request"),
        )
        .await
        .expect("send the admin request");

    assert_eq!(admin.status(), StatusCode::OK);
}
