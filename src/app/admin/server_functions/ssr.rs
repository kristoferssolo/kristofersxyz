use super::projects::ProjectLinkField;
use crate::{
    app::admin::error::AdminError,
    app::content::{PortfolioContent, store_server_content},
    authentication::{
        Authenticated, AxumAuthSession, OwnerSession, RetryAfter, SessionState, Unverified,
    },
    db,
    domain::{
        ProjectLink, ProjectLinkLabel, ProjectLinkUrl, ProjectLinks, ProjectTechnologies,
        TechnologyName,
    },
    security_events::SecurityEvent,
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
            SecurityEvent::AuthorizationRejected.record();
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

/// Turns the raw Technology fields a form submits into the validated
/// collection, naming the first line that failed. One rejection stops the
/// save, so nothing reaches the database.
pub fn validated_technologies(values: &[String]) -> Result<ProjectTechnologies, AdminError> {
    let names = values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            value.parse::<TechnologyName>().map_err(|_| {
                rejected(AdminError::InvalidTechnology {
                    position: line(index),
                })
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    ProjectTechnologies::try_from(names).map_err(|repeat| {
        rejected(AdminError::RepeatedTechnology {
            position: line(repeat.index),
        })
    })
}

/// Turns the raw label and URL pairs a form submits into the validated
/// collection, naming the first line and field that failed.
pub fn validated_links(values: &[ProjectLinkField]) -> Result<ProjectLinks, AdminError> {
    let links = values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            Ok(ProjectLink {
                label: value.label.parse::<ProjectLinkLabel>().map_err(|_| {
                    rejected(AdminError::InvalidLinkLabel {
                        position: line(index),
                    })
                })?,
                href: value.href.parse::<ProjectLinkUrl>().map_err(|_| {
                    rejected(AdminError::InvalidLinkUrl {
                        position: line(index),
                    })
                })?,
            })
        })
        .collect::<Result<Vec<_>, AdminError>>()?;

    ProjectLinks::try_from(links).map_err(|repeat| {
        rejected(AdminError::RepeatedLinkLabel {
            position: line(repeat.index),
        })
    })
}

/// The one-based line the Owner sees for a zero-based collection index.
const fn line(index: usize) -> usize {
    index.saturating_add(1)
}

/// Answers a rejected edit without applying any part of it.
fn rejected(error: AdminError) -> AdminError {
    with_status(StatusCode::UNPROCESSABLE_ENTITY, error)
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
