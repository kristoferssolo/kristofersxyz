use crate::app::{
    content::PortfolioContent,
    pages::{HomePage, NotFoundPage},
};
use leptos::prelude::*;
use leptos::serde_json;
use leptos_meta::{Link, Meta, MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

const FONTS: &str = "https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:wght@400;500;600&family=IBM+Plex+Sans:wght@400;500;600&display=swap";

// `use<>` keeps the returned view from capturing the `content` borrow: it is
// serialized into an owned string here and never referenced afterwards.
#[must_use]
pub fn shell(options: LeptosOptions, content: &PortfolioContent) -> impl IntoView + use<> {
    // The content the client reads back into context before hydrating, so both
    // sides render from identical values. It lives in the head, which
    // hydrate_body does not walk, so it never reads as a hydration mismatch.
    let content = serde_json::to_string(content).unwrap_or_default();
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
                <script id="portfolio-content" type="application/json" inner_html=content></script>
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
    let site = expect_context::<PortfolioContent>().site;

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

/// Reads the portfolio the shell embedded in the page back into context, so
/// hydration renders from the same values the server did.
#[cfg(feature = "hydrate")]
fn embedded_content() -> PortfolioContent {
    use leptos::web_sys;

    let json = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id("portfolio-content"))
        .and_then(|element| element.text_content())
        .unwrap_or_default();

    serde_json::from_str(&json).expect("the embedded portfolio content is valid JSON")
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    // Put the portfolio into context so the pages read one small type. The
    // source differs by side: the boot-loaded singleton on the server, the
    // embedded JSON on the client, which renders identical values.
    #[cfg(feature = "ssr")]
    provide_context(crate::app::content::server_content());
    #[cfg(feature = "hydrate")]
    provide_context(embedded_content());

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
