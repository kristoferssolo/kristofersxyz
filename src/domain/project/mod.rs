//! Project domain types.
//!
//! Constructors validate the route slug and require a non-empty description
//! before persistence or rendering can use a Project. The ordered Technology
//! and Project Link collections live in [`collections`].

mod collections;
mod movement;

pub use self::collections::{
    ProjectLink, ProjectLinkLabel, ProjectLinkUrl, ProjectLinkUrlError, ProjectLinks,
    ProjectTechnologies, RepeatedEntry, TechnologyName, VisibleNameError,
};
pub use self::movement::{ProjectMove, ProjectMoveError};
use crate::serde_helpers::impl_deserialize_from_str;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Project {
    pub slug: ProjectSlug,
    pub title: String,
    pub summary: String,
    pub description: ProjectDescription,
    pub technologies: ProjectTechnologies,
    pub links: ProjectLinks,
}

impl Project {
    /// The canonical public route for this project.
    #[must_use]
    pub fn path(&self) -> String {
        Self::path_for_slug(&self.slug)
    }

    /// Builds the canonical public route from a validated project slug.
    #[must_use]
    pub fn path_for_slug(slug: &ProjectSlug) -> String {
        format!("/work/{slug}")
    }
}

/// Stable, URL-safe identity for a [`Project`].
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub struct ProjectSlug(String);

impl ProjectSlug {
    /// Returns the validated slug as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ProjectSlug {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for ProjectSlug {
    type Err = ProjectSlugError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let first = value.chars().next().ok_or(ProjectSlugError::Empty)?;
        let last = value.chars().next_back().ok_or(ProjectSlugError::Empty)?;
        if !first.is_ascii_lowercase() && !first.is_ascii_digit()
            || !last.is_ascii_lowercase() && !last.is_ascii_digit()
        {
            return Err(ProjectSlugError::InvalidEdge);
        }

        if let Some(character) = value.chars().find(|character| {
            !character.is_ascii_lowercase() && !character.is_ascii_digit() && *character != '-'
        }) {
            return Err(ProjectSlugError::InvalidCharacter { character });
        }

        Ok(Self(value.to_owned()))
    }
}

impl_deserialize_from_str!(ProjectSlug);

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ProjectSlugError {
    #[error("a project slug cannot be empty")]
    Empty,
    #[error("a project slug must start and end with a lowercase letter or digit")]
    InvalidEdge,
    #[error("a project slug cannot contain '{character}'")]
    InvalidCharacter { character: char },
}

/// A non-empty, Markdown-authored project description.
#[derive(Clone, Debug, Serialize)]
#[serde(transparent)]
pub struct ProjectDescription(String);

impl ProjectDescription {
    /// Returns the validated Markdown source.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for ProjectDescription {
    type Err = ProjectDescriptionError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.trim().is_empty() {
            return Err(ProjectDescriptionError::Empty);
        }

        Ok(Self(value.to_owned()))
    }
}

impl_deserialize_from_str!(ProjectDescription);

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ProjectDescriptionError {
    #[error("a project description cannot be empty")]
    Empty,
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_err, assert_ok};

    #[test]
    fn project_slugs_accept_url_safe_text() {
        assert_ok!("guenther".parse::<ProjectSlug>());
        assert_ok!("cipher-workshop-2".parse::<ProjectSlug>());
    }

    #[test]
    fn project_slugs_reject_invalid_text() {
        for value in ["", "Guenther", "project page", "-project", "project-"] {
            assert_err!(value.parse::<ProjectSlug>());
        }
    }

    #[test]
    fn project_description_requires_visible_content() {
        assert_err!("\n  \t".parse::<ProjectDescription>());
        assert_ok!("# What it solves".parse::<ProjectDescription>());
    }

    #[test]
    fn project_path_uses_the_validated_slug() {
        let project = Project {
            slug: crate::test_support::parse("guenther"),
            title: "guenther".to_owned(),
            summary: "Telegram media bot".to_owned(),
            description: crate::test_support::parse("# What it solves"),
            technologies: ProjectTechnologies::default(),
            links: ProjectLinks::default(),
        };

        assert_eq!(project.path(), "/work/guenther");
    }
}
