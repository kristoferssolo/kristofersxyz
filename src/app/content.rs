//! The portfolio content model.
//!
//! [`PortfolioContent`] is the shape every view renders. It is owned and
//! serializable, so it can cross the server-to-client boundary as a Leptos
//! resource and be built from database rows. Views read it through
//! [`portfolio_content`]; the database is the source of truth, and this
//! function is the fixture the reducer tests and the current page still read
//! until the loader is wired in.

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
    pub stack: Vec<String>,
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
pub struct Project {
    pub name: String,
    pub summary: String,
    pub stack: Vec<String>,
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

/// The single seam every view reads content through.
#[must_use]
pub fn portfolio_content() -> PortfolioContent {
    let profile = Profile {
        name: "Kristofers Solo".to_owned(),
        title: "Rust-focused software developer building reliable web systems and developer tools."
            .to_owned(),
        summary: "I build practical software with an emphasis on Rust, typed interfaces, \
                  maintainable web systems and tooling that makes day-to-day development simpler."
            .to_owned(),
        about: "I focus on Rust and web systems where correctness, maintainability and clear \
                operational behavior matter. My preferred work is close to the boundary between \
                product needs and engineering infrastructure: APIs, server-rendered applications, \
                developer tools and deployment surfaces that stay understandable over time."
            .to_owned(),
        stack: stack(&["Rust", "Leptos", "Axum", "Tailwind"]),
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
            body: "Mail is the fastest route. Repositories and posts sit behind the links below."
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

fn stack(items: &[&str]) -> Vec<String> {
    items.iter().copied().map(str::to_owned).collect()
}

/// Entry names double as slugs, so they stay lowercase and URL safe.
fn projects() -> Vec<Project> {
    vec![
        Project {
            name: "guenther".to_owned(),
            summary: "Telegram bot that takes a social media link and sends the media back \
                      inline, so a shared post plays in the chat instead of opening a browser."
                .to_owned(),
            stack: stack(&["Rust", "Telegram", "yt-dlp"]),
            links: vec![project_link(
                "GitHub",
                "https://github.com/kristoferssolo/guenther",
            )],
        },
        Project {
            name: "traxor".to_owned(),
            summary: "Terminal UI for managing Transmission torrents: queue, inspect and control \
                      transfers without leaving the shell."
                .to_owned(),
            stack: stack(&["Rust", "ratatui", "Transmission RPC"]),
            links: vec![project_link(
                "Codeberg",
                "https://codeberg.org/kristoferssolo/traxor",
            )],
        },
        Project {
            name: "cipher-workshop".to_owned(),
            summary: "Rust workspace implementing cipher algorithms, AES-128 and CBC among them, \
                      exposed through both a CLI and a web interface."
                .to_owned(),
            stack: stack(&["Rust", "AES-128", "CLI", "WebAssembly"]),
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
                .map(|project| project.name.as_str()),
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

    /// Names double as URL slugs, so a capital or a space here would show up
    /// as a broken hash fragment.
    #[test]
    fn project_names_are_slug_safe() {
        for project in portfolio_content().projects {
            assert!(
                project
                    .name
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
                "{} is not slug safe",
                project.name
            );
        }
    }
}
