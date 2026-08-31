//! The router security boundary, authentication, and admin editing through the
//! Leptos route and server-function adapters.

#![cfg(feature = "ssr")]
#![allow(clippy::expect_used, clippy::indexing_slicing, clippy::panic)]

use std::{
    fmt::Write as _,
    io::{self, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{Arc, Mutex},
};

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
use tracing::instrument::WithSubscriber;
use tracing_subscriber::fmt::MakeWriter;

const TEST_PEER: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 41_000);
const TEST_ORIGIN: &str = "http://localhost:3000";
const TEST_HOST: &str = "localhost:3000";
const PRODUCTION_ORIGIN: &str = "https://kristofers.xyz";
const PRODUCTION_HOST: &str = "kristofers.xyz";

#[derive(Clone, Default)]
struct CapturedLogs(Arc<Mutex<Vec<u8>>>);

impl CapturedLogs {
    fn text(&self) -> String {
        let bytes = self.0.lock().expect("lock captured logs").clone();
        String::from_utf8(bytes).expect("captured logs are UTF-8")
    }
}

struct CapturedLogWriter(Arc<Mutex<Vec<u8>>>);

impl Write for CapturedLogWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0
            .lock()
            .map_err(|_| io::Error::other("captured log lock poisoned"))?
            .extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'writer> MakeWriter<'writer> for CapturedLogs {
    type Writer = CapturedLogWriter;

    fn make_writer(&'writer self) -> Self::Writer {
        CapturedLogWriter(Arc::clone(&self.0))
    }
}

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
        .header(header::HOST, TEST_HOST)
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
    let mut builder = Request::get(uri)
        .header(header::ACCEPT, "text/html")
        .header(header::HOST, TEST_HOST);
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    builder.body(Body::empty()).expect("build the request")
}

/// Re-addresses a request built for the local test origin, for the routers
/// whose `PUBLIC_ORIGIN` is the production one.
fn addressed_to(mut request: Request<Body>, host: &str) -> Request<Body> {
    request
        .headers_mut()
        .insert(header::HOST, host.parse().expect("build the host header"));
    request
}

/// A public page request addressed to `host`, or one that names no authority at
/// all.
fn page_request(host: Option<&str>) -> Request<Body> {
    let mut request = get_request("/", None);
    if let Some(host) = host {
        return addressed_to(request, host);
    }
    request.headers_mut().remove(header::HOST);
    request
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

/// A project save carrying both ordered collections, encoded the way a browser
/// encodes the editor's indexed field names.
fn project_edit(
    slug: &str,
    title: &str,
    summary: &str,
    markdown: &str,
    technologies: &[&str],
    links: &[(&str, &str)],
    cookie: Option<&str>,
) -> Request<Body> {
    let mut body = format!("slug={slug}&title={title}&summary={summary}&markdown={markdown}");
    for (index, technology) in technologies.iter().enumerate() {
        let _ = write!(
            body,
            "&technologies%5B{index}%5D={}",
            form_encode(technology)
        );
    }
    for (index, (label, href)) in links.iter().enumerate() {
        let _ = write!(
            body,
            "&links%5B{index}%5D%5Blabel%5D={}&links%5B{index}%5D%5Bhref%5D={}",
            form_encode(label),
            form_encode(href)
        );
    }

    form_post("/api/save_project", &body, cookie)
}

fn form_encode(value: &str) -> String {
    value
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                char::from(byte).to_string()
            }
            _ => format!("%{byte:02X}"),
        })
        .collect()
}

