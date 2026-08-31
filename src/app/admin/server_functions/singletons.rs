//! The server functions that edit the single-row portfolio content.

#[cfg(feature = "ssr")]
use super::ssr::{authenticated_session, reload, require_fields};
use crate::app::{admin::error::AdminError, content::PortfolioContent};
#[cfg(feature = "ssr")]
use crate::{
    db,
    security_events::{PortfolioResource, SecurityEvent},
    startup::ApplicationState,
};
use leptos::prelude::*;

#[cfg(feature = "ssr")]
macro_rules! save_singleton {
    ($resource:expr, $setter:path; $($field:ident),+ $(,)?) => {{
        let session = authenticated_session().await?;
        require_fields(&[$(&$field),+])?;
        let state = expect_context::<ApplicationState>();
        ($setter)(&state.pool, $(&$field),+)
            .await
            .map_err(|_| AdminError::Save)?;
        SecurityEvent::PortfolioChanged {
            owner_id: session.owner_id(),
            resource: $resource,
        }
        .record();
        reload(&state).await
    }};
}

/// Saves the editable profile fields and returns the refreshed portfolio.
#[server(endpoint = "save_profile")]
#[tracing::instrument(name = "Save portfolio profile", skip_all, err)]
pub async fn save_profile(
    name: String,
    title: String,
    summary: String,
    about: String,
    email: String,
) -> Result<PortfolioContent, AdminError> {
    save_singleton!(PortfolioResource::Profile, db::portfolio::set_profile; name, title, summary, about, email)
}

/// Saves the editable contact fields and returns the refreshed portfolio.
#[server(endpoint = "save_contact")]
#[tracing::instrument(name = "Save portfolio contact", skip_all, err)]
pub async fn save_contact(name: String, body: String) -> Result<PortfolioContent, AdminError> {
    save_singleton!(PortfolioResource::Contact, db::portfolio::set_contact; name, body)
}

/// Saves the editable site metadata and returns the refreshed portfolio.
#[server(endpoint = "save_site")]
#[tracing::instrument(name = "Save portfolio site metadata", skip_all, err)]
pub async fn save_site(
    url: String,
    title: String,
    description: String,
    og_image: String,
) -> Result<PortfolioContent, AdminError> {
    save_singleton!(PortfolioResource::SiteMetadata, db::portfolio::set_site; url, title, description, og_image)
}
