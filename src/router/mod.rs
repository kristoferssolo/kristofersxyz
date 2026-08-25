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

    Router::new()
        .route("/login", get(auth::login_form).post(auth::login))
        .route("/logout", post(auth::logout))
        .route("/admin", get(auth::admin))
        .route(
            "/admin/project/{slug}",
            get(auth::project_form).post(auth::edit_project),
        )
        .route(
            "/admin/profile",
            get(auth::profile_form).post(auth::edit_profile),
        )
        .route(
            "/admin/contact",
            get(auth::contact_form).post(auth::edit_contact),
        )
        .route("/admin/site", get(auth::site_form).post(auth::edit_site))
        .leptos_routes(&state, routes, {
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
fn session_layer(pool: crate::db::DbPool, secure: bool) -> SessionManagerLayer<SqliteSessionStore> {
    SessionManagerLayer::new(SqliteSessionStore::new(pool))
        .with_http_only(true)
        .with_same_site(SameSite::Strict)
        .with_secure(secure)
        .with_expiry(Expiry::OnInactivity(Duration::hours(1)))
}
