//! Reading the project tables into validated domain types.

use crate::{
    db::{
        DbPool,
        portfolio::{
            LoadError,
            rows::{ProjectItemRow, ProjectLinkRow, ProjectRow},
        },
    },
    domain::{
        Project, ProjectDescription, ProjectLink, ProjectLinkLabel, ProjectLinkUrl, ProjectLinks,
        ProjectSlug, ProjectTechnologies, TechnologyName,
    },
};

/// Loads every Project in the public order, with its ordered Technologies
/// and links.
///
/// # Errors
///
/// Returns [`LoadError`] when a query fails or a stored value no longer parses
/// into its domain type.
pub async fn load(pool: &DbPool) -> Result<Vec<Project>, LoadError> {
    let rows = sqlx::query_as!(
        ProjectRow,
        r#"
SELECT
    id,
    slug,
    title,
    summary,
    description_markdown
FROM
    project
ORDER BY
    sort_order
    "#,
    )
    .fetch_all(pool)
    .await?;

    let technologies = sqlx::query_as!(
        ProjectItemRow,
        r#"
SELECT
    project_id,
    item
FROM
    project_technology
ORDER BY
    project_id,
    sort_order
    "#,
    )
    .fetch_all(pool)
    .await?;

    let links = sqlx::query_as!(
        ProjectLinkRow,
        r#"
SELECT
    project_id,
    label,
    href
FROM
    project_link
ORDER BY
    project_id,
    sort_order
    "#,
    )
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|project| assemble(project, &technologies, &links))
        .collect()
}

/// Turns one project row and the child rows that belong to it into a
/// [`Project`], validating every stored value on the way in. A row the
/// application itself wrote always passes; a row edited by hand may not.
fn assemble(
    project: ProjectRow,
    technologies: &[ProjectItemRow],
    links: &[ProjectLinkRow],
) -> Result<Project, LoadError> {
    let slug =
        project
            .slug
            .parse::<ProjectSlug>()
            .map_err(|source| LoadError::InvalidProjectSlug {
                value: project.slug,
                source,
            })?;
    let description = project
        .description_markdown
        .parse::<ProjectDescription>()
        .map_err(|source| LoadError::InvalidProjectDescription {
            slug: slug.clone(),
            source,
        })?;

    let names = technologies
        .iter()
        .filter(|row| row.project_id == project.id)
        .map(|row| {
            row.item
                .parse::<TechnologyName>()
                .map_err(|source| LoadError::InvalidTechnology {
                    slug: slug.clone(),
                    source,
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let destinations = links
        .iter()
        .filter(|row| row.project_id == project.id)
        .map(|row| {
            Ok(ProjectLink {
                label: row.label.parse::<ProjectLinkLabel>().map_err(|source| {
                    LoadError::InvalidLinkLabel {
                        slug: slug.clone(),
                        source,
                    }
                })?,
                href: row.href.parse::<ProjectLinkUrl>().map_err(|source| {
                    LoadError::InvalidLinkUrl {
                        slug: slug.clone(),
                        source,
                    }
                })?,
            })
        })
        .collect::<Result<Vec<_>, LoadError>>()?;

    Ok(Project {
        technologies: ProjectTechnologies::try_from(names)
            .map_err(|_| LoadError::RepeatedTechnology { slug: slug.clone() })?,
        links: ProjectLinks::try_from(destinations)
            .map_err(|_| LoadError::RepeatedLinkLabel { slug: slug.clone() })?,
        slug,
        title: project.title,
        summary: project.summary,
        description,
    })
}
