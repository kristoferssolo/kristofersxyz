use crate::app::{
    admin::server_functions::Logout,
    content::Portfolio,
    layout::{CollapsibleSidebar, SidebarPreference},
    markdown,
};
use leptos::{form::ActionForm, prelude::*};
use leptos_router::components::A;
use lucide_leptos::{Box as BoxIcon, ChevronRight, Globe, Mail, User};

#[derive(Clone, Copy)]
pub enum EntryIcon {
    Project,
    Profile,
    Contact,
    Site,
}

#[component]
pub fn Icon(kind: EntryIcon) -> impl IntoView {
    let class = "inline-flex h-4 w-4 shrink-0 text-[#767d87] \
        group-hover:text-[#e2a340] group-aria-[current=page]:text-[#8b939d]";

    match kind {
        EntryIcon::Project => view! {
            <span class=class aria-hidden="true">
                <BoxIcon size=16 />
            </span>
        }
        .into_any(),
        EntryIcon::Profile => view! {
            <span class=class aria-hidden="true">
                <User size=16 />
            </span>
        }
        .into_any(),
        EntryIcon::Contact => view! {
            <span class=class aria-hidden="true">
                <Mail size=16 />
            </span>
        }
        .into_any(),
        EntryIcon::Site => view! {
            <span class=class aria-hidden="true">
                <Globe size=16 />
            </span>
        }
        .into_any(),
    }
}

#[component]
pub fn Affordance() -> impl IntoView {
    view! {
        <span
            class="inline-flex items-center text-[#6b7280] group-hover:text-[#e2a340]"
            aria-hidden="true"
        >
            <ChevronRight size=14 />
        </span>
    }
}

#[component]
pub fn LogoutForm() -> impl IntoView {
    let action = ServerAction::<Logout>::new();
    view! {
        <ActionForm action>
            <button
                class="mt-[1.6rem] w-auto cursor-pointer border border-[#30363d] bg-[#080a0d] px-[1.4rem] py-[.55rem] font-[inherit] text-[13px] text-white hover:border-[#e2a340]"
                type="submit"
            >
                "Sign out"
            </button>
        </ActionForm>
    }
}

#[component]
pub fn SaveButton() -> impl IntoView {
    view! {
        <button
            class="mt-[1.6rem] w-auto cursor-pointer border border-[#30363d] bg-[#080a0d] px-[1.4rem] py-[.55rem] font-[inherit] text-[13px] text-white hover:border-[#e2a340]"
            type="submit"
        >
            "Save"
        </button>
    }
}

#[component]
pub fn EditorLayout(
    active: String,
    heading: &'static str,
    breadcrumb: String,
    wide: bool,
    children: Children,
) -> impl IntoView {
    let sidebar = use_context::<SidebarPreference>().unwrap_or_default();
    let open = Signal::derive(move || sidebar.open());
    let wrap = if wide {
        "mx-auto w-full max-w-[1200px]"
    } else {
        "mx-auto w-full max-w-[760px]"
    };

    view! {
        <div class="h-dvh overflow-hidden bg-black font-mono text-[#d4d7db]">
            <AdminSidebarShortcut sidebar />
            <CollapsibleSidebar
                id="admin-navigation"
                label="admin navigation"
                width="320px"
                open
                on_toggle=Callback::new(move |()| sidebar.toggle())
                navigation=view! { <EditorNavigation active /> }.into_any()
            >
                <main class="relative flex h-full flex-col overflow-x-hidden overflow-y-auto px-[3.25rem] py-12">
                    <div class=wrap>
                        <p class="text-[10px] tracking-[.24em] text-[#767d87] uppercase">
                            {breadcrumb}
                        </p>
                        <h1 class="mt-[.7rem] font-sans text-2xl font-semibold text-white">
                            {heading}
                        </h1>
                        {children()}
                    </div>
                </main>
            </CollapsibleSidebar>
        </div>
    }
}

#[component]
fn AdminSidebarShortcut(sidebar: SidebarPreference) -> impl IntoView {
    Effect::new(move |_| {
        let handle = window_event_listener(leptos::ev::keydown, move |event| {
            if event.ctrl_key()
                && !event.alt_key()
                && !event.meta_key()
                && event.key().eq_ignore_ascii_case("b")
            {
                event.prevent_default();
                sidebar.toggle();
            }
        });
        on_cleanup(move || handle.remove());
    });
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
                    <A
                        href=href
                        attr:class="group flex items-center gap-[1.1ch] py-[.45rem] text-[13px] text-[#8b939d] no-underline hover:text-[#e2a340] aria-[current=page]:text-white"
                        attr:aria-current=current.then_some("page")
                    >
                        <Icon kind=EntryIcon::Project />
                        <span>{project.title}</span>
                    </A>
                </li>
            }
        })
        .collect_view();

    view! {
        <aside class="flex min-h-0 flex-col border-b border-[#1e2126] px-9 py-12 md:h-full md:border-r md:border-b-0">
            <p class="text-[10px] tracking-[.24em] text-[#767d87] uppercase">
                <A href="/admin" attr:class="text-inherit no-underline hover:text-[#8b939d]">
                    "Admin"
                </A>
            </p>
            <h1 class="mt-[.7rem] font-sans text-2xl font-semibold text-white">"Edit"</h1>
            <p class="mt-8 mb-[.8rem] text-[10px] tracking-[.2em] text-[#767d87] uppercase">
                "Projects"
            </p>
            <ul class="m-0 list-none p-0">{projects}</ul>
            <p class="mt-8 mb-[.8rem] text-[10px] tracking-[.2em] text-[#767d87] uppercase">
                "Site"
            </p>
            <ul class="m-0 list-none p-0">
                <NavigationLink
                    active=active.clone()
                    href="/admin/profile"
                    label="Profile"
                    icon=EntryIcon::Profile
                />
                <NavigationLink
                    active=active.clone()
                    href="/admin/contact"
                    label="Contact"
                    icon=EntryIcon::Contact
                />
                <NavigationLink
                    active
                    href="/admin/site"
                    label="Site metadata"
                    icon=EntryIcon::Site
                />
            </ul>
            <div class="mt-auto pt-10">
                <LogoutForm />
                <p class="mt-auto pt-8 text-[11px] text-[#767d87]">"kristofers.xyz"</p>
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
            <A
                href
                attr:class="group flex items-center gap-[1.1ch] py-[.45rem] text-[13px] text-[#8b939d] no-underline hover:text-[#e2a340] aria-[current=page]:text-white"
                attr:aria-current=current.then_some("page")
            >
                <Icon kind=icon />
                <span>{label}</span>
            </A>
        </li>
    }
}

