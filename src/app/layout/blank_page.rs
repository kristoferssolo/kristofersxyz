use super::{
    StatusBarState,
    help_panel::HelpPanel,
    notice_view::NoticeView,
    sidebar::{Sidebar, SidebarToggle},
    status_bar::StatusBar,
};
use crate::app::{editor::EntryId, editor_controller::EditorController};
use leptos::prelude::Effect as ReactiveEffect;
use leptos::{ev, prelude::*};

/// The empty portfolio frame. Pages supply an editor session, status, and
/// content; this module owns input, feedback, the viewport, and chrome.
#[component]
pub fn BlankPage(
    editor: EditorController,
    #[prop(into)] status: Signal<StatusBarState>,
    #[prop(optional)] on_select: Option<Callback<EntryId>>,
    children: Children,
) -> impl IntoView {
    let active = Signal::derive(move || editor.active());
    let sidebar = Signal::derive(move || editor.sidebar());

    // The frame owns global input so command mode behaves identically on
    // home, Project Details, and error pages.
    ReactiveEffect::new(move |_| {
        let handle = window_event_listener(ev::keydown, move |event| {
            editor.handle_keydown(&event);
        });
        on_cleanup(move || handle.remove());
    });

    view! {
        <main class="grid h-dvh grid-rows-[minmax(0,1fr)_1.75rem] overflow-hidden bg-black font-mono text-[#d4d7db]">
            <NoticeView editor />
            <HelpPanel editor />
            // Only the column animates. The stacked layout drops the sidebar's
            // row instead of narrowing it, and `auto` tracks do not interpolate.
            <div class=move || {
                if sidebar.get() {
                    "relative grid min-h-0 grid-rows-[auto_minmax(0,1fr)] transition-[grid-template-columns] duration-150 ease-out md:grid-cols-[340px_minmax(0,1fr)] md:grid-rows-1"
                } else {
                    "relative grid min-h-0 grid-rows-[minmax(0,1fr)] transition-[grid-template-columns] duration-150 ease-out md:grid-cols-[0px_minmax(0,1fr)] md:grid-rows-1"
                }
            }>
                <SidebarToggle editor />
                <Sidebar active visible=sidebar on_select=on_select />
                // Wide pages have room for the floating toggle inside their
                // own left margin. Narrow ones do not, so the content starts
                // below it instead.
                <div class=move || {
                    if sidebar.get() {
                        "flex min-h-0 min-w-0 flex-col"
                    } else {
                        "flex min-h-0 min-w-0 flex-col pt-10 md:pt-0"
                    }
                }>{children()}</div>
            </div>
            <StatusBar state=status />
        </main>
    }
}
