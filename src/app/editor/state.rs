//! State read and produced by the editor reducers in `normal` and `line`.

use super::{Destination, EntryId, Selection};
use std::fmt::{self, Display, Formatter};

/// The editor's current input mode.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Mode {
    Normal,
    /// Command line text, without the leading colon.
    Command(String),
    /// Search query, without the leading slash.
    Search(String),
}

/// State shared by every input mode.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorState {
    pub mode: Mode,
    pub active: Selection,
    /// The which-key panel.
    pub help: bool,
    /// Whether the portfolio navigation is visible. Only `Ctrl+B` and its
    /// button change this value.
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

/// Messages emitted by editor transitions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Notification {
    NotAnEditorCommand(String),
    TrailingCharacters(String),
    NoMatchingBuffer(String),
    PatternNotFound(String),
    SearchWrapped,
    NothingToOpen,
}

impl Display for Notification {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAnEditorCommand(input) => write!(f, "E492: Not an editor command: {input}"),
            Self::TrailingCharacters(rest) => write!(f, "E488: Trailing characters: {rest}"),
            Self::NoMatchingBuffer(name) => write!(f, "E94: No matching buffer for {name}"),
            Self::PatternNotFound(query) => write!(f, "E486: Pattern not found: {query}"),
            Self::SearchWrapped => f.write_str("search hit BOTTOM, continuing at TOP"),
            Self::NothingToOpen => f.write_str("Nothing to open here"),
        }
    }
}

/// Side effects for the browser adapter to perform.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Effect {
    /// Bring the entry's content into view.
    ScrollTo(EntryId),
    /// Open a repository, in a new tab.
    Navigate(Destination),
    /// Replace whatever notification is showing.
    Notify(Notification),
    /// Clear the current notification before its timer expires.
    Dismiss,
    /// Return keyboard focus to the page, after the command line closes.
    FocusPage,
}

/// New state and browser effects from one input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Transition {
    pub state: EditorState,
    pub effects: Vec<Effect>,
}

impl Transition {
    pub(super) const fn new(state: EditorState, effects: Vec<Effect>) -> Self {
        Self { state, effects }
    }

    /// Returns an unchanged state with no effects.
    pub(super) fn unchanged(state: &EditorState) -> Self {
        Self::new(state.clone(), Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(
        Notification::NotAnEditorCommand("wrok".to_owned()),
        "E492: Not an editor command: wrok"
    )]
    #[case(
        Notification::PatternNotFound("kubernetes".to_owned()),
        "E486: Pattern not found: kubernetes"
    )]
    #[case(Notification::SearchWrapped, "search hit BOTTOM, continuing at TOP")]
    #[case(Notification::NothingToOpen, "Nothing to open here")]
    fn notifications_read_the_way_vim_reports_them(
        #[case] notification: Notification,
        #[case] expected: &str,
    ) {
        assert_eq!(notification.to_string(), expected);
    }
}
