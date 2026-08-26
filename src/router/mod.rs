use crate::{
    app::{App, content::server_content, shell},
    db::DbPool,
    sessions::SqliteSessionStore,
    startup::AppState,
};
use axum::Router;
use leptos_axum::{LeptosRoutes, file_and_error_handler, generate_route_list};
use time::Duration;
use tower_sessions::{Expiry, SessionManagerLayer, cookie::SameSite};

pub fn route(state: AppState) -> Router {
    let routes = generate_route_list(App);

    Router::new()
        .leptos_routes_with_context(&state, routes, || {}, {
            let leptos_options = state.leptos_options.clone();
            move || shell(leptos_options.clone(), server_content().as_ref())
        })
        .fallback(file_and_error_handler::<AppState, _>(|options| {
            shell(options, server_content().as_ref())
        }))
        .layer(session_layer(state.pool.clone(), state.secure_cookie))
        .with_state(state)
}

/// The server-side session middleware, backed by the shared SQLite pool.
///
/// `secure` controls the cookie's `Secure` attribute. Production defaults to
/// true; local HTTP development can disable it.
fn session_layer(pool: DbPool, secure: bool) -> SessionManagerLayer<SqliteSessionStore> {
    SessionManagerLayer::new(SqliteSessionStore::new(pool))
        .with_http_only(true)
        .with_same_site(SameSite::Strict)
        .with_secure(secure)
        .with_expiry(Expiry::OnInactivity(Duration::hours(1)))
}
