//! The portfolio content model.
//!
//! Every view renders [`PortfolioContent`]. The server loads it from SQLite and
//! serializes it into the page for hydration. Tests use the static fixture and
//! check that it matches the seed database.

use crate::domain::Project;
#[cfg(test)]
use crate::domain::{ProjectDescription, ProjectLink, ProjectSlug};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// The portfolio content shared by every route.
///
/// Admin saves replace the value before client-side navigation, so the public
/// pages and metadata render the same content as the server cache.
#[derive(Clone, Copy)]
pub struct Portfolio(RwSignal<PortfolioContent>);

impl Portfolio {
    #[must_use]
    pub fn new(content: PortfolioContent) -> Self {
        Self(RwSignal::new(content))
    }

    #[must_use]
    pub fn current(self) -> PortfolioContent {
        self.0.get()
    }

    pub fn replace(self, content: PortfolioContent) {
        self.0.set(content);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PortfolioContent {
    pub site: Site,
    pub profile: Profile,
    pub projects: Vec<Project>,
    pub contact: Contact,
}

/// Required metadata for browser titles, search results, and link previews.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Site {
    pub url: String,
    pub title: String,
    /// Shared by the meta description and Open Graph tags.
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

/// The mail-only contact entry.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Contact {
    pub name: String,
    pub body: String,
}

/// Server-side portfolio storage for the shell, which cannot read router
/// context during SSR.
#[cfg(feature = "ssr")]
mod server {
    use super::PortfolioContent;
    use arc_swap::ArcSwap;
    use std::sync::{Arc, OnceLock};

    /// [`ArcSwap`] gives renders lock-free reads and admin edits atomic writes.
    static CONTENT: OnceLock<ArcSwap<PortfolioContent>> = OnceLock::new();

    /// Replaces the portfolio at startup or after an admin edit.
    pub fn store(content: PortfolioContent) {
        let content = Arc::new(content);
        match CONTENT.get() {
            Some(cell) => cell.store(content),
            None => {
                let _ = CONTENT.set(ArcSwap::new(content));
            }
        }
    }

    /// Returns the current portfolio in a cloned [`Arc`].
    ///
    /// # Process termination
    ///
    /// Aborts if called before [`store`], which startup always calls first.
    #[must_use]
    pub fn content() -> Arc<PortfolioContent> {
        let Some(content) = CONTENT.get() else {
            std::process::abort();
        };
        content.load_full()
    }
}

#[cfg(feature = "ssr")]
pub use server::{content as server_content, store as store_server_content};

/// Static content used by reducer and page tests.
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
