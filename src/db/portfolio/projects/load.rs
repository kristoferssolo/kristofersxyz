//! Reading the project tables into validated domain types.

use crate::{
    db::{
        DbPool,
        portfolio::{
            LoadError,
            rows::{ProjectItemRow, ProjectLinkRow, ProjectRow, ProjectScreenshotRow},
        },
    },
    domain::{
        Project, ProjectDescription, ProjectLink, ProjectLinkLabel, ProjectLinkUrl, ProjectLinks,
        ProjectScreenshot, ProjectScreenshots, ProjectSlug, ProjectTechnologies, ScreenshotAltText,
        ScreenshotCaption, ScreenshotId, ScreenshotMediaType, ScreenshotSize, TechnologyName,
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

/// Loads every Project in the public order, with its ordered Technologies,
/// links, and Project Screenshots.
///
/// The screenshot query reads metadata only. Image bytes stay in the database
/// until the media route asks for one, so loading the portfolio never carries
/// them into memory or into the page.
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

    let screenshots = sqlx::query_as!(
        ProjectScreenshotRow,
        r#"
        SELECT
            project_id,
            screenshot_id,
            media_type,
            width,
            height,
            alt_text,
            caption
        FROM
            project_screenshot
        ORDER BY
            project_id,
            sort_order
        "#,
    )
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|project| assemble(project, &technologies, &links, &screenshots))
        .collect()
}

/// Turns one project row and the child rows that belong to it into a
/// [`Project`], validating every stored value on the way in. A row the
/// application itself wrote always passes; a row edited by hand may not.
fn assemble(
    project: ProjectRow,
    technologies: &[ProjectItemRow],
    links: &[ProjectLinkRow],
    screenshots: &[ProjectScreenshotRow],
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

    let screenshot_range = project_row_range(screenshots, project.id, |row| row.project_id);
    let screenshot_rows = screenshots.get(screenshot_range).unwrap_or_default();
    let evidence = screenshot_rows
        .iter()
        .map(|row| screenshot(row, &slug))
        .collect::<Result<Vec<_>, LoadError>>()?;

    Ok(Project {
        technologies: ProjectTechnologies::try_from(names)
            .map_err(|_| LoadError::RepeatedTechnology { slug: slug.clone() })?,
        links: ProjectLinks::try_from(destinations)
            .map_err(|_| LoadError::RepeatedLinkLabel { slug: slug.clone() })?,
        screenshots: ProjectScreenshots::try_from(evidence)
            .map_err(|_| LoadError::RepeatedScreenshot { slug: slug.clone() })?,
        slug,
        title: project.title,
        summary: project.summary,
        description,
    })
}

/// Validates one stored screenshot row into the metadata a figure renders
/// from. The bytes are not part of this row, so nothing here loads an image.
fn screenshot(
    row: &ProjectScreenshotRow,
    slug: &ProjectSlug,
) -> Result<ProjectScreenshot, LoadError> {
    Ok(ProjectScreenshot {
        id: row
            .screenshot_id
            .parse::<ScreenshotId>()
            .map_err(|source| LoadError::InvalidScreenshotId {
                slug: slug.clone(),
                source,
            })?,
        media_type: row
            .media_type
            .parse::<ScreenshotMediaType>()
            .map_err(|source| LoadError::InvalidScreenshotMediaType {
                slug: slug.clone(),
                source,
            })?,
        size: ScreenshotSize::try_from((dimension(row.width), dimension(row.height))).map_err(
            |source| LoadError::InvalidScreenshotSize {
                slug: slug.clone(),
                source,
            },
        )?,
        alt: row
            .alt_text
            .parse::<ScreenshotAltText>()
            .map_err(|source| LoadError::InvalidScreenshotAltText {
                slug: slug.clone(),
                source,
            })?,
        caption: row
            .caption
            .as_deref()
            .map(str::parse::<ScreenshotCaption>)
            .transpose()
            .map_err(|source| LoadError::InvalidScreenshotCaption {
                slug: slug.clone(),
                source,
            })?,
    })
}

/// A stored dimension as the domain type reads it. SQLite holds an integer
/// wide enough to carry a value the application never wrote, so anything
/// outside `u32` is pushed to an end the domain already rejects.
fn dimension(value: i64) -> u32 {
    u32::try_from(value).unwrap_or_else(|_| if value.is_negative() { 0 } else { u32::MAX })
}
