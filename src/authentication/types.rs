use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// A plaintext password supplied by the owner.
#[derive(Debug)]
pub struct Password(SecretString);

impl Password {
    /// Creates a password containing at least one non-whitespace character.
    ///
    /// # Errors
    ///
    /// Returns [`PasswordError::Empty`] if `value` is blank.
    pub fn new(value: SecretString) -> Result<Self, PasswordError> {
        if value.expose_secret().trim().is_empty() {
            Err(PasswordError::Empty)
        } else {
            Ok(Self(value))
        }
    }

    /// Creates a password without validating its domain invariants.
    #[cfg(test)]
    #[must_use]
    pub const fn new_unchecked(value: SecretString) -> Self {
        Self(value)
    }

    pub(crate) fn expose_secret(&self) -> &str {
        self.0.expose_secret()
    }
}

impl TryFrom<String> for Password {
    type Error = PasswordError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(SecretString::from(value))
    }
}

impl TryFrom<SecretString> for Password {
    type Error = PasswordError;

    fn try_from(value: SecretString) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum PasswordError {
    #[error("a password cannot be empty")]
    Empty,
}

/// An encoded password hash suitable for persistent storage.
pub struct PasswordHash(SecretString);

impl PasswordHash {
    pub(crate) fn expose_secret(&self) -> &str {
        self.0.expose_secret()
    }
}

impl From<String> for PasswordHash {
    fn from(value: String) -> Self {
        Self(SecretString::from(value))
    }
}

/// The stable identifier of the portfolio owner.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct OwnerId(Uuid);

impl OwnerId {
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for OwnerId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Uuid> for OwnerId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl TryFrom<&str> for OwnerId {
    type Error = uuid::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Uuid::parse_str(value).map(Self)
    }
}

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
    fn passwords_require_visible_content() {
        assert_err!(Password::try_from(String::new()));
        assert_err!(Password::try_from(" \t".to_owned()));
        assert_ok!(Password::try_from("correct horse".to_owned()));
    }
}
