use leptos::prelude::*;

use crate::app::{
    content::{PROFILE, PROJECTS},
    layout::{ExternalLink, PageShell, ProjectCard, SectionWrapper},
};

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <PageShell>
            <section id="top" class="hero" aria-labelledby="hero-title">
                <p class="eyebrow">"Portfolio"</p>
                <h1 id="hero-title">{PROFILE.name}</h1>
                <p class="hero__title">{PROFILE.title}</p>
                <p class="hero__summary">{PROFILE.summary}</p>
                <div class="hero__links" aria-label="Primary links">
                    {PROFILE
                        .links
                        .iter()
                        .map(|link| view! { <ExternalLink link/> })
                        .collect_view()}
                </div>
            </section>

            <SectionWrapper id="projects" heading="Selected Work">
                <div class="project-grid">
                    {PROJECTS
                        .iter()
                        .map(|project| view! { <ProjectCard project/> })
                        .collect_view()}
                </div>
            </SectionWrapper>

            <SectionWrapper id="about" heading="About" class="section--text">
                <p>
                    "I focus on Rust and web systems where correctness, maintainability, and clear operational behavior matter. My preferred work is close to the boundary between product needs and engineering infrastructure: APIs, server-rendered applications, developer tools, and deployment surfaces that stay understandable over time."
                </p>
                <p>
                    "The common thread is practical software: typed interfaces where they help, tests that protect important behavior, and simple architecture that leaves room for future change."
                </p>
            </SectionWrapper>

            <SectionWrapper id="working-style" heading="Working Style">
                <ul class="focus-list">
                    <li>
                        <strong>"Rust web services"</strong>
                        <span>"Backend systems with explicit data flow and predictable runtime behavior."</span>
                    </li>
                    <li>
                        <strong>"Typed interfaces"</strong>
                        <span>"Small contracts that make invalid states harder to express."</span>
                    </li>
                    <li>
                        <strong>"Pragmatic testing"</strong>
                        <span>"Coverage aimed at behavior, integrations, and regression-prone edges."</span>
                    </li>
                    <li>
                        <strong>"Maintainable deployment surfaces"</strong>
                        <span>"Operational choices that are easy to inspect, document, and repeat."</span>
                    </li>
                </ul>
            </SectionWrapper>
        </PageShell>
    }
}

#[component]
pub fn NotFoundPage() -> impl IntoView {
    view! {
        <PageShell>
            <section class="section not-found">
                <h1>"Page not found."</h1>
                <p>"The portfolio currently has a single homepage."</p>
                <a href="/">"Return home"</a>
            </section>
        </PageShell>
    }
}
