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
    view::{admin_page, contact_page, login_page, profile_page, project_page, site_page},
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
    title: String,
    summary: String,
    markdown: String,
}

/// Saves a project's editable fields, then refreshes the cached portfolio.
/// Requires an owner session, a known slug, and non-empty fields.
pub async fn edit_project(
    session: Session,
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Form(form): Form<ProjectEdit>,
) -> Response {
    if owner(&session).await.is_none() {
        return Redirect::to("/login").into_response();
    }

    if !all_filled(&[&form.title, &form.summary, &form.markdown]) {
        return unprocessable();
    }

    match db::portfolio::set_project(&state.pool, &slug, &form.title, &form.summary, &form.markdown)
        .await
    {
        Ok(true) => apply(&state).await,
        Ok(false) => (StatusCode::NOT_FOUND, "No such project.").into_response(),
        Err(_) => internal(),
    }
}

#[derive(Deserialize)]
pub struct PreviewInput {
    markdown: String,
}

/// Renders Markdown to HTML for the live editor preview, using the same
/// renderer as the public site. Owner only; the renderer discards raw HTML, so
/// the result is safe to inject into the preview pane.
pub async fn preview(session: Session, Form(form): Form<PreviewInput>) -> Response {
    if owner(&session).await.is_none() {
        return Redirect::to("/login").into_response();
    }
    Html(crate::app::markdown::render_source(&form.markdown)).into_response()
}

/// The profile edit form.
pub async fn profile_form(session: Session) -> Response {
    if owner(&session).await.is_none() {
        return Redirect::to("/login").into_response();
    }
    Html(profile_page(&server_content().profile)).into_response()
}

#[derive(Deserialize)]
pub struct ProfileEdit {
    name: String,
    title: String,
    summary: String,
    about: String,
    email: String,
}

/// Saves the profile singleton, then refreshes the cached portfolio. Requires
/// an owner session and non-empty fields.
pub async fn edit_profile(
    session: Session,
    State(state): State<AppState>,
    Form(form): Form<ProfileEdit>,
) -> Response {
    if owner(&session).await.is_none() {
        return Redirect::to("/login").into_response();
    }
    if !all_filled(&[
        &form.name,
        &form.title,
        &form.summary,
        &form.about,
        &form.email,
    ]) {
        return unprocessable();
    }
    match db::portfolio::set_profile(
        &state.pool,
        &form.name,
        &form.title,
        &form.summary,
        &form.about,
        &form.email,
    )
    .await
    {
        Ok(()) => apply(&state).await,
        Err(_) => internal(),
    }
}

/// The contact edit form.
pub async fn contact_form(session: Session) -> Response {
    if owner(&session).await.is_none() {
        return Redirect::to("/login").into_response();
    }
    Html(contact_page(&server_content().contact)).into_response()
}

#[derive(Deserialize)]
pub struct ContactEdit {
    name: String,
    body: String,
}

/// Saves the contact singleton, then refreshes the cached portfolio. Requires
/// an owner session and non-empty fields.
pub async fn edit_contact(
    session: Session,
    State(state): State<AppState>,
    Form(form): Form<ContactEdit>,
) -> Response {
    if owner(&session).await.is_none() {
        return Redirect::to("/login").into_response();
    }
    if !all_filled(&[&form.name, &form.body]) {
        return unprocessable();
    }
    match db::portfolio::set_contact(&state.pool, &form.name, &form.body).await {
        Ok(()) => apply(&state).await,
        Err(_) => internal(),
    }
}

/// The site metadata edit form.
pub async fn site_form(session: Session) -> Response {
    if owner(&session).await.is_none() {
        return Redirect::to("/login").into_response();
    }
    Html(site_page(&server_content().site)).into_response()
}

#[derive(Deserialize)]
pub struct SiteEdit {
    url: String,
    title: String,
    description: String,
    og_image: String,
}

/// Saves the site singleton, then refreshes the cached portfolio. Requires an
/// owner session and non-empty fields.
pub async fn edit_site(
    session: Session,
    State(state): State<AppState>,
    Form(form): Form<SiteEdit>,
) -> Response {
    if owner(&session).await.is_none() {
        return Redirect::to("/login").into_response();
    }
    if !all_filled(&[&form.url, &form.title, &form.description, &form.og_image]) {
        return unprocessable();
    }
    match db::portfolio::set_site(
        &state.pool,
        &form.url,
        &form.title,
        &form.description,
        &form.og_image,
    )
    .await
    {
        Ok(()) => apply(&state).await,
        Err(_) => internal(),
    }
}

/// Whether every required text field carries a value.
fn all_filled(fields: &[&str]) -> bool {
    fields.iter().all(|field| !field.trim().is_empty())
}

/// The response for a submission with an empty required field.
fn unprocessable() -> Response {
    (
        StatusCode::UNPROCESSABLE_ENTITY,
        "Every field is required.",
    )
        .into_response()
}

/// The response for an edit that could not be saved.
fn internal() -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Could not save the edit.",
    )
        .into_response()
}

/// Reloads the portfolio into the cache after a save, then returns to the admin
/// area. The row is already written, so a reload failure is reported as such.
async fn apply(state: &AppState) -> Response {
    db::portfolio::load(&state.pool).await.map_or_else(
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
    )
}

/// Re-renders the login form with a fixed message under the given status.
fn error_page(status: StatusCode, message: &str) -> Response {
    (status, Html(login_page(Some(message)))).into_response()
}
