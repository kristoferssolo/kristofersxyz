//! Pure editor state transitions.
//!
//! This module has no DOM, timers, or `web_sys`. The Leptos adapter converts
//! browser events into [`KeyInput`] values and applies the returned [`Effect`]s.
//!
//! `normal` and `line` reduce input by mode. `buffer`, `command`, and `key`
//! define the values they consume.

mod buffer;
mod command;
mod key;
mod line;
mod normal;
mod state;

pub use buffer::{
    Buffer, BufferEntry, Destination, EntryId, PageStep, SearchHit, SectionId, Selection,
};
pub use command::Command;
pub use key::{Key, KeyInput};
pub use state::{EditorState, Effect, Mode, Notification, Transition};

/// Returns the transition for one key press.
#[must_use]
pub fn reduce(state: &EditorState, input: KeyInput, buffer: &Buffer) -> Transition {
    if input.foreign() {
        return Transition::unchanged(state);
    }

    match &state.mode {
        Mode::Normal => normal::reduce(state, input, buffer),
        Mode::Command(text) => line::command(state, input, buffer, text),
        Mode::Search(query) => line::search(state, input, buffer, query),
    }
}

/// Shows or hides the portfolio navigation.
///
/// Both `Ctrl+B` and the toggle button call this function.
#[must_use]
pub fn toggle_sidebar(state: &EditorState) -> Transition {
    Transition::new(
        EditorState {
            sidebar: !state.sidebar,
            ..state.clone()
        },
        Vec::new(),
    )
}

/// Selects an entry by id.
///
/// Rows, action links, and URL fragments use this path. Selection also closes
/// an open command or search line.
#[must_use]
pub fn select(state: &EditorState, entry: &EntryId, buffer: &Buffer) -> Transition {
    let Some(active) = buffer.get(entry).map(BufferEntry::selection) else {
        return Transition::unchanged(state);
    };

    let scrolled = Effect::ScrollTo(active.entry.clone());
    Transition::new(
        EditorState {
            mode: Mode::Normal,
            active,
            ..state.clone()
        },
        vec![scrolled],
    )
}

#[cfg(test)]
mod tests;
