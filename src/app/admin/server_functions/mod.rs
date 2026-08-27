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
        AuthError, Credentials, Password, RetryAfter, SessionState, validate_credentials,
    },
    db,
    startup::AppState,
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
    let state = expect_context::<AppState>();
    let ConnectInfo(peer) = leptos_axum::extract::<ConnectInfo<SocketAddr>>()
        .await
        .map_err(|_| AdminError::Internal)?;
    state
        .login_throttle
        .check_source(peer.ip())
        .map_err(too_many_attempts)?;
    let username = Username::new(username).map_err(|_| invalid_credentials())?;
    tracing::Span::current().record("username", tracing::field::display(&username));
    state
        .login_throttle
        .check_account(&username)
        .map_err(too_many_attempts)?;
    let password = Password::try_from(password).map_err(|_| {
        state.login_throttle.record_failure(&username);
        invalid_credentials()
    })?;
    let credentials = Credentials {
        username: username.clone(),
        password,
    };

    let owner_id = match validate_credentials(credentials, &state.pool).await {
        Ok(owner_id) => {
            state.login_throttle.record_success(&username);
            owner_id
        }
        Err(AuthError::InvalidCredentials) => {
            state.login_throttle.record_failure(&username);
            return Err(invalid_credentials());
        }
        Err(AuthError::PasswordTasksUnavailable) => {
            return Err(too_many_attempts(RetryAfter::password_verification_busy()));
        }
        Err(_) => return Err(AdminError::Internal),
    };
    tracing::Span::current().record("owner_id", tracing::field::display(owner_id));
    session
        .sign_in(&state.pool, owner_id, username)
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
    let _session = authenticated_session().await?;
    require_fields(&[&title, &summary, &markdown])?;
    let state = expect_context::<AppState>();
    let saved = db::portfolio::set_project(&state.pool, &slug, &title, &summary, &markdown)
        .await
        .map_err(|_| AdminError::Save)?;
    if !saved {
        return Err(with_status(
            StatusCode::NOT_FOUND,
            AdminError::ProjectNotFound,
        ));
    }
    reload(&state).await
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
    let _session = authenticated_session().await?;
    require_fields(&[&name, &title, &summary, &about, &email])?;
    let state = expect_context::<AppState>();
    db::portfolio::set_profile(&state.pool, &name, &title, &summary, &about, &email)
        .await
        .map_err(|_| AdminError::Save)?;
    reload(&state).await
}

/// Saves the editable contact fields and returns the refreshed portfolio.
#[server(endpoint = "save_contact")]
#[tracing::instrument(name = "Save portfolio contact", skip_all, err)]
pub async fn save_contact(name: String, body: String) -> Result<PortfolioContent, AdminError> {
    let _session = authenticated_session().await?;
    require_fields(&[&name, &body])?;
    let state = expect_context::<AppState>();
    db::portfolio::set_contact(&state.pool, &name, &body)
        .await
        .map_err(|_| AdminError::Save)?;
    reload(&state).await
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
    let _session = authenticated_session().await?;
    require_fields(&[&url, &title, &description, &og_image])?;
    let state = expect_context::<AppState>();
    db::portfolio::set_site(&state.pool, &url, &title, &description, &og_image)
        .await
        .map_err(|_| AdminError::Save)?;
    reload(&state).await
}
