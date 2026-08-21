//! The portfolio as static data.
//!
//! Everything the views render comes from [`portfolio_content`]. Views never
//! reach for the consts below, so when this content moves to SQLite the
//! function becomes an async loader and the page components keep their shape.

pub struct PortfolioContent {
    pub site: Site,
    pub profile: Profile,
    pub projects: &'static [Project],
    pub contact: Contact,
}

/// Site level metadata. Title and description are never optional, because a
/// missing one is what search results and link previews show as a blank.
#[derive(Clone, Copy)]
pub struct Site {
    pub url: &'static str,
    pub title: &'static str,
    /// The meta and Open Graph description. Reuses the profile summary, so the
    /// search snippet and the page never drift apart.
    pub description: &'static str,
    /// Absolute, because Open Graph consumers do not resolve relative paths.
    pub og_image: &'static str,
}

#[derive(Clone, Copy)]
pub struct Profile {
    pub name: &'static str,
    pub title: &'static str,
    pub summary: &'static str,
    pub about: &'static str,
    pub stack: &'static [&'static str],
    /// Shown as a short list under the about text.
    pub working_style: &'static [FocusArea],
    pub email: &'static str,
    pub links: &'static [SocialLink],
}

#[derive(Clone, Copy)]
pub struct SocialLink {
    pub label: &'static str,
    pub href: &'static str,
    pub rel: &'static str,
}

#[derive(Clone, Copy)]
pub struct Project {
    pub name: &'static str,
    pub summary: &'static str,
    pub stack: &'static [&'static str],
    pub links: &'static [ProjectLink],
}

#[derive(Clone, Copy)]
pub struct ProjectLink {
    pub label: &'static str,
    pub href: &'static str,
}

#[derive(Clone, Copy)]
pub struct FocusArea {
    pub label: &'static str,
    pub detail: &'static str,
}

/// The contact entry. Mail only: no form, and therefore no spam surface.
#[derive(Clone, Copy)]
pub struct Contact {
    pub name: &'static str,
    pub body: &'static str,
}

/// The single seam every view reads content through.
#[must_use]
pub const fn portfolio_content() -> PortfolioContent {
    PORTFOLIO
}

const SITE: Site = Site {
    url: "https://kristofers.xyz/",
    title: "Kristofers Solo, Rust software developer",
    description: PROFILE.summary,
    og_image: "https://kristofers.xyz/og.png",
};

const PROFILE: Profile = Profile {
    name: "Kristofers Solo",
    title: "Rust-focused software developer building reliable web systems and developer tools.",
    summary: "I build practical software with an emphasis on Rust, typed interfaces, maintainable web systems and tooling that makes day-to-day development simpler.",
    about: "I focus on Rust and web systems where correctness, maintainability and clear operational behavior matter. My preferred work is close to the boundary between product needs and engineering infrastructure: APIs, server-rendered applications, developer tools and deployment surfaces that stay understandable over time.",
    stack: &["Rust", "Leptos", "Axum", "Tailwind"],
    working_style: WORKING_STYLE,
    email: "mailto:dev@kristofers.xyz",
    links: &[
        SocialLink {
            label: "Codeberg",
            href: "https://codeberg.org/kristoferssolo",
            rel: "me noopener noreferrer",
        },
        SocialLink {
            label: "GitHub",
            href: "https://github.com/kristoferssolo",
            rel: "me noopener noreferrer",
        },
        SocialLink {
            label: "Mastodon",
            href: "https://fosstodon.org/@kristofers_solo",
            rel: "me noopener noreferrer",
        },
        SocialLink {
            label: "Email",
            href: "mailto:dev@kristofers.xyz",
            rel: "noopener noreferrer",
        },
    ],
};

/// Entry names double as slugs, so they stay lowercase and URL safe.
const PROJECTS: &[Project] = &[
    Project {
        name: "guenther",
        summary: "Telegram bot that takes a social media link and sends the media back inline, so a shared post plays in the chat instead of opening a browser.",
        stack: &["Rust", "Telegram", "yt-dlp"],
        links: &[ProjectLink {
            label: "GitHub",
            href: "https://github.com/kristoferssolo/guenther",
        }],
    },
    Project {
        name: "traxor",
        summary: "Terminal UI for managing Transmission torrents: queue, inspect and control transfers without leaving the shell.",
        stack: &["Rust", "ratatui", "Transmission RPC"],
        links: &[ProjectLink {
            label: "Codeberg",
            href: "https://codeberg.org/kristoferssolo/traxor",
        }],
    },
    Project {
        name: "cipher-workshop",
        summary: "Rust workspace implementing cipher algorithms, AES-128 and CBC among them, exposed through both a CLI and a web interface.",
        stack: &["Rust", "AES-128", "CLI", "WebAssembly"],
        links: &[ProjectLink {
            label: "GitHub",
            href: "https://github.com/kristoferssolo/cipher-workshop",
        }],
    },
];

const WORKING_STYLE: &[FocusArea] = &[
    FocusArea {
        label: "Rust web services",
        detail: "Backend systems with explicit data flow and predictable runtime behavior.",
    },
    FocusArea {
        label: "Typed interfaces",
        detail: "Small contracts that make invalid states harder to express.",
    },
    FocusArea {
        label: "Pragmatic testing",
        detail: "Coverage aimed at behavior, integrations and regression-prone edges.",
    },
    FocusArea {
        label: "Maintainable deployment surfaces",
        detail: "Operational choices that are easy to inspect, document and repeat.",
    },
];

const CONTACT: Contact = Contact {
    name: "Write to me",
    body: "Mail is the fastest route. Repositories and posts sit behind the links below.",
};

const PORTFOLIO: PortfolioContent = PortfolioContent {
    site: SITE,
    profile: PROFILE,
    projects: PROJECTS,
    contact: CONTACT,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portfolio_leads_with_the_profile_and_the_first_project() {
        let content = portfolio_content();
        assert_eq!(content.profile.name, "Kristofers Solo");
        assert_eq!(
            content.projects.first().map(|project| project.name),
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
