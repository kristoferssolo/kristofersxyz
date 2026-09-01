//! Project Screenshots: the visual Project Evidence a Project Detail shows.
//!
//! A screenshot is identified by a generated UUID rather than by a filename,
//! so replacing the image is a new screenshot and no Owner-supplied name ever
//! reaches a route. The image bytes stay in the database; a [`Project`] only
//! carries the metadata a page needs to render the figure and the route that
//! serves the bytes.
//!
//! Alternative text is required, because a screenshot that a reader cannot see
//! is evidence only if it is described. A caption is optional, and a present
//! one follows the same visible-text rule as every other name in the domain.

use super::collections::{RepeatedEntry, VisibleNameError, first_repeat, visible_name};
use crate::serde_helpers::impl_deserialize_from_str;
use serde::{Deserialize, Deserializer, Serialize};
use std::{fmt, num::NonZeroU32, slice, str::FromStr, vec};

/// The route prefix that serves a screenshot's stored bytes. The router owns
/// the matching handler; this is the one place the public path is built.
pub const SCREENSHOT_MEDIA_PREFIX: &str = "/media/project";

/// The longest edge a stored screenshot may have. A larger image costs the
/// reader more than the detail it adds, and bounds what a decoder allocates.
pub const MAX_SCREENSHOT_EDGE: u32 = 4096;

/// The largest screenshot an upload may carry. The upload route bounds the
/// request body just above this, so an image over the limit is answered with
/// the Owner-facing reason rather than a bare rejection.
pub const MAX_SCREENSHOT_BYTES: usize = 5 * 1_024 * 1_024;

/// The identity of one Project Screenshot: a v4 UUID in its canonical
/// lowercase, hyphenated form.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub struct ScreenshotId(String);

impl ScreenshotId {
    /// A new identity for an image that is about to be stored. Replacing the
    /// bytes of a screenshot means generating one of these, so a cached
    /// response can never describe the wrong image.
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
        /// The hyphen-separated group lengths of a canonical UUID.
        const GROUPS: [usize; 5] = [8, 4, 4, 4, 12];

        let mut groups = value.split('-');
        for length in GROUPS {
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

/// A rejected screenshot identity. The text is not repeated back, so a crafted
/// path cannot echo through an error message or a log line.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ScreenshotIdError {
    #[error("a screenshot id must be a canonical lowercase UUID")]
    Malformed,
}

/// What a reader who cannot see the screenshot is told instead. Required,
/// because a described screenshot is evidence and an undescribed one is not.
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

/// The line printed under a screenshot. A Project Screenshot may have none;
/// a present caption carries visible text like every other name.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub struct ScreenshotCaption(String);

impl ScreenshotCaption {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Reads an optional caption field, where empty text means no caption at
    /// all rather than a rejected one. Whitespace still fails, so a caption
    /// cannot be stored with padding that changes how it renders.
    ///
    /// # Errors
    ///
    /// Returns [`VisibleNameError`] when the text is present but padded.
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

/// The image formats a Project Screenshot may be stored in. The stored bytes
/// are the ones that were uploaded, so this is both what the decoder found and
/// what the media route answers with.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum ScreenshotMediaType {
    Png,
    Jpeg,
    Webp,
}

impl ScreenshotMediaType {
    /// The media type sent as `Content-Type` and stored beside the bytes.
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

/// The stored pixel dimensions of a screenshot, so a page can reserve the
/// space before the image arrives. Neither edge can be zero, and neither can
/// exceed [`MAX_SCREENSHOT_EDGE`].
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

/// One Project Screenshot as every reader-facing consumer sees it: the
/// metadata a figure needs, and the route that serves the image. The bytes
/// themselves never leave the server.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProjectScreenshot {
    pub id: ScreenshotId,
    pub media_type: ScreenshotMediaType,
    pub size: ScreenshotSize,
    pub alt: ScreenshotAltText,
    pub caption: Option<ScreenshotCaption>,
}

impl ProjectScreenshot {
    /// The public route that answers with this screenshot's stored bytes.
    #[must_use]
    pub fn media_path(&self) -> String {
        format!("{SCREENSHOT_MEDIA_PREFIX}/{}", self.id)
    }
}

/// A Project's screenshots, in the order the Owner arranged them and with no
/// identity repeated. A Project may have none.
#[derive(Clone, Debug, Default, Serialize)]
#[serde(transparent)]
pub struct ProjectScreenshots(Vec<ProjectScreenshot>);

impl ProjectScreenshots {
    /// Returns the screenshots in their stored order.
    #[must_use]
    pub fn as_slice(&self) -> &[ProjectScreenshot] {
        &self.0
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> slice::Iter<'_, ProjectScreenshot> {
        self.0.iter()
    }
}

impl TryFrom<Vec<ProjectScreenshot>> for ProjectScreenshots {
    type Error = RepeatedEntry;

