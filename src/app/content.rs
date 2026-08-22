//! The portfolio content model.
//!
//! [`PortfolioContent`] is the shape every view renders. It is owned and
//! serializable, so the database loader can build it and the server can embed
//! it in the page for the client to hydrate from. The database is the source
//! of truth; the static `portfolio_content` below is only a test fixture, and
//! the db loader tests assert the two never drift.

use serde::{Deserialize, Deserializer, Serialize, de};
use std::{fmt, str::FromStr};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PortfolioContent {
    pub site: Site,
    pub profile: Profile,
    pub projects: Vec<ProjectSummary>,
    pub contact: Contact,
}

/// Site level metadata. Title and description are never optional, because a
/// missing one is what search results and link previews show as a blank.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Site {
    pub url: String,
    pub title: String,
    /// The meta and Open Graph description. Reuses the profile summary, so the
    /// search snippet and the page never drift apart.
    pub description: String,
    /// Absolute, because Open Graph consumers do not resolve relative paths.
    pub og_image: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub title: String,
    pub summary: String,
    pub about: String,
    pub technologies: Vec<String>,
    /// Shown as a short list under the about text.
    pub working_style: Vec<FocusArea>,
    pub email: String,
    pub links: Vec<SocialLink>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SocialLink {
    pub label: String,
    pub href: String,
    pub rel: String,
}

/// Stable, URL-safe identity for a Project.
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

