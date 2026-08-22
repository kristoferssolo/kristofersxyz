use crate::app::editor::EntryId;
use leptos::{prelude::*, wasm_bindgen::JsCast, web_sys};

/// Below this width the buffer list and content pane stack, so a selection has
/// to pull the view down to the content. Matches Tailwind's `md` breakpoint.
const STACK_BELOW_PX: f64 = 768.0;

/// Moves keyboard focus onto an entry's row, so screen readers announce the
/// row the vim keys just moved to.
pub(super) fn focus_row(entry: &EntryId) {
    if let Some(element) = document().get_element_by_id(&row_id(entry))
        && let Ok(element) = element.dyn_into::<web_sys::HtmlElement>()
    {
        let _ = element.focus();
    }
}

/// The DOM id of an entry's row in the buffer list.
pub(super) fn row_id(entry: &EntryId) -> String {
    format!("buffer-{}", entry.fragment())
}

/// On the stacked phone layout the content sits below the whole list, so a
/// selection has to bring it into view.
pub(super) fn reveal_content() {
    if viewport_is_stacked()
        && let Some(section) = document().get_element_by_id("buffer-content")
    {
        section.scroll_into_view();
    }
}

fn viewport_is_stacked() -> bool {
    web_sys::window()
        .and_then(|window| window.inner_width().ok())
        .and_then(|width| width.as_f64())
        .is_some_and(|width| width < STACK_BELOW_PX)
}