    fn try_from(screenshots: Vec<ProjectScreenshot>) -> Result<Self, Self::Error> {
        first_repeat(&screenshots, |earlier, current| earlier.id == current.id)
            .map_or(Ok(Self(screenshots)), |index| Err(RepeatedEntry { index }))
    }
}

impl<'de> Deserialize<'de> for ProjectScreenshots {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Vec::<ProjectScreenshot>::deserialize(deserializer)?
            .try_into()
            .map_err(serde::de::Error::custom)
    }
}

impl IntoIterator for ProjectScreenshots {
    type Item = ProjectScreenshot;
    type IntoIter = vec::IntoIter<ProjectScreenshot>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a ProjectScreenshots {
    type Item = &'a ProjectScreenshot;
    type IntoIter = slice::Iter<'a, ProjectScreenshot>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// A requested move through one Project's screenshot order.
///
/// Screenshots are arranged by button rather than by dragging, so the two
/// steps are the whole vocabulary, and the field survives a form round trip
/// before the page hydrates.
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

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_err_eq, assert_none, assert_ok, assert_ok_eq, assert_some};
    use rstest::rstest;

    fn screenshot(id: &str, caption: Option<&str>) -> ProjectScreenshot {
        ProjectScreenshot {
            id: crate::test_support::parse(id),
            media_type: ScreenshotMediaType::Png,
            size: assert_ok!(ScreenshotSize::try_from((1600, 1000))),
            alt: crate::test_support::parse("A terminal showing the transfer queue."),
            caption: caption.map(crate::test_support::parse),
        }
    }

    const FIRST: &str = "9c1f4e2a-1f2b-4a3c-8d4e-5f6a7b8c9d0e";
    const SECOND: &str = "0a1b2c3d-4e5f-4a6b-8c7d-9e0f1a2b3c4d";

    #[rstest]
    #[case(FIRST)]
    #[case(SECOND)]
    fn a_screenshot_id_accepts_a_canonical_uuid(#[case] value: &str) {
        assert_ok!(value.parse::<ScreenshotId>());
    }

    #[rstest]
    #[case("")]
    #[case("not-a-uuid")]
    #[case("9C1F4E2A-1F2B-4A3C-8D4E-5F6A7B8C9D0E")]
    #[case("9c1f4e2a1f2b4a3c8d4e5f6a7b8c9d0e")]
    #[case("9c1f4e2a-1f2b-4a3c-8d4e-5f6a7b8c9d0e-extra")]
    #[case("../../etc/passwd")]
    fn a_screenshot_id_rejects_anything_else(#[case] value: &str) {
        assert_err_eq!(value.parse::<ScreenshotId>(), ScreenshotIdError::Malformed);
    }

    #[test]
    fn alternative_text_needs_visible_unpadded_content() {
        assert_ok!("A terminal showing four transfers.".parse::<ScreenshotAltText>());
        assert_err_eq!("   ".parse::<ScreenshotAltText>(), VisibleNameError::Empty);
        assert_err_eq!(
            " A terminal. ".parse::<ScreenshotAltText>(),
            VisibleNameError::SurroundingWhitespace
        );
    }

