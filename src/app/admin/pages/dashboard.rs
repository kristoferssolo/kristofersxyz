use super::super::{
    components::{Affordance, EntryIcon, Icon, LogoutForm},
    server_functions::SessionUser,
};
use crate::app::content::Portfolio;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::A;

/// The authenticated landing page and its editable entries.
#[component]
pub fn DashboardPage() -> impl IntoView {
    let content = expect_context::<Portfolio>().current();
    let user = expect_context::<SessionUser>();
    let projects_count = content.projects.len();
    let pages_count = projects_count.saturating_add(2);
    let projects = content
        .projects
        .into_iter()
        .map(|project| {
            let words = project.description.as_str().split_whitespace().count();
            let links = project.links.len();
            let link_label = if links == 1 { "link" } else { "links" };
            let href = format!("/admin/project/{}", project.slug);
            let path = project.path();
            view! {
                <li>
                    <A href attr:class="group block py-5 text-inherit no-underline">
                        <div class="flex items-baseline justify-between gap-[2ch]">
                            <span class="flex items-center gap-[1.1ch]">
                                <Icon kind=EntryIcon::Project />
                                <span class="text-[15px] text-white group-hover:text-[#e2a340]">{project.title}</span>
                            </span>
                            <Affordance />
                        </div>
                        <p class="mt-2 text-[13px] leading-[1.55] text-[#8b939d]">{project.summary}</p>
                        <p class="mt-[.6rem] text-[11px] tracking-[.04em] text-[#767d87]">
                            <b class="font-normal text-[#8b939d]">{project.technologies.len()}</b>" tech · "
                            <b class="font-normal text-[#8b939d]">{links}</b>" "{link_label}" · "
                            <b class="font-normal text-[#8b939d]">{words}</b>" words · "
                            <span class="text-[#6b7280]">{path}</span>
                        </p>
                    </A>
                </li>
            }
        })
        .collect_view();

    view! {
        <Title text="Admin" />
        <div class="grid h-dvh grid-cols-[320px_1fr] overflow-hidden bg-black font-mono text-[#d4d7db]">
            <aside class="flex min-h-0 flex-col border-r border-[#1e2126] px-9 py-12">
                <p class="text-[10px] tracking-[.24em] text-[#767d87] uppercase">"Admin"</p>
                <h1 class="mt-[.7rem] font-sans text-2xl font-semibold text-white">"Signed in"</h1>
                <p class="mt-[.6rem] text-[13px] leading-[1.6] text-[#8b939d]">"Owner session. Pick an entry to edit."</p>
                <p class="mt-8 mb-[.8rem] text-[10px] tracking-[.2em] text-[#767d87] uppercase">"Session"</p>
                <dl class="m-0 grid grid-cols-[13ch_1fr] gap-x-[2ch] gap-y-[.55rem] text-[13px] [&_dt]:text-[#8b939d] [&_dd]:m-0 [&_dd]:text-[#c3c9cf]">
                    <dt>"status"</dt><dd>"active"</dd>
                    <dt>"as"</dt><dd>{user.username.to_string()}</dd>
                    <dt>"idle limit"</dt><dd>"1 hour"</dd>
                </dl>
                <p class="mt-8 mb-[.8rem] text-[10px] tracking-[.2em] text-[#767d87] uppercase">"Content"</p>
                <dl class="m-0 grid grid-cols-[13ch_1fr] gap-x-[2ch] gap-y-[.55rem] text-[13px] [&_dt]:text-[#8b939d] [&_dd]:m-0 [&_dd]:text-[#c3c9cf] [&_dd_b]:font-medium [&_dd_b]:text-[#e2a340]">
                    <dt>"store"</dt><dd>"SQLite"</dd>
                    <dt>"projects"</dt><dd><b>{projects_count}</b></dd>
                    <dt>"pages"</dt><dd><b>{pages_count}</b></dd>
                </dl>
                <div class="mt-auto pt-10">
                    <LogoutForm />
                    <p class="mt-auto pt-8 text-[11px] text-[#767d87]">"kristofers.xyz"</p>
                </div>
            </aside>
            <main class="relative flex flex-col overflow-x-hidden overflow-y-auto px-[3.25rem] py-12">
                <p class="text-[10px] tracking-[.24em] text-[#767d87] uppercase">"Projects"</p>
                <ul class="mt-[1.4rem] max-w-[720px] list-none p-0 [&>li]:border-b [&>li]:border-[#1e2126]">{projects}</ul>
                <p class="text-[10px] tracking-[.24em] text-[#767d87] uppercase">"Site"</p>
                <ul class="mt-[1.4rem] max-w-[720px] list-none p-0 [&>li]:border-b [&>li]:border-[#1e2126]">
                    <EntryLink href="/admin/profile" label="Profile" detail="name, title, summary, about, email" icon=EntryIcon::Profile />
                    <EntryLink href="/admin/contact" label="Contact" detail="name, body" icon=EntryIcon::Contact />
                    <EntryLink href="/admin/site" label="Site metadata" detail="url, title, description, OpenGraph image" icon=EntryIcon::Site />
                </ul>
            </main>
        </div>
    }
}

#[component]
fn EntryLink(
    href: &'static str,
    label: &'static str,
    detail: &'static str,
    icon: EntryIcon,
) -> impl IntoView {
    view! {
        <li>
            <A href attr:class="group block py-5 text-inherit no-underline">
                <div class="flex items-baseline justify-between gap-[2ch]">
                    <span class="flex items-center gap-[1.1ch]">
                        <Icon kind=icon />
                        <span class="text-[15px] text-white group-hover:text-[#e2a340]">{label}</span>
                    </span>
                    <Affordance />
                </div>
                <p class="mt-[.6rem] text-[11px] tracking-[.04em] text-[#767d87]">{detail}</p>
            </A>
        </li>
    }
}
