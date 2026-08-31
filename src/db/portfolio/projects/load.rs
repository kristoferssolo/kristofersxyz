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

/// Finds the contiguous child-row range for one Project.
///
/// The queries above order child rows by `project_id`, so each Project can
/// locate its rows without scanning every other Project's children.
fn project_row_range<T>(
    rows: &[T],
    project_id: i64,
    row_project_id: impl Fn(&T) -> i64,
) -> std::ops::Range<usize> {
    let start = rows.partition_point(|row| row_project_id(row) < project_id);
    let tail = rows.get(start..).unwrap_or_default();
    let end = start
        .checked_add(tail.partition_point(|row| row_project_id(row) == project_id))
        .unwrap_or(rows.len());
    start..end
}

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

    let technology_range = project_row_range(technologies, project.id, |row| row.project_id);
    let technology_rows = technologies.get(technology_range).unwrap_or_default();
    let names = technology_rows
        .iter()
        .map(|row| {
            row.item
                .parse::<TechnologyName>()
                .map_err(|source| LoadError::InvalidTechnology {
                    slug: slug.clone(),
                    source,
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let link_range = project_row_range(links, project.id, |row| row.project_id);
    let link_rows = links.get(link_range).unwrap_or_default();
    let destinations = link_rows
        .iter()
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
