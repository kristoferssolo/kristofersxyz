//! Loads the portfolio from SQLite into the content model.
//!
//! The database is the source of truth. This module owns the queries and the
//! row shapes; it maps them into [`PortfolioContent`], which stays free of any
//! persistence concern so it can also compile for the WASM client.

use crate::app::content::{
    Contact, FocusArea, PortfolioContent, Profile, Project, ProjectLink, Site, SocialLink,
};
use crate::db::DbPool;

/// Reads the whole portfolio in a handful of ordered queries and assembles it
/// into [`PortfolioContent`]. Rows come back by `sort_order`, so the buffer
/// lines keep the order the author gave them.
///
/// # Errors
///
/// Returns [`sqlx::Error`] if a query fails or a singleton row is missing.
pub async fn load(pool: &DbPool) -> Result<PortfolioContent, sqlx::Error> {
    let site = sqlx::query_as::<_, SiteRow>(
        "SELECT url, title, description, og_image FROM site WHERE id = 1",
    )
    .fetch_one(pool)
    .await?;

    let profile = sqlx::query_as::<_, ProfileRow>(
        "SELECT name, title, summary, about, email FROM profile WHERE id = 1",
    )
    .fetch_one(pool)
    .await?;

    let stack =
        sqlx::query_scalar::<_, String>("SELECT item FROM profile_stack ORDER BY sort_order")
            .fetch_all(pool)
            .await?;

    let working_style = sqlx::query_as::<_, FocusRow>(
        "SELECT label, detail FROM working_principle ORDER BY sort_order",
    )
    .fetch_all(pool)
    .await?;

    let links = sqlx::query_as::<_, SocialRow>(
        "SELECT label, href, rel FROM social_link ORDER BY sort_order",
    )
    .fetch_all(pool)
    .await?;

    let contact = sqlx::query_as::<_, ContactRow>("SELECT name, body FROM contact WHERE id = 1")
        .fetch_one(pool)
        .await?;

    Ok(PortfolioContent {
        site: Site {
            url: site.url,
            title: site.title,
            description: site.description,
            og_image: site.og_image,
        },
        profile: Profile {
            name: profile.name,
            title: profile.title,
            summary: profile.summary,
            about: profile.about,
            stack,
            working_style: working_style
                .into_iter()
                .map(|row| FocusArea {
                    label: row.label,
                    detail: row.detail,
                })
                .collect(),
            email: profile.email,
            links: links
                .into_iter()
                .map(|row| SocialLink {
                    label: row.label,
                    href: row.href,
                    rel: row.rel,
                })
                .collect(),
        },
        projects: projects(pool).await?,
        contact: Contact {
            name: contact.name,
            body: contact.body,
        },
    })
}

/// Projects, with each project's stack and links grouped in memory rather than
/// queried per row, so the whole set costs three queries.
async fn projects(pool: &DbPool) -> Result<Vec<Project>, sqlx::Error> {
    let rows = sqlx::query_as::<_, ProjectRow>(
        "SELECT id, name, summary FROM project ORDER BY sort_order",
    )
    .fetch_all(pool)
    .await?;

    let stacks = sqlx::query_as::<_, ProjectItemRow>(
        "SELECT project_id, item FROM project_stack ORDER BY project_id, sort_order",
    )
    .fetch_all(pool)
    .await?;

    let links = sqlx::query_as::<_, ProjectLinkRow>(
        "SELECT project_id, label, href FROM project_link ORDER BY project_id, sort_order",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|project| Project {
            stack: stacks
                .iter()
                .filter(|row| row.project_id == project.id)
                .map(|row| row.item.clone())
                .collect(),
            links: links
                .iter()
                .filter(|row| row.project_id == project.id)
                .map(|row| ProjectLink {
                    label: row.label.clone(),
                    href: row.href.clone(),
                })
                .collect(),
            name: project.name,
            summary: project.summary,
        })
        .collect())
}

#[derive(sqlx::FromRow)]
struct SiteRow {
    url: String,
    title: String,
    description: String,
    og_image: String,
}

#[derive(sqlx::FromRow)]
struct ProfileRow {
    name: String,
    title: String,
    summary: String,
    about: String,
    email: String,
}

#[derive(sqlx::FromRow)]
struct FocusRow {
    label: String,
    detail: String,
}

#[derive(sqlx::FromRow)]
struct SocialRow {
    label: String,
    href: String,
    rel: String,
}

#[derive(sqlx::FromRow)]
struct ContactRow {
    name: String,
    body: String,
}

#[derive(sqlx::FromRow)]
struct ProjectRow {
    id: i64,
    name: String,
    summary: String,
}

#[derive(sqlx::FromRow)]
struct ProjectItemRow {
    project_id: i64,
    item: String,
}

#[derive(sqlx::FromRow)]
struct ProjectLinkRow {
    project_id: i64,
    label: String,
    href: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    /// A single-connection in-memory database: SQLite gives each connection
    /// its own `:memory:` database, so a wider pool would migrate one and
    /// query another. Migrated and seeded from the same files the app ships.
    async fn seeded_pool() -> DbPool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("connect to an in-memory database");

        sqlx::migrate!()
            .run(&pool)
            .await
            .expect("run the migrations");

        sqlx::raw_sql(include_str!("../../seeds/portfolio.sql"))
            .execute(&pool)
            .await
            .expect("apply the seed");

        pool
    }

    #[tokio::test]
    async fn the_seed_loads_into_the_content_model() {
        let content = load(&seeded_pool().await)
            .await
            .expect("load the seeded portfolio");

        assert_eq!(
            content.site.title,
            "Kristofers Solo, Rust software developer"
        );
        assert_eq!(content.profile.name, "Kristofers Solo");
        assert_eq!(
            content.profile.stack,
            ["Rust", "Leptos", "Axum", "Tailwind"]
        );
        assert_eq!(content.profile.working_style.len(), 4);
        assert_eq!(content.profile.links.len(), 4);
        assert_eq!(content.contact.name, "Write to me");
    }

    #[tokio::test]
    async fn projects_keep_their_order_stacks_and_links() {
        let content = load(&seeded_pool().await)
            .await
            .expect("load the seeded portfolio");

        let names = content
            .projects
            .iter()
            .map(|project| project.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(names, ["guenther", "traxor", "cipher-workshop"]);

        let cipher = &content.projects[2];
        assert_eq!(cipher.stack, ["Rust", "AES-128", "CLI", "WebAssembly"]);
        assert_eq!(cipher.links.len(), 1);
        assert_eq!(cipher.links[0].label, "GitHub");
        assert_eq!(
            cipher.links[0].href,
            "https://github.com/kristoferssolo/cipher-workshop"
        );
    }

    /// The seed exists to reproduce the static fixture exactly. If they drift,
    /// the page renders different content depending on where it was loaded.
    #[tokio::test]
    async fn the_seed_matches_the_static_fixture() {
        use crate::app::content::portfolio_content;

        let loaded = load(&seeded_pool().await)
            .await
            .expect("load the seeded portfolio");
        let fixture = portfolio_content();

        assert_eq!(loaded.profile.about, fixture.profile.about);
        assert_eq!(loaded.projects.len(), fixture.projects.len());
        for (loaded, fixture) in loaded.projects.iter().zip(&fixture.projects) {
            assert_eq!(loaded.name, fixture.name);
            assert_eq!(loaded.summary, fixture.summary);
            assert_eq!(loaded.stack, fixture.stack);
        }
    }
}
