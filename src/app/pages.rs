use crate::app::content::{PROFILE, PROJECTS, Project};
use leptos::prelude::*;

const ABOUT_COPY: &str = "I focus on Rust and web systems where correctness, maintainability, and clear operational behavior matter. My preferred work is close to the boundary between product needs and engineering infrastructure: APIs, server-rendered applications, developer tools, and deployment surfaces that stay understandable over time.";

const WORKING_STYLE: &[(&str, &str)] = &[
    (
        "Rust web services",
        "Backend systems with explicit data flow and predictable runtime behavior.",
    ),
    (
        "Typed interfaces",
        "Small contracts that make invalid states harder to express.",
    ),
    (
        "Pragmatic testing",
        "Coverage aimed at behavior, integrations, and regression-prone edges.",
    ),
    (
        "Maintainable deployment surfaces",
        "Operational choices that are easy to inspect, document, and repeat.",
    ),
];

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <main class="relative min-h-screen overflow-hidden bg-[#0b0d10] font-mono text-[#d8dee9]">
            <div
                class="pointer-events-none absolute inset-0 opacity-30"
                style="background-image: linear-gradient(rgba(130, 149, 167, 0.08) 1px, transparent 1px); background-size: 100% 1.6rem;"
            ></div>
            <div
                class="pointer-events-none absolute inset-0 opacity-50"
                style="background: radial-gradient(circle at top right, rgba(83, 161, 191, 0.16), transparent 32%), radial-gradient(circle at left center, rgba(191, 155, 83, 0.12), transparent 28%);"
            ></div>
            <div class="relative mx-auto max-w-5xl px-5 py-6 sm:px-7">
                <header class="flex flex-wrap items-center justify-between gap-4 pb-4 text-sm text-[#8f9baa]">
                    <a class="text-[#c8d1dc] no-underline" href="#top">
                        "~/kristoferssolo/portfolio"
                    </a>
                </header>

                <section id="top" class="py-16 sm:py-24">
                    <p class="text-sm text-[#6ea3bd]">"$ whoami"</p>
                    <h1 class="mt-4 text-4xl leading-tight text-[#eef2f7] sm:text-6xl">
                        {PROFILE.name}
                    </h1>
                    <p class="mt-6 max-w-3xl text-lg leading-8 text-[#c7d0da]">{PROFILE.title}</p>
                    <div class="mt-8 max-w-3xl text-base leading-8 text-[#9da9b8]">
                        <span class="text-[#f3c677]">"summary: "</span>
                        <span>{PROFILE.summary}</span>
                    </div>
                    <div class="mt-10 flex flex-wrap gap-x-5 gap-y-2 text-sm">
                        <ProfileLinks />
                    </div>
                </section>

                <section id="projects" class="py-12">
                    <p class="text-sm text-[#6ea3bd]">"$ ls selected-work"</p>
                    <h2 class="mt-3 text-2xl text-[#eef2f7]">"Selected Work"</h2>
                    <div class="mt-8 space-y-5">
                        {PROJECTS
                            .iter()
                            .map(|project| view! { <TerminalProject project /> })
                            .collect_view()}
                    </div>
                </section>

                <section id="about" class="grid gap-6 py-12 md:grid-cols-[180px_1fr]">
                    <h2 class="text-xl text-[#eef2f7]">"about.md"</h2>
                    <p class="leading-8 text-[#c7d0da]">{ABOUT_COPY}</p>
                </section>

                <section class="py-12">
                    <p class="text-sm text-[#6ea3bd]">"$ cat working-style.txt"</p>
                    <ul class="mt-6 space-y-3">
                        {WORKING_STYLE
                            .iter()
                            .map(|(label, detail)| {
                                view! {
                                    <li class="grid gap-1 md:grid-cols-[220px_1fr]">
                                        <strong class="text-[#f3c677]">{*label}</strong>
                                        <span class="text-[#aeb8c4]">{*detail}</span>
                                    </li>
                                }
                            })
                            .collect_view()}
                    </ul>
                </section>

                <footer id="contact" class="py-8">
                    <TerminalFooterLinks />
                </footer>
            </div>
        </main>
    }
}

#[component]
pub fn NotFoundPage() -> impl IntoView {
    view! {
        <main class="grid min-h-screen place-items-center bg-[#0b0d10] px-5 font-mono text-[#d8dee9]">
            <section class="max-w-md">
                <p class="text-sm text-[#6ea3bd]">"$ cd /"</p>
                <h1 class="mt-3 text-2xl text-[#eef2f7]">"Page not found."</h1>
                <p class="mt-3 text-[#9da9b8]">
                    "This portfolio currently lives at the root URL only."
                </p>
                <a
                    class="mt-5 inline-block text-[#8db7cc] no-underline hover:text-[#f3c677]"
                    href="/"
                >
                    "Return home"
                </a>
            </section>
        </main>
    }
}

#[component]
fn ProfileLinks() -> impl IntoView {
    view! {
        {PROFILE
            .links
            .iter()
            .map(|link| {
                view! {
                    <a
                        class="text-[#8db7cc] no-underline hover:text-[#f3c677]"
                        href=link.href
                        rel=link.rel
                    >
                        {link.label}
                    </a>
                }
            })
            .collect_view()}
    }
}

#[component]
fn TerminalProject(project: &'static Project) -> impl IntoView {
    view! {
        <article class="grid gap-3 border-b border-[#20262d] pb-5 last:border-b-0">
            <div class="flex flex-wrap items-baseline gap-x-3 gap-y-1">
                <span class="text-[#6ea3bd]">"+"</span>
                <h3 class="text-lg text-[#eef2f7]">{project.name}</h3>
            </div>
            <p class="pl-5 leading-7 text-[#b7c1cc]">{project.summary}</p>
            <div class="space-y-3 pl-5">
                <p class="text-xs text-[#7f8b99]">
                    {project
                        .stack
                        .iter()
                        .enumerate()
                        .map(|(index, tag)| {
                            if index + 1 == project.stack.len() {
                                (*tag).to_owned()
                            } else {
                                format!("{tag} · ")
                            }
                        })
                        .collect::<String>()}
                </p>
                {(!project.links.is_empty())
                    .then(|| {
                        view! {
                            <div class="flex flex-wrap gap-x-4 gap-y-1 text-sm">
                                {project
                                    .links
                                    .iter()
                                    .map(|link| {
                                        view! {
                                            <a
                                                class="text-[#8db7cc] no-underline hover:text-[#f3c677]"
                                                href=link.href
                                                rel="noopener noreferrer"
                                            >
                                                {link.label}
                                            </a>
                                        }
                                    })
                                    .collect_view()}
                            </div>
                        }
                    })}
            </div>
        </article>
    }
}

#[component]
fn TerminalFooterLinks() -> impl IntoView {
    view! {
        <div class="flex flex-wrap items-center gap-x-4 gap-y-2 text-sm text-[#8f9baa]">
            <span class="text-[#6ea3bd]">"$ open"</span>
            <a class="text-[#8db7cc] no-underline hover:text-[#f3c677]" href=PROFILE.email>
                "dev@kristofers.xyz"
            </a>
            {PROFILE
                .links
                .iter()
                .filter(|link| link.label != "Email")
                .map(|link| {
                    view! {
                        <a
                            class="text-[#8db7cc] no-underline hover:text-[#f3c677]"
                            href=link.href
                            rel=link.rel
                        >
                            {link.label}
                        </a>
                    }
                })
                .collect_view()}
            <span class="text-[#6b7480]">"© 2026 Kristofers Solo"</span>
        </div>
    }
}
