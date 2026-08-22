use super::site_meta::SiteMeta;
#[cfg(feature = "hydrate")]
use crate::app::content::PortfolioContent;
use crate::app::pages::{HomePage, NotFoundPage};
use leptos::prelude::*;
#[cfg(feature = "hydrate")]
use leptos::serde_json;
use leptos_meta::{Stylesheet, provide_meta_context};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

const FONTS: &str = "https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:wght@400;500;600&family=IBM+Plex+Sans:wght@400;500;600&display=swap";

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
