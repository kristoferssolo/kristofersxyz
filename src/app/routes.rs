use crate::app::pages::{HomePage, NotFoundPage};
use leptos::prelude::*;
use leptos_meta::{Meta, MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

#[must_use]
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/kristofersxyz.css" />
        <Title text="Kristofers Solo" />
        <Meta
            name="description"
            content="Personal portfolio of Kristofers Solo, a Rust-focused software developer."
        />
        <Router>
            <Routes fallback=|| view! { <NotFoundPage /> }.into_view()>
                <Route path=path!("/") view=HomePage />
            </Routes>
        </Router>
    }
}
