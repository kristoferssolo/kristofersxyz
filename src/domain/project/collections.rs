//! The ordered child collections of a [`Project`](super::Project).
//!
//! Technology names and Project Link labels carry visible text without
//! surrounding whitespace, and a Project Link URL is an absolute HTTP or HTTPS
//! destination. Each collection keeps the order the Owner arranged and rejects
//! a repeated entry, so persistence never receives one it would have to
//! reconcile. A Project may hold neither collection.
//!
//! Rejections name the position that failed rather than the text that failed,
//! which keeps a mistyped URL out of errors and tracing.

use crate::serde_helpers::impl_deserialize_from_str;
use serde::{Deserialize, Deserializer, Serialize};
use std::{fmt, slice, str::FromStr, vec};

/// A language, framework, platform, protocol, algorithm, or major tool that
/// materially shaped a Project.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub struct TechnologyName(String);

impl TechnologyName {
    /// Returns the validated name as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TechnologyName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for TechnologyName {
    type Err = VisibleNameError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        visible_name(value).map(Self)
    }
}

impl_deserialize_from_str!(TechnologyName);

/// The reader-facing text of a Project Link, such as `GitHub`.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub struct ProjectLinkLabel(String);

impl ProjectLinkLabel {
    /// Returns the validated label as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ProjectLinkLabel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for ProjectLinkLabel {
    type Err = VisibleNameError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        visible_name(value).map(Self)
    }
}

impl_deserialize_from_str!(ProjectLinkLabel);

/// Accepts a single-line name a reader can see, with no surrounding
/// whitespace to make two entries look alike.
fn visible_name(value: &str) -> Result<String, VisibleNameError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(VisibleNameError::Empty);
    }
    if trimmed.len() != value.len() {
        return Err(VisibleNameError::SurroundingWhitespace);
    }

    Ok(value.to_owned())
}

/// Why a Technology name or Project Link label was rejected.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum VisibleNameError {
    #[error("a name needs visible text")]
    Empty,
    #[error("a name cannot start or end with whitespace")]
    SurroundingWhitespace,
}

/// An absolute HTTP or HTTPS destination for a Project Link.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub struct ProjectLinkUrl(String);

impl ProjectLinkUrl {
    /// Returns the validated URL as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ProjectLinkUrl {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for ProjectLinkUrl {
    type Err = ProjectLinkUrlError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value
            .chars()
            .any(|character| character.is_whitespace() || character.is_control())
        {
            return Err(ProjectLinkUrlError::Whitespace);
        }

        let (scheme, rest) = value
            .split_once("://")
            .ok_or(ProjectLinkUrlError::NotAbsolute)?;
        if !scheme.eq_ignore_ascii_case("http") && !scheme.eq_ignore_ascii_case("https") {
            return Err(ProjectLinkUrlError::Scheme);
        }

        let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
        if authority.is_empty() {
            return Err(ProjectLinkUrlError::MissingHost);
        }
        if authority.contains('@') {
            return Err(ProjectLinkUrlError::Credentials);
        }

        Ok(Self(value.to_owned()))
    }
}

impl_deserialize_from_str!(ProjectLinkUrl);

/// Why a Project Link URL was rejected. No variant carries part of the URL,
/// so a mistyped destination cannot reach an error message or a log line.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ProjectLinkUrlError {
    #[error("a project link cannot contain whitespace")]
    Whitespace,
    #[error("a project link must be an absolute URL")]
    NotAbsolute,
    #[error("a project link must use http or https")]
    Scheme,
    #[error("a project link must name a host")]
    MissingHost,
    #[error("a project link cannot carry credentials")]
    Credentials,
}

/// A labelled destination shown on a Project Detail.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProjectLink {
    pub label: ProjectLinkLabel,
    pub href: ProjectLinkUrl,
}

/// A Project's Technologies, in the order the Owner arranged them and with no
/// name repeated.
#[derive(Clone, Debug, Default, Serialize)]
#[serde(transparent)]
pub struct ProjectTechnologies(Vec<TechnologyName>);

