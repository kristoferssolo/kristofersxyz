//! Persistent chrome shared by every portfolio page.

mod blank_page;
mod command_shell;
mod help_panel;
mod notice_view;
mod sidebar;
mod status_bar;

pub use blank_page::BlankPage;
pub use command_shell::CommandShell;
pub use sidebar::{CollapsibleSidebar, SidebarPreference};
pub use status_bar::{StatusBarState, StatusLocation};

use crate::app::editor_controller::EditorController;
use help_panel::HelpPanel;
use leptos::prelude::Effect as ReactiveEffect;
use leptos::{ev, prelude::*};
use notice_view::NoticeView;
use status_bar::StatusBar;

#[component]
pub(crate) fn EditorShell(
    editor: EditorController,
    #[prop(into)] status: Signal<StatusBarState>,
    children: Children,
) -> impl IntoView {
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
            <div class="h-full min-h-0 min-w-0">{children()}</div>
            <StatusBar state=status />
        </main>
    }
}
