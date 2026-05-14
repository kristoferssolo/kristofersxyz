use leptos::prelude::*;

use crate::app::content::{FocusArea, PortfolioContent, Profile, Project, get_portfolio_content};

#[component]
pub fn HomePage() -> impl IntoView {
    let portfolio = Resource::new(|| (), |()| async { get_portfolio_content().await });

    view! {
        <Suspense fallback=TerminalLoadingPage>
            {move || Suspend::new(async move {
                portfolio.await.map_or_else(
                    |_| view! { <TerminalErrorPage/> }.into_any(),
                    |content| view! { <PortfolioPage content/> }.into_any(),
                )
            })}
        </Suspense>
    }
}

#[component]
pub fn NotFoundPage() -> impl IntoView {
    view! {
        <main class="grid min-h-screen place-items-center bg-[#0b0d10] px-5 font-mono text-[#d8dee9]">
            <section class="max-w-md">
                <p class="text-sm text-[#6ea3bd]">"$ cd /"</p>
                <h1 class="mt-3 text-2xl text-[#eef2f7]">"Page not found."</h1>
                <p class="mt-3 text-[#9da9b8]">"This portfolio currently lives at the root URL only."</p>
                <a class="mt-5 inline-block text-[#8db7cc] no-underline hover:text-[#f3c677]" href="/">
                    "Return home"
                </a>
            </section>
        </main>
    }
}

#[component]
fn PortfolioPage(content: PortfolioContent) -> impl IntoView {
    let PortfolioContent {
        profile,
        projects,
        working_style,
    } = content;

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
                    <a class="text-[#c8d1dc] no-underline" href="#top">"~/kristoferssolo/portfolio"</a>
                </header>

                <section id="top" class="py-16 sm:py-24">
                    <p class="text-sm text-[#6ea3bd]">"$ whoami"</p>
                    <h1 class="mt-4 text-4xl leading-tight text-[#eef2f7] sm:text-6xl">
                        {profile.name.clone()}
                    </h1>
                    <p class="mt-6 max-w-3xl text-lg leading-8 text-[#c7d0da]">
                        {profile.title.clone()}
                    </p>
                    <div class="mt-8 max-w-3xl text-base leading-8 text-[#9da9b8]">
                        <span class="text-[#f3c677]">"summary: "</span>
                        <span>{profile.summary.clone()}</span>
                    </div>
                    <div class="mt-10 flex flex-wrap gap-x-5 gap-y-2 text-sm">
                        <ProfileLinks profile=profile.clone()/>
                    </div>
                </section>

                <section id="projects" class="py-12">
                    <p class="text-sm text-[#6ea3bd]">"$ ls selected-work"</p>
                    <h2 class="mt-3 text-2xl text-[#eef2f7]">"Selected Work"</h2>
                    <div class="mt-8 space-y-5">
                        {projects
                            .into_iter()
                            .map(|project| view! { <TerminalProject project/> })
                            .collect_view()}
                    </div>
                </section>

                <section id="about" class="grid gap-6 py-12 md:grid-cols-[180px_1fr]">
                    <h2 class="text-xl text-[#eef2f7]">"about.md"</h2>
                    <p class="leading-8 text-[#c7d0da]">{profile.about.clone()}</p>
                </section>

                <section class="py-12">
                    <p class="text-sm text-[#6ea3bd]">"$ cat working-style.txt"</p>
                    <ul class="mt-6 space-y-3">
                        {working_style
                            .into_iter()
                            .map(|focus| view! { <FocusAreaRow focus/> })
                            .collect_view()}
                    </ul>
                </section>

                <footer id="contact" class="py-8">
                    <TerminalFooterLinks profile/>
                </footer>
            </div>
        </main>
    }
}

#[component]
fn TerminalLoadingPage() -> impl IntoView {
    view! {
        <main class="grid min-h-screen place-items-center bg-[#0b0d10] px-5 font-mono text-[#d8dee9]">
            <div class="max-w-md">
                <p class="text-sm text-[#6ea3bd]">"$ sqlite3 data/portfolio.db"</p>
                <p class="mt-3 text-[#9da9b8]">"Loading portfolio content…"</p>
            </div>
        </main>
    }
}

#[component]
fn TerminalErrorPage() -> impl IntoView {
    view! {
        <main class="grid min-h-screen place-items-center bg-[#0b0d10] px-5 font-mono text-[#d8dee9]">
            <div class="max-w-md">
                <p class="text-sm text-[#6ea3bd]">"$ sqlite3 data/portfolio.db"</p>
                <h1 class="mt-3 text-2xl text-[#eef2f7]">"Content unavailable."</h1>
                <p class="mt-3 text-[#9da9b8]">"The portfolio database could not be read."</p>
            </div>
        </main>
    }
}

#[component]
fn ProfileLinks(profile: Profile) -> impl IntoView {
    view! {
        {profile
            .links
            .into_iter()
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
fn TerminalProject(project: Project) -> impl IntoView {
    view! {
        <article class="grid gap-3 border-b border-[#20262d] pb-5 last:border-b-0">
            <div class="flex flex-wrap items-baseline gap-x-3 gap-y-1">
                <span class="text-[#6ea3bd]">"+"</span>
                <h3 class="text-lg text-[#eef2f7]">{project.name.clone()}</h3>
            </div>
            <p class="pl-5 leading-7 text-[#b7c1cc]">{project.summary.clone()}</p>
            <div class="space-y-3 pl-5">
                <p class="text-xs text-[#7f8b99]">
                    {project
                        .stack
                        .iter()
                        .enumerate()
                        .map(|(index, tag)| {
                            if index + 1 == project.stack.len() {
                                tag.clone()
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
                                    .into_iter()
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
fn FocusAreaRow(focus: FocusArea) -> impl IntoView {
    view! {
        <li class="grid gap-1 md:grid-cols-[220px_1fr]">
            <strong class="text-[#f3c677]">{focus.label}</strong>
            <span class="text-[#aeb8c4]">{focus.detail}</span>
        </li>
    }
}

#[component]
fn TerminalFooterLinks(profile: Profile) -> impl IntoView {
    let email_href = profile.email.clone();
    let email_label = email_href.trim_start_matches("mailto:").to_owned();

    view! {
        <div class="flex flex-wrap items-center gap-x-4 gap-y-2 text-sm text-[#8f9baa]">
            <span class="text-[#6ea3bd]">"$ open"</span>
            <a class="text-[#8db7cc] no-underline hover:text-[#f3c677]" href=email_href>
                {email_label}
            </a>
            {profile
                .links
                .into_iter()
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
