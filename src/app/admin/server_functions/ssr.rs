use super::AdminError;
use crate::{
    app::content::{PortfolioContent, store_server_content},
    authentication::{AuthSession, Authenticated, SessionState, Unverified},
    db,
    startup::AppState,
};
use axum::http::StatusCode;
use leptos::prelude::expect_context;
use leptos_axum::{ResponseOptions, extract, redirect};
use tower_sessions::Session;

pub async fn session_state() -> Result<SessionState, AdminError> {
    let session = extract::<Session>()
        .await
        .map_err(|_| AdminError::Internal)?;
    AuthSession::<Unverified>::new(session)
        .resolve()
        .await
        .map_err(|_| AdminError::Internal)
}

pub async fn authenticated_session() -> Result<AuthSession<Authenticated>, AdminError> {
    match session_state().await? {
        SessionState::Authenticated(session) => Ok(session),
        SessionState::Anonymous(_) => {
            redirect("/login");
            Err(with_status(
                StatusCode::UNAUTHORIZED,
                AdminError::Unauthenticated,
            ))
        }
    }
}

pub fn require_fields(fields: &[&str]) -> Result<(), AdminError> {
    if fields.iter().all(|field| !field.trim().is_empty()) {
        Ok(())
    } else {
        Err(with_status(
            StatusCode::UNPROCESSABLE_ENTITY,
            AdminError::MissingField,
        ))
    }
}

pub async fn reload(state: &AppState) -> Result<PortfolioContent, AdminError> {
    let content = db::portfolio::load(&state.pool)
        .await
        .map_err(|_| AdminError::Reload)?;
    store_server_content(content.clone());
    Ok(content)
}

pub fn with_status(status: StatusCode, error: AdminError) -> AdminError {
    expect_context::<ResponseOptions>().set_status(status);
    error
}

pub fn invalid_credentials() -> AdminError {
    with_status(StatusCode::UNAUTHORIZED, AdminError::InvalidCredentials)
}
