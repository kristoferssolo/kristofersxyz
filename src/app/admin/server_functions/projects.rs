//! The server functions that edit, create, and order Projects.

use crate::app::{admin::error::AdminError, content::PortfolioContent};
#[cfg(feature = "ssr")]
use crate::{
    db,
    domain::{ProjectDescription, ProjectSlug},
    security_events::{PortfolioResource, SecurityEvent},
    startup::ApplicationState,
};
#[cfg(feature = "ssr")]
use super::ssr::{
    authenticated_session, reload, require_fields, validated_links, validated_technologies,
    with_status,
};
#[cfg(feature = "ssr")]
use axum::http::StatusCode;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

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
