//! The owner session: who is signed in, signing in, and signing out.

#[cfg(feature = "ssr")]
use super::ssr::{invalid_credentials, session_state, too_many_attempts};
use crate::{app::admin::error::AdminError, domain::Username};
#[cfg(feature = "ssr")]
use crate::{
    authentication::{
        AuthError, Credentials, OwnerSessionError, Password, RetryAfter, SessionState,
    },
    security_events::{AuthenticationFailure, LoginThrottleScope, SecurityEvent},
    startup::ApplicationState,
};
#[cfg(feature = "ssr")]
use axum::extract::ConnectInfo;
use leptos::prelude::*;
#[cfg(feature = "ssr")]
use leptos_axum::redirect;
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use std::net::SocketAddr;
use tracing::field::Empty;

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
