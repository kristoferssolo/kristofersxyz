use std::{fmt, str::FromStr};

/// A single step through one Project's screenshot order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScreenshotMove {
    Up,
    Down,
}

impl fmt::Display for ScreenshotMove {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Up => "up",
            Self::Down => "down",
        })
    }
}

impl FromStr for ScreenshotMove {
    type Err = ScreenshotMoveError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "up" => Ok(Self::Up),
            "down" => Ok(Self::Down),
            _ => Err(ScreenshotMoveError::Unknown),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ScreenshotMoveError {
    #[error("a screenshot move must be up or down")]
    Unknown,
}
