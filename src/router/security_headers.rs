use crate::configuration::DeploymentMode;
use axum::{
    extract::{Request, State},
    http::{HeaderName, HeaderValue, header},
    middleware::Next,
    response::Response,
};
use leptos::{nonce::use_nonce, prelude::use_context};
use leptos_axum::ResponseOptions;

const PERMISSIONS_POLICY: HeaderName = HeaderName::from_static("permissions-policy");

/// Adds the request-specific nonce to the document's Content Security Policy.
/// Leptos generates this nonce and applies it to hydration and streaming
/// scripts before this additional render context runs.
pub fn provide_content_security_policy(deployment: DeploymentMode) {
    let (Some(nonce), Some(response)) = (use_nonce(), use_context::<ResponseOptions>()) else {
        return;
    };
    let connect_sources = match deployment {
        DeploymentMode::Local => "'self' ws: wss:",
        DeploymentMode::ProductionBehindTrustedProxy => "'self'",
    };
    let policy = format!(
        "default-src 'self'; base-uri 'none'; object-src 'none'; frame-ancestors 'none'; \
         form-action 'self'; script-src 'nonce-{nonce}' 'strict-dynamic' 'wasm-unsafe-eval'; \
         script-src-attr 'none'; style-src 'self' https://fonts.googleapis.com; \
         style-src-attr 'unsafe-inline'; font-src 'self' https://fonts.gstatic.com; \
         img-src 'self' data:; connect-src {connect_sources}"
    );
    let Ok(policy) = HeaderValue::from_str(&policy) else {
        std::process::abort();
    };
    response.insert_header(header::CONTENT_SECURITY_POLICY, policy);
}

/// Adds response headers that do not depend on the per-render CSP nonce.
pub async fn add(
    State(deployment): State<DeploymentMode>,
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    headers.insert(
        PERMISSIONS_POLICY,
        HeaderValue::from_static(
            "accelerometer=(), camera=(), geolocation=(), gyroscope=(), magnetometer=(), \
             microphone=(), payment=(), usb=()",
        ),
    );
    if deployment == DeploymentMode::ProductionBehindTrustedProxy {
        headers.insert(
            header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=31536000"),
        );
    }
    response
}
