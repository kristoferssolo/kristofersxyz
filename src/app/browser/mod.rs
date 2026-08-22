//! Browser navigation at the edge of the application.

use crate::app::editor::{Destination, EntryId};
use leptos::web_sys;

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
