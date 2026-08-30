use super::rows::{ProjectItemRow, ProjectLinkRow, ProjectRow};
use crate::{
    db::DbPool,
    domain::{
        Project, ProjectDescription, ProjectLink, ProjectLinkLabel, ProjectLinkUrl, ProjectLinks,
        ProjectSlug, ProjectTechnologies, TechnologyName,
    },
};

/// Replaces a project's editable fields by slug, returning whether a row
/// matched. The slug is the route identity and stays fixed.
pub async fn set(
    pool: &DbPool,
    slug: &str,
    title: &str,
    summary: &str,
    description: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"
UPDATE
    project
SET
    title = ?1,
    summary = ?2,
    description_markdown = ?3
WHERE
    slug = ?4
    "#,
        title,
        summary,
        description,
        slug
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Loads projects with Technologies and links grouped in memory, avoiding
/// queries per project.
pub async fn load(pool: &DbPool) -> Result<Vec<Project>, super::LoadError> {
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
) -> Result<Project, super::LoadError> {
    let slug = project.slug.parse::<ProjectSlug>().map_err(|source| {
        super::LoadError::InvalidProjectSlug {
            value: project.slug,
            source,
        }
    })?;
    let description = project
        .description_markdown
        .parse::<ProjectDescription>()
        .map_err(|source| super::LoadError::InvalidProjectDescription {
            slug: slug.clone(),
            source,
        })?;

    let names = technologies
        .iter()
        .filter(|row| row.project_id == project.id)
        .map(|row| {
            row.item.parse::<TechnologyName>().map_err(|source| {
                super::LoadError::InvalidTechnology {
                    slug: slug.clone(),
                    source,
                }
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let destinations = links
        .iter()
        .filter(|row| row.project_id == project.id)
        .map(|row| {
            Ok(ProjectLink {
                label: row.label.parse::<ProjectLinkLabel>().map_err(|source| {
                    super::LoadError::InvalidLinkLabel {
                        slug: slug.clone(),
                        source,
                    }
                })?,
                href: row.href.parse::<ProjectLinkUrl>().map_err(|source| {
                    super::LoadError::InvalidLinkUrl {
                        slug: slug.clone(),
                        source,
                    }
                })?,
            })
        })
        .collect::<Result<Vec<_>, super::LoadError>>()?;

    Ok(Project {
        technologies: ProjectTechnologies::try_from(names)
            .map_err(|_| super::LoadError::RepeatedTechnology { slug: slug.clone() })?,
        links: ProjectLinks::try_from(destinations)
            .map_err(|_| super::LoadError::RepeatedLinkLabel { slug: slug.clone() })?,
        slug,
        title: project.title,
        summary: project.summary,
        description,
    })
}