#[component]
pub fn TextInput(label: &'static str, name: &'static str, value: String) -> impl IntoView {
    view! {
        <label class="mt-[1.3rem] block text-xs text-[#8b939d]">
            {label}
            <input
                class="mt-[.4rem] block w-full border border-[#2b3037] bg-[#0b0e11] px-[.65rem] py-2 font-[inherit] text-[13px] text-white focus:border-[#e2a340] focus:outline-none"
                name=name
                value=value
            />
        </label>
    }
}

#[component]
pub fn TextArea(label: &'static str, name: &'static str, value: String) -> impl IntoView {
    view! {
        <label class="mt-[1.3rem] block text-xs text-[#8b939d]">
            {label}
            <textarea
                class="mt-[.4rem] block min-h-[55vh] w-full resize-y border border-[#2b3037] bg-[#0b0e11] px-[.7rem] py-[.6rem] font-[inherit] text-[13px] leading-[1.65] text-white focus:border-[#e2a340] focus:outline-none"
                name=name
                spellcheck="false"
            >
                {value}
            </textarea>
        </label>
    }
}

#[component]
pub fn MarkdownEditor(value: String) -> impl IntoView {
    let source = RwSignal::new(value);

    view! {
        <p class="mt-[1.3rem] text-xs text-[#8b939d]">"Description (Markdown)"</p>
        <div class="mt-2 grid grid-cols-1 gap-6 min-[961px]:grid-cols-2">
            <div>
                <p class="mb-2 text-[10px] tracking-[.2em] text-[#767d87] uppercase">"Markdown"</p>
                <textarea
                    id="md"
                    class="block min-h-[55vh] w-full resize-y border border-[#2b3037] bg-[#0b0e11] px-[.7rem] py-[.6rem] font-[inherit] text-[13px] leading-[1.65] text-white focus:border-[#e2a340] focus:outline-none"
                    name="markdown"
                    prop:value=move || source.get()
                    on:input=move |event| source.set(event_target_value(&event))
                    spellcheck="false"
                ></textarea>
            </div>
            <div>
                <p class="mb-2 text-[10px] tracking-[.2em] text-[#767d87] uppercase">"Preview"</p>
                <div class="min-h-[55vh] overflow-auto border border-[#2b3037] bg-[#050607] px-[1.15rem] py-4">
                    <div
                        id="pv"
                        class="font-sans text-sm leading-[1.65] text-[#c3c9cf]
                        [&>:first-child]:mt-0
                        [&_h1]:mt-[1.4em] [&_h1]:mb-[.5em] [&_h1]:font-semibold [&_h1]:leading-[1.3] [&_h1]:text-white
                        [&_h2]:mt-[1.4em] [&_h2]:mb-[.5em] [&_h2]:font-semibold [&_h2]:leading-[1.3] [&_h2]:text-[1.1rem] [&_h2]:text-white
                        [&_h3]:mt-[1.4em] [&_h3]:mb-[.5em] [&_h3]:font-semibold [&_h3]:leading-[1.3] [&_h3]:text-base [&_h3]:text-white
                        [&_p]:my-[.8em] [&_a]:text-[#e2a340] [&_strong]:text-white
                        [&_code]:rounded-[3px] [&_code]:border [&_code]:border-[#1e2126] [&_code]:bg-[#0b0e11] [&_code]:px-[.35em] [&_code]:py-[.1em] [&_code]:font-mono [&_code]:text-[.85em]
                        [&_pre]:overflow-auto [&_pre]:rounded-sm [&_pre]:border [&_pre]:border-[#1e2126] [&_pre]:bg-[#0b0e11] [&_pre]:p-[.9rem]
                        [&_pre_code]:border-0 [&_pre_code]:bg-transparent [&_pre_code]:p-0
                        [&_ul]:my-[.8em] [&_ul]:pl-[1.4em] [&_ol]:my-[.8em] [&_ol]:pl-[1.4em] [&_li]:my-[.3em]"
                        inner_html=move || markdown::render_source(&source.get())
                    ></div>
                </div>
            </div>
        </div>
    }
}
