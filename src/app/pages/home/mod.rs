mod action_links;
mod browser;
mod buffer_list;
mod command_line;
mod content_pane;
mod entry_details;
mod external_links;
mod model;
mod next_entry;
mod notice_view;
mod status_line;
mod view_model;

use self::{
    buffer_list::BufferList, content_pane::ContentPane, notice_view::NoticeView,
    status_line::StatusLine, view_model::HomeViewModel,
};
use crate::app::{
    content::PortfolioContent,
    editor::{Key, KeyInput},
};
use leptos::prelude::Effect as ReactiveEffect;
use leptos::{ev, prelude::*};

/// The portfolio as a modal editor. The component wires browser input to the
/// editor reducer while each pane owns its own rendering module.
#[component]
pub fn HomePage() -> impl IntoView {
    let content = expect_context::<PortfolioContent>();
    let view_model = HomeViewModel::new(&content);
    provide_context(view_model);

    // Bound globally so the keys work wherever the reader's focus sits.
    ReactiveEffect::new(move |_| {
        let handle = window_event_listener(ev::keydown, move |event| {
            let input = KeyInput {
                key: Key::from_name(&event.key()),
                ctrl: event.ctrl_key(),
                alt: event.alt_key(),
                meta: event.meta_key(),
            };
            let current = view_model.state.get_untracked();
            let transition = view_model.transition_for(input);

            // A key the editor does not bind changes nothing, so leave it to
            // the browser rather than swallowing it.
            if transition.state == current && transition.effects.is_empty() {
                return;
            }

            event.prevent_default();
            view_model.advance(transition);
        });
        on_cleanup(move || handle.remove());
    });

    view! {
        <main class="flex min-h-dvh flex-col bg-black font-mono text-[#d4d7db] md:h-dvh md:overflow-hidden">
            <NoticeView />
            <div class="grid min-h-0 flex-1 md:grid-cols-[minmax(260px,340px)_minmax(0,1fr)]">
                <BufferList />
                <ContentPane />
            </div>
            <StatusLine />
        </main>
    }
}
