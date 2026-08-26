use serde::{Deserialize, Deserializer, Serialize, de};
use std::fmt;
#[cfg(feature = "ssr")]
use uuid::Uuid;

/// The name that identifies the portfolio owner during authentication.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Username(String);

impl Username {
    /// Creates a non-empty username without surrounding whitespace.
    ///
    /// # Errors
    ///
    /// Returns [`UsernameError::Empty`] if `value` is blank, or
    /// [`UsernameError::SurroundingWhitespace`] if it starts or ends with
    /// whitespace.
    pub fn new(value: String) -> Result<Self, UsernameError> {
        if value.trim().is_empty() {
            return Err(UsernameError::Empty);
        }
        if value.trim() != value {
            return Err(UsernameError::SurroundingWhitespace);
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
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

impl<'de> Deserialize<'de> for Username {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(String::deserialize(deserializer)?).map_err(de::Error::custom)
    }
}

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
}
