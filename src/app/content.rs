use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use sqlx::{FromRow, SqlitePool};
#[cfg(feature = "ssr")]
use std::collections::BTreeMap;
#[cfg(feature = "ssr")]
use thiserror::Error;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PortfolioContent {
    pub profile: Profile,
    pub projects: Vec<Project>,
    pub working_style: Vec<FocusArea>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Profile {
    pub name: String,
    pub title: String,
    pub summary: String,
    pub about: String,
    pub email: String,
    pub links: Vec<SocialLink>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SocialLink {
    pub label: String,
    pub href: String,
    pub rel: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Project {
    pub name: String,
    pub summary: String,
    pub stack: Vec<String>,
    pub links: Vec<ProjectLink>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProjectLink {
    pub label: String,
    pub href: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FocusArea {
    pub label: String,
    pub detail: String,
}

#[server]
pub async fn get_portfolio_content() -> Result<PortfolioContent, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let pool = expect_context::<SqlitePool>();
        return load_portfolio_content(&pool)
            .await
            .map_err(ServerFnError::from);
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new(
            "portfolio content is only available on the server",
        ))
    }
}

#[cfg(feature = "ssr")]
#[derive(Debug, Error)]
pub enum ContentStoreError {
    #[error("failed to query portfolio content")]
    Query(#[from] sqlx::Error),
    #[error("missing profile row in portfolio database")]
    MissingProfile,
}

#[cfg(feature = "ssr")]
#[derive(FromRow)]
struct ProfileRow {
    name: String,
    title: String,
    summary: String,
    about: String,
    email: String,
}

#[cfg(feature = "ssr")]
#[derive(FromRow)]
struct SocialLinkRow {
    label: String,
    href: String,
    rel: String,
}

#[cfg(feature = "ssr")]
#[derive(FromRow)]
struct ProjectRow {
    id: i64,
    name: String,
    summary: String,
}

#[cfg(feature = "ssr")]
#[derive(FromRow)]
struct ProjectStackRow {
    project_id: i64,
    stack: String,
}

#[cfg(feature = "ssr")]
#[derive(FromRow)]
struct ProjectLinkRow {
    project_id: i64,
    label: String,
    href: String,
}

#[cfg(feature = "ssr")]
#[derive(FromRow)]
struct FocusAreaRow {
    label: String,
    detail: String,
}

#[cfg(feature = "ssr")]
pub async fn load_portfolio_content(
    pool: &SqlitePool,
) -> Result<PortfolioContent, ContentStoreError> {
    let profile = sqlx::query_as::<_, ProfileRow>(
        "SELECT name, title, summary, about, email FROM profile LIMIT 1",
    )
    .fetch_optional(pool)
    .await?
    .ok_or(ContentStoreError::MissingProfile)?;

    let links = sqlx::query_as::<_, SocialLinkRow>(
        "SELECT label, href, rel FROM social_links ORDER BY sort_order",
    )
    .fetch_all(pool)
    .await?;

    let projects = sqlx::query_as::<_, ProjectRow>(
        "SELECT id, name, summary FROM projects ORDER BY sort_order",
    )
    .fetch_all(pool)
    .await?;

    let stacks = sqlx::query_as::<_, ProjectStackRow>(
        "SELECT project_id, stack FROM project_stacks ORDER BY project_id, sort_order",
    )
    .fetch_all(pool)
    .await?;

    let project_links = sqlx::query_as::<_, ProjectLinkRow>(
        "SELECT project_id, label, href FROM project_links ORDER BY project_id, sort_order",
    )
    .fetch_all(pool)
    .await?;

    let working_style = sqlx::query_as::<_, FocusAreaRow>(
        "SELECT label, detail FROM focus_areas ORDER BY sort_order",
    )
    .fetch_all(pool)
    .await?;

    let mut stacks_by_project = BTreeMap::<i64, Vec<String>>::new();
    for row in stacks {
        stacks_by_project
            .entry(row.project_id)
            .or_default()
            .push(row.stack);
    }

    let mut links_by_project = BTreeMap::<i64, Vec<ProjectLink>>::new();
    for row in project_links {
        links_by_project
            .entry(row.project_id)
            .or_default()
            .push(ProjectLink {
                label: row.label,
                href: row.href,
            });
    }

    Ok(PortfolioContent {
        profile: Profile {
            name: profile.name,
            title: profile.title,
            summary: profile.summary,
            about: profile.about,
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
        projects: projects
            .into_iter()
            .map(|row| Project {
                name: row.name,
                summary: row.summary,
                stack: stacks_by_project.remove(&row.id).unwrap_or_default(),
                links: links_by_project.remove(&row.id).unwrap_or_default(),
            })
            .collect(),
        working_style: working_style
            .into_iter()
            .map(|row| FocusArea {
                label: row.label,
                detail: row.detail,
            })
            .collect(),
    })
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use claims::assert_some_eq;
    use sqlx::SqlitePool;

    use super::load_portfolio_content;

    #[tokio::test]
    async fn loads_seeded_portfolio_content() {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("in-memory sqlite pool");

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("migrations should apply");

        let content = load_portfolio_content(&pool)
            .await
            .expect("seeded content should load");

        assert_eq!(content.profile.name, "Kristofers Solo");
        assert_some_eq!(
            content
                .projects
                .first()
                .map(|project| project.name.as_str()),
            "kristofers.xyz"
        );
    }
}
