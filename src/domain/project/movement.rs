//! How the Owner asks to move a Project through the public order.
//!
//! The public order is the one sequence every reader-facing consumer follows,
//! so a move is described by intent rather than by a position the caller
//! calculated. [`ProjectMove`] survives a form round trip as one field, which
//! keeps the editor's buttons working before hydration.

use std::{fmt, str::FromStr};

use super::{ProjectSlug, ProjectSlugError};

/// The prefix of the anchored variant's wire form, `place-of:<slug>`.
const PLACE_OF: &str = "place-of:";

/// A requested move through the public project order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProjectMove {
    /// One position toward the start of the order.
    Up,
    /// One position toward the end of the order.
    Down,
    /// To the position another Project holds, which is what a drag reports.
    /// Naming the anchor rather than an index keeps a request from a stale
    /// page from landing somewhere the Owner did not point at.
    ToPlaceOf(ProjectSlug),
    /// To the end of the public order, which is the drop target after the
    /// final Project.
    ToEnd,
}

impl fmt::Display for ProjectMove {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Up => formatter.write_str("up"),
            Self::Down => formatter.write_str("down"),
            Self::ToPlaceOf(slug) => write!(formatter, "{PLACE_OF}{slug}"),
            Self::ToEnd => formatter.write_str("to-end"),
        }
    }
}

impl FromStr for ProjectMove {
    type Err = ProjectMoveError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "up" => Ok(Self::Up),
            "down" => Ok(Self::Down),
            "to-end" => Ok(Self::ToEnd),
            other => other
                .strip_prefix(PLACE_OF)
                .ok_or(ProjectMoveError::Unknown)?
                .parse::<ProjectSlug>()
                .map(Self::ToPlaceOf)
                .map_err(|source| ProjectMoveError::Anchor { source }),
        }
    }
}

/// Why a submitted move could not be read. Neither variant repeats the text it
/// was given, so a crafted field cannot echo through an error message.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ProjectMoveError {
    #[error("a project move must be up, down, or a place to take")]
    Unknown,
    #[error("a project move names an invalid project")]
    Anchor {
        #[source]
        source: ProjectSlugError,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_err_eq, assert_ok_eq};
    use rstest::rstest;

    #[rstest]
    #[case("up", ProjectMove::Up)]
    #[case("down", ProjectMove::Down)]
    #[case("to-end", ProjectMove::ToEnd)]
    fn a_step_reads_back_from_its_field(#[case] value: &str, #[case] expected: ProjectMove) {
        assert_ok_eq!(value.parse::<ProjectMove>(), expected);
        assert_eq!(expected.to_string(), value);
    }

    #[test]
    fn an_anchored_move_carries_a_validated_slug() {
        let movement = assert_ok_eq!(
            "place-of:cipher-workshop".parse::<ProjectMove>(),
            ProjectMove::ToPlaceOf(crate::test_support::parse("cipher-workshop"))
        );

        assert_eq!(movement.to_string(), "place-of:cipher-workshop");
    }

    #[rstest]
    #[case("sideways")]
    #[case("")]
    #[case("UP")]
    fn an_unreadable_move_is_rejected(#[case] value: &str) {
        assert_err_eq!(value.parse::<ProjectMove>(), ProjectMoveError::Unknown);
    }

    #[test]
    fn an_anchor_that_is_not_a_slug_is_rejected() {
        assert_err_eq!(
            "place-of:Cipher Workshop".parse::<ProjectMove>(),
            ProjectMoveError::Anchor {
                source: ProjectSlugError::InvalidEdge
            }
        );
    }
}