impl ProjectTechnologies {
    /// Returns the names in their stored order.
    #[must_use]
    pub fn as_slice(&self) -> &[TechnologyName] {
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

    pub fn iter(&self) -> slice::Iter<'_, TechnologyName> {
        self.0.iter()
    }
}

impl TryFrom<Vec<TechnologyName>> for ProjectTechnologies {
    type Error = RepeatedEntry;

    fn try_from(names: Vec<TechnologyName>) -> Result<Self, Self::Error> {
        first_repeat(&names).map_or(Ok(Self(names)), |index| Err(RepeatedEntry { index }))
    }
}

impl<'de> Deserialize<'de> for ProjectTechnologies {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Vec::<TechnologyName>::deserialize(deserializer)?
            .try_into()
            .map_err(serde::de::Error::custom)
    }
}

impl IntoIterator for ProjectTechnologies {
    type Item = TechnologyName;
    type IntoIter = vec::IntoIter<TechnologyName>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a ProjectTechnologies {
    type Item = &'a TechnologyName;
    type IntoIter = slice::Iter<'a, TechnologyName>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// A Project's links, in the order the Owner arranged them and with no label
/// repeated. Two links may share a destination; their labels distinguish them.
#[derive(Clone, Debug, Default, Serialize)]
#[serde(transparent)]
pub struct ProjectLinks(Vec<ProjectLink>);

impl ProjectLinks {
    /// Returns the links in their stored order.
    #[must_use]
    pub fn as_slice(&self) -> &[ProjectLink] {
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

    pub fn iter(&self) -> slice::Iter<'_, ProjectLink> {
        self.0.iter()
    }
}

impl TryFrom<Vec<ProjectLink>> for ProjectLinks {
    type Error = RepeatedEntry;

    fn try_from(links: Vec<ProjectLink>) -> Result<Self, Self::Error> {
        let labels = links.iter().map(|link| &link.label).collect::<Vec<_>>();
        first_repeat(&labels).map_or(Ok(Self(links)), |index| Err(RepeatedEntry { index }))
    }
}

impl<'de> Deserialize<'de> for ProjectLinks {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Vec::<ProjectLink>::deserialize(deserializer)?
            .try_into()
            .map_err(serde::de::Error::custom)
    }
}

impl IntoIterator for ProjectLinks {
    type Item = ProjectLink;
    type IntoIter = vec::IntoIter<ProjectLink>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a ProjectLinks {
    type Item = &'a ProjectLink;
    type IntoIter = slice::Iter<'a, ProjectLink>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// A collection held an entry that an earlier position already used. `index`
/// is the zero-based position of the later entry, so a caller can point the
/// Owner at the row to change.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("an entry repeats one at an earlier position")]
pub struct RepeatedEntry {
    pub index: usize,
}

/// The position of the first entry that an earlier position already used.
/// Both collections stay short, so the pairwise scan avoids the ordering and
/// hashing bounds a set would demand of the entry types.
fn first_repeat<T: PartialEq>(items: &[T]) -> Option<usize> {
    items
        .iter()
        .enumerate()
        .find(|(index, item)| items.iter().take(*index).any(|earlier| earlier == *item))
        .map(|(index, _)| index)
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_err, assert_err_eq, assert_ok, assert_ok_eq};
    use rstest::rstest;

    fn technology(value: &str) -> TechnologyName {
        crate::test_support::parse(value)
    }

    fn link(label: &str, href: &str) -> ProjectLink {
        ProjectLink {
            label: crate::test_support::parse(label),
            href: crate::test_support::parse(href),
        }
    }

    #[rstest]
    #[case("Rust")]
    #[case("SQLx and SQLite")]
    #[case("C++")]
    fn a_name_accepts_visible_text(#[case] value: &str) {
        assert_ok_eq!(value.parse::<TechnologyName>(), technology(value));
        assert_ok!(value.parse::<ProjectLinkLabel>());
    }

    #[rstest]
    #[case("", VisibleNameError::Empty)]
    #[case("   ", VisibleNameError::Empty)]
    #[case("\n\t", VisibleNameError::Empty)]
    #[case(" Rust", VisibleNameError::SurroundingWhitespace)]
    #[case("Rust ", VisibleNameError::SurroundingWhitespace)]
    fn a_name_rejects_blank_and_padded_text(
        #[case] value: &str,
        #[case] expected: VisibleNameError,
    ) {
        assert_err_eq!(value.parse::<TechnologyName>(), expected);
        assert_err_eq!(value.parse::<ProjectLinkLabel>(), expected);
    }

    #[rstest]
    #[case("https://github.com/kristoferssolo/guenther")]
    #[case("http://localhost:3000/work")]
    #[case("HTTPS://codeberg.org")]
    #[case("https://example.com/path?query=1#fragment")]
    fn a_project_link_url_accepts_absolute_http(#[case] value: &str) {
        assert_ok!(value.parse::<ProjectLinkUrl>());
    }

    #[rstest]
    #[case("github.com/kristoferssolo", ProjectLinkUrlError::NotAbsolute)]
    #[case("/work/guenther", ProjectLinkUrlError::NotAbsolute)]
    #[case("mailto:dev@kristofers.xyz", ProjectLinkUrlError::NotAbsolute)]
    #[case("ftp://example.com", ProjectLinkUrlError::Scheme)]
    #[case("javascript://example.com", ProjectLinkUrlError::Scheme)]
    #[case("https:///work", ProjectLinkUrlError::MissingHost)]
    #[case("https://owner:hunter2@example.com", ProjectLinkUrlError::Credentials)]
    #[case("https://example.com/a b", ProjectLinkUrlError::Whitespace)]
    #[case(" https://example.com", ProjectLinkUrlError::Whitespace)]
    fn a_project_link_url_rejects_anything_else(
        #[case] value: &str,
        #[case] expected: ProjectLinkUrlError,
    ) {
        assert_err_eq!(value.parse::<ProjectLinkUrl>(), expected);
    }

    /// The URL is the one field an Owner could paste a secret into, so no
    /// rejection may quote it back.
    #[test]
    fn a_rejected_url_stays_out_of_its_error() {
        let error = assert_err!("https://owner:hunter2@example.com".parse::<ProjectLinkUrl>());

        assert!(!error.to_string().contains("hunter2"));
    }

    #[test]
    fn a_collection_keeps_the_order_it_was_given() {
        let names = ["Rust", "teloxide", "Cobalt"].map(technology).to_vec();

        let technologies = assert_ok!(ProjectTechnologies::try_from(names));

        assert_eq!(
            technologies
                .iter()
                .map(TechnologyName::as_str)
                .collect::<Vec<_>>(),
            ["Rust", "teloxide", "Cobalt"]
        );
    }

    #[test]
    fn a_collection_may_be_empty() {
        assert_ok!(ProjectTechnologies::try_from(Vec::new()));
        assert_ok!(ProjectLinks::try_from(Vec::new()));
    }

    #[test]
    fn technologies_reject_a_repeated_name() {
        let names = ["Rust", "teloxide", "Rust"].map(technology).to_vec();

        assert_err_eq!(
            ProjectTechnologies::try_from(names),
            RepeatedEntry { index: 2 }
        );
    }

    #[test]
    fn links_reject_a_repeated_label_but_not_a_repeated_url() {
        let repeated_label = vec![
            link("GitHub", "https://github.com/kristoferssolo/guenther"),
            link("GitHub", "https://codeberg.org/kristoferssolo/guenther"),
        ];
        assert_err_eq!(
            ProjectLinks::try_from(repeated_label),
            RepeatedEntry { index: 1 }
        );

        let repeated_url = vec![
            link("GitHub", "https://github.com/kristoferssolo/guenther"),
            link("Source", "https://github.com/kristoferssolo/guenther"),
        ];
        assert_ok!(ProjectLinks::try_from(repeated_url));
    }
}
