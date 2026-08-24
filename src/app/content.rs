//! The portfolio content model.
//!
//! [`PortfolioContent`] is the shape every view renders. It is owned and
//! serializable, so the database loader can build it and the server can embed
//! it in the page for the client to hydrate from. The database is the source
//! of truth; the static `portfolio_content` below is only a test fixture, and
//! the db loader tests assert the two never drift.

use crate::domain::Project;
#[cfg(test)]
use crate::domain::{ProjectDescription, ProjectLink, ProjectSlug};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PortfolioContent {
    pub site: Site,
    pub profile: Profile,
    pub projects: Vec<Project>,
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
    use arc_swap::ArcSwap;
    use std::sync::{Arc, OnceLock};

    /// The live portfolio. An [`ArcSwap`] so page renders read it without a
    /// lock on the hot path, while a content edit can replace it atomically.
    static CONTENT: OnceLock<ArcSwap<PortfolioContent>> = OnceLock::new();

    /// Sets the current portfolio: once at boot, and again after every edit.
    /// A later render sees the new value on its next read.
    pub fn store(content: PortfolioContent) {
        let content = Arc::new(content);
        match CONTENT.get() {
            Some(cell) => cell.store(content),
            None => {
                let _ = CONTENT.set(ArcSwap::new(content));
            }
        }
    }

    /// The current portfolio, cheap to clone since it hands back an [`Arc`].
    ///
    /// # Panics
    ///
    /// Panics if called before [`store`], which startup always calls first.
    #[must_use]
    pub fn content() -> Arc<PortfolioContent> {
        CONTENT
            .get()
            .expect("portfolio content is set at startup")
            .load_full()
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

    fn project_description(value: &str) -> ProjectDescription {
        value
            .parse()
            .unwrap_or_else(|error| panic!("invalid fixture project description: {error}"))
    }

    fn projects() -> Vec<Project> {
        vec![
            Project {
                slug: project_slug("guenther"),
                title: "guenther".to_owned(),
                summary: "Telegram bot that takes a social media link and sends the media back \
                      inline, so a shared post plays in the chat instead of opening a browser."
                    .to_owned(),
                description: project_description(
                    "## What it solves\n\nGuenther turns supported Instagram, TikTok, X, and \
                     YouTube Shorts links into media that plays inside the Telegram conversation. \
                     Public posts stay in the chat instead of sending everyone through a browser \
                     or login prompt.\n\n## System shape\n\n```text\nTelegram update\n    -> \
                     Guenther router\n    -> private Cobalt sidecar\n    -> Telegram media response\n\
                     ```\n\nThe Rust process classifies each URL, sends the download request to Cobalt, \
                     and builds the Telegram response. Optional modules handle F1 schedules, \
                     persistent bingo games, and reusable voice lines without making those \
                     dependencies mandatory for the media path.\n\n## Engineering evidence\n\nThe \
                     bingo module stores concurrent chat-local games in SQLite through SQLx. Entry \
                     imports are transactional, and existing cards keep their original entry text \
                     after later edits.\n\nThe Compose deployment keeps Cobalt on a private network \
                     with no host port. An optional proxy applies only to Cobalt traffic, so it \
                     never receives the Telegram bot token.",
                ),
                technologies: technologies(&[
                    "Rust",
                    "teloxide",
                    "Cobalt",
                    "SQLx and SQLite",
                    "Docker Compose",
                ]),
                links: vec![project_link(
                    "GitHub",
                    "https://github.com/kristoferssolo/guenther",
                )],
            },
            Project {
                slug: project_slug("traxor"),
                title: "traxor".to_owned(),
                summary:
                    "Terminal UI for managing Transmission torrents: queue, inspect and control \
                      transfers without leaving the shell."
                        .to_owned(),
                description: project_description(
                    "## What it solves\n\nTorrent operations stay in the terminal.\n\n## System\n\n\
                     Traxor presents Transmission RPC state through a keyboard-driven terminal \
                     interface.",
                ),
                technologies: technologies(&["Rust", "ratatui", "Transmission RPC"]),
                links: vec![project_link(
                    "Codeberg",
                    "https://codeberg.org/kristoferssolo/traxor",
                )],
            },
            Project {
                slug: project_slug("cipher-workshop"),
                title: "cipher-workshop".to_owned(),
                summary:
                    "Rust workspace implementing cipher algorithms, AES-128 and CBC among them, \
                      exposed through both a CLI and a web interface."
                        .to_owned(),
                description: project_description(
                    "## What it explores\n\nCipher implementations share one Rust workspace and \
                     support command-line and browser interfaces.",
                ),
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
}
