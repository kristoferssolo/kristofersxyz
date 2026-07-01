pub struct PortfolioContent {
    pub profile: Profile,
    pub projects: &'static [Project],
    pub working_style: &'static [FocusArea],
}

#[derive(Clone, Copy)]
pub struct Profile {
    pub name: &'static str,
    pub title: &'static str,
    pub summary: &'static str,
    pub about: &'static str,
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

pub const PROFILE: Profile = Profile {
    name: "Kristofers Solo",
    title: "Rust-focused software developer building reliable web systems and developer tools.",
    summary: "I build practical software with an emphasis on Rust, typed interfaces, maintainable web systems and tooling that makes day-to-day development simpler.",
    about: "I focus on Rust and web systems where correctness, maintainability and clear operational behavior matter. My preferred work is close to the boundary between product needs and engineering infrastructure: APIs, server-rendered applications, developer tools and deployment surfaces that stay understandable over time.",
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
        summary: "A terminal-styled personal portfolio built with Rust, Leptos, Axum and server-side rendering.",
        stack: &["Rust", "Leptos", "Axum", "Tailwind"],
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
        summary: "Backend and service work focused on typed APIs, clear operational boundaries and maintainable deployment surfaces.",
        stack: &["Rust", "Axum", "PostgreSQL", "Docker"],
        links: &[],
    },
    Project {
        name: "Developer Tooling",
        summary: "CLI and automation work that keeps development workflows fast, explicit and easy to reason about.",
        stack: &["Rust", "CLI", "Automation"],
        links: &[],
    },
];

pub const WORKING_STYLE: &[FocusArea] = &[
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

pub const PORTFOLIO: PortfolioContent = PortfolioContent {
    profile: PROFILE,
    projects: PROJECTS,
    working_style: WORKING_STYLE,
};

#[cfg(test)]
mod tests {
    use super::{PORTFOLIO, PROJECTS};

    #[test]
    fn portfolio_contains_root_identity_and_project() {
        assert_eq!(PORTFOLIO.profile.name, "Kristofers Solo");
        assert_eq!(
            PROJECTS.first().map(|project| project.name),
            Some("kristofers.xyz")
        );
    }
}
