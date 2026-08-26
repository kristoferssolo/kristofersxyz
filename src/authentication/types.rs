use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// A plaintext password supplied by the owner.
pub struct Password(SecretString);

impl Password {
    pub(crate) fn expose_secret(&self) -> &str {
        self.0.expose_secret()
    }
}

impl From<String> for Password {
    fn from(value: String) -> Self {
        Self(SecretString::from(value))
    }
}

impl From<SecretString> for Password {
    fn from(value: SecretString) -> Self {
        Self(value)
    }
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
