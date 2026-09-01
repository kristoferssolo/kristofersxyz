mod cache_policy;
mod csrf;
mod media;
mod request_authority;
mod request_limits;
mod security_headers;
mod trailing_slash;

use crate::{
    app::{App, content::server_content, shell},
    authentication::AuthBackend,
    configuration::SessionPolicy,
    db::DbPool,
    sessions::SqliteSessionStore,
    startup::ApplicationState,
};
use axum::{Router, middleware, routing::get};
use axum_login::AuthManagerLayerBuilder;
use leptos_axum::{LeptosRoutes, file_and_error_handler_with_context, generate_route_list};
use tower_sessions::{Expiry, SessionManagerLayer, cookie::SameSite};

pub fn route(state: ApplicationState) -> Router {
    let routes = generate_route_list(App);
    let sessions = session_layer(state.pool.clone(), state.session_policy);
    let deployment = state.deployment;
    let authentication =
        AuthManagerLayerBuilder::new(AuthBackend::new(state.pool.clone()), sessions).build();

    Router::new()
        .leptos_routes_with_context(
            &state,
            routes,
            move || {
                security_headers::provide_content_security_policy(deployment);
            },
            {
                let leptos_options = state.leptos_options.clone();
                move || shell(leptos_options.clone(), server_content().as_ref())
            },
        )
        .route(media::SCREENSHOT_ROUTE, get(media::screenshot))
        .fallback(file_and_error_handler_with_context::<ApplicationState, _>(
            move || security_headers::provide_content_security_policy(deployment),
            |options| shell(options, server_content().as_ref()),
        ))
        .layer(authentication)
        .layer(middleware::from_fn(request_limits::enforce))
        .layer(middleware::from_fn(trailing_slash::redirect))
        .layer(middleware::from_fn_with_state(
            state.public_origin.clone(),
            csrf::verify_origin,
        ))
        .layer(middleware::from_fn_with_state(
            state.public_origin.clone(),
            request_authority::verify,
        ))
        .layer(middleware::from_fn_with_state(
            deployment,
            security_headers::add,
        ))
        .layer(middleware::from_fn(cache_policy::add))
        .with_state(state)
}

/// The server-side session middleware, backed by the shared SQLite pool.
///
/// The validated deployment mode supplies a coherent cookie name and transport
/// policy. No `Domain` attribute is set, so the cookie remains host-only.
fn session_layer(pool: DbPool, policy: SessionPolicy) -> SessionManagerLayer<SqliteSessionStore> {
    SessionManagerLayer::new(SqliteSessionStore::new(pool))
        .with_name(policy.name())
        .with_http_only(true)
        .with_same_site(SameSite::Strict)
        .with_secure(policy.secure())
        .with_path("/")
        .with_expiry(Expiry::OnInactivity(policy.idle_timeout()))
}
