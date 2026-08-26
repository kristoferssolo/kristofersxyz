mod portfolio;

pub use portfolio::PortfolioNavigation;

use leptos::prelude::*;
use lucide_leptos::PanelLeft;

/// Remembers whether navigation is open as routes replace their page layouts.
#[derive(Clone, Copy)]
pub struct SidebarPreference(RwSignal<bool>);

impl Default for SidebarPreference {
    fn default() -> Self {
        Self(RwSignal::new(true))
    }
}

impl SidebarPreference {
    #[must_use]
    pub fn open(self) -> bool {
        self.0.get()
    }

    #[must_use]
    pub fn open_untracked(self) -> bool {
        self.0.get_untracked()
    }

    pub fn set(self, open: bool) {
        self.0.set(open);
    }

    pub fn toggle(self) {
        self.0.update(|open| *open = !*open);
    }
}

/// A fully collapsible navigation column and the content beside it.
///
/// Callers supply their navigation adapter and decide how a toggle changes
/// state. This keeps portfolio reducer behavior out of the admin layout.
#[component]
pub fn CollapsibleSidebar(
    id: &'static str,
    label: &'static str,
    width: &'static str,
    #[prop(into)] open: Signal<bool>,
    on_toggle: Callback<()>,
    navigation: AnyView,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=move || {
                if open.get() {
                    "relative grid h-full min-h-0 grid-rows-[auto_minmax(0,1fr)] transition-[grid-template-columns] duration-150 ease-out md:grid-cols-[var(--sidebar-width)_minmax(0,1fr)] md:grid-rows-1"
                } else {
                    "relative grid h-full min-h-0 grid-rows-[minmax(0,1fr)] transition-[grid-template-columns] duration-150 ease-out md:grid-cols-[0px_minmax(0,1fr)] md:grid-rows-1"
                }
            }
            style=format!("--sidebar-width: {width}")
        >
            <button
                type="button"
                aria-controls=id
                aria-expanded=move || open.get().to_string()
                aria-keyshortcuts="Control+B"
                aria-label=move || {
                    if open.get() {
                        format!("Collapse {label}, Ctrl+B")
                    } else {
                        format!("Expand {label}, Ctrl+B")
                    }
                }
                on:click=move |_| on_toggle.run(())
                class="absolute top-3 left-3 z-10 grid size-8 cursor-pointer place-items-center bg-transparent text-[#767d87] hover:bg-[#0c1013] hover:text-white focus-visible:outline focus-visible:outline-1 focus-visible:outline-[#e2a340]"
            >
                <PanelLeft size=16 />
            </button>
            <div
                id=id
                class=move || {
                    if open.get() {
                        "min-h-0 min-w-0 overflow-hidden opacity-100 transition-[opacity,visibility] duration-150 ease-out"
                    } else {
                        "invisible min-h-0 min-w-0 overflow-hidden opacity-0 transition-[opacity,visibility] duration-150 ease-out"
                    }
                }
            >
                {navigation}
            </div>
            <div class=move || {
                if open.get() {
                    "min-h-0 min-w-0"
                } else {
                    "min-h-0 min-w-0 pt-10 md:pt-0"
                }
            }>
                {children()}
            </div>
        </div>
    }
}
