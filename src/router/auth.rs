//! The owner login flow: a form, its verification, a session guard for the
//! admin area, and logout.
//!
//! These are plain Axum handlers rather than Leptos server functions. The
//! login and admin pages are minimal functional HTML, not a designed
//! interface; the visual pass is deferred.

use crate::{
    authentication::{AuthError, Credentials, validate_credentials},
    startup::AppState,
};
use axum::{
    Form,
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
};
use secrecy::SecretString;
use serde::Deserialize;
use tower_sessions::{Session, session};
use uuid::Uuid;

/// Session key under which the authenticated user's id lives.
const USER_ID_KEY: &str = "user_id";

#[derive(Deserialize)]
pub struct LoginForm {
    username: String,
    password: String,
}

/// Renders the empty login form.
pub async fn login_form() -> Html<String> {
    Html(login_page(None))
}

/// Verifies the submitted credentials and, on success, starts a session and
/// sends the owner to the admin area. A failure re-renders the form with a
/// fixed message, never echoing the submitted values.
pub async fn login(
    session: Session,
    State(state): State<AppState>,
    Form(form): Form<LoginForm>,
) -> Response {
    let credentials = Credentials {
        username: form.username,
        password: SecretString::from(form.password),
    };

    match validate_credentials(credentials, &state.pool).await {
        Ok(user_id) => match establish_session(&session, user_id).await {
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

/// The admin area, reachable only with a session. A signed-out visitor is
/// sent to the login form.
pub async fn admin(session: Session) -> Response {
    match session.get::<Uuid>(USER_ID_KEY).await {
        Ok(Some(_)) => Html(admin_page()).into_response(),
        _ => Redirect::to("/login").into_response(),
    }
}

/// Rotates the session id to defeat fixation, then records the user.
async fn establish_session(session: &Session, user_id: Uuid) -> Result<(), session::Error> {
    session.cycle_id().await?;
    session.insert(USER_ID_KEY, user_id).await
}

fn error_page(status: StatusCode, message: &str) -> Response {
    (status, Html(login_page(Some(message)))).into_response()
}

fn document(title: &str, body: &str) -> String {
    format!(
        "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
         <title>{title}</title><link rel=\"stylesheet\" href=\"/pkg/kristofersxyz.css\"></head>\
         <body class=\"grid min-h-dvh place-items-center bg-black font-mono text-[#d4d7db]\">{body}</body></html>"
    )
}

fn login_page(error: Option<&str>) -> String {
    let error = error.map_or_else(String::new, |message| {
        format!("<p class=\"mt-4 text-[12px] text-[#e2a340]\">{message}</p>")
    });
    let body = format!(
        "<form method=\"post\" action=\"/login\" class=\"w-full max-w-[320px] px-6\">\
         <p class=\"text-[10px] tracking-[0.24em] text-[#4c525a] uppercase\">Admin</p>\
         <h1 class=\"mt-3 font-sans text-2xl font-semibold text-white\">Sign in</h1>{error}\
         <label class=\"mt-6 block text-[12px] text-[#8b939d]\">Username\
         <input name=\"username\" autocomplete=\"username\" class=\"mt-1 block w-full border border-[#2b3037] bg-[#0b0e11] px-2.5 py-1.5 text-white focus-visible:outline-none\"></label>\
         <label class=\"mt-4 block text-[12px] text-[#8b939d]\">Password\
         <input name=\"password\" type=\"password\" autocomplete=\"current-password\" class=\"mt-1 block w-full border border-[#2b3037] bg-[#0b0e11] px-2.5 py-1.5 text-white focus-visible:outline-none\"></label>\
         <button type=\"submit\" class=\"mt-6 w-full border border-[#30363d] bg-[#080a0d] px-3 py-1.5 text-[13px] text-white hover:border-[#3d444d]\">Sign in</button></form>"
    );
    document("Sign in", &body)
}

fn admin_page() -> String {
    let body = "<div class=\"w-full max-w-[320px] px-6\">\
         <p class=\"text-[10px] tracking-[0.24em] text-[#4c525a] uppercase\">Admin</p>\
         <h1 class=\"mt-3 font-sans text-2xl font-semibold text-white\">Signed in</h1>\
         <p class=\"mt-4 text-[13px] text-[#aab2bb]\">Content editing lands here next.</p>\
         <form method=\"post\" action=\"/logout\" class=\"mt-6\">\
         <button type=\"submit\" class=\"border border-[#30363d] bg-[#080a0d] px-3 py-1.5 text-[13px] text-white hover:border-[#3d444d]\">Sign out</button></form></div>";
    document("Admin", body)
}
