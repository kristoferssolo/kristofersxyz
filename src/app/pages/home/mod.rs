mod action_links;
mod browser;
mod content_pane;
mod entry_details;
mod external_links;
mod model;
mod next_entry;
mod notice_view;
mod view_model;

use self::{content_pane::ContentPane, notice_view::NoticeView, view_model::HomeViewModel};
use crate::app::{
    browser::current_fragment,
    content::PortfolioContent,
    editor::{Key, KeyInput, Mode},
    layout::{BlankPage, StatusBarState, StatusLocation},
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
    let active = Signal::derive(move || view_model.state.get().active.entry);
    let on_select = Callback::new(move |entry| view_model.pick(&entry));
    let status = Signal::derive(move || match view_model.state.get().mode {
        Mode::Normal => StatusBarState::normal(
            "kristofers.xyz",
            StatusLocation::Page {
                current: view_model.position(),
                total: view_model.total(),
            },
        )
        .with_help()
        .with_progress(),
        Mode::Command(text) => StatusBarState::command(':', text),
        Mode::Search(text) => StatusBarState::command('/', text),
    });

    // Fragments are not sent to the server, so restore them once the browser
    // has mounted the homepage.
    ReactiveEffect::new(move |_| {
        if let Some(fragment) = current_fragment() {
            view_model.pick_fragment(&fragment);
        }
    });

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
        <NoticeView />
        <BlankPage active status on_select>
            <ContentPane />
        </BlankPage>
    }
}
