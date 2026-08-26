use crate::app::{
    content::{Portfolio, PortfolioContent},
    editor::{Buffer, EntryId, SectionId},
    editor_controller::EditorController,
};
use leptos::prelude::*;
use leptos_router::components::A;
use lucide_leptos::PanelLeft;

#[derive(Clone)]
struct NavigationEntry {
    id: EntryId,
    name: String,
    href: String,
}

/// The navigation's identity, shared with the toggle's `aria-controls`.
const NAVIGATION_ID: &str = "portfolio-navigation";

/// Portfolio navigation. On wide screens it occupies the same place a site
/// header would, while the selected page scrolls beside it. Collapsed, it
/// stays in the document so the toggle keeps naming something real.
///
/// `visibility` is in the collapse transition, so the entries stay painted
/// through the fade and leave the tab order only once it ends.
#[component]
pub fn Sidebar(
    #[prop(into)] active: Signal<EntryId>,
    #[prop(into)] visible: Signal<bool>,
    on_select: Option<Callback<EntryId>>,
) -> impl IntoView {
    let content = expect_context::<Portfolio>().current();
    let groups = navigation(&content);

    view! {
        <nav
            id=NAVIGATION_ID
            aria-label="Portfolio"
            class=move || {
                if visible.get() {
                    "border-b border-[#1e2126] pt-12 pb-3 text-[13px] transition-[opacity,visibility] duration-150 ease-out md:h-full md:min-h-0 md:visible md:overflow-x-hidden md:overflow-y-auto md:border-r md:border-b-0 md:opacity-100"
                } else {
                    "hidden border-b border-[#1e2126] pt-12 pb-3 text-[13px] transition-[opacity,visibility] duration-150 ease-out md:invisible md:block md:h-full md:min-h-0 md:overflow-x-hidden md:overflow-y-auto md:border-r md:border-b-0 md:opacity-0"
                }
            }
        >
            {groups
                .into_iter()
                .map(|(section, entries)| {
                    view! {
                        <div class="mt-4 first:mt-0">
                            <p
                                class="mb-1 pl-[7ch] text-[10px] tracking-[0.24em] whitespace-nowrap text-[#4c525a] uppercase"
                            >
                                {section.label()}
                            </p>
                            {entries
                                .into_iter()
                                .enumerate()
                                .map(|(index, entry)| {
                                    let id = entry.id.clone();
                                    let selected_id = entry.id.clone();
                                    let is_active = Memo::new(move |_| active.get() == selected_id);

                                    view! {
                                        <A
                                            href=entry.href
                                            attr:id=format!("buffer-{}", entry.id.fragment())
                                            attr:aria-current=move || {
                                                is_active.get().then_some("page")
                                            }
                                            on:click=move |_| {
                                                if let Some(callback) = on_select
                                                    && !matches!(id, EntryId::Project(_))
                                                {
                                                    callback.run(id.clone());
                                                }
                                            }
                                            attr:class=move || {
                                                if is_active.get() {
                                                    "flex w-full items-baseline gap-[1ch] bg-[#14181d] px-3 py-[3px] text-left hover:bg-[#101317] focus-visible:outline-none"
                                                } else {
                                                    "flex w-full items-baseline gap-[1ch] px-3 py-[3px] text-left hover:bg-[#101317] focus-visible:outline-none"
                                                }
                                            }
                                        >
                                            <span
                                                aria-hidden="true"
                                                class=move || {
                                                    if is_active.get() {
                                                        "w-[1ch] shrink-0 text-[#e2a340]"
                                                    } else {
                                                        "w-[1ch] shrink-0 text-transparent"
                                                    }
                                                }
                                            >
                                                "\u{258e}"
                                            </span>
                                            <span
                                                aria-hidden="true"
                                                class=move || {
                                                    if is_active.get() {
                                                        "w-[3ch] shrink-0 text-right text-[#e2a340] tabular-nums"
                                                    } else {
                                                        "w-[3ch] shrink-0 text-right text-[#3c424a] tabular-nums"
                                                    }
                                                }
                                            >
                                            {index.saturating_add(1)}
                                            </span>
                                            <span
                                                class=move || {
                                                    if is_active.get() {
                                                        "truncate text-white"
                                                    } else {
                                                        "truncate text-[#8b939d]"
                                                    }
                                                }
                                            >
                                                {entry.name}
                                            </span>
                                        </A>
                                    }
                                })
                                .collect_view()}
                        </div>
                    }
                })
                .collect_view()}
        </nav>
    }
}

/// The navigation's visibility control. It sits outside the `<nav>` so it
/// survives the collapse, and dispatches the transition `Ctrl+B` dispatches.
#[component]
pub fn SidebarToggle(editor: EditorController) -> impl IntoView {
    let open = Signal::derive(move || editor.sidebar());

    view! {
        <button
            type="button"
            aria-controls=NAVIGATION_ID
            aria-expanded=move || open.get().to_string()
            aria-keyshortcuts="Control+B"
            aria-label=move || {
                if open.get() {
                    "Collapse portfolio navigation, Ctrl+B"
                } else {
                    "Expand portfolio navigation, Ctrl+B"
                }
            }
            on:click=move |_| editor.toggle_sidebar()
            class="absolute top-2 left-2 z-10 flex h-7 items-center gap-[1ch] border border-[#30363d] bg-[#080a0d] px-1.5 text-[10px] text-[#b8bfc7] hover:border-[#3d444d] hover:text-white"
        >
            <span aria-hidden="true"><PanelLeft size=14 /></span>
            <span aria-hidden="true" class="text-[#e2a340]">"^B"</span>
        </button>
    }
}

fn navigation(content: &PortfolioContent) -> Vec<(SectionId, Vec<NavigationEntry>)> {
    let buffer = Buffer::from_content(content);

    [SectionId::Profile, SectionId::Work, SectionId::Contact]
        .into_iter()
        .map(|section| {
            let entries = buffer
                .entries()
                .iter()
                .filter(|entry| entry.section == section)
                .map(|entry| NavigationEntry {
                    id: entry.id.clone(),
                    name: entry.name.clone(),
                    href: entry.id.path(),
                })
                .collect();
            (section, entries)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::content::portfolio_content;

    #[test]
    fn projects_link_to_their_detail_routes() {
        let links = navigation(&portfolio_content())
            .into_iter()
            .flat_map(|(_, entries)| entries)
            .map(|entry| entry.href)
            .collect::<Vec<_>>();

        assert!(links.contains(&"/work/guenther".to_owned()));
        assert!(links.contains(&"/work/traxor".to_owned()));
    }
}
