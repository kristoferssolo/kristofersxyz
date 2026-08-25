//! Owner login, session checks, and logout.
//!
//! Axum handlers return HTML from [`view`] and store authentication data through
//! [`session`].

mod session;
mod view;

use crate::{
    app::content::{server_content, store_server_content},
    authentication::{AuthError, Credentials, validate_credentials},
    db,
    domain::ProjectDescription,
    startup::AppState,
};
use axum::{
    Form,
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
};
use secrecy::SecretString;
use serde::Deserialize;
use tower_sessions::Session;

use self::{
    session::{establish_session, owner, username},
    view::{admin_page, login_page, project_page},
};

#[derive(Deserialize)]
pub struct LoginForm {
    username: String,
    password: String,
}

/// Renders the empty login form.
pub async fn login_form() -> Html<String> {
    Html(login_page(None))
}

/// Starts a session for valid credentials. Failures return a fixed message and
/// never echo submitted values.
pub async fn login(
    session: Session,
    State(state): State<AppState>,
    Form(form): Form<LoginForm>,
) -> Response {
    let username = form.username.clone();
    let credentials = Credentials {
        username: form.username,
        password: SecretString::from(form.password),
    };

    match validate_credentials(credentials, &state.pool).await {
        Ok(user_id) => match establish_session(&session, user_id, &username).await {
            Ok(()) => Redirect::to("/admin").into_response(),
            Err(_) => error_page(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Could not start a session.",
            ),
        },
        Err(AuthError::InvalidCredentials) => {
            error_page(StatusCode::UNAUTHORIZED, "Invalid username or password.")
        }
        Err(_) => error_page(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Something went wrong. Please try again.",
        ),
    }
}

/// Ends the session and returns to the portfolio.
pub async fn logout(session: Session) -> Redirect {
    let _ = session.delete().await;
    Redirect::to("/")
}

/// Renders the admin area or redirects a signed-out visitor to login.
pub async fn admin(session: Session) -> Response {
    if owner(&session).await.is_none() {
        return Redirect::to("/login").into_response();
    }
    let name = username(&session)
        .await
        .unwrap_or_else(|| "owner".to_owned());
    Html(admin_page(&name)).into_response()
}

/// Renders an owner's project Markdown form. Unknown slugs return 404.
pub async fn project_form(session: Session, Path(slug): Path<String>) -> Response {
    if owner(&session).await.is_none() {
        return Redirect::to("/login").into_response();
    }

    let content = server_content();
    content
        .projects
        .iter()
        .find(|project| project.slug.as_str() == slug)
        .map_or_else(
            || (StatusCode::NOT_FOUND, "No such project.").into_response(),
            |project| Html(project_page(project)).into_response(),
        )
}

#[derive(Deserialize)]
pub struct ProjectEdit {
    markdown: String,
}

/// Saves non-empty Project Description Markdown, then refreshes the cached
/// portfolio. Requires an owner session and a known slug.
pub async fn edit_project(
    session: Session,
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Form(form): Form<ProjectEdit>,
) -> Response {
    if owner(&session).await.is_none() {
        return Redirect::to("/login").into_response();
    }

    if form.markdown.parse::<ProjectDescription>().is_err() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            "A project description cannot be empty.",
        )
            .into_response();
    }

    match db::portfolio::set_project_description(&state.pool, &slug, &form.markdown).await {
        Ok(true) => db::portfolio::load(&state.pool).await.map_or_else(
            |_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Saved, but the portfolio could not be reloaded.",
                )
                    .into_response()
            },
            |content| {
                store_server_content(content);
                Redirect::to("/admin").into_response()
            },
        ),
        Ok(false) => (StatusCode::NOT_FOUND, "No such project.").into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Could not save the edit.",
        )
            .into_response(),
    }
}

/// Re-renders the login form with a fixed message under the given status.
fn error_page(status: StatusCode, message: &str) -> Response {
    (status, Html(login_page(Some(message)))).into_response()
}
