//! Writing a Project and its ordered child collections.
//!
//! A Project owns two ordered child tables, so every write replaces them from
//! position one inside the same transaction as the parent row. A failure
//! anywhere leaves the stored Project exactly as it was rather than half
//! replaced.

use super::CreateError;
use crate::{
    db::DbPool,
    domain::{ProjectDescription, ProjectLinks, ProjectSlug, ProjectTechnologies},
};
use sqlx::{Sqlite, Transaction};

/// Replaces a project's editable fields by slug, returning whether a row
/// matched. The slug is the route identity and stays fixed.
///
/// # Errors
///
/// Returns [`sqlx::Error`] if the transaction fails.
pub async fn set(
    pool: &DbPool,
    slug: &ProjectSlug,
    title: &str,
    summary: &str,
    description: &ProjectDescription,
    technologies: &ProjectTechnologies,
    links: &ProjectLinks,
) -> Result<bool, sqlx::Error> {
    let slug = slug.as_str();
    let markdown = description.as_str();
    let mut transaction = pool.begin().await?;

    let project_id = sqlx::query_scalar!(
        r#"
        UPDATE
            project
        SET
            title = ?1,
            summary = ?2,
            description_markdown = ?3
        WHERE
            slug = ?4
        RETURNING
            id AS "id!"
        "#,
        title,
        summary,
        markdown,
        slug
    )
    .fetch_optional(&mut *transaction)
    .await?;
    let Some(project_id) = project_id else {
        return Ok(false);
    };

    clear_collections(&mut transaction, project_id).await?;
    write_collections(&mut transaction, project_id, technologies, links).await?;

    transaction.commit().await?;
    Ok(true)
}

/// Stores a new Project after the current final Project, with its ordered
/// Technologies and links, and publishes it immediately.
///
/// # Errors
///
/// Returns [`CreateError::DuplicateSlug`] when the slug is taken, leaving no
/// row behind, and [`CreateError::Transaction`] when the transaction fails.
pub async fn create(
    pool: &DbPool,
    slug: &ProjectSlug,
    title: &str,
    summary: &str,
    description: &ProjectDescription,
    technologies: &ProjectTechnologies,
    links: &ProjectLinks,
) -> Result<(), CreateError> {
    let slug = slug.as_str();
    let markdown = description.as_str();
    let mut transaction = pool.begin().await?;

    let taken = sqlx::query_scalar!(
        r#"
        SELECT
            EXISTS (
                SELECT
                    1
                FROM
                    project
                WHERE
                    slug = ?1
            ) AS "taken!"
        "#,
        slug
    )
    .fetch_one(&mut *transaction)
    .await?;
    if taken != 0 {
        return Err(CreateError::DuplicateSlug);
    }

    let project_id = sqlx::query_scalar!(
        r#"
        INSERT INTO
            project (slug, title, summary, description_markdown, sort_order)
        SELECT
            ?1,
            ?2,
            ?3,
            ?4,
            COALESCE(MAX(sort_order), 0) + 1
        FROM
            project
        RETURNING
            id AS "id!"
        "#,
        slug,
        title,
        summary,
        markdown
    )
    .fetch_one(&mut *transaction)
    .await?;

    write_collections(&mut transaction, project_id, technologies, links).await?;

    transaction.commit().await?;
    Ok(())
}

/// Removes the child rows of a project that is about to be rewritten.
async fn clear_collections(
    transaction: &mut Transaction<'_, Sqlite>,
    project_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        DELETE FROM
            project_technology
        WHERE
            project_id = ?1
        "#,
        project_id
    )
    .execute(&mut **transaction)
    .await?;

    sqlx::query!(
        r#"
        DELETE FROM
            project_link
        WHERE
            project_id = ?1
        "#,
        project_id
    )
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

