//! The server functions that edit, create, and order Projects.

#[cfg(feature = "ssr")]
use super::ssr::{
    authenticated_session, reload, require_fields, validated_links, validated_technologies,
    with_status,
};
use crate::app::{admin::error::AdminError, content::PortfolioContent};
#[cfg(feature = "ssr")]
use crate::{
    db,
    domain::{ProjectDescription, ProjectMove, ProjectSlug},
    security_events::{PortfolioResource, SecurityEvent},
    startup::ApplicationState,
};
#[cfg(feature = "ssr")]
use axum::http::StatusCode;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// Route segments under `/admin/project/` that address a page rather than a
/// Project. A Project holding one of these slugs would have no reachable
/// editor.
#[cfg(feature = "ssr")]
const RESERVED_SLUGS: [&str; 1] = ["new"];

/// The label and URL of one Project Link as a form encodes them. Leptos needs
/// a plain shape to decode into; [`save_project`] converts it into a validated
/// [`ProjectLink`](crate::domain::ProjectLink) before anything is stored.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProjectLinkField {
    pub label: String,
    pub href: String,
}

/// Saves the editable project fields, Technologies, and links, and returns the
/// refreshed portfolio. Both collections may be empty. The slug identifies the
/// project and does not change.
#[server(endpoint = "save_project")]
#[tracing::instrument(
    name = "Save portfolio project",
    skip_all,
    fields(
        slug = %slug,
        technologies = technologies.len(),
        links = links.len(),
    ),
    err,
)]
pub async fn save_project(
    slug: String,
    title: String,
    summary: String,
    markdown: String,
    #[server(default)] technologies: Vec<String>,
    #[server(default)] links: Vec<ProjectLinkField>,
) -> Result<PortfolioContent, AdminError> {
    let session = authenticated_session().await?;
    require_fields(&[&title, &summary])?;
    let slug = slug
        .parse::<ProjectSlug>()
        .map_err(|_| with_status(StatusCode::NOT_FOUND, AdminError::ProjectNotFound))?;
    let description = markdown
        .parse::<ProjectDescription>()
        .map_err(|_| with_status(StatusCode::UNPROCESSABLE_ENTITY, AdminError::MissingField))?;
    let technologies = validated_technologies(&technologies)?;
    let links = validated_links(&links)?;

    let state = expect_context::<ApplicationState>();
    let saved = db::portfolio::set_project(
        &state.pool,
        &slug,
        &title,
        &summary,
        &description,
        &technologies,
        &links,
    )
    .await
    .map_err(|_| AdminError::Save)?;
    if !saved {
        return Err(with_status(
            StatusCode::NOT_FOUND,
            AdminError::ProjectNotFound,
        ));
    }
    SecurityEvent::PortfolioChanged {
        owner_id: session.owner_id(),
        resource: PortfolioResource::Project,
    }
    .record();
    reload(&state).await
}

/// Creates a Project after the current final Project and returns the refreshed
/// portfolio. The slug is the route identity and cannot change afterwards, so
/// it is validated before anything else is read.
///
/// The Project is public as soon as it is stored. Both collections may be
/// empty.
#[server(endpoint = "create_project")]
#[tracing::instrument(
    name = "Create portfolio project",
    skip_all,
    fields(
        slug = %slug,
        technologies = technologies.len(),
        links = links.len(),
    ),
    err,
)]
pub async fn create_project(
    slug: String,
    title: String,
    summary: String,
    markdown: String,
    #[server(default)] technologies: Vec<String>,
    #[server(default)] links: Vec<ProjectLinkField>,
) -> Result<PortfolioContent, AdminError> {
    let session = authenticated_session().await?;
    require_fields(&[&title, &summary])?;
    let slug = slug
        .parse::<ProjectSlug>()
        .map_err(|_| with_status(StatusCode::UNPROCESSABLE_ENTITY, AdminError::InvalidSlug))?;
    if RESERVED_SLUGS.contains(&slug.as_str()) {
        return Err(with_status(
            StatusCode::UNPROCESSABLE_ENTITY,
            AdminError::ReservedSlug,
        ));
    }
    let description = markdown
        .parse::<ProjectDescription>()
        .map_err(|_| with_status(StatusCode::UNPROCESSABLE_ENTITY, AdminError::MissingField))?;
    let technologies = validated_technologies(&technologies)?;
    let links = validated_links(&links)?;

    let state = expect_context::<ApplicationState>();
    db::portfolio::create_project(
        &state.pool,
        &slug,
        &title,
        &summary,
        &description,
        &technologies,
        &links,
    )
    .await
    .map_err(|error| match error {
        db::portfolio::CreateError::DuplicateSlug => {
            with_status(StatusCode::CONFLICT, AdminError::SlugTaken)
        }
        db::portfolio::CreateError::Transaction(_) => AdminError::Save,
    })?;
    SecurityEvent::PortfolioChanged {
        owner_id: session.owner_id(),
        resource: PortfolioResource::Project,
    }
    .record();
    reload(&state).await
}

/// Moves one Project through the public order and returns the refreshed
/// portfolio, which every ordered consumer then reads.
#[server(endpoint = "move_project")]
#[tracing::instrument(
    name = "Move portfolio project",
    skip_all,
    fields(slug = %slug, movement = %movement),
    err,
)]
pub async fn move_project(slug: String, movement: String) -> Result<PortfolioContent, AdminError> {
    let session = authenticated_session().await?;
    let slug = slug
        .parse::<ProjectSlug>()
        .map_err(|_| with_status(StatusCode::NOT_FOUND, AdminError::ProjectNotFound))?;
    let movement = movement.parse::<ProjectMove>().map_err(|_| {
        with_status(
            StatusCode::UNPROCESSABLE_ENTITY,
            AdminError::InvalidMovement,
        )
    })?;

    let state = expect_context::<ApplicationState>();
    db::portfolio::move_project(&state.pool, &slug, &movement)
        .await
        .map_err(|error| match error {
            db::portfolio::MoveError::UnknownProject => {
                with_status(StatusCode::NOT_FOUND, AdminError::ProjectNotFound)
            }
            db::portfolio::MoveError::InvalidMovement => with_status(
                StatusCode::UNPROCESSABLE_ENTITY,
                AdminError::InvalidMovement,
            ),
            db::portfolio::MoveError::Transaction(_) => AdminError::Save,
        })?;
    SecurityEvent::PortfolioChanged {
        owner_id: session.owner_id(),
        resource: PortfolioResource::Project,
    }
    .record();
    reload(&state).await
}
