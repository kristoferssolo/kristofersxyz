use super::error::AdminError;
use crate::app::content::PortfolioContent;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// The owner identity exposed to authenticated Leptos routes.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SessionUser {
    pub username: String,
}

/// Returns the owner for a complete authenticated session.
#[server(endpoint = "admin_session")]
pub async fn current_user() -> Result<Option<SessionUser>, AdminError> {
    use crate::authentication::SessionState;
    use leptos_axum::redirect;

    match session_state().await? {
        SessionState::Anonymous(_) => {
            redirect("/login");
            Ok(None)
        }
        SessionState::Authenticated(session) => Ok(Some(SessionUser {
            username: session.username().to_owned(),
        })),
    }
}

/// Verifies credentials and starts an owner session.
#[server(endpoint = "login")]
pub async fn login(username: String, password: String) -> Result<(), AdminError> {
    use crate::{
        authentication::{AuthError, Credentials, SessionState, validate_credentials},
        startup::AppState,
    };
    use leptos_axum::redirect;
    use secrecy::SecretString;

    let session = match session_state().await? {
        SessionState::Authenticated(_) => {
            redirect("/admin");
            return Ok(());
        }
        SessionState::Anonymous(session) => session,
    };
    let state = expect_context::<AppState>();
    let credentials = Credentials {
        username: username.clone(),
        password: SecretString::from(password),
    };

    let owner_id = validate_credentials(credentials, &state.pool)
        .await
        .map_err(|error| match error {
            AuthError::InvalidCredentials => with_status(
                axum::http::StatusCode::UNAUTHORIZED,
                AdminError::InvalidCredentials,
            ),
            _ => AdminError::Internal,
        })?;
    session
        .sign_in(owner_id, username)
        .await
        .map_err(|_| AdminError::Internal)?;
    redirect("/admin");
    Ok(())
}

/// Ends an authenticated owner session.
#[server(endpoint = "logout")]
pub async fn logout() -> Result<(), AdminError> {
    use crate::authentication::SessionState;
    use leptos_axum::redirect;

    if let SessionState::Authenticated(session) = session_state().await? {
        session.sign_out().await.map_err(|_| AdminError::Internal)?;
    }
    redirect("/");
    Ok(())
}

/// Saves the editable project fields and returns the refreshed portfolio.
#[server(endpoint = "save_project")]
pub async fn save_project(
    slug: String,
    title: String,
    summary: String,
    markdown: String,
) -> Result<PortfolioContent, AdminError> {
    use crate::{db, startup::AppState};

    let _session = authenticated_session().await?;
    require_fields(&[&title, &summary, &markdown])?;
    let state = expect_context::<AppState>();
    let saved = db::portfolio::set_project(&state.pool, &slug, &title, &summary, &markdown)
        .await
        .map_err(|_| AdminError::Save)?;
    if !saved {
        return Err(with_status(
            axum::http::StatusCode::NOT_FOUND,
            AdminError::ProjectNotFound,
        ));
    }
    reload(&state).await
}

/// Saves the editable profile fields and returns the refreshed portfolio.
#[server(endpoint = "save_profile")]
pub async fn save_profile(
    name: String,
    title: String,
    summary: String,
    about: String,
    email: String,
) -> Result<PortfolioContent, AdminError> {
    use crate::{db, startup::AppState};

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
pub async fn save_contact(name: String, body: String) -> Result<PortfolioContent, AdminError> {
    use crate::{db, startup::AppState};

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
pub async fn save_site(
    url: String,
    title: String,
    description: String,
    og_image: String,
) -> Result<PortfolioContent, AdminError> {
    use crate::{db, startup::AppState};

    let _session = authenticated_session().await?;
    require_fields(&[&url, &title, &description, &og_image])?;
    let state = expect_context::<AppState>();
    db::portfolio::set_site(&state.pool, &url, &title, &description, &og_image)
        .await
        .map_err(|_| AdminError::Save)?;
    reload(&state).await
}

#[cfg(feature = "ssr")]
async fn session_state() -> Result<crate::authentication::SessionState, AdminError> {
    use crate::authentication::{AuthSession, Unverified};
    use tower_sessions::Session;

    let session = leptos_axum::extract::<Session>()
        .await
        .map_err(|_| AdminError::Internal)?;
    AuthSession::<Unverified>::new(session)
        .resolve()
        .await
        .map_err(|_| AdminError::Internal)
}

#[cfg(feature = "ssr")]
async fn authenticated_session()
-> Result<crate::authentication::AuthSession<crate::authentication::Authenticated>, AdminError> {
    use crate::authentication::SessionState;
    use leptos_axum::redirect;

    match session_state().await? {
        SessionState::Authenticated(session) => Ok(session),
        SessionState::Anonymous(_) => {
            redirect("/login");
            Err(with_status(
                axum::http::StatusCode::UNAUTHORIZED,
                AdminError::Unauthenticated,
            ))
        }
    }
}

#[cfg(feature = "ssr")]
fn require_fields(fields: &[&str]) -> Result<(), AdminError> {
    if fields.iter().all(|field| !field.trim().is_empty()) {
        Ok(())
    } else {
        Err(with_status(
            axum::http::StatusCode::UNPROCESSABLE_ENTITY,
            AdminError::MissingField,
        ))
    }
}

#[cfg(feature = "ssr")]
async fn reload(state: &crate::startup::AppState) -> Result<PortfolioContent, AdminError> {
    let content = crate::db::portfolio::load(&state.pool)
        .await
        .map_err(|_| AdminError::Reload)?;
    crate::app::content::store_server_content(content.clone());
    Ok(content)
}

#[cfg(feature = "ssr")]
fn with_status(status: axum::http::StatusCode, error: AdminError) -> AdminError {
    expect_context::<leptos_axum::ResponseOptions>().set_status(status);
    error
}
