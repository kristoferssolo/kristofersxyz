use crate::app::content::PortfolioContent;
use leptos::prelude::*;
use leptos_meta::{Link, Meta, Title};
use leptos_router::hooks::use_location;

/// Route-aware title, description, canonical URL, and Open Graph tags.
#[component]
pub(super) fn SiteMeta() -> impl IntoView {
    let content = expect_context::<PortfolioContent>();
    let pathname = use_location().pathname;

    move || {
        let path = pathname.get();
        let project = content
            .projects
            .iter()
            .find(|project| project.path() == path);
        let title = project.map_or_else(
            || content.site.title.clone(),
            |project| format!("{} | Kristofers Solo", project.title),
        );
        let description = project.map_or_else(
            || content.site.description.clone(),
            |project| project.summary.clone(),
        );
        let url = project.map_or_else(
            || content.site.url.clone(),
            |project| format!("{}work/{}", content.site.url, project.slug),
        );
        let page_type = if project.is_some() {
            "article"
        } else {
            "website"
        };

        view! {
            <Title text=title.clone() />
            <Meta name="description" content=description.clone() />
            <Link rel="canonical" href=url.clone() />
            <Meta property="og:type" content=page_type />
            <Meta property="og:url" content=url />
            <Meta property="og:title" content=title />
            <Meta property="og:description" content=description />
            <Meta property="og:image" content=content.site.og_image.clone() />
        }
    }
}
