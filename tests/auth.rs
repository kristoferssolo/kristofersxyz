//! Authentication and admin editing through the Leptos route and server-function adapters.

#![cfg(feature = "ssr")]
#![allow(clippy::expect_used, clippy::indexing_slicing, clippy::panic)]

use axum::{
    Router,
    body::{Body, to_bytes},
    extract::ConnectInfo,
    http::{Request, StatusCode, header},
};
use kristofersxyz::{
    admin_cli::set_password,
    authentication::Password,
    configuration::{DatabaseSettings, HttpSettings, PublicOrigin, SessionSettings, Settings},
    domain::Username,
    router::route,
    startup::App,
};
use tempfile::NamedTempFile;
use tower::ServiceExt;

const TEST_PEER: std::net::SocketAddr =
    std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST), 41_000);
const TEST_ORIGIN: &str = "http://localhost:3000";

fn username(value: &str) -> Username {
    Username::new(value.to_owned())
        .unwrap_or_else(|error| panic!("invalid username in test fixture: {error}"))
}

fn password(value: &str) -> Password {
    Password::try_from(value.to_owned())
        .unwrap_or_else(|error| panic!("invalid password in test fixture: {error}"))
}

async fn app_with_owner() -> (Router, NamedTempFile) {
    let database = NamedTempFile::new().expect("create a temporary database");
    let settings = Settings {
        database: DatabaseSettings {
            url: format!("sqlite://{}", database.path().display()),
        },
        http: HttpSettings {
            public_origin: "http://localhost:3000"
                .parse::<PublicOrigin>()
                .expect("the test origin is valid"),
        },
        session: SessionSettings {
            secure_cookie: false,
        },
    };
    set_password(&settings, &username("owner"), &password("s3cret"))
        .await
        .expect("create the owner");
    let app = App::new(&settings).await.expect("build the application");
    (route(app), database)
}

fn form_post(uri: &str, body: &str, cookie: Option<&str>) -> Request<Body> {
    let mut builder = Request::post(uri)
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(header::ORIGIN, TEST_ORIGIN);
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    builder
        .body(Body::from(body.to_owned()))
        .expect("build the form post")
}

fn login_request(username: &str, password: &str) -> Request<Body> {
    let mut request = form_post(
        "/api/login",
        &format!("username={username}&password={password}"),
        None,
    );
    request.extensions_mut().insert(ConnectInfo(TEST_PEER));
    request
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
async fn unsafe_requests_require_the_canonical_origin() {
    let (router, _database) = app_with_owner().await;
    let requests = [
        login_request("owner", "wrong"),
        form_post("/api/logout", "", None),
        project_edit("traxor", "Title", "Summary", "Body", None),
        form_post(
            "/api/save_profile",
            "name=Name&title=Title&summary=Summary&about=About&email=me%40example.com",
            None,
        ),
        form_post("/api/save_contact", "name=Name&body=Body", None),
        form_post(
            "/api/save_site",
            "url=https%3A%2F%2Fexample.com&title=Title&description=Description&og_image=%2Fog.png",
            None,
        ),
    ];

    for mut request in requests {
        request.headers_mut().insert(
            header::ORIGIN,
            "https://attacker.example".parse().expect("header"),
        );
        let response = router
            .clone()
            .oneshot(request)
            .await
            .expect("send a cross-origin request");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    let mut missing = login_request("owner", "wrong");
    missing.headers_mut().remove(header::ORIGIN);
    let response = router
        .oneshot(missing)
        .await
        .expect("send a request without origin proof");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn same_origin_referer_is_accepted_when_origin_is_absent() {
    let (router, _database) = app_with_owner().await;
    let mut request = login_request("owner", "wrong");
    request.headers_mut().remove(header::ORIGIN);
    request.headers_mut().insert(
        header::REFERER,
        "http://localhost:3000/login".parse().expect("header"),
    );

    let response = router
        .oneshot(request)
        .await
        .expect("send a same-origin request");
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn repeated_account_failures_return_a_retry_delay() {
    let (router, _database) = app_with_owner().await;

    for _ in 0..6 {
        let response = router
            .clone()
            .oneshot(login_request("owner", "wrong"))
            .await
            .expect("send a failed login");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    let throttled = router
        .oneshot(login_request("owner", "wrong"))
        .await
        .expect("send a throttled login");
    assert_eq!(throttled.status(), StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(throttled.headers()[header::RETRY_AFTER], "1");
}

#[tokio::test]
async fn source_limit_ignores_spoofed_forwarding_headers() {
    let (router, _database) = app_with_owner().await;

    for attempt in 0..20 {
        let mut request = login_request("", "wrong");
        request.headers_mut().insert(
            "x-forwarded-for",
            format!("198.51.100.{attempt}")
                .parse()
                .expect("build the spoofed address"),
        );
        let response = router
            .clone()
            .oneshot(request)
            .await
            .expect("send a malformed login");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    let throttled = router
        .oneshot(login_request("", "wrong"))
        .await
        .expect("send a throttled login");
    assert_eq!(throttled.status(), StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(throttled.headers()[header::RETRY_AFTER], "60");
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
    assert!(body.contains("id=\"admin-navigation\""));
    assert!(body.contains("aria-controls=\"admin-navigation\""));
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