impl<'de> Deserialize<'de> for ProjectSlug {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ProjectSlugError {
    #[error("a project slug cannot be empty")]
    Empty,
    #[error("a project slug must start and end with a lowercase letter or digit")]
    InvalidEdge,
    #[error("a project slug cannot contain '{character}'")]
    InvalidCharacter { character: char },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectSummary {
    /// Stable identity used in URLs and editor state.
    pub slug: ProjectSlug,
    /// Reader-facing name, independent from the stable slug.
    pub title: String,
    pub summary: String,
    pub technologies: Vec<String>,
    pub links: Vec<ProjectLink>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectLink {
    pub label: String,
    pub href: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FocusArea {
    pub label: String,
    pub detail: String,
}

/// The contact entry. Mail only: no form, and therefore no spam surface.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Contact {
    pub name: String,
    pub body: String,
}

/// The server's copy of the loaded portfolio. `App` reads it here during SSR:
/// Leptos context from the router does not reach the shell-level component
/// tree, so the boot-loaded singleton is stored here instead.
#[cfg(feature = "ssr")]
mod server {
    use super::PortfolioContent;
    use std::sync::OnceLock;

    static CONTENT: OnceLock<PortfolioContent> = OnceLock::new();

    /// Stores the loaded portfolio. Startup calls this once, before serving.
    pub fn store(content: PortfolioContent) {
        let _ = CONTENT.set(content);
    }

    /// The loaded portfolio.
    ///
    /// # Panics
    ///
    /// Panics if called before [`store`], which startup always calls first.
    #[must_use]
    pub fn content() -> PortfolioContent {
        CONTENT
            .get()
            .expect("portfolio content is set at startup")
            .clone()
    }
}

#[cfg(feature = "ssr")]
pub use server::{content as server_content, store as store_server_content};

/// The static portfolio the reducer and page tests build their buffers from.
/// The running app loads the same content from the database instead.
#[cfg(test)]
pub(crate) use fixture::portfolio_content;

#[cfg(test)]
mod fixture {
    use super::*;

    pub fn portfolio_content() -> PortfolioContent {
        let profile = Profile {
            name: "Kristofers Solo".to_owned(),
            title:
                "Rust-focused software developer building reliable web systems and developer tools."
                    .to_owned(),
            summary: "I build practical software with an emphasis on Rust, typed interfaces, \
                  maintainable web systems and tooling that makes day-to-day development simpler."
                .to_owned(),
            about: "I focus on Rust and web systems where correctness, maintainability and clear \
                operational behavior matter. My preferred work is close to the boundary between \
                product needs and engineering infrastructure: APIs, server-rendered applications, \
                developer tools and deployment surfaces that stay understandable over time."
                .to_owned(),
            technologies: technologies(&["Rust", "Leptos", "Axum", "Tailwind"]),
            working_style: working_style(),
            email: "mailto:dev@kristofers.xyz".to_owned(),
            links: vec![
                social(
                    "Codeberg",
                    "https://codeberg.org/kristoferssolo",
                    "me noopener noreferrer",
                ),
                social(
                    "GitHub",
                    "https://github.com/kristoferssolo",
                    "me noopener noreferrer",
                ),
                social(
                    "Mastodon",
                    "https://fosstodon.org/@kristofers_solo",
                    "me noopener noreferrer",
                ),
                social("Email", "mailto:dev@kristofers.xyz", "noopener noreferrer"),
            ],
        };

        let site = Site {
            url: "https://kristofers.xyz/".to_owned(),
            title: "Kristofers Solo, Rust software developer".to_owned(),
            description: profile.summary.clone(),
            og_image: "https://kristofers.xyz/og.png".to_owned(),
        };

        PortfolioContent {
            site,
            profile,
            projects: projects(),
            contact: Contact {
                name: "Write to me".to_owned(),
                body:
                    "Mail is the fastest route. Repositories and posts sit behind the links below."
                        .to_owned(),
            },
        }
    }

    fn social(label: &str, href: &str, rel: &str) -> SocialLink {
        SocialLink {
            label: label.to_owned(),
            href: href.to_owned(),
            rel: rel.to_owned(),
        }
    }

    fn project_link(label: &str, href: &str) -> ProjectLink {
        ProjectLink {
            label: label.to_owned(),
            href: href.to_owned(),
        }
    }

    fn focus(label: &str, detail: &str) -> FocusArea {
        FocusArea {
            label: label.to_owned(),
            detail: detail.to_owned(),
        }
    }

    fn technologies(items: &[&str]) -> Vec<String> {
        items.iter().copied().map(str::to_owned).collect()
    }

    fn project_slug(value: &str) -> ProjectSlug {
        value
            .parse()
            .unwrap_or_else(|error| panic!("invalid fixture project slug: {error}"))
    }

    fn projects() -> Vec<ProjectSummary> {
        vec![
            ProjectSummary {
                slug: project_slug("guenther"),
                title: "guenther".to_owned(),
                summary: "Telegram bot that takes a social media link and sends the media back \
                      inline, so a shared post plays in the chat instead of opening a browser."
                    .to_owned(),
                technologies: technologies(&["Rust", "Telegram", "yt-dlp"]),
                links: vec![project_link(
                    "GitHub",
                    "https://github.com/kristoferssolo/guenther",
                )],
            },
            ProjectSummary {
                slug: project_slug("traxor"),
                title: "traxor".to_owned(),
                summary:
                    "Terminal UI for managing Transmission torrents: queue, inspect and control \
                      transfers without leaving the shell."
                        .to_owned(),
                technologies: technologies(&["Rust", "ratatui", "Transmission RPC"]),
                links: vec![project_link(
                    "Codeberg",
                    "https://codeberg.org/kristoferssolo/traxor",
                )],
            },
            ProjectSummary {
                slug: project_slug("cipher-workshop"),
                title: "cipher-workshop".to_owned(),
                summary:
                    "Rust workspace implementing cipher algorithms, AES-128 and CBC among them, \
                      exposed through both a CLI and a web interface."
                        .to_owned(),
                technologies: technologies(&["Rust", "AES-128", "CLI", "WebAssembly"]),
                links: vec![project_link(
                    "GitHub",
                    "https://github.com/kristoferssolo/cipher-workshop",
                )],
            },
        ]
    }

    fn working_style() -> Vec<FocusArea> {
        vec![
            focus(
                "Rust web services",
                "Backend systems with explicit data flow and predictable runtime behavior.",
            ),
            focus(
                "Typed interfaces",
                "Small contracts that make invalid states harder to express.",
            ),
            focus(
                "Pragmatic testing",
                "Coverage aimed at behavior, integrations and regression-prone edges.",
            ),
            focus(
                "Maintainable deployment surfaces",
                "Operational choices that are easy to inspect, document and repeat.",
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portfolio_leads_with_the_profile_and_the_first_project() {
        let content = portfolio_content();
        assert_eq!(content.profile.name, "Kristofers Solo");
        assert_eq!(
            content
                .projects
                .first()
                .map(|project| project.slug.as_str()),
            Some("guenther")
        );
    }

    /// The buffer is five lines: profile, three projects, contact.
    #[test]
    fn portfolio_holds_three_projects() {
        assert_eq!(portfolio_content().projects.len(), 3);
    }

    /// Link previews truncate past roughly 160 characters, and a cut sentence
    /// is what the preview then shows.
    #[test]
    fn the_meta_description_fits_a_search_result() {
        let description = portfolio_content().site.description;
        assert!(
            description.len() <= 160,
            "{} characters is too long",
            description.len()
        );
    }

    #[test]
    fn project_slugs_accept_url_safe_text() {
        use claims::assert_ok;

        assert_ok!("guenther".parse::<ProjectSlug>());
        assert_ok!("cipher-workshop-2".parse::<ProjectSlug>());
    }

    #[test]
    fn project_slugs_reject_invalid_text() {
        use claims::assert_err;

        for value in ["", "Guenther", "project page", "-project", "project-"] {
            assert_err!(value.parse::<ProjectSlug>());
        }
    }
}
