use super::server_functions::Logout;
use crate::app::{content::Portfolio, markdown};
use leptos::{form::ActionForm, prelude::*};
use leptos_router::components::A;

#[derive(Clone, Copy)]
pub(super) enum EntryIcon {
    Project,
    Profile,
    Contact,
    Site,
}

#[component]
pub(super) fn Icon(kind: EntryIcon) -> impl IntoView {
    match kind {
        EntryIcon::Project => view! {
            <svg class="ico" viewBox="0 0 24 24" aria-hidden="true">
                <path d="M21 8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16Z" />
                <path d="m3.3 7 8.7 5 8.7-5" />
                <path d="M12 22V12" />
            </svg>
        }.into_any(),
        EntryIcon::Profile => view! {
            <svg class="ico" viewBox="0 0 24 24" aria-hidden="true">
                <path d="M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2" />
                <circle cx="12" cy="7" r="4" />
            </svg>
        }.into_any(),
        EntryIcon::Contact => view! {
            <svg class="ico" viewBox="0 0 24 24" aria-hidden="true">
                <rect width="20" height="16" x="2" y="4" rx="2" />
                <path d="m22 7-8.97 5.7a1.94 1.94 0 0 1-2.06 0L2 7" />
            </svg>
        }.into_any(),
        EntryIcon::Site => view! {
            <svg class="ico" viewBox="0 0 24 24" aria-hidden="true">
                <circle cx="12" cy="12" r="10" />
                <path d="M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20" />
                <path d="M2 12h20" />
            </svg>
        }.into_any(),
    }
}

#[component]
pub(super) fn Affordance() -> impl IntoView {
    view! {
        <span class="edit" aria-hidden="true">
            <svg class="i" viewBox="0 0 24 24"><path d="m9 18 6-6-6-6" /></svg>
        </span>
    }
}

#[component]
pub(super) fn LogoutForm() -> impl IntoView {
    let action = ServerAction::<Logout>::new();
    view! {
        <ActionForm action>
            <button class="admin-button" type="submit">"Sign out"</button>
        </ActionForm>
    }
}

#[component]
pub(super) fn EditorLayout(
    active: String,
    heading: &'static str,
    breadcrumb: String,
    wide: bool,
    children: Children,
) -> impl IntoView {
    let wrap = if wide { "wrap" } else { "wrap narrow" };

    view! {
        <div class="dash">
            <EditorNavigation active />
            <main class="stage">
                <div class=wrap>
                    <p class="eyebrow">{breadcrumb}</p>
                    <h1 class="admin-heading">{heading}</h1>
                    {children()}
                </div>
            </main>
        </div>
    }
}

#[component]
fn EditorNavigation(active: String) -> impl IntoView {
    let content = expect_context::<Portfolio>().current();
    let projects = content
        .projects
        .into_iter()
        .map(|project| {
            let href = format!("/admin/project/{}", project.slug);
            let current = href == active;
            view! {
                <li>
                    <A href=href attr:aria-current=current.then_some("page")>
                        <Icon kind=EntryIcon::Project />
                        <span>{project.title}</span>
                    </A>
                </li>
            }
        })
        .collect_view();

    view! {
        <aside class="admin-aside">
            <p class="eyebrow"><A href="/admin">"Admin"</A></p>
            <h1 class="admin-heading">"Edit"</h1>
            <p class="grp">"Projects"</p>
            <ul class="nav">{projects}</ul>
            <p class="grp">"Site"</p>
            <ul class="nav">
                <NavigationLink active=active.clone() href="/admin/profile" label="Profile" icon=EntryIcon::Profile />
                <NavigationLink active=active.clone() href="/admin/contact" label="Contact" icon=EntryIcon::Contact />
                <NavigationLink active href="/admin/site" label="Site metadata" icon=EntryIcon::Site />
            </ul>
            <div class="bottom">
                <LogoutForm />
                <p class="foot">"kristofers.xyz"</p>
            </div>
        </aside>
    }
}

#[component]
fn NavigationLink(
    active: String,
    href: &'static str,
    label: &'static str,
    icon: EntryIcon,
) -> impl IntoView {
    let current = active == href;
    view! {
        <li>
            <A href attr:aria-current=current.then_some("page")>
                <Icon kind=icon />
                <span>{label}</span>
            </A>
        </li>
    }
}

#[component]
pub(super) fn TextInput(label: &'static str, name: &'static str, value: String) -> impl IntoView {
    view! {
        <label class="admin-label">
            {label}
            <input class="admin-input" name=name value=value />
        </label>
    }
}

#[component]
pub(super) fn TextArea(label: &'static str, name: &'static str, value: String) -> impl IntoView {
    view! {
        <label class="admin-label">
            {label}
            <textarea class="admin-textarea" name=name spellcheck="false">{value}</textarea>
        </label>
    }
}

#[component]
pub(super) fn MarkdownEditor(value: String) -> impl IntoView {
    let source = RwSignal::new(value);

    view! {
        <p class="mdlabel">"Description (Markdown)"</p>
        <div class="md">
            <div>
                <p class="panelabel">"Markdown"</p>
                <textarea
                    id="md"
                    class="admin-textarea"
                    name="markdown"
                    prop:value=move || source.get()
                    on:input=move |event| source.set(event_target_value(&event))
                    spellcheck="false"
                ></textarea>
            </div>
            <div>
                <p class="panelabel">"Preview"</p>
                <div class="preview">
                    <div id="pv" class="prose" inner_html=move || markdown::render_source(&source.get())></div>
                </div>
            </div>
        </div>
    }
}
