//! End-to-end auth flow over the real router: the session guard blocks a
//! signed-out visitor, wrong credentials are rejected, and a completed login
//! carries its cookie back into the admin area.

#![cfg(feature = "ssr")]

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use kristofersxyz::{
    admin_cli::set_password,
    configuration::{DatabaseSettings, SessionSettings, Settings},
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
        session: SessionSettings {
            secure_cookie: false,
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

/// Signs in as the owner and returns the session cookie to replay.
async fn sign_in(router: &Router) -> String {
    let login = router
        .clone()
        .oneshot(login_request("owner", "s3cret"))
        .await
        .expect("send the login");
    login.headers()[header::SET_COOKIE]
        .to_str()
        .expect("cookie is text")
        .split(';')
        .next()
        .expect("cookie has a value")
        .to_owned()
}

fn edit_request(slug: &str, markdown: &str, cookie: Option<&str>) -> Request<Body> {
    let mut builder = Request::post(format!("/admin/project/{slug}"))
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded");
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    builder
        .body(Body::from(format!("markdown={markdown}")))
        .expect("build the edit request")
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

fn get_request(uri: &str, cookie: Option<&str>) -> Request<Body> {
    let mut builder = Request::get(uri);
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    builder.body(Body::empty()).expect("build the request")
}

async fn body_text(response: axum::response::Response) -> String {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read the body");
    String::from_utf8_lossy(&body).into_owned()
}

#[tokio::test]
async fn the_admin_page_links_to_each_project() {
    let (router, _database) = app_with_owner().await;
    let cookie = sign_in(&router).await;

    let page = router
        .oneshot(get_request("/admin", Some(&cookie)))
        .await
        .expect("send the admin request");
    assert_eq!(page.status(), StatusCode::OK);
    assert!(body_text(page).await.contains("/admin/project/traxor"));
}

#[tokio::test]
async fn the_edit_page_prefills_the_current_description() {
    let (router, _database) = app_with_owner().await;
    let cookie = sign_in(&router).await;

    router
        .clone()
        .oneshot(edit_request("traxor", "PREFILLMARKER", Some(&cookie)))
        .await
        .expect("send the edit");

    let page = router
        .oneshot(get_request("/admin/project/traxor", Some(&cookie)))
        .await
        .expect("send the edit-page request");
    assert_eq!(page.status(), StatusCode::OK);
    assert!(body_text(page).await.contains("PREFILLMARKER"));
}

#[tokio::test]
async fn the_edit_page_redirects_a_signed_out_visitor() {
    let (router, _database) = app_with_owner().await;

    let response = router
        .oneshot(get_request("/admin/project/traxor", None))
        .await
        .expect("send the request");

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(response.headers()[header::LOCATION], "/login");
}

#[tokio::test]
async fn the_edit_page_for_an_unknown_project_is_not_found() {
    let (router, _database) = app_with_owner().await;
    let cookie = sign_in(&router).await;

    let response = router
        .oneshot(get_request("/admin/project/ghost", Some(&cookie)))
        .await
        .expect("send the request");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn an_owner_can_edit_a_project_and_the_page_updates() {
    let (router, _database) = app_with_owner().await;
    let cookie = sign_in(&router).await;

    let save = router
        .clone()
        .oneshot(edit_request("traxor", "EDITMARKER42", Some(&cookie)))
        .await
        .expect("send the edit");
    assert_eq!(save.status(), StatusCode::SEE_OTHER);
    assert_eq!(save.headers()[header::LOCATION], "/admin");

    // The refreshed cache feeds the detail page render.
    let page = router
        .oneshot(
            Request::get("/work/traxor")
                .body(Body::empty())
                .expect("build request"),
        )
        .await
        .expect("send the page request");
    let body = to_bytes(page.into_body(), usize::MAX)
        .await
        .expect("read the body");
    assert!(String::from_utf8_lossy(&body).contains("EDITMARKER42"));
}

#[tokio::test]
async fn an_unauthenticated_edit_is_rejected() {
    let (router, _database) = app_with_owner().await;

    let response = router
        .oneshot(edit_request("traxor", "EDITMARKER42", None))
        .await
        .expect("send the edit");

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(response.headers()[header::LOCATION], "/login");
}

#[tokio::test]
async fn an_empty_description_is_rejected() {
    let (router, _database) = app_with_owner().await;
    let cookie = sign_in(&router).await;

    let response = router
        .oneshot(edit_request("traxor", "", Some(&cookie)))
        .await
        .expect("send the edit");

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn editing_an_unknown_project_is_not_found() {
    let (router, _database) = app_with_owner().await;
    let cookie = sign_in(&router).await;

    let response = router
        .oneshot(edit_request("ghost", "EDITMARKER42", Some(&cookie)))
        .await
        .expect("send the edit");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
