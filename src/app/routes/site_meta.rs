use crate::app::content::PortfolioContent;
use leptos::prelude::*;
use leptos_meta::{Link, Meta, Title};

/// Title, description, canonical URL and Open Graph tags. One route, so one
/// set of tags; per page metadata arrives with the first real subpage.
#[component]
pub(super) fn SiteMeta() -> impl IntoView {
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
