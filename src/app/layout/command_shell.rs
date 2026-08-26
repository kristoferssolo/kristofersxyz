use super::{
    StatusBarState, StatusLocation, help_panel::HelpPanel, notice_view::NoticeView,
    status_bar::StatusBar,
};
use crate::app::{content::Portfolio, editor::EntryId, editor_controller::EditorController};
use leptos::prelude::Effect as ReactiveEffect;
use leptos::{ev, prelude::*};

#[component]
pub fn CommandShell(#[prop(into)] filename: String, children: Children) -> impl IntoView {
    let content = expect_context::<Portfolio>().current();
    let editor = EditorController::restricted(&content, &EntryId::Profile);
    let status = Signal::derive(move || {
        StatusBarState::from_editor_mode(
            editor.mode(),
            filename.clone(),
            StatusLocation::Cursor { line: 0, column: 0 },
        )
        .with_help()
    });

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
