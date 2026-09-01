use super::super::collections::{VisibleNameError, visible_name};
use crate::serde_helpers::impl_deserialize_from_str;
use serde::{Deserialize, Serialize};
use std::{fmt, num::NonZeroU32, str::FromStr};

/// The longest edge a stored screenshot may have.
pub const MAX_SCREENSHOT_EDGE: u32 = 4096;

/// The largest screenshot an upload may carry.
pub const MAX_SCREENSHOT_BYTES: usize = 5 * 1_024 * 1_024;

/// What a reader who cannot see the screenshot is told instead.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub struct ScreenshotAltText(String);

impl ScreenshotAltText {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ScreenshotAltText {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for ScreenshotAltText {
    type Err = VisibleNameError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        visible_name(value).map(Self)
    }
}

impl_deserialize_from_str!(ScreenshotAltText);

/// The optional line printed under a screenshot.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub struct ScreenshotCaption(String);

impl ScreenshotCaption {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Treats an empty field as no caption and validates present text.
    ///
    /// # Errors
    ///
    /// Returns [`VisibleNameError`] when the text is blank or padded.
    pub fn parse_optional(value: &str) -> Result<Option<Self>, VisibleNameError> {
        if value.is_empty() {
            return Ok(None);
        }
        value.parse().map(Some)
    }
}

impl fmt::Display for ScreenshotCaption {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for ScreenshotCaption {
    type Err = VisibleNameError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        visible_name(value).map(Self)
    }
}

impl_deserialize_from_str!(ScreenshotCaption);

/// An image format accepted for a Project Screenshot.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum ScreenshotMediaType {
    Png,
    Jpeg,
    Webp,
}

impl ScreenshotMediaType {
    /// The value sent as `Content-Type` and stored beside the bytes.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::Webp => "image/webp",
        }
    }
}

impl fmt::Display for ScreenshotMediaType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for ScreenshotMediaType {
    type Err = ScreenshotMediaTypeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "image/png" => Ok(Self::Png),
            "image/jpeg" => Ok(Self::Jpeg),
            "image/webp" => Ok(Self::Webp),
            _ => Err(ScreenshotMediaTypeError::Unsupported),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ScreenshotMediaTypeError {
    #[error("a screenshot must be a PNG, JPEG, or WebP image")]
    Unsupported,
}

/// Stored pixel dimensions with non-zero edges no longer than [`MAX_SCREENSHOT_EDGE`] pixels.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(into = "(u32, u32)", try_from = "(u32, u32)")]
pub struct ScreenshotSize {
    width: NonZeroU32,
    height: NonZeroU32,
}

impl ScreenshotSize {
    #[must_use]
    pub const fn width(self) -> u32 {
        self.width.get()
    }

    #[must_use]
    pub const fn height(self) -> u32 {
        self.height.get()
    }
}

impl TryFrom<(u32, u32)> for ScreenshotSize {
    type Error = ScreenshotSizeError;

    fn try_from((width, height): (u32, u32)) -> Result<Self, Self::Error> {
        if width > MAX_SCREENSHOT_EDGE || height > MAX_SCREENSHOT_EDGE {
            return Err(ScreenshotSizeError::TooLarge);
        }
        let (Some(width), Some(height)) = (NonZeroU32::new(width), NonZeroU32::new(height)) else {
            return Err(ScreenshotSizeError::Empty);
        };

        Ok(Self { width, height })
    }
}

impl From<ScreenshotSize> for (u32, u32) {
    fn from(size: ScreenshotSize) -> Self {
        (size.width(), size.height())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ScreenshotSizeError {
    #[error("a screenshot cannot have a zero width or height")]
    Empty,
    #[error("a screenshot can be at most {MAX_SCREENSHOT_EDGE} by {MAX_SCREENSHOT_EDGE} pixels")]
    TooLarge,
}
