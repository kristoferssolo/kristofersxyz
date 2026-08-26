use super::{
    StatusBarState,
    help_panel::HelpPanel,
    notice_view::NoticeView,
    sidebar::{CollapsibleSidebar, PortfolioNavigation},
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
            <CollapsibleSidebar
                id="portfolio-navigation"
                label="portfolio navigation"
                width="340px"
                open=sidebar
                on_toggle=Callback::new(move |()| editor.toggle_sidebar())
                navigation=view! { <PortfolioNavigation active on_select /> }.into_any()
            >
                <div class="flex h-full min-h-0 min-w-0 flex-col">{children()}</div>
            </CollapsibleSidebar>
            <StatusBar state=status />
        </main>
    }
}
