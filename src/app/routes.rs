use crate::app::{
    content::portfolio_content,
    pages::{HomePage, NotFoundPage},
};
use leptos::prelude::*;
use leptos_meta::{Link, Meta, MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

const FONTS: &str = "https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:wght@400;500;600&family=IBM+Plex+Sans:wght@400;500;600&display=swap";

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

/// Title, description, canonical URL and Open Graph tags. One route, so one
/// set of tags; per page metadata arrives with the first real subpage.
#[component]
fn SiteMeta() -> impl IntoView {
    let site = portfolio_content().site;

    view! {
        <Title text=site.title.clone() />
        <Meta name="description" content=site.description.clone() />
        <Link rel="canonical" href=site.url.clone() />
        <Meta property="og:type" content="website" />
        <Meta property="og:url" content=site.url />
        <Meta property="og:title" content=site.title />
        <Meta property="og:description" content=site.description />
        <Meta property="og:image" content=site.og_image />
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="fonts" href=FONTS />
        <Stylesheet id="leptos" href="/pkg/kristofersxyz.css" />
        <SiteMeta />
        <Router>
            <Routes fallback=|| view! { <NotFoundPage /> }.into_view()>
                <Route path=path!("/") view=HomePage />
            </Routes>
        </Router>
    }
}