/// The order two values appear in, so a test can assert on stored order rather
/// than mere presence.
fn appears_before(haystack: &str, first: &str, second: &str) -> bool {
    match (haystack.find(first), haystack.find(second)) {
        (Some(first), Some(second)) => first < second,
        _ => false,
    }
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
        PRODUCTION_ORIGIN,
    );
    let app = ApplicationState::new(&settings)
        .await
        .expect("build the application");
    let response = route(app)
        .oneshot(addressed_to(get_request("/", None), PRODUCTION_HOST))
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
async fn oversized_login_bodies_are_rejected_before_throttling() {
    let (router, _database) = app_with_owner().await;
    let oversized_password = "x".repeat(5 * 1_024);

    for _ in 0..6 {
        let response = router
            .clone()
            .oneshot(login_request("owner", &oversized_password))
            .await
            .expect("send the oversized login request");
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }

    let response = router
        .oneshot(login_request("owner", "s3cret"))
        .await
        .expect("send a valid login after oversized requests");
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn oversized_content_edits_are_rejected_before_form_extraction() {
    let (router, _database) = app_with_owner().await;
    let cookie = sign_in(&router).await;
    let oversized_markdown = "x".repeat(257 * 1_024);

    let response = router
        .oneshot(project_edit(
            "traxor",
            "Traxor",
            "Summary",
            &oversized_markdown,
            &[],
            &[],
            Some(&cookie),
        ))
        .await
        .expect("send the oversized content request");

    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
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
async fn failed_authentication_logs_safe_structured_fields() {
    let (router, _database) = app_with_owner().await;
    let password = "submitted-secret-value";
    let logs = CapturedLogs::default();
    let subscriber = tracing_subscriber::fmt()
        .without_time()
        .with_ansi(false)
        .with_writer(logs.clone())
        .finish();

    let response = router
        .oneshot(login_request("owner", password))
        .with_subscriber(subscriber)
        .await
        .expect("send the failed login");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let logs = logs.text();
    assert!(logs.contains("kristofersxyz::security"));
    assert!(logs.contains("security_event=\"authentication_failed\""));
    assert!(logs.contains("reason=\"invalid_credentials\""));
    assert!(logs.contains("username=owner"));
    assert!(!logs.contains(password));
}

#[tokio::test]
async fn successful_authentication_logs_session_without_its_cookie() {
    let (router, _database) = app_with_owner().await;
    let logs = CapturedLogs::default();
    let subscriber = tracing_subscriber::fmt()
        .without_time()
        .with_ansi(false)
        .with_writer(logs.clone())
        .finish();

    let response = router
        .oneshot(login_request("owner", "s3cret"))
        .with_subscriber(subscriber)
        .await
        .expect("send the successful login");

    assert_eq!(response.status(), StatusCode::OK);
    let cookie = response.headers()[header::SET_COOKIE]
        .to_str()
        .expect("session cookie is text");
    let logs = logs.text();
    assert!(logs.contains("security_event=\"authentication_succeeded\""));
    assert!(logs.contains("security_event=\"session_started\""));
    assert!(logs.contains("session_rotated=true"));
    assert!(logs.contains("username=owner"));
    assert!(!logs.contains(cookie));
    assert!(!logs.contains("session_id"));
}

#[tokio::test]
async fn the_configured_authority_reaches_a_public_page() {
    let (router, _database) = app_with_owner().await;
    let response = router
        .oneshot(page_request(Some(TEST_HOST)))
        .await
        .expect("send the public request");

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn the_configured_authority_reaches_the_login_server_function() {
    let (router, _database) = app_with_owner().await;
    let response = router
        .oneshot(login_request("owner", "s3cret"))
        .await
        .expect("send the login");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::LOCATION], "/admin");
}

#[tokio::test]
async fn requests_addressed_to_another_authority_are_misdirected() {
    let (router, _database) = app_with_owner().await;
    let hosts = [
        None,
        Some("attacker.example"),
        Some("kristofers.xyz.attacker.example"),
        Some("localhost.attacker.example:3000"),
        Some("localhost:3001"),
        Some("localhost"),
        Some("localhost:3000/admin"),
        Some(""),
    ];

    for host in hosts {
        let response = router
            .clone()
            .oneshot(page_request(host))
            .await
            .expect("send the misdirected request");
        assert_eq!(
            response.status(),
            StatusCode::MISDIRECTED_REQUEST,
            "host: {host:?}"
        );
    }
}

#[tokio::test]
async fn a_request_target_that_disagrees_with_the_host_header_is_misdirected() {
    let (router, _database) = app_with_owner().await;
    let absolute = |target: &str, host: &str| {
        Request::get(target)
            .header(header::ACCEPT, "text/html")
            .header(header::HOST, host)
            .body(Body::empty())
            .expect("build the absolute-form request")
    };

    let agreeing = router
        .clone()
        .oneshot(absolute("http://localhost:3000/", TEST_HOST))
        .await
        .expect("send the agreeing request");
    assert_eq!(agreeing.status(), StatusCode::OK);

    for request in [
        absolute("http://localhost:3000/", "attacker.example"),
        absolute("http://attacker.example/", TEST_HOST),
    ] {
        let response = router
            .clone()
            .oneshot(request)
            .await
            .expect("send the conflicting request");
        assert_eq!(response.status(), StatusCode::MISDIRECTED_REQUEST);
    }
}

#[tokio::test]
async fn forwarding_headers_cannot_replace_the_request_authority() {
    let (router, _database) = app_with_owner().await;
    let mut spoofed = page_request(Some("attacker.example"));
    spoofed.headers_mut().insert(
        "x-forwarded-host",
        TEST_HOST.parse().expect("build the forwarded host"),
    );
    spoofed.headers_mut().insert(
        "forwarded",
        format!("host={TEST_HOST}")
            .parse()
            .expect("build the forwarded header"),
    );

    let response = router
        .clone()
        .oneshot(spoofed)
        .await
        .expect("send the spoofed request");
    assert_eq!(response.status(), StatusCode::MISDIRECTED_REQUEST);

    let mut genuine = page_request(Some(TEST_HOST));
    genuine.headers_mut().insert(
        "x-forwarded-host",
        "attacker.example"
            .parse()
            .expect("build the forwarded host"),
    );
    let response = router
        .oneshot(genuine)
        .await
        .expect("send the genuine request");
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn a_misdirected_login_keeps_security_headers_and_starts_no_session() {
    let (router, _database) = app_with_owner().await;
    let response = router
        .oneshot(addressed_to(
            login_request("owner", "s3cret"),
            "attacker.example",
        ))
        .await
        .expect("send the misdirected login");

    assert_eq!(response.status(), StatusCode::MISDIRECTED_REQUEST);
    assert!(!response.headers().contains_key(header::SET_COOKIE));
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    assert_eq!(
        response.headers()[header::X_CONTENT_TYPE_OPTIONS],
        "nosniff"
    );
    assert_eq!(response.headers()[header::X_FRAME_OPTIONS], "DENY");
    assert_eq!(
        response.headers()[header::REFERRER_POLICY],
        "strict-origin-when-cross-origin"
    );
    assert!(response.headers().contains_key("permissions-policy"));
}

#[tokio::test]
async fn a_misdirected_request_logs_its_reason_without_the_host_it_carried() {
    let (router, _database) = app_with_owner().await;
    let host = "kristofers.xyz.attacker.example";
    let logs = CapturedLogs::default();
    let subscriber = tracing_subscriber::fmt()
        .without_time()
        .with_ansi(false)
        .with_writer(logs.clone())
        .finish();

    let response = router
        .oneshot(page_request(Some(host)))
        .with_subscriber(subscriber)
        .await
        .expect("send the misdirected request");

    assert_eq!(response.status(), StatusCode::MISDIRECTED_REQUEST);
    let logs = logs.text();
    assert!(logs.contains("security_event=\"host_rejected\""));
    assert!(logs.contains("reason=\"unexpected\""));
    assert!(!logs.contains(host));
}

#[tokio::test]
async fn unsafe_requests_require_the_canonical_origin() {
    let (router, _database) = app_with_owner().await;
    let requests = [
        login_request("owner", "wrong"),
        form_post("/api/logout", "", None),
        project_edit("traxor", "Title", "Summary", "Body", &[], &[], None),
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
    let mut request = addressed_to(login_request("owner", "s3cret"), PRODUCTION_HOST);
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
            &["Rust", "ratatui"],
            &[("Codeberg", "https://codeberg.org/kristoferssolo/traxor")],
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
        .oneshot(project_edit(
            "traxor",
            "Title",
            "Summary",
            "Body",
            &["Rust"],
            &[],
            None,
        ))
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
        .oneshot(project_edit(
            "traxor",
            "",
            "Summary",
            "Body",
            &[],
            &[],
            Some(&cookie),
        ))
        .await
        .expect("send the empty edit");
    assert_eq!(empty.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let missing = router
        .oneshot(project_edit(
            "ghost",
            "Title",
            "Summary",
            "Body",
            &[],
            &[],
            Some(&cookie),
        ))
        .await
        .expect("send the unknown edit");
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn saving_a_project_replaces_its_technologies_and_links_in_order() {
    let (router, _database) = app_with_owner().await;
    let cookie = sign_in(&router).await;

    let save = router
        .clone()
        .oneshot(project_edit(
            "guenther",
            "guenther",
            "A summary",
            "A description",
            &["Docker Compose", "Rust", "teloxide"],
            &[
                ("Cobalt", "https://github.com/imputnet/cobalt"),
                ("GitHub", "https://github.com/kristoferssolo/guenther"),
            ],
            Some(&cookie),
        ))
        .await
        .expect("send the edit");
    assert_eq!(save.status(), StatusCode::OK);

    // The save answers with the refreshed portfolio, which is what the editor
    // puts back into the page.
    let portfolio = body_text(save).await;
    assert!(appears_before(&portfolio, "Docker Compose", "teloxide"));
    assert!(appears_before(
        &portfolio,
        "github.com/imputnet/cobalt",
        "github.com/kristoferssolo/guenther"
    ));
    assert!(
        !portfolio.contains("SQLx and SQLite"),
        "the replaced technologies are gone"
    );
    assert!(portfolio.contains("https://github.com/imputnet/cobalt"));
}

#[tokio::test]
async fn the_project_detail_renders_every_link_in_stored_order() {
    let (router, _database) = app_with_owner().await;
    let cookie = sign_in(&router).await;
    router
        .clone()
        .oneshot(project_edit(
            "guenther",
            "guenther",
            "A summary",
            "A description",
            &["Rust"],
            &[
                (
                    "Codeberg mirror",
                    "https://codeberg.org/kristoferssolo/guenther",
                ),
                ("GitHub", "https://github.com/kristoferssolo/guenther"),
                ("Cobalt", "https://github.com/imputnet/cobalt"),
            ],
            Some(&cookie),
        ))
        .await
        .expect("send the edit");

    let page = router
        .oneshot(get_request("/work/guenther", None))
        .await
        .expect("send the public page request");
    let body = body_text(page).await;

    // Anchors, not the portfolio JSON the page also carries for hydration.
    let anchor = |href: &str| format!(r#"href="{href}""#);
    assert!(appears_before(
        &body,
        &anchor("https://codeberg.org/kristoferssolo/guenther"),
        &anchor("https://github.com/kristoferssolo/guenther")
    ));
    assert!(appears_before(
        &body,
        &anchor("https://github.com/kristoferssolo/guenther"),
        &anchor("https://github.com/imputnet/cobalt")
    ));
    assert!(body.contains("Codeberg mirror"));
    assert!(body.contains(r#"rel="noopener noreferrer""#));
    assert!(body.contains(r#"target="_blank""#));
}

#[tokio::test]
async fn the_project_editor_renders_the_stored_technologies_and_links() {
    let (router, _database) = app_with_owner().await;
    let cookie = sign_in(&router).await;

    let response = router
        .oneshot(get_request("/admin/project/guenther", Some(&cookie)))
        .await
        .expect("send the editor request");

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert!(body.contains(r#"name="technologies[0]""#));
    assert!(body.contains(r#"value="SQLx and SQLite""#));
    assert!(body.contains(r#"name="links[0][label]""#));
    assert!(body.contains(r#"name="links[0][href]""#));
    assert!(body.contains("https://github.com/kristoferssolo/guenther"));
    assert!(body.contains("Move technology 1 down"));
    assert!(body.contains("Remove link 1"));
}

#[tokio::test]
async fn an_invalid_technology_or_link_url_rejects_the_whole_save() {
    let (router, _database) = app_with_owner().await;
    let cookie = sign_in(&router).await;

    let padded = router
        .clone()
        .oneshot(project_edit(
            "guenther",
            "Renamed",
            "A summary",
            "A description",
            &["Rust", "teloxide "],
            &[],
            Some(&cookie),
        ))
        .await
        .expect("send the padded technology");
    assert_eq!(padded.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let relative = router
        .clone()
        .oneshot(project_edit(
            "guenther",
            "Renamed",
            "A summary",
            "A description",
            &["Rust"],
            &[("Cobalt", "github.com/imputnet/cobalt")],
            Some(&cookie),
        ))
        .await
        .expect("send the relative URL");
    assert_eq!(relative.status(), StatusCode::UNPROCESSABLE_ENTITY);

    // Neither rejection wrote any part of its edit.
    let page = router
        .oneshot(get_request("/work/guenther", None))
        .await
        .expect("send the public page request");
    let body = body_text(page).await;
    assert!(!body.contains("Renamed"));
    assert!(body.contains("SQLx and SQLite"));
    assert!(body.contains("https://github.com/kristoferssolo/guenther"));
}

#[tokio::test]
async fn repeated_technologies_and_link_labels_are_rejected() {
    let (router, _database) = app_with_owner().await;
    let cookie = sign_in(&router).await;

    let technologies = router
        .clone()
        .oneshot(project_edit(
            "guenther",
            "Renamed",
            "A summary",
            "A description",
            &["Rust", "teloxide", "Rust"],
            &[],
            Some(&cookie),
        ))
        .await
        .expect("send the repeated technology");
    assert_eq!(technologies.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let labels = router
        .clone()
        .oneshot(project_edit(
            "guenther",
            "Renamed",
            "A summary",
            "A description",
            &["Rust"],
            &[
                ("GitHub", "https://github.com/kristoferssolo/guenther"),
                ("GitHub", "https://codeberg.org/kristoferssolo/guenther"),
            ],
            Some(&cookie),
        ))
        .await
        .expect("send the repeated label");
    assert_eq!(labels.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let page = router
        .oneshot(get_request("/work/guenther", None))
        .await
        .expect("send the public page request");
    assert!(!body_text(page).await.contains("Renamed"));
}

#[tokio::test]
async fn an_unauthenticated_request_cannot_change_either_collection() {
    let (router, _database) = app_with_owner().await;

    let response = router
        .clone()
        .oneshot(project_edit(
            "guenther",
            "guenther",
            "A summary",
            "A description",
            &["Borrowed"],
            &[("Elsewhere", "https://example.com")],
            None,
        ))
        .await
        .expect("send the signed out edit");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(response.headers()[header::LOCATION], "/login");

    let page = router
        .oneshot(get_request("/work/guenther", None))
        .await
        .expect("send the public page request");
    let body = body_text(page).await;
    assert!(!body.contains("Borrowed"));
    assert!(!body.contains("Elsewhere"));
    assert!(body.contains("SQLx and SQLite"));
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
