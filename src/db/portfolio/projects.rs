use super::rows::{ProjectItemRow, ProjectLinkRow, ProjectRow};
use crate::{
    db::DbPool,
    domain::{Project, ProjectDescription, ProjectLink, ProjectSlug},
};

/// Loads projects with Technologies and links grouped in memory, avoiding
/// queries per project.
pub(super) async fn load(pool: &DbPool) -> Result<Vec<Project>, super::LoadError> {
    let rows = sqlx::query_as::<_, ProjectRow>(
        "SELECT id, slug, title, summary, description_markdown FROM project ORDER BY sort_order",
    )
    .fetch_all(pool)
    .await?;

    let technologies = sqlx::query_as::<_, ProjectItemRow>(
        "SELECT project_id, item FROM project_technology ORDER BY project_id, sort_order",
    )
    .fetch_all(pool)
    .await?;

    let links = sqlx::query_as::<_, ProjectLinkRow>(
        "SELECT project_id, label, href FROM project_link ORDER BY project_id, sort_order",
    )
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|project| {
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

            Ok(Project {
                technologies: technologies
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
                slug,
                title: project.title,
                summary: project.summary,
                description,
            })
        })
        .collect()
}
