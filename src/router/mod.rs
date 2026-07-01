use crate::{
    app::{App, shell},
    startup::AppState,
};
use axum::Router;
use leptos_axum::{LeptosRoutes, file_and_error_handler, generate_route_list};

pub fn route(state: AppState) -> Router {
    let routes = generate_route_list(App);

    Router::new()
        .leptos_routes(&state, routes, {
            let leptos_options = state.leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(file_and_error_handler::<AppState, _>(shell))
        .with_state(state)
}
