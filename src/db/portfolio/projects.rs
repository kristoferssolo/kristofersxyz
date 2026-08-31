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
///
/// The project row and both ordered child collections change in one
/// transaction. The child rows are removed and rewritten from position one, so
/// a failure anywhere leaves the stored project exactly as it was rather than
/// half replaced.
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
    let description = description.as_str();
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
        description,
        slug
    )
    .fetch_optional(&mut *transaction)
    .await?;
    let Some(project_id) = project_id else {
        return Ok(false);
    };

    sqlx::query!(
        r#"
DELETE FROM project_technology
WHERE
    project_id = ?1
    "#,
        project_id
    )
    .execute(&mut *transaction)
    .await?;

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
        .execute(&mut *transaction)
        .await?;
    }

    sqlx::query!(
        r#"
DELETE FROM project_link
WHERE
    project_id = ?1
    "#,
        project_id
    )
    .execute(&mut *transaction)
    .await?;

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
        .execute(&mut *transaction)
        .await?;
    }

    transaction.commit().await?;
    Ok(true)
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
