//! Project Screenshots, the visual Project Evidence a Project Detail shows.
//!
//! A generated identity addresses immutable bytes. Reader-facing metadata stays
//! separate from the stored image, and each Project keeps its screenshots ordered.

mod identity;
mod metadata;
mod movement;

pub use identity::{ScreenshotId, ScreenshotIdError};
pub use metadata::{
    MAX_SCREENSHOT_BYTES, MAX_SCREENSHOT_EDGE, ScreenshotAltText, ScreenshotCaption,
    ScreenshotMediaType, ScreenshotMediaTypeError, ScreenshotSize, ScreenshotSizeError,
};
pub use movement::{ScreenshotMove, ScreenshotMoveError};

use super::collections::{RepeatedEntry, first_repeat};
use serde::{Deserialize, Deserializer, Serialize};
use std::{slice, vec};

/// The route prefix that serves a screenshot's stored bytes.
pub const SCREENSHOT_MEDIA_PREFIX: &str = "/media/project";

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::VisibleNameError;
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
