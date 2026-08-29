//! Authentication and admin editing through the Leptos route and server-function adapters.

#![cfg(feature = "ssr")]
#![allow(clippy::expect_used, clippy::indexing_slicing, clippy::panic)]

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use axum::{
    Router,
    body::{Body, to_bytes},
    extract::ConnectInfo,
    http::{Request, StatusCode, header},
};
use kristofersxyz::{
    admin_cli::set_password,
    authentication::Password,
    configuration::{DeploymentMode, PublicOrigin, Settings},
    db,
    domain::Username,
    router::route,
    startup::ApplicationState,
};
use tempfile::NamedTempFile;
use tower::ServiceExt;

const TEST_PEER: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 41_000);
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
    let settings = settings_for(&database);
    set_password(&settings, &username("owner"), &password("s3cret"))
        .await
        .expect("create the owner");
    let app = ApplicationState::new(&settings)
        .await
        .expect("build the application");
    (route(app), database)
}

fn settings_for(database: &NamedTempFile) -> Settings {
    settings_for_deployment(database, DeploymentMode::Local, TEST_ORIGIN)
}

fn settings_for_deployment(
    database: &NamedTempFile,
    deployment: DeploymentMode,
    public_origin: &str,
) -> Settings {
    Settings::new(
        format!("sqlite://{}", database.path().display()),
        deployment,
        public_origin
            .parse::<PublicOrigin>()
            .expect("the test origin is valid"),
    )
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

fn header_text<'a>(response: &'a axum::response::Response, name: &str) -> &'a str {
    response.headers()[name]
        .to_str()
        .unwrap_or_else(|error| panic!("{name} is not text: {error}"))
}

#[tokio::test]
async fn documents_include_a_nonce_based_content_security_policy() {
    let (router, _database) = app_with_owner().await;
    let response = router
        .oneshot(get_request("/", None))
        .await
        .expect("send the public request");

    let policy = header_text(&response, header::CONTENT_SECURITY_POLICY.as_str()).to_owned();
    assert!(policy.contains("default-src 'self'"));
    assert!(policy.contains("object-src 'none'"));
    assert!(policy.contains("frame-ancestors 'none'"));
    assert!(policy.contains("script-src 'nonce-"));
    assert!(policy.contains("'strict-dynamic' 'wasm-unsafe-eval'"));
    assert!(policy.contains("style-src 'self' https://fonts.googleapis.com"));
    assert!(policy.contains("font-src 'self' https://fonts.gstatic.com"));
    assert!(policy.contains("connect-src 'self' ws: wss:"));

    let nonce = policy
        .split_once("'nonce-")
        .and_then(|(_, rest)| rest.split_once('\''))
        .map(|(nonce, _)| nonce)
        .expect("the policy contains a nonce");
    let body = body_text(response).await;
    assert!(body.contains(&format!("nonce=\"{nonce}\"")));
    assert!(body.contains(&format!(
        "id=\"portfolio-content\" type=\"application/json\" nonce=\"{nonce}\""
    )));
}

#[tokio::test]
async fn every_response_includes_static_security_headers() {
    let (router, _database) = app_with_owner().await;
    let response = router
        .oneshot(get_request("/", None))
        .await
        .expect("send the public request");

    assert_eq!(
        response.headers()[header::X_CONTENT_TYPE_OPTIONS],
        "nosniff"
    );
    assert_eq!(
        response.headers()[header::REFERRER_POLICY],
        "strict-origin-when-cross-origin"
    );
    assert_eq!(response.headers()[header::X_FRAME_OPTIONS], "DENY");
    assert_eq!(
        header_text(&response, "permissions-policy"),
        "accelerometer=(), camera=(), geolocation=(), gyroscope=(), magnetometer=(), microphone=(), payment=(), usb=()"
    );
    assert!(
        !response
            .headers()
            .contains_key(header::STRICT_TRANSPORT_SECURITY)
    );
}

