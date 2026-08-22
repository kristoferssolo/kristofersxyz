//! Browser navigation at the edge of the application.

use crate::app::editor::{Destination, EntryId};
use leptos::{prelude::document, wasm_bindgen::JsCast, web_sys};

/// Below this width the sidebar and content stack, so a selection has to pull
/// the content into view. Matches Tailwind's `md` breakpoint.
const STACK_BELOW_PX: f64 = 768.0;

/// Navigates inside the portfolio or opens an external destination in a new
/// tab.
pub(super) fn navigate(destination: &Destination) {
    let Some(window) = web_sys::window() else {
        return;
    };

    match destination {
        Destination::Internal(path) => {
            let _ = window.location().set_href(path);
        }
        Destination::External(url) => {
            let _ = window.open_with_url_and_target(url, "_blank");
        }
    }
}

/// Opens one entry in the portfolio's ordered page sequence.
pub(super) fn navigate_to(entry: &EntryId) {
    navigate(&Destination::Internal(entry.path()));
}

/// Returns the current URL fragment when it addresses part of the homepage.
pub(super) fn current_fragment() -> Option<String> {
    web_sys::window()?
        .location()
        .hash()
        .ok()
        .filter(|fragment| !fragment.is_empty())
}

/// Moves keyboard focus onto a sidebar entry so assistive technology announces
/// the row selected by an editor command.
pub(super) fn focus_row(entry: &EntryId) {
    if let Some(element) = document().get_element_by_id(&row_id(entry))
        && let Ok(element) = element.dyn_into::<web_sys::HtmlElement>()
    {
        let _ = element.focus();
    }
}

/// On the stacked phone layout, selected homepage content sits below the
/// sidebar and must be brought into view.
pub(super) fn reveal_content() {
    if viewport_is_stacked()
        && let Some(section) = document().get_element_by_id("buffer-content")
    {
        section.scroll_into_view();
    }
}

fn row_id(entry: &EntryId) -> String {
    format!("buffer-{}", entry.fragment())
}

fn viewport_is_stacked() -> bool {
    web_sys::window()
        .and_then(|window| window.inner_width().ok())
        .and_then(|width| width.as_f64())
        .is_some_and(|width| width < STACK_BELOW_PX)
}
