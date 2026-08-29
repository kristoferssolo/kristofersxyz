//! Loads the portfolio from SQLite into the content model.
//!
//! The database is the source of truth. This module owns the queries and the
//! row shapes; it maps them into [`PortfolioContent`], which stays free of any
//! persistence concern so it can also compile for the WASM client.

mod projects;
mod rows;

use self::rows::ProfileRow;
use crate::{
    app::content::{Contact, FocusArea, PortfolioContent, Profile, Site, SocialLink},
    db::DbPool,
    domain::{ProjectDescriptionError, ProjectSlug, ProjectSlugError},
};

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("failed to query portfolio content")]
    Database(#[from] sqlx::Error),
    #[error("project slug '{value}' is invalid")]
    InvalidProjectSlug {
        value: String,
        #[source]
        source: ProjectSlugError,
    },
    #[error("project '{slug}' has an invalid description")]
    InvalidProjectDescription {
        slug: ProjectSlug,
        #[source]
        source: ProjectDescriptionError,
    },
}

/// Reads the whole portfolio in a handful of ordered queries and assembles it
/// into [`PortfolioContent`]. Rows come back by `sort_order`, so the buffer
/// lines keep the order the author gave them.
///
/// # Errors
///
/// Returns [`sqlx::Error`] if a query fails or a singleton row is missing.
pub async fn load(pool: &DbPool) -> Result<PortfolioContent, LoadError> {
    let site = sqlx::query_as!(
        Site,
        r#"
SELECT
    url,
    title,
    description,
    og_image
FROM
    site
WHERE
    id = 1
    "#,
    )
    .fetch_one(pool)
    .await?;

    let contact = sqlx::query_as!(
        Contact,
        r#"
SELECT
    name,
    body
FROM
    contact
WHERE
    id = 1
    "#
    )
    .fetch_one(pool)
    .await?;

    Ok(PortfolioContent {
        site,
        profile: load_profile(pool).await?,
        projects: projects::load(pool).await?,
        contact,
    })
}

/// Reads the profile singleton with its ordered technology, working-style, and
/// link lists and assembles them into [`Profile`].
async fn load_profile(pool: &DbPool) -> Result<Profile, LoadError> {
    let profile = sqlx::query_as!(
        ProfileRow,
        r#"
SELECT
    name,
    title,
    summary,
    about,
    email
FROM
    profile
WHERE
    id = 1
    "#,
    )
    .fetch_one(pool)
    .await?;

    let technologies = sqlx::query_scalar!(
        r#"
SELECT
    item
FROM
    profile_technology
ORDER BY
    sort_order
    "#
    )
    .fetch_all(pool)
    .await?;

    let working_style = sqlx::query_as!(
        FocusArea,
        r#"
SELECT
    label,
    detail
FROM
    working_principle
ORDER BY
    sort_order
    "#,
    )
    .fetch_all(pool)
    .await?;

    let links = sqlx::query_as!(
        SocialLink,
        r#"
SELECT
    label,
    href,
    rel
FROM
    social_link
ORDER BY
    sort_order
    "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(Profile {
        name: profile.name,
        title: profile.title,
        summary: profile.summary,
        about: profile.about,
        technologies,
        working_style,
        email: profile.email,
        links,
    })
}

/// Replaces a project's editable fields by slug, returning whether the project
/// exists. The caller reloads and re-caches the portfolio so the edit takes
/// effect.
///
/// # Errors
///
/// Returns [`sqlx::Error`] if the update fails.
pub async fn set_project(
    pool: &DbPool,
    slug: &str,
    title: &str,
    summary: &str,
    description: &str,
) -> Result<bool, sqlx::Error> {
    projects::set(pool, slug, title, summary, description).await
}

/// Replaces the profile singleton's scalar fields. The caller reloads and
/// re-caches the portfolio afterward.
///
/// # Errors
///
/// Returns [`sqlx::Error`] if the update fails.
pub async fn set_profile(
    pool: &DbPool,
    name: &str,
    title: &str,
    summary: &str,
    about: &str,
    email: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
UPDATE
    profile
SET
    name = ?1,
    title = ?2,
    summary = ?3,
    about = ?4,
    email = ?5
WHERE
    id = 1
    "#,
        name,
        title,
        summary,
        about,
        email
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Replaces the contact singleton's fields. The caller reloads and re-caches
/// the portfolio afterward.
///
/// # Errors
///
/// Returns [`sqlx::Error`] if the update fails.
pub async fn set_contact(pool: &DbPool, name: &str, body: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
UPDATE
    contact
SET
    name = ?1,
    body = ?2
WHERE
    id = 1
    "#,
        name,
        body
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Replaces the site singleton's metadata. The caller reloads and re-caches the
/// portfolio afterward.
///
/// # Errors
///
/// Returns [`sqlx::Error`] if the update fails.
pub async fn set_site(
    pool: &DbPool,
    url: &str,
    title: &str,
    description: &str,
    og_image: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
UPDATE
    site
SET
    url = ?1,
    title = ?2,
    description = ?3,
    og_image = ?4
WHERE
    id = 1
    "#,
        url,
        title,
        description,
        og_image
    )
    .execute(pool)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{seed_if_empty, test_support::migrated_pool};

    async fn seeded_pool() -> DbPool {
        let pool = migrated_pool().await;
        seed_if_empty(&pool).await.expect("seed the database");
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
            content.profile.technologies,
            ["Rust", "Leptos", "Axum", "Tailwind"]
        );
        assert_eq!(content.profile.working_style.len(), 4);
        assert_eq!(content.profile.links.len(), 4);
        assert_eq!(content.contact.name, "Write to me");
    }

    #[tokio::test]
    async fn projects_keep_their_order_technologies_and_links() {
        let content = load(&seeded_pool().await)
            .await
            .expect("load the seeded portfolio");

        let names = content
            .projects
            .iter()
            .map(|project| project.slug.as_str())
            .collect::<Vec<_>>();
        assert_eq!(names, ["guenther", "traxor", "cipher-workshop"]);

        let cipher = &content.projects[2];
        assert_eq!(
            cipher.technologies,
            ["Rust", "AES-128", "CLI", "WebAssembly"]
        );
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
            assert_eq!(loaded.slug, fixture.slug);
            assert_eq!(loaded.title, fixture.title);
            assert_eq!(loaded.summary, fixture.summary);
            assert_eq!(loaded.description.as_str(), fixture.description.as_str());
            assert_eq!(loaded.technologies, fixture.technologies);
        }
    }
}
