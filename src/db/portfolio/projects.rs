use super::rows::{ProjectItemRow, ProjectLinkRow, ProjectRow};
use crate::{
    app::content::{Project, ProjectLink},
    db::DbPool,
};

/// Loads projects with stacks and links grouped in memory, avoiding queries per
/// project.
pub(super) async fn load(pool: &DbPool) -> Result<Vec<Project>, sqlx::Error> {
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
