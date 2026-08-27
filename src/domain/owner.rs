use crate::serde_helpers::impl_deserialize_from_str;
#[cfg(feature = "ssr")]
use serde::Deserialize;
use serde::Serialize;
use std::{fmt, str::FromStr};
use unicode_segmentation::UnicodeSegmentation;
#[cfg(feature = "ssr")]
use uuid::Uuid;

/// The longest username, in grapheme clusters. Long enough for any real name
/// while bounding what the login path stores and compares.
const MAX_USERNAME_GRAPHEMES: usize = 256;

/// Characters that have structural meaning in paths, markup, or shells, kept
/// out of usernames as a defensive measure. Mirrors the set used for names in
/// *Zero to Production in Rust*.
const FORBIDDEN_CHARACTERS: [char; 9] = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];

/// The name that identifies the portfolio owner during authentication.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Username(String);

impl Username {
    /// Creates a validated username.
    ///
    /// # Errors
    ///
    /// Returns the matching [`UsernameError`] when `value` is blank, starts or
    /// ends with whitespace, exceeds `MAX_USERNAME_GRAPHEMES`, or contains a
    /// forbidden character.
    pub fn new(value: String) -> Result<Self, UsernameError> {
        if value.trim().is_empty() {
            return Err(UsernameError::Empty);
        }
        if value.trim() != value {
            return Err(UsernameError::SurroundingWhitespace);
        }
        if value.graphemes(true).count() > MAX_USERNAME_GRAPHEMES {
            return Err(UsernameError::TooLong);
        }
        if let Some(character) = value.chars().find(|c| FORBIDDEN_CHARACTERS.contains(c)) {
            return Err(UsernameError::ForbiddenCharacter(character));
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for Username {
    type Err = UsernameError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value.to_owned())
    }
}

impl TryFrom<String> for Username {
    type Error = UsernameError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for Username {
    type Error = UsernameError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value.to_owned())
    }
}

impl_deserialize_from_str!(Username);

impl fmt::Display for Username {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum UsernameError {
    #[error("a username cannot be empty")]
    Empty,
    #[error("a username cannot start or end with whitespace")]
    SurroundingWhitespace,
    #[error("a username cannot be longer than {MAX_USERNAME_GRAPHEMES} characters")]
    TooLong,
    #[error("a username cannot contain the character '{0}'")]
    ForbiddenCharacter(char),
}

/// The stable identifier of the portfolio owner.
#[cfg(feature = "ssr")]
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct OwnerId(Uuid);

#[cfg(feature = "ssr")]
impl OwnerId {
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[cfg(feature = "ssr")]
impl Default for OwnerId {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "ssr")]
impl From<Uuid> for OwnerId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

#[cfg(feature = "ssr")]
impl TryFrom<&str> for OwnerId {
    type Error = uuid::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Uuid::parse_str(value).map(Self)
    }
}

#[cfg(feature = "ssr")]
impl fmt::Display for OwnerId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_err, assert_ok};

    #[test]
    fn usernames_require_visible_content() {
        assert_err!(Username::new(String::new()));
        assert_err!(Username::new(" \t".to_owned()));
        assert_ok!(Username::new("owner".to_owned()));
    }

    #[test]
    fn usernames_reject_surrounding_whitespace() {
        assert_err!(Username::new(" owner".to_owned()));
        assert_err!(Username::new("owner ".to_owned()));
        assert_ok!(Username::new("site owner".to_owned()));
    }

    #[test]
    fn usernames_reject_forbidden_characters() {
        for character in FORBIDDEN_CHARACTERS {
            assert_err!(Username::new(format!("own{character}er")));
        }
    }

    #[test]
    fn usernames_are_bounded_in_length() {
        assert_ok!(Username::new("a".repeat(MAX_USERNAME_GRAPHEMES)));
        assert_err!(Username::new("a".repeat(MAX_USERNAME_GRAPHEMES + 1)));
    }
}
