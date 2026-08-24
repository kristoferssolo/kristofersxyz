mod auth;

use crate::{
    app::{App, content::server_content, shell},
    sessions::SqliteSessionStore,
    startup::AppState,
};
use axum::{
    Router,
    routing::{get, post},
};
use leptos_axum::{LeptosRoutes, file_and_error_handler, generate_route_list};
use time::Duration;
use tower_sessions::{Expiry, SessionManagerLayer, cookie::SameSite};

pub fn route(state: AppState) -> Router {
    let routes = generate_route_list(App);

    // The shell serves both the page routes and the 404 fallback. Each render
    // reads the current portfolio rather than a snapshot taken here, so a
    // content edit shows on the next request without a restart. The admin
    // routes sit alongside it, ahead of the Leptos fallback.
    Router::new()
        .route("/login", get(auth::login_form).post(auth::login))
        .route("/logout", post(auth::logout))
        .route("/admin", get(auth::admin))
        .leptos_routes(&state, routes, {
            let leptos_options = state.leptos_options.clone();
            move || shell(leptos_options.clone(), server_content().as_ref())
        })
        .fallback(file_and_error_handler::<AppState, _>(|options| {
            shell(options, server_content().as_ref())
        }))
        .layer(session_layer(state.pool.clone()))
        .with_state(state)
}

/// The server-side session middleware, backed by the shared SQLite pool.
///
/// `with_secure(false)` lets the cookie ride local HTTP during development; a
/// production deploy behind TLS should turn it on, which the login work will
/// make configurable.
fn session_layer(pool: crate::db::DbPool) -> SessionManagerLayer<SqliteSessionStore> {
    SessionManagerLayer::new(SqliteSessionStore::new(pool))
        .with_http_only(true)
        .with_same_site(SameSite::Strict)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::hours(1)))
}
