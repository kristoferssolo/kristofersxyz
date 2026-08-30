use crate::{
    app::{admin::error::AdminError, content::PortfolioContent},
    domain::Username,
};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::field::Empty;

#[cfg(feature = "ssr")]
mod ssr;

#[cfg(feature = "ssr")]
use self::ssr::{
    authenticated_session, invalid_credentials, reload, require_fields, session_state,
    too_many_attempts, with_status,
};
#[cfg(feature = "ssr")]
use crate::{
    authentication::{
        AuthError, Credentials, OwnerSessionError, Password, RetryAfter, SessionState,
    },
    db,
    security_events::{
        AuthenticationFailure, LoginThrottleScope, PortfolioResource, SecurityEvent,
    },
    startup::ApplicationState,
};
#[cfg(feature = "ssr")]
use axum::extract::ConnectInfo;
#[cfg(feature = "ssr")]
use axum::http::StatusCode;
#[cfg(feature = "ssr")]
use leptos_axum::redirect;
#[cfg(feature = "ssr")]
use std::net::SocketAddr;

/// The owner identity exposed to authenticated Leptos routes.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SessionUser {
    pub username: Username,
}

/// Returns the owner for a complete authenticated session.
#[server(endpoint = "admin_session")]
#[tracing::instrument(name = "Get current owner session", skip_all, err)]
pub async fn current_user() -> Result<Option<SessionUser>, AdminError> {
    match session_state().await? {
        SessionState::Anonymous(_) => {
            redirect("/login");
            Ok(None)
        }
        SessionState::Authenticated(session) => Ok(Some(SessionUser {
            username: session.username().clone(),
        })),
    }
}

/// Verifies credentials and starts an owner session.
#[server(endpoint = "login")]
#[tracing::instrument(
    name = "Log in owner",
    skip_all,
    fields(
        username = Empty,
        owner_id = Empty,
    ),
    err,
)]
pub async fn login(username: String, password: String) -> Result<(), AdminError> {
    let session = match session_state().await? {
        SessionState::Authenticated(_) => {
            redirect("/admin");
            return Ok(());
        }
        SessionState::Anonymous(session) => session,
    };
    let state = expect_context::<ApplicationState>();
    let ConnectInfo(peer) = leptos_axum::extract::<ConnectInfo<SocketAddr>>()
        .await
        .map_err(|_| {
            SecurityEvent::AuthenticationFailed {
                username: None,
                reason: AuthenticationFailure::Internal,
            }
            .record();
            AdminError::Internal
        })?;
    if let Err(retry_after) = state.login_throttle.check_source(peer.ip()) {
        SecurityEvent::LoginThrottled {
            username: None,
            scope: LoginThrottleScope::Source,
            retry_after,
        }
        .record();
        return Err(too_many_attempts(retry_after));
    }
    let username = Username::new(username).map_err(|_| {
        SecurityEvent::AuthenticationFailed {
            username: None,
            reason: AuthenticationFailure::InvalidInput,
        }
        .record();
        invalid_credentials()
    })?;
    tracing::Span::current().record("username", tracing::field::display(&username));
    if let Err(retry_after) = state.login_throttle.check_account(&username) {
        SecurityEvent::LoginThrottled {
            username: Some(&username),
            scope: LoginThrottleScope::Account,
            retry_after,
        }
        .record();
        return Err(too_many_attempts(retry_after));
    }
    let password = Password::try_from(password).map_err(|_| {
        state.login_throttle.record_failure(&username);
        SecurityEvent::AuthenticationFailed {
            username: Some(&username),
            reason: AuthenticationFailure::InvalidInput,
        }
        .record();
        invalid_credentials()
    })?;
    let credentials = Credentials {
        username: username.clone(),
        password,
    };

    let owner = match session.authenticate(credentials).await {
        Ok(Some(owner)) => {
            state.login_throttle.record_success(&username);
            owner
        }
        Ok(None) | Err(OwnerSessionError::Backend(AuthError::InvalidCredentials)) => {
            state.login_throttle.record_failure(&username);
            SecurityEvent::AuthenticationFailed {
                username: Some(&username),
                reason: AuthenticationFailure::InvalidCredentials,
            }
            .record();
            return Err(invalid_credentials());
        }
        Err(OwnerSessionError::Backend(AuthError::PasswordTasksUnavailable)) => {
            let retry_after = RetryAfter::password_verification_busy();
            SecurityEvent::LoginThrottled {
                username: Some(&username),
                scope: LoginThrottleScope::PasswordCapacity,
                retry_after,
            }
            .record();
            return Err(too_many_attempts(retry_after));
        }
        Err(_) => {
            SecurityEvent::AuthenticationFailed {
                username: Some(&username),
                reason: AuthenticationFailure::Internal,
            }
            .record();
            return Err(AdminError::Internal);
        }
    };
    tracing::Span::current().record("owner_id", tracing::field::display(owner.id()));
    session
        .sign_in(&state.pool, owner)
        .await
        .map_err(|_| AdminError::Internal)?;
    redirect("/admin");
    Ok(())
}

/// Ends an authenticated owner session.
#[server(endpoint = "logout")]
#[tracing::instrument(name = "Log out owner", skip_all, err)]
pub async fn logout() -> Result<(), AdminError> {
    if let SessionState::Authenticated(session) = session_state().await? {
        session.sign_out().await.map_err(|_| AdminError::Internal)?;
    }
    redirect("/");
    Ok(())
}

/// Saves the editable project fields and returns the refreshed portfolio.
#[server(endpoint = "save_project")]
#[tracing::instrument(
    name = "Save portfolio project",
    skip_all,
    fields(slug = %slug),
    err,
)]
pub async fn save_project(
    slug: String,
    title: String,
    summary: String,
    markdown: String,
) -> Result<PortfolioContent, AdminError> {
    let session = authenticated_session().await?;
    require_fields(&[&title, &summary, &markdown])?;
    let state = expect_context::<ApplicationState>();
    let saved = db::portfolio::set_project(&state.pool, &slug, &title, &summary, &markdown)
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
