use serde::{Deserialize, Serialize};
use std::fmt;

/// The name that identifies the portfolio owner during authentication.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Username(String);

impl Username {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for Username {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for Username {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl fmt::Display for Username {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
