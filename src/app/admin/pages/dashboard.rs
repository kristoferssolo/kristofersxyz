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
                    <A href>
                        <div class="row">
                            <span class="name-row"><Icon kind=EntryIcon::Project /><span class="name">{project.title}</span></span>
                            <Affordance />
                        </div>
                        <p class="sum">{project.summary}</p>
                        <p class="meta">
                            <b>{project.technologies.len()}</b>" tech · "
                            <b>{links}</b>" "{link_label}" · "
                            <b>{words}</b>" words · "
                            <span class="path">{path}</span>
                        </p>
                    </A>
                </li>
            }
        })
        .collect_view();

    view! {
        <Title text="Admin" />
        <div class="dash">
            <aside class="admin-aside">
                <p class="eyebrow">"Admin"</p>
                <h1 class="admin-heading">"Signed in"</h1>
                <p class="lede">"Owner session. Pick an entry to edit."</p>
                <p class="grp">"Session"</p>
                <dl class="admin-dl">
                    <dt>"status"</dt><dd>"active"</dd>
                    <dt>"as"</dt><dd>{user.username}</dd>
                    <dt>"idle limit"</dt><dd>"1 hour"</dd>
                </dl>
                <p class="grp">"Content"</p>
                <dl class="admin-dl">
                    <dt>"store"</dt><dd>"SQLite"</dd>
                    <dt>"projects"</dt><dd><b>{projects_count}</b></dd>
                    <dt>"pages"</dt><dd><b>{pages_count}</b></dd>
                </dl>
                <div class="bottom">
                    <LogoutForm />
                    <p class="foot">"kristofers.xyz"</p>
                </div>
            </aside>
            <main class="stage">
                <p class="eyebrow">"Projects"</p>
                <ul class="projects">{projects}</ul>
                <p class="eyebrow">"Site"</p>
                <ul class="projects">
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
            <A href>
                <div class="row">
                    <span class="name-row"><Icon kind=icon /><span class="name">{label}</span></span>
                    <Affordance />
                </div>
                <p class="meta">{detail}</p>
            </A>
        </li>
    }
}
