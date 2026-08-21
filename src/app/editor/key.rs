//! The browser event, normalized at the Leptos boundary so the reducer never
//! sees `web_sys`.

/// A key, as the editor cares about it. Anything unbound arrives as
/// [`Key::Other`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Enter,
    Escape,
    Backspace,
    ArrowUp,
    ArrowDown,
    Other,
}

impl Key {
    /// Maps a `KeyboardEvent.key` string. Single character names become
    /// [`Key::Char`], which carries the shift state through its case.
    #[must_use]
    pub fn from_name(name: &str) -> Self {
        match name {
            "Enter" => Self::Enter,
            "Escape" => Self::Escape,
            "Backspace" => Self::Backspace,
            "ArrowUp" => Self::ArrowUp,
            "ArrowDown" => Self::ArrowDown,
            _ => {
                let mut chars = name.chars();
                match (chars.next(), chars.next()) {
                    (Some(character), None) => Self::Char(character),
                    _ => Self::Other,
                }
            }
        }
    }
}

/// A key press with the modifiers the editor binds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyInput {
    pub key: Key,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}

impl KeyInput {
    /// A press with no modifiers held.
    #[must_use]
    pub const fn plain(key: Key) -> Self {
        Self {
            key,
            ctrl: false,
            alt: false,
            meta: false,
        }
    }

    /// A press with only Ctrl held.
    #[must_use]
    pub const fn ctrl(key: Key) -> Self {
        Self {
            key,
            ctrl: true,
            alt: false,
            meta: false,
        }
    }

    /// True when a modifier the editor does not bind is held, which is the
    /// signal to leave the key to the browser.
    pub(super) const fn foreign(self) -> bool {
        self.alt || self.meta
    }
}
