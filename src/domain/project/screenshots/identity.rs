use crate::serde_helpers::impl_deserialize_from_str;
use serde::Serialize;
use std::{fmt, str::FromStr};

/// A UUID in canonical lowercase, hyphenated form.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub struct ScreenshotId(String);

impl ScreenshotId {
    /// Generates an identity for an image that is about to be stored.
    #[cfg(feature = "ssr")]
    #[must_use]
    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ScreenshotId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for ScreenshotId {
    type Err = ScreenshotIdError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        const UUID_GROUP_LENGTHS: [usize; 5] = [8, 4, 4, 4, 12];

        let mut groups = value.split('-');
        for length in UUID_GROUP_LENGTHS {
            let group = groups.next().ok_or(ScreenshotIdError::Malformed)?;
            if group.len() != length || !group.bytes().all(is_lowercase_hex) {
                return Err(ScreenshotIdError::Malformed);
            }
        }
        if groups.next().is_some() {
            return Err(ScreenshotIdError::Malformed);
        }

        Ok(Self(value.to_owned()))
    }
}

const fn is_lowercase_hex(byte: u8) -> bool {
    byte.is_ascii_digit() || matches!(byte, b'a'..=b'f')
}

impl_deserialize_from_str!(ScreenshotId);

/// A rejected screenshot identity. Errors omit the supplied path text.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ScreenshotIdError {
    #[error("a screenshot id must be a canonical lowercase UUID")]
    Malformed,
}
