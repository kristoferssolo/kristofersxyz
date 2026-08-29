use super::AdminError;
use crate::{
    app::content::{PortfolioContent, store_server_content},
    authentication::{
        Authenticated, AxumAuthSession, OwnerSession, RetryAfter, SessionState, Unverified,
    },
    db,
    startup::ApplicationState,
};
use axum::http::{HeaderValue, StatusCode, header};
use leptos::prelude::expect_context;
use leptos_axum::{ResponseOptions, extract, redirect};

pub async fn session_state() -> Result<SessionState, AdminError> {
    let session = extract::<AxumAuthSession>()
        .await
        .map_err(|_| AdminError::Internal)?;
    let state = expect_context::<ApplicationState>();
    OwnerSession::<Unverified>::new(session)
        .resolve(state.session_policy)
        .await
        .map_err(|_| AdminError::Internal)
}

pub async fn authenticated_session() -> Result<OwnerSession<Authenticated>, AdminError> {
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

pub async fn reload(state: &ApplicationState) -> Result<PortfolioContent, AdminError> {
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

pub fn too_many_attempts(retry_after: RetryAfter) -> AdminError {
    let response = expect_context::<ResponseOptions>();
    response.set_status(StatusCode::TOO_MANY_REQUESTS);
    let seconds = retry_after.seconds();
    HeaderValue::from_str(&seconds.to_string()).map_or(AdminError::Internal, |value| {
        response.insert_header(header::RETRY_AFTER, value);
        AdminError::TooManyAttempts {
            retry_after_seconds: seconds,
        }
    })
}
