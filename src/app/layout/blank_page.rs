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

/// Wraps page content in the shared sidebar and status bar, and routes global
/// keyboard input through the supplied editor session.
#[component]
pub fn BlankPage(
    editor: EditorController,
    #[prop(into)] status: Signal<StatusBarState>,
    #[prop(optional)] on_select: Option<Callback<EntryId>>,
    children: Children,
) -> impl IntoView {
    let active = Signal::derive(move || editor.active());
    let sidebar = Signal::derive(move || editor.sidebar());

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
            <div class=move || {
                if sidebar.get() {
                    "relative grid min-h-0 grid-rows-[auto_minmax(0,1fr)] transition-[grid-template-columns] duration-150 ease-out md:grid-cols-[340px_minmax(0,1fr)] md:grid-rows-1"
                } else {
                    "relative grid min-h-0 grid-rows-[minmax(0,1fr)] transition-[grid-template-columns] duration-150 ease-out md:grid-cols-[0px_minmax(0,1fr)] md:grid-rows-1"
                }
            }>
                <SidebarToggle editor />
                <Sidebar active visible=sidebar on_select=on_select />
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
