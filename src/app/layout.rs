use leptos::prelude::*;

use crate::app::content::{PROFILE, Project, SocialLink};

#[component]
pub fn PageShell(children: Children) -> impl IntoView {
    view! {
        <div class="site-shell">
            <SiteHeader/>
            <main>{children()}</main>
            <SiteFooter/>
        </div>
    }
}

#[component]
pub fn SiteHeader() -> impl IntoView {
    view! {
        <header class="site-header">
            <a class="site-mark" href="#top">{PROFILE.name}</a>
            <nav class="site-nav" aria-label="Primary navigation">
                <a href="#projects">"Projects"</a>
                <a href="#about">"About"</a>
                <a href="#contact">"Contact"</a>
            </nav>
        </header>
    }
}

#[component]
pub fn SectionWrapper(
    id: &'static str,
    heading: &'static str,
    #[prop(optional)] class: &'static str,
    children: Children,
) -> impl IntoView {
    let class_name = if class.is_empty() {
        "section".to_owned()
    } else {
        format!("section {class}")
    };

    view! {
        <section id=id class=class_name>
            <div class="section-header">
                <h2>{heading}</h2>
            </div>
            {children()}
        </section>
    }
}

#[component]
pub fn ExternalLink(link: &'static SocialLink) -> impl IntoView {
    view! {
        <a class="external-link" href=link.href rel=link.rel>
            {link.label}
        </a>
    }
}

#[component]
pub fn ProjectCard(project: &'static Project) -> impl IntoView {
    view! {
        <article class="project-card">
            <div class="project-card__body">
                <h3>{project.name}</h3>
                <p>{project.summary}</p>
            </div>
            <ul class="tag-list" aria-label=format!("{} stack", project.name)>
                {project
                    .stack
                    .iter()
                    .map(|tag| view! { <li>{*tag}</li> })
                    .collect_view()}
            </ul>
            {(!project.links.is_empty())
                .then(|| {
                    view! {
                        <div class="project-links">
                            {project
                                .links
                                .iter()
                                .map(|link| {
                                    view! {
                                        <a href=link.href rel="noopener noreferrer">
                                            {link.label}
                                        </a>
                                    }
                                })
                                .collect_view()}
                        </div>
                    }
                })}
        </article>
    }
}

#[component]
pub fn SiteFooter() -> impl IntoView {
    view! {
        <footer id="contact" class="site-footer">
            <div>
                <h2>"Contact"</h2>
                <a href=PROFILE.email>"dev@kristofers.xyz"</a>
            </div>
            <nav aria-label="Public profiles">
                {PROFILE
                    .links
                    .iter()
                    .map(|link| view! { <ExternalLink link/> })
                    .collect_view()}
            </nav>
            <p>"© 2026 Kristofers Solo"</p>
        </footer>
    }
}
