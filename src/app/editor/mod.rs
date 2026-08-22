//! The editor core: a pure reducer over key input.
//!
//! No `web_sys`, no timers, no DOM, so every key is exhaustively testable.
//! The Leptos adapter normalizes the browser event into a [`KeyInput`], calls
//! [`reduce`], and applies the resulting [`Effect`]s. The whole [`EditorState`]
//! lives here; none of it is half managed by the view.
//!
//! `state` holds the data model, `normal` and `line` hold the reduction for
//! each mode, and `buffer`, `command` and `key` hold the vocabulary the
//! reducer reads.

mod buffer;
mod command;
mod key;
mod line;
mod normal;
mod state;

pub use buffer::{Buffer, BufferEntry, Destination, EntryId, SearchHit, SectionId, Selection};
pub use command::Command;
pub use key::{Key, KeyInput};
pub use state::{EditorState, Effect, Mode, Notification, Transition};

/// Applies one key press. Pure: the same state, input and buffer always give
/// the same transition.
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

/// Selects an entry by id, rather than by key press.
///
/// The two paths that need it are a click on a row or an action link, and a
/// hash fragment on load. Closes any open line, the way clicking away from
/// vim's command line does.
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
