use crate::app::{
    content::{Portfolio, PortfolioContent},
    editor::{Buffer, EntryId, SectionId},
};
use leptos::prelude::*;
use leptos_router::components::A;

#[derive(Clone)]
struct NavigationEntry {
    id: EntryId,
    name: String,
    href: String,
}

/// Portfolio navigation content for the shared collapsible sidebar.
#[component]
pub fn PortfolioNavigation(
    #[prop(into)] active: Signal<EntryId>,
    on_select: Option<Callback<EntryId>>,
) -> impl IntoView {
    let content = expect_context::<Portfolio>().snapshot();
    let groups = navigation(&content);

    view! {
        <nav
            aria-label="Portfolio"
            class="min-h-0 overflow-x-hidden overflow-y-auto border-b border-[#1e2126] pt-12 pb-3 text-[13px] md:h-full md:border-r md:border-b-0"
        >
            {groups
                .into_iter()
                .map(|(section, entries)| {
                    view! {
                        <div class="mt-4 first:mt-0">
                            <p class="mb-1 pl-[7ch] text-[10px] tracking-[0.24em] whitespace-nowrap text-[#4c525a] uppercase">
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
                                            attr:aria-current=move || is_active.get().then_some("page")
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
                                            <span aria-hidden="true" class=move || {
                                                if is_active.get() {
                                                    "w-[1ch] shrink-0 text-[#e2a340]"
                                                } else {
                                                    "w-[1ch] shrink-0 text-transparent"
                                                }
                                            }>
                                                "\u{258e}"
                                            </span>
                                            <span aria-hidden="true" class=move || {
                                                if is_active.get() {
                                                    "w-[3ch] shrink-0 text-right text-[#e2a340] tabular-nums"
                                                } else {
                                                    "w-[3ch] shrink-0 text-right text-[#3c424a] tabular-nums"
                                                }
                                            }>
                                                {index.saturating_add(1)}
                                            </span>
                                            <span class=move || {
                                                if is_active.get() {
                                                    "truncate text-white"
                                                } else {
                                                    "truncate text-[#8b939d]"
                                                }
                                            }>
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
