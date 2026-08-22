use crate::{
    app::{App, shell},
    startup::AppState,
};
use axum::Router;
use leptos_axum::{LeptosRoutes, file_and_error_handler, generate_route_list};

pub fn route(state: AppState) -> Router {
    let routes = generate_route_list(App);
    let content = state.content.clone();

    // The shell serves both the page routes and the 404 fallback. It provides
    // the portfolio into context for the server render and serializes it into
    // the page for the client to hydrate from.
    Router::new()
        .leptos_routes(&state, routes, {
            let leptos_options = state.leptos_options.clone();
            let content = content.clone();
            move || shell(leptos_options.clone(), &content)
        })
        .fallback(file_and_error_handler::<AppState, _>(move |options| {
            shell(options, &content)
        }))
        .with_state(state)
}