#[tokio::test]
async fn production_responses_enable_strict_transport_security() {
    let database = NamedTempFile::new().expect("create a temporary database");
    let settings = settings_for_deployment(
        &database,
        DeploymentMode::ProductionBehindTrustedProxy,
        "https://kristofers.xyz",
    );
    let app = ApplicationState::new(&settings)
        .await
        .expect("build the application");
    let response = route(app)
        .oneshot(get_request("/", None))
        .await
        .expect("send the production request");

    assert_eq!(
        response.headers()[header::STRICT_TRANSPORT_SECURITY],
        "max-age=31536000"
    );
    let policy = header_text(&response, header::CONTENT_SECURITY_POLICY.as_str());
    assert!(policy.contains("connect-src 'self'"));
    assert!(!policy.contains("ws: wss:"));
}

#[tokio::test]
async fn owner_pages_and_server_functions_are_not_cacheable() {
    let (router, _database) = app_with_owner().await;

    let login_page = router
        .clone()
        .oneshot(get_request("/login", None))
        .await
        .expect("send the login page request");
    assert_eq!(login_page.headers()[header::CACHE_CONTROL], "no-store");

    let login_attempt = router
        .clone()
        .oneshot(login_request("owner", "wrong"))
        .await
        .expect("send the login attempt");
    assert_eq!(login_attempt.headers()[header::CACHE_CONTROL], "no-store");

    let cookie = sign_in(&router).await;
    let admin_page = router
        .oneshot(get_request("/admin", Some(&cookie)))
        .await
        .expect("send the admin page request");
    assert_eq!(admin_page.headers()[header::CACHE_CONTROL], "no-store");
}

