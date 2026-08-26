use super::site_meta::SiteMeta;
#[cfg(any(feature = "ssr", feature = "hydrate"))]
use crate::app::content::Portfolio;
#[cfg(feature = "hydrate")]
use crate::app::content::PortfolioContent;
use crate::app::{
    admin::{
        AuthenticatedAdmin, ContactEditorPage, DashboardPage, LoginPage, ProfileEditorPage,
        ProjectEditorPage, SiteEditorPage,
    },
    layout::SidebarPreference,
    pages::{HomePage, NotFoundPage, ProjectPage},
};
use leptos::prelude::*;
#[cfg(feature = "hydrate")]
use leptos::serde_json;
use leptos_meta::{Stylesheet, provide_meta_context};
use leptos_router::{
    SsrMode,
    components::{ParentRoute, Route, Router, Routes},
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

    let Ok(content) = serde_json::from_str(&json) else {
        std::process::abort();
    };
    content
}

/// Provides portfolio and sidebar state to every route.
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    #[cfg(all(feature = "ssr", not(feature = "hydrate")))]
    provide_context(Portfolio::new(
        crate::app::content::server_content().as_ref().clone(),
    ));
    #[cfg(feature = "hydrate")]
    provide_context(Portfolio::new(embedded_content()));

    provide_context(SidebarPreference::default());

    view! {
        <Stylesheet id="fonts" href=FONTS />
        <Stylesheet id="leptos" href="/pkg/kristofersxyz.css" />
        <Router>
            <SiteMeta />
            <Routes fallback=|| view! { <NotFoundPage /> }.into_view()>
                <Route path=path!("/") view=HomePage />
                <Route path=path!("/work/:slug") view=ProjectPage />
                <Route path=path!("/login") view=LoginPage />
                <ParentRoute path=path!("/admin") view=AuthenticatedAdmin ssr=SsrMode::Async>
                    <Route path=path!("") view=DashboardPage />
                    <Route path=path!("project/:slug") view=ProjectEditorPage />
                    <Route path=path!("profile") view=ProfileEditorPage />
                    <Route path=path!("contact") view=ContactEditorPage />
                    <Route path=path!("site") view=SiteEditorPage />
                </ParentRoute>
            </Routes>
        </Router>
    }
}
