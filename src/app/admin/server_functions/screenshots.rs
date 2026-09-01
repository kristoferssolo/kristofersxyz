//! The server functions that add, describe, order, and remove Project
//! Screenshots.
//!
//! The upload takes a multipart body, which the pinned Leptos version encodes
//! and decodes on its own, so no hand-written Axum route is involved. Its
//! request body is bounded separately from the other server functions, because
//! it is the only one that carries an image.
//!
//! Every one of them resolves the Owner session first and answers with the
//! refreshed portfolio, so the editor and every public consumer read the same
//! content after a change.

#[cfg(feature = "ssr")]
use super::{
    ssr::{authenticated_session, reload, with_status},
    upload,
};
use crate::app::{admin::error::AdminError, content::PortfolioContent};
#[cfg(feature = "ssr")]
use crate::{
    db,
    domain::{ScreenshotAltText, ScreenshotCaption, ScreenshotId, ScreenshotMove},
    security_events::{PortfolioResource, SecurityEvent},
    startup::ApplicationState,
};
#[cfg(feature = "ssr")]
use axum::http::StatusCode;
use leptos::{
    prelude::*,
    server_fn::codec::{MultipartData, MultipartFormData},
};

/// Stores one uploaded image after the Project's current final screenshot and
/// returns the refreshed portfolio.
///
/// The browser's filename and declared content type are ignored. The stored
/// format and dimensions are the ones the decoder found, and the stored bytes
/// are the ones that were uploaded.
#[server(input = MultipartFormData, endpoint = "upload_project_screenshot")]
#[tracing::instrument(name = "Upload project screenshot", skip_all, err)]
pub async fn upload_project_screenshot(
    data: MultipartData,
) -> Result<PortfolioContent, AdminError> {
    let session = authenticated_session().await?;
    let uploaded = upload::read(data).await.map_err(rejected)?;

    let state = expect_context::<ApplicationState>();
    db::portfolio::append_project_screenshot(
        &state.pool,
        &uploaded.slug,
        uploaded.media_type,
        &uploaded.bytes,
        uploaded.size,
        &uploaded.alt,
        None,
    )
    .await
    .map_err(|error| stored(&error))?;

    changed(&session);
    reload(&state).await
}

/// Replaces one screenshot's alternative text and caption. The identity and
/// the stored image are untouched, because neither describes the other.
#[server(endpoint = "save_screenshot_details")]
#[tracing::instrument(name = "Save project screenshot details", skip_all, err)]
pub async fn save_screenshot_details(
    id: String,
    alt: String,
    #[server(default)] caption: String,
) -> Result<PortfolioContent, AdminError> {
    let session = authenticated_session().await?;
    let id = screenshot_id(&id)?;
    let alt = alt
        .parse::<ScreenshotAltText>()
        .map_err(|_| rejected(AdminError::InvalidAltText))?;
    let caption = ScreenshotCaption::parse_optional(&caption)
        .map_err(|_| rejected(AdminError::InvalidCaption))?;

    let state = expect_context::<ApplicationState>();
    db::portfolio::set_project_screenshot_details(&state.pool, &id, &alt, caption.as_ref())
        .await
        .map_err(|error| stored(&error))?;

    changed(&session);
    reload(&state).await
}

/// Moves one screenshot a single step through its Project's order.
#[server(endpoint = "move_project_screenshot")]
#[tracing::instrument(name = "Move project screenshot", skip_all, fields(movement = %movement), err)]
pub async fn move_project_screenshot(
    id: String,
    movement: String,
) -> Result<PortfolioContent, AdminError> {
    let session = authenticated_session().await?;
    let id = screenshot_id(&id)?;
    let movement = movement
        .parse::<ScreenshotMove>()
        .map_err(|_| rejected(AdminError::InvalidScreenshotMovement))?;

    let state = expect_context::<ApplicationState>();
    db::portfolio::move_project_screenshot(&state.pool, &id, movement)
        .await
        .map_err(|error| stored(&error))?;

    changed(&session);
    reload(&state).await
}

/// Removes one screenshot and its stored image.
#[server(endpoint = "delete_project_screenshot")]
#[tracing::instrument(name = "Delete project screenshot", skip_all, err)]
pub async fn delete_project_screenshot(id: String) -> Result<PortfolioContent, AdminError> {
    let session = authenticated_session().await?;
    let id = screenshot_id(&id)?;

    let state = expect_context::<ApplicationState>();
    db::portfolio::delete_project_screenshot(&state.pool, &id)
        .await
        .map_err(|error| stored(&error))?;

    changed(&session);
    reload(&state).await
}

/// Reads the identity a form carried. A field that is not an identity names no
/// screenshot, which is the same answer as one that no row holds.
#[cfg(feature = "ssr")]
fn screenshot_id(value: &str) -> Result<ScreenshotId, AdminError> {
    value
        .parse::<ScreenshotId>()
        .map_err(|_| with_status(StatusCode::NOT_FOUND, AdminError::ScreenshotNotFound))
}

/// Answers a refused change without applying any part of it.
#[cfg(feature = "ssr")]
fn rejected(error: AdminError) -> AdminError {
    with_status(StatusCode::UNPROCESSABLE_ENTITY, error)
}

/// Turns a persistence failure into the Owner-facing reason. No variant
/// carries a database message.
#[cfg(feature = "ssr")]
fn stored(error: &db::portfolio::ScreenshotError) -> AdminError {
    use db::portfolio::ScreenshotError;

    match *error {
        ScreenshotError::UnknownProject => {
            with_status(StatusCode::NOT_FOUND, AdminError::ProjectNotFound)
        }
        ScreenshotError::UnknownScreenshot | ScreenshotError::Corrupt => {
            with_status(StatusCode::NOT_FOUND, AdminError::ScreenshotNotFound)
        }
        ScreenshotError::InvalidMovement => rejected(AdminError::InvalidScreenshotMovement),
        ScreenshotError::Transaction(_) => AdminError::Save,
    }
}

/// Records the audit event every screenshot change shares.
#[cfg(feature = "ssr")]
fn changed(session: &crate::authentication::OwnerSession<crate::authentication::Authenticated>) {
    SecurityEvent::PortfolioChanged {
        owner_id: session.owner_id(),
        resource: PortfolioResource::ProjectScreenshot,
    }
    .record();
}
