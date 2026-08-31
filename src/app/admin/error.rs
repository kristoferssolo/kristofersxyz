use leptos::prelude::{FromServerFnError, ServerFnErrorErr};
use leptos::server_fn::codec::JsonEncoding;
use serde::{Deserialize, Serialize};

/// Errors safe to show on the owner-facing forms.
#[derive(Clone, Debug, Deserialize, Serialize, thiserror::Error)]
pub enum AdminError {
    #[error("Invalid username or password.")]
    InvalidCredentials,
    #[error("Too many sign-in attempts. Try again in {retry_after_seconds} seconds.")]
    TooManyAttempts { retry_after_seconds: u64 },
    #[error("Your session has expired. Sign in again.")]
    Unauthenticated,
    #[error("Every field is required.")]
    MissingField,
    #[error("No such project.")]
    ProjectNotFound,
    #[error("Use lowercase letters, digits, and dashes, starting and ending with one.")]
    InvalidSlug,
    #[error("A project already uses that slug. Pick another.")]
    SlugTaken,
    #[error("That slug is reserved. Pick another.")]
    ReservedSlug,
    #[error("That project cannot move that way.")]
    InvalidMovement,
    /// The line rejections carry the one-based position the Owner sees rather
    /// than the text that failed, so a mistyped URL never reaches an error.
    #[error("Needs visible text, with no space at the start or end.")]
    InvalidTechnology { position: usize },
    #[error("Repeats an earlier technology. Each one can appear once.")]
    RepeatedTechnology { position: usize },
    #[error("A link label needs visible text, with no space at the start or end.")]
    InvalidLinkLabel { position: usize },
    #[error("Use an absolute http or https URL.")]
    InvalidLinkUrl { position: usize },
    #[error("Repeats an earlier label. Each link needs its own.")]
    RepeatedLinkLabel { position: usize },
    #[error("Could not save the edit.")]
    Save,
    #[error("Saved, but the portfolio could not be reloaded.")]
    Reload,
    #[error("Something went wrong. Please try again.")]
    Internal,
}

impl FromServerFnError for AdminError {
    type Encoder = JsonEncoding;

    fn from_server_fn_error(_error: ServerFnErrorErr) -> Self {
        Self::Internal
    }
}
