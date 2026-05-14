pub struct Profile {
    pub name: &'static str,
    pub title: &'static str,
    pub summary: &'static str,
    pub email: &'static str,
    pub links: &'static [SocialLink],
}

pub struct SocialLink {
    pub label: &'static str,
    pub href: &'static str,
    pub rel: &'static str,
}

pub struct Project {
    pub name: &'static str,
    pub summary: &'static str,
    pub stack: &'static [&'static str],
    pub links: &'static [ProjectLink],
}

pub struct ProjectLink {
    pub label: &'static str,
    pub href: &'static str,
}

pub const PROFILE: Profile = Profile {
    name: "Kristofers Solo",
    title: "Rust-focused software developer building reliable web systems and developer tools.",
    summary: "I build practical software with an emphasis on Rust, typed interfaces, maintainable web systems, and tooling that makes day-to-day development simpler.",
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

pub const PROJECTS: &[Project] = &[
    Project {
        name: "kristofers.xyz",
        summary: "A single-page personal portfolio built with Rust, Leptos, Axum, server-side rendering, and a small typed content model.",
        stack: &["Rust", "Leptos", "Axum", "SCSS"],
        links: &[
            ProjectLink {
                label: "Codeberg",
                href: "https://codeberg.org/kristoferssolo",
            },
            ProjectLink {
                label: "GitHub",
                href: "https://github.com/kristoferssolo",
            },
        ],
    },
    Project {
        name: "Rust Web Services",
        summary: "Backend and service work focused on typed APIs, clear operational boundaries, and maintainable deployment surfaces.",
        stack: &["Rust", "Axum", "PostgreSQL", "Docker"],
        links: &[],
    },
    Project {
        name: "Developer Tooling",
        summary: "CLI and automation work that keeps development workflows fast, explicit, and easy to reason about.",
        stack: &["Rust", "CLI", "Automation"],
        links: &[],
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_some, assert_some_eq};

    #[test]
    fn portfolio_content_contains_public_identity() {
        assert_eq!(PROFILE.name, "Kristofers Solo");
        let mastodon = assert_some!(PROFILE.links.iter().find(|link| link.label == "Mastodon"));
        assert_eq!(mastodon.href, "https://fosstodon.org/@kristofers_solo");
        assert_some_eq!(
            PROJECTS.first().map(|project| project.name),
            "kristofers.xyz"
        );
        assert_some!(
            PROJECTS
                .iter()
                .find(|project| project.name == "kristofers.xyz")
        );
    }
}
