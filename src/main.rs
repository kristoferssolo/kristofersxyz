#[cfg(feature = "ssr")]
#[allow(clippy::unwrap_used)]
#[tokio::main]
async fn main() {
    use axum::Router;
    use kristofersxyz::{
        app::{App, shell},
        configuration::get_settings,
        state::build_app_state,
    };
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{LeptosRoutes, file_and_error_handler_with_context, generate_route_list};

    let conf = get_configuration(None).unwrap();
    let settings = get_settings().unwrap();
    let addr = conf.leptos_options.site_addr;
    let state = build_app_state(conf.leptos_options, &settings.database)
        .await
        .unwrap();
    let leptos_options = state.leptos_options.clone();
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);
    let render_pool = state.pool.clone();
    let fallback_pool = state.pool.clone();

    let app = Router::new()
        .leptos_routes_with_context(
            &state,
            routes,
            move || provide_context(render_pool.clone()),
            {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
        .fallback(file_and_error_handler_with_context::<
            kristofersxyz::state::AppState,
            _,
        >(
            move || provide_context(fallback_pool.clone()), shell
        ))
        .with_state(state);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
