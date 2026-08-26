//! Authentication and admin editing through the Leptos route and server-function adapters.

#![cfg(feature = "ssr")]
#![allow(clippy::expect_used, clippy::indexing_slicing, clippy::panic)]

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

fn form_post(uri: &str, body: &str, cookie: Option<&str>) -> Request<Body> {
    let mut builder =
        Request::post(uri).header(header::CONTENT_TYPE, "application/x-www-form-urlencoded");
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    builder
        .body(Body::from(body.to_owned()))
        .expect("build the form post")
}

fn login_request(username: &str, password: &str) -> Request<Body> {
    form_post(
        "/api/login",
        &format!("username={username}&password={password}"),
        None,
    )
}

fn get_request(uri: &str, cookie: Option<&str>) -> Request<Body> {
    let mut builder = Request::get(uri).header(header::ACCEPT, "text/html");
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    builder.body(Body::empty()).expect("build the request")
}

async fn sign_in(router: &Router) -> String {
    let response = router
        .clone()
        .oneshot(login_request("owner", "s3cret"))
        .await
        .expect("send the login");
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::LOCATION], "/admin");
    response.headers()[header::SET_COOKIE]
        .to_str()
        .expect("cookie is text")
        .split(';')
        .next()
        .expect("cookie has a value")
        .to_owned()
}

fn project_edit(
    slug: &str,
    title: &str,
    summary: &str,
    markdown: &str,
    cookie: Option<&str>,
) -> Request<Body> {
    form_post(
        "/api/save_project",
        &format!("slug={slug}&title={title}&summary={summary}&markdown={markdown}"),
        cookie,
    )
}

async fn body_text(response: axum::response::Response) -> String {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read the body");
    String::from_utf8_lossy(&body).into_owned()
}

#[tokio::test]
async fn signed_out_visitors_are_redirected_before_admin_routes_render() {
    let (router, _database) = app_with_owner().await;
    let response = router
        .oneshot(get_request("/admin", None))
        .await
        .expect("send the request");

    assert_eq!(response.status(), StatusCode::FOUND);
    assert_eq!(response.headers()[header::LOCATION], "/login");
}

#[tokio::test]
async fn wrong_credentials_are_rejected() {
    let (router, _database) = app_with_owner().await;
    let response = router
        .oneshot(login_request("owner", "wrong"))
        .await
        .expect("send the login");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn login_starts_a_session_that_reaches_the_dashboard() {
    let (router, _database) = app_with_owner().await;
    let cookie = sign_in(&router).await;
    let response = router
        .oneshot(get_request("/admin", Some(&cookie)))
        .await
        .expect("send the admin request");

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert!(body.contains("owner"));
    assert!(body.contains("/admin/project/traxor"));
}

#[tokio::test]
async fn logout_invalidates_the_owner_session() {
    let (router, _database) = app_with_owner().await;
    let cookie = sign_in(&router).await;
    let logout = router
        .clone()
        .oneshot(form_post("/api/logout", "", Some(&cookie)))
        .await
        .expect("send the logout");
    assert_eq!(logout.status(), StatusCode::OK);
    assert_eq!(logout.headers()[header::LOCATION], "/");

    let admin = router
        .oneshot(get_request("/admin", Some(&cookie)))
        .await
        .expect("send the old session");
    assert_eq!(admin.status(), StatusCode::FOUND);
    assert_eq!(admin.headers()[header::LOCATION], "/login");
}

#[tokio::test]
async fn project_editor_is_a_leptos_form_with_local_preview() {
    let (router, _database) = app_with_owner().await;
    let cookie = sign_in(&router).await;
    let response = router
        .oneshot(get_request("/admin/project/traxor", Some(&cookie)))
        .await
        .expect("send the editor request");

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert!(body.contains("action=\"/api/save_project\""));
    assert!(body.contains("id=\"md\""));
    assert!(body.contains("id=\"pv\""));
    assert!(body.contains("/admin/profile"));
}

#[tokio::test]
async fn unknown_project_routes_are_not_found() {
    let (router, _database) = app_with_owner().await;
    let cookie = sign_in(&router).await;
    let response = router
        .oneshot(get_request("/admin/project/ghost", Some(&cookie)))
        .await
        .expect("send the request");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn authenticated_owner_can_save_a_project() {
    let (router, _database) = app_with_owner().await;
    let cookie = sign_in(&router).await;
    let save = router
        .clone()
        .oneshot(project_edit(
            "traxor",
            "Traxor Reborn",
            "A fresh summary",
            "EDITMARKER42",
            Some(&cookie),
        ))
        .await
        .expect("send the edit");
    assert_eq!(save.status(), StatusCode::OK);

    let page = router
        .oneshot(get_request("/work/traxor", None))
        .await
        .expect("send the public page request");
    let body = body_text(page).await;
    assert!(body.contains("Traxor Reborn"));
    assert!(body.contains("EDITMARKER42"));
}

#[tokio::test]
async fn project_save_requires_an_authenticated_session() {
    let (router, _database) = app_with_owner().await;
    let response = router
        .oneshot(project_edit("traxor", "Title", "Summary", "Body", None))
        .await
        .expect("send the edit");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(response.headers()[header::LOCATION], "/login");
}

#[tokio::test]
async fn project_save_validates_required_fields_and_slug() {
    let (router, _database) = app_with_owner().await;
    let cookie = sign_in(&router).await;

    let empty = router
        .clone()
        .oneshot(project_edit("traxor", "", "Summary", "Body", Some(&cookie)))
        .await
        .expect("send the empty edit");
    assert_eq!(empty.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let missing = router
        .oneshot(project_edit(
            "ghost",
            "Title",
            "Summary",
            "Body",
            Some(&cookie),
        ))
        .await
        .expect("send the unknown edit");
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn authenticated_owner_can_save_singleton_content() {
    let (router, _database) = app_with_owner().await;
    let cookie = sign_in(&router).await;
    let edits = [
        (
            "/api/save_profile",
            "name=New+Name&title=A+title&summary=A+summary&about=About+me&email=me%40example.com",
        ),
        ("/api/save_contact", "name=Reach+out&body=Send+an+email"),
        (
            "/api/save_site",
            "url=https%3A%2F%2Fexample.com&title=New+Title&description=A+description&og_image=%2Fog.png",
        ),
    ];

    for (endpoint, body) in edits {
        let response = router
            .clone()
            .oneshot(form_post(endpoint, body, Some(&cookie)))
            .await
            .expect("send the singleton edit");
        assert_eq!(response.status(), StatusCode::OK);
    }

    let page = router
        .oneshot(get_request("/admin", Some(&cookie)))
        .await
        .expect("send the dashboard request");
    let body = body_text(page).await;
    assert!(body.contains("New Name"));
    assert!(body.contains("New Title"));
}