#[tokio::test]
async fn public_pages_do_not_inherit_the_owner_cache_policy() {
    let (router, _database) = app_with_owner().await;
    let response = router
        .oneshot(get_request("/work/traxor", None))
        .await
        .expect("send the public request");

    assert!(!response.headers().contains_key(header::CACHE_CONTROL));
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
async fn production_login_emits_a_host_only_secure_cookie() {
    const PRODUCTION_ORIGIN: &str = "https://kristofers.xyz";

    let database = NamedTempFile::new().expect("create a temporary database");
    let settings = settings_for_deployment(
        &database,
        DeploymentMode::ProductionBehindTrustedProxy,
        PRODUCTION_ORIGIN,
    );
    set_password(&settings, &username("owner"), &password("s3cret"))
        .await
        .expect("create the owner");
    let app = ApplicationState::new(&settings)
        .await
        .expect("build the application");
    let router = route(app);
    let mut request = login_request("owner", "s3cret");
    request.headers_mut().insert(
        header::ORIGIN,
        PRODUCTION_ORIGIN.parse().expect("build the origin header"),
    );

    let response = router
        .oneshot(request)
        .await
        .expect("send the production login");
    assert_eq!(response.status(), StatusCode::OK);
    let set_cookie = response.headers()[header::SET_COOKIE]
        .to_str()
        .expect("cookie is text");
    assert!(set_cookie.starts_with("__Host-kristofersxyz-session="));
    assert!(set_cookie.contains("HttpOnly"));
    assert!(set_cookie.contains("SameSite=Strict"));
    assert!(set_cookie.contains("Secure"));
    assert!(set_cookie.contains("Path=/"));
    assert!(!set_cookie.contains("Domain="));
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
async fn changing_the_password_invalidates_existing_sessions() {
    let (router, database) = app_with_owner().await;
    let cookie = sign_in(&router).await;

    set_password(
        &settings_for(&database),
        &username("owner"),
        &password("replacement password"),
    )
    .await
    .expect("replace the owner password");

    let old_session = router
        .clone()
        .oneshot(get_request("/admin", Some(&cookie)))
        .await
        .expect("send the old session");
    assert_eq!(old_session.status(), StatusCode::FOUND);
    assert_eq!(old_session.headers()[header::LOCATION], "/login");

    let old_password = router
        .clone()
        .oneshot(login_request("owner", "s3cret"))
        .await
        .expect("try the old password");
    assert_eq!(old_password.status(), StatusCode::UNAUTHORIZED);

    let new_password = router
        .oneshot(login_request("owner", "replacement password"))
        .await
        .expect("try the replacement password");
    assert_eq!(new_password.status(), StatusCode::OK);
}

#[tokio::test]
async fn deleting_the_owner_invalidates_an_existing_session() {
    let (router, database) = app_with_owner().await;
    let cookie = sign_in(&router).await;
    let pool = db::connect(&settings_for(&database).database.url)
        .await
        .expect("connect to the database");
    sqlx::query!(
        "DELETE FROM
    users"
    )
    .execute(&pool)
    .await
    .expect("delete the owner");

    let response = router
        .oneshot(get_request("/admin", Some(&cookie)))
        .await
        .expect("send the old session");
    assert_eq!(response.status(), StatusCode::FOUND);
    assert_eq!(response.headers()[header::LOCATION], "/login");

    let sessions = sqlx::query_scalar!(
        "SELECT
    COUNT(*)
FROM
    sessions"
    )
    .fetch_one(&pool)
    .await
    .expect("count sessions");
    assert_eq!(sessions, 0);
}

#[tokio::test]
async fn rotating_the_session_version_invalidates_an_existing_session() {
    let (router, database) = app_with_owner().await;
    let cookie = sign_in(&router).await;
    let pool = db::connect(&settings_for(&database).database.url)
        .await
        .expect("connect to the database");
    let session_version = uuid::Uuid::new_v4().to_string();
    sqlx::query!(
        r#"
UPDATE
    users
SET
    session_version = ?1
    "#,
        session_version
    )
    .execute(&pool)
    .await
    .expect("rotate the session version");

    let response = router
        .oneshot(get_request("/admin", Some(&cookie)))
        .await
        .expect("send the old session");
    assert_eq!(response.status(), StatusCode::FOUND);
    assert_eq!(response.headers()[header::LOCATION], "/login");

    let sessions = sqlx::query_scalar!(
        "SELECT
    COUNT(*)
FROM
    sessions"
    )
    .fetch_one(&pool)
    .await
    .expect("count sessions");
    assert_eq!(sessions, 0);
}

#[tokio::test]
async fn the_absolute_lifetime_invalidates_an_existing_session() {
    let (router, database) = app_with_owner().await;
    let cookie = sign_in(&router).await;
    let pool = db::connect(&settings_for(&database).database.url)
        .await
        .expect("connect to the database");
    let data = sqlx::query_scalar!(
        "SELECT
    data
FROM
    sessions"
    )
    .fetch_one(&pool)
    .await
    .expect("load the session");
    let mut record: tower_sessions::session::Record =
        serde_json::from_str(&data).expect("stored session record is valid");
    record
        .data
        .insert("owner-issued-at".to_owned(), serde_json::json!(0));
    let data = serde_json::to_string(&record).expect("serialize the expired session");
    sqlx::query!(
        "UPDATE
    sessions
SET
    data = ?1",
        data
    )
    .execute(&pool)
    .await
    .expect("expire the session");

    let response = router
        .oneshot(get_request("/admin", Some(&cookie)))
        .await
        .expect("send the expired session");
    let status = response.status();
    let location = response.headers().get(header::LOCATION).cloned();
    let body = body_text(response).await;
    assert_eq!(status, StatusCode::FOUND, "{body}");
    assert_eq!(location.expect("redirect location"), "/login");

    let sessions = sqlx::query_scalar!(
        "SELECT
    COUNT(*)
FROM
    sessions"
    )
    .fetch_one(&pool)
    .await
    .expect("count sessions");
    assert_eq!(sessions, 0);
}

#[tokio::test]
async fn a_new_login_replaces_the_previous_session() {
    let (router, _database) = app_with_owner().await;
    let first_cookie = sign_in(&router).await;
    let second_cookie = sign_in(&router).await;
    assert_ne!(first_cookie, second_cookie);

    let first = router
        .clone()
        .oneshot(get_request("/admin", Some(&first_cookie)))
        .await
        .expect("send the first session");
    assert_eq!(first.status(), StatusCode::FOUND);
    assert_eq!(first.headers()[header::LOCATION], "/login");

    let second = router
        .oneshot(get_request("/admin", Some(&second_cookie)))
        .await
        .expect("send the second session");
    assert_eq!(second.status(), StatusCode::OK);
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