/// Writes both ordered collections from position one, so the stored order is
/// the order the Owner arranged.
async fn write_collections(
    transaction: &mut Transaction<'_, Sqlite>,
    project_id: i64,
    technologies: &ProjectTechnologies,
    links: &ProjectLinks,
) -> Result<(), sqlx::Error> {
    let mut sort_order: i64 = 0;
    for name in technologies {
        sort_order = sort_order.saturating_add(1);
        let item = name.as_str();
        sqlx::query!(
            r#"
            INSERT INTO
                project_technology (project_id, item, sort_order)
            VALUES
                (?1, ?2, ?3)
            "#,
            project_id,
            item,
            sort_order
        )
        .execute(&mut **transaction)
        .await?;
    }

    let mut sort_order: i64 = 0;
    for link in links {
        sort_order = sort_order.saturating_add(1);
        let label = link.label.as_str();
        let href = link.href.as_str();
        sqlx::query!(
            r#"
            INSERT INTO
                project_link (project_id, label, href, sort_order)
            VALUES
                (?1, ?2, ?3, ?4)
            "#,
            project_id,
            label,
            href,
            sort_order
        )
        .execute(&mut **transaction)
        .await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db::{
            portfolio::{load, projects},
            seed_if_empty,
            test_support::migrated_pool,
        },
        domain::{Project, ProjectLink, TechnologyName},
    };
    use claims::{assert_matches, assert_ok, assert_some};

    async fn seeded_pool() -> DbPool {
        let pool = migrated_pool().await;
        seed_if_empty(&pool).await.expect("seed the database");
        pool
    }

    fn slug(value: &str) -> ProjectSlug {
        crate::test_support::parse(value)
    }

    fn technologies(items: &[&str]) -> ProjectTechnologies {
        let names = items
            .iter()
            .map(|item| crate::test_support::parse::<TechnologyName>(item))
            .collect::<Vec<_>>();
        assert_ok!(ProjectTechnologies::try_from(names))
    }

    fn links(items: &[(&str, &str)]) -> ProjectLinks {
        let links = items
            .iter()
            .map(|(label, href)| ProjectLink {
                label: crate::test_support::parse(label),
                href: crate::test_support::parse(href),
            })
            .collect::<Vec<_>>();
        assert_ok!(ProjectLinks::try_from(links))
    }

    async fn create_portfolio_project(pool: &DbPool, value: &str) -> Result<(), CreateError> {
        projects::create(
            pool,
            &slug(value),
            "kristofers.xyz",
            "The portfolio itself",
            &crate::test_support::parse::<ProjectDescription>("## What it solves\n\nA portfolio."),
            &technologies(&["Rust", "Leptos"]),
            &links(&[("GitHub", "https://github.com/kristoferssolo/kristofersxyz")]),
        )
        .await
    }

    async fn stored(pool: &DbPool) -> Vec<Project> {
        assert_ok!(load(pool).await).projects
    }

    #[tokio::test]
    async fn a_new_project_lands_after_the_final_project() {
        let pool = seeded_pool().await;

        assert_ok!(create_portfolio_project(&pool, "kristofersxyz").await);

        let slugs = stored(&pool)
            .await
            .iter()
            .map(|project| project.slug.to_string())
            .collect::<Vec<_>>();
        assert_eq!(
            slugs,
            ["guenther", "traxor", "cipher-workshop", "kristofersxyz"]
        );
    }

    #[tokio::test]
    async fn a_new_project_keeps_its_description_and_both_collections() {
        let pool = seeded_pool().await;

        assert_ok!(create_portfolio_project(&pool, "kristofersxyz").await);

        let projects = stored(&pool).await;
        let created = assert_some!(projects.last());
        assert_eq!(
            created.description.as_str(),
            "## What it solves\n\nA portfolio."
        );
        assert_eq!(
            created
                .technologies
                .iter()
                .map(TechnologyName::to_string)
                .collect::<Vec<_>>(),
            ["Rust", "Leptos"]
        );
        assert_eq!(
            created
                .links
                .iter()
                .map(|link| link.href.to_string())
                .collect::<Vec<_>>(),
            ["https://github.com/kristoferssolo/kristofersxyz"]
        );
    }

    /// The child rows are written in the same transaction as the parent, so a
    /// rejected slug cannot leave a project row or a stray Technology behind.
    #[tokio::test]
    async fn a_duplicate_slug_stores_nothing() {
        let pool = seeded_pool().await;

        let rejected = create_portfolio_project(&pool, "traxor").await;

        assert_matches!(rejected, Err(CreateError::DuplicateSlug));
        let projects = stored(&pool).await;
        assert_eq!(projects.len(), 3);
        let traxor = assert_some!(projects.get(1));
        assert_eq!(traxor.title, "traxor");
        assert_eq!(
            traxor
                .technologies
                .iter()
                .map(TechnologyName::to_string)
                .collect::<Vec<_>>(),
            ["Rust", "ratatui", "Transmission RPC"]
        );
    }
}
