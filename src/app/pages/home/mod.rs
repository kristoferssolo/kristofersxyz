mod action_links;
mod content_pane;
mod entry_details;
mod external_links;
mod model;
mod next_entry;
mod view_model;

use self::{content_pane::ContentPane, view_model::HomeViewModel};
use crate::app::{
    browser::current_fragment,
    content::PortfolioContent,
    layout::{BlankPage, StatusBarState, StatusLocation},
};
use leptos::prelude::Effect as ReactiveEffect;
use leptos::prelude::*;

/// The portfolio as a modal editor. The component wires browser input to the
/// editor reducer while each pane owns its own rendering module.
#[component]
pub fn HomePage() -> impl IntoView {
    let content = expect_context::<PortfolioContent>();
    let view_model = HomeViewModel::new(&content);
    provide_context(view_model);
    let editor = view_model.editor();
    let on_select = Callback::new(move |entry| view_model.pick(&entry));
    let status = Signal::derive(move || {
        StatusBarState::from_editor_mode(
            editor.mode(),
            "kristofers.xyz",
            StatusLocation::Page {
                current: view_model.position(),
                total: view_model.total(),
            },
        )
        .with_help()
        .with_progress()
    });

    ReactiveEffect::new(move |_| {
        if let Some(fragment) = current_fragment() {
            view_model.pick_fragment(&fragment);
        }
    });

    view! {
        <BlankPage editor status on_select>
            <ContentPane />
        </BlankPage>
    }
}
