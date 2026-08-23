//! The editor's data model: the state a key press reads, and the
//! transition it produces. The reduction that moves between these values
//! lives in the `normal` and `line` sibling modules.

use super::{Destination, EntryId, Selection};
use std::fmt::{self, Display, Formatter};

/// Which keys mean what right now.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Mode {
    Normal,
    /// Command line text, without the leading colon.
    Command(String),
    /// Search query, without the leading slash.
    Search(String),
}

/// Everything the editor knows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorState {
    pub mode: Mode,
    pub active: Selection,
    /// The which-key panel.
    pub help: bool,
    /// The portfolio navigation. `Ctrl+B` and the toggle button are the only
    /// two ways to change it, so a collapse is always the reader's choice.
    pub sidebar: bool,
}

impl EditorState {
    /// Normal mode on `active`, help closed and the navigation open.
    #[must_use]
    pub const fn new(active: Selection) -> Self {
        Self {
            mode: Mode::Normal,
            active,
            help: false,
            sidebar: true,
        }
    }
}

/// The full message set. Vim codes where vim has them, plain language where
/// it does not: inventing a code for a case vim lacks is where the bit would
/// become a lie.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Notification {
    NotAnEditorCommand(String),
    TrailingCharacters(String),
    PatternNotFound(String),
    SearchWrapped,
    NothingToOpen,
}

impl Display for Notification {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAnEditorCommand(input) => write!(f, "E492: Not an editor command: {input}"),
            Self::TrailingCharacters(rest) => write!(f, "E488: Trailing characters: {rest}"),
            Self::PatternNotFound(query) => write!(f, "E486: Pattern not found: {query}"),
            Self::SearchWrapped => f.write_str("search hit BOTTOM, continuing at TOP"),
            Self::NothingToOpen => f.write_str("Nothing to open here"),
        }
    }
}

/// Something the adapter has to do to the world. Timing lives out there too:
/// the reducer emits [`Effect::Notify`], the adapter schedules its removal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Effect {
    /// Bring the entry's content into view.
    ScrollTo(EntryId),
    /// Open a repository, in a new tab.
    Navigate(Destination),
    /// Replace whatever notification is showing.
    Notify(Notification),
    /// Clear the notification now, rather than waiting for its timer.
    Dismiss,
    /// Return keyboard focus to the page, after the command line closes.
    FocusPage,
}

/// The result of one key press.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Transition {
    pub state: EditorState,
    pub effects: Vec<Effect>,
}

impl Transition {
    pub(super) const fn new(state: EditorState, effects: Vec<Effect>) -> Self {
        Self { state, effects }
    }

    /// What an unbound key produces: nothing at all.
    pub(super) fn unchanged(state: &EditorState) -> Self {
        Self::new(state.clone(), Vec::new())
    }
}