    #[test]
    fn a_caption_may_be_absent_but_not_padded() {
        assert_none!(assert_ok!(ScreenshotCaption::parse_optional("")));
        assert_some!(assert_ok!(ScreenshotCaption::parse_optional(
            "The queue, ratios, and peers in one view."
        )));
        assert_err_eq!(
            ScreenshotCaption::parse_optional("  padded  "),
            VisibleNameError::SurroundingWhitespace
        );
        assert_err_eq!(
            ScreenshotCaption::parse_optional("   "),
            VisibleNameError::Empty
        );
    }

    #[rstest]
    #[case("image/png", ScreenshotMediaType::Png)]
    #[case("image/jpeg", ScreenshotMediaType::Jpeg)]
    #[case("image/webp", ScreenshotMediaType::Webp)]
    fn a_media_type_reads_back_from_its_stored_text(
        #[case] value: &str,
        #[case] expected: ScreenshotMediaType,
    ) {
        assert_ok_eq!(value.parse::<ScreenshotMediaType>(), expected);
        assert_eq!(expected.to_string(), value);
    }

    #[rstest]
    #[case("image/gif")]
    #[case("image/svg+xml")]
    #[case("video/mp4")]
    #[case("")]
    fn an_unsupported_media_type_is_rejected(#[case] value: &str) {
        assert_err_eq!(
            value.parse::<ScreenshotMediaType>(),
            ScreenshotMediaTypeError::Unsupported
        );
    }

    #[rstest]
    #[case(0, 800, ScreenshotSizeError::Empty)]
    #[case(1200, 0, ScreenshotSizeError::Empty)]
    #[case(4097, 800, ScreenshotSizeError::TooLarge)]
    #[case(800, 4097, ScreenshotSizeError::TooLarge)]
    fn a_size_rejects_empty_and_oversized_images(
        #[case] width: u32,
        #[case] height: u32,
        #[case] expected: ScreenshotSizeError,
    ) {
        assert_err_eq!(ScreenshotSize::try_from((width, height)), expected);
    }

    #[test]
    fn a_size_keeps_both_edges_up_to_the_limit() {
        let size = assert_ok!(ScreenshotSize::try_from((
            MAX_SCREENSHOT_EDGE,
            MAX_SCREENSHOT_EDGE
        )));

        assert_eq!((size.width(), size.height()), (4096, 4096));
    }

    #[test]
    fn a_screenshot_addresses_its_bytes_by_identity() {
        assert_eq!(
            screenshot(FIRST, None).media_path(),
            format!("/media/project/{FIRST}")
        );
    }

    #[test]
    fn screenshots_keep_their_order_and_reject_a_repeated_identity() {
        let ordered = vec![screenshot(FIRST, Some("First")), screenshot(SECOND, None)];
        let screenshots = assert_ok!(ProjectScreenshots::try_from(ordered));
        assert_eq!(
            screenshots
                .iter()
                .map(|shot| shot.id.to_string())
                .collect::<Vec<_>>(),
            [FIRST, SECOND]
        );

        assert_err_eq!(
            ProjectScreenshots::try_from(vec![screenshot(FIRST, None), screenshot(FIRST, None)]),
            RepeatedEntry { index: 1 }
        );
    }

    #[test]
    fn a_project_may_hold_no_screenshots() {
        assert!(ProjectScreenshots::default().is_empty());
        assert_ok!(ProjectScreenshots::try_from(Vec::new()));
    }

    #[rstest]
    #[case("up", ScreenshotMove::Up)]
    #[case("down", ScreenshotMove::Down)]
    fn a_screenshot_move_reads_back_from_its_field(
        #[case] value: &str,
        #[case] expected: ScreenshotMove,
    ) {
        assert_ok_eq!(value.parse::<ScreenshotMove>(), expected);
        assert_eq!(expected.to_string(), value);
    }

    #[rstest]
    #[case("sideways")]
    #[case("UP")]
    #[case("")]
    fn an_unreadable_screenshot_move_is_rejected(#[case] value: &str) {
        assert_err_eq!(
            value.parse::<ScreenshotMove>(),
            ScreenshotMoveError::Unknown
        );
    }
}
