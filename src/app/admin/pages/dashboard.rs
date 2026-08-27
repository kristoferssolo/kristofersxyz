use super::super::{
    components::{Affordance, EntryIcon, Icon, InfoList, InfoRow, LogoutForm},
    server_functions::SessionUser,
};
use crate::app::{content::Portfolio, layout::CommandShell};
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
                <EntryLink href label=project.title icon=EntryIcon::Project>
                    <p class="mt-2 text-[13px] leading-[1.55] text-[#8b939d]">{project.summary}</p>
                    <p class="mt-[.6rem] text-[11px] tracking-[.04em] text-[#767d87]">
                        <b class="font-normal text-[#8b939d]">{project.technologies.len()}</b>" tech · "
                        <b class="font-normal text-[#8b939d]">{links}</b>" "{link_label}" · "
                        <b class="font-normal text-[#8b939d]">{words}</b>" words · "
                        <span class="text-[#6b7280]">{path}</span>
                    </p>
                </EntryLink>
            }
        })
        .collect_view();

    view! {
        <Title text="Admin" />
        <CommandShell filename="admin">
        <div class="grid h-full grid-cols-[320px_1fr] overflow-hidden bg-black font-mono text-[#d4d7db]">
            <aside class="flex min-h-0 flex-col overflow-y-auto border-r border-[#1e2126] px-9 py-12">
                <p class="text-[10px] tracking-[.24em] text-[#767d87] uppercase">"Admin"</p>
                <h1 class="mt-[.7rem] font-sans text-2xl font-semibold text-white">"Signed in"</h1>
                <p class="mt-[.6rem] text-[13px] leading-[1.6] text-[#8b939d]">"Owner session. Pick an entry to edit."</p>
                <p class="mt-8 mb-[.8rem] text-[10px] tracking-[.2em] text-[#767d87] uppercase">"Session"</p>
                <InfoList
                    rows=vec![
                        InfoRow { label: "status", value: "active".to_owned(), emphasized: false },
                        InfoRow { label: "as", value: user.username.to_string(), emphasized: false },
                        InfoRow { label: "idle limit", value: "1 hour".to_owned(), emphasized: false },
                    ]
                />
                <p class="mt-8 mb-[.8rem] text-[10px] tracking-[.2em] text-[#767d87] uppercase">"Content"</p>
                <InfoList
                    rows=vec![
                        InfoRow { label: "store", value: "SQLite".to_owned(), emphasized: false },
                        InfoRow { label: "projects", value: projects_count.to_string(), emphasized: true },
                        InfoRow { label: "pages", value: pages_count.to_string(), emphasized: true },
                    ]
                />
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
                    <EntryLink href="/admin/profile".to_owned() label="Profile".to_owned() icon=EntryIcon::Profile>
                        <p class="mt-[.6rem] text-[11px] tracking-[.04em] text-[#767d87]">"name, title, summary, about, email"</p>
                    </EntryLink>
                    <EntryLink href="/admin/contact".to_owned() label="Contact".to_owned() icon=EntryIcon::Contact>
                        <p class="mt-[.6rem] text-[11px] tracking-[.04em] text-[#767d87]">"name, body"</p>
                    </EntryLink>
                    <EntryLink href="/admin/site".to_owned() label="Site metadata".to_owned() icon=EntryIcon::Site>
                        <p class="mt-[.6rem] text-[11px] tracking-[.04em] text-[#767d87]">"url, title, description, OpenGraph image"</p>
                    </EntryLink>
                </ul>
            </main>
        </div>
        </CommandShell>
    }
}

#[component]
pub fn EntryLink(
    href: String,
    label: String,
    icon: EntryIcon,
    children: Children,
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
                {children()}
            </A>
        </li>
    }
}
