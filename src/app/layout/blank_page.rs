use super::{StatusBarState, sidebar::Sidebar, status_bar::StatusBar};
use crate::app::editor::EntryId;
use leptos::prelude::*;

/// The empty portfolio frame. Pages supply only their active entry, status,
/// and content; this module owns the viewport and persistent chrome.
#[component]
pub fn BlankPage(
    #[prop(into)] active: Signal<EntryId>,
    #[prop(into)] status: Signal<StatusBarState>,
    #[prop(optional)] on_select: Option<Callback<EntryId>>,
    children: Children,
) -> impl IntoView {
    view! {
        <main class="grid h-dvh grid-rows-[minmax(0,1fr)_1.75rem] overflow-hidden bg-black font-mono text-[#d4d7db]">
            <div class="grid min-h-0 grid-rows-[auto_minmax(0,1fr)] md:grid-cols-[minmax(260px,340px)_minmax(0,1fr)] md:grid-rows-1">
                <Sidebar active on_select=on_select />
                <div class="flex min-h-0 min-w-0 flex-col">{children()}</div>
            </div>
            <StatusBar state=status />
        </main>
    }
}
