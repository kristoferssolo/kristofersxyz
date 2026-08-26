use leptos::prelude::{FromServerFnError, ServerFnErrorErr};
use leptos::server_fn::codec::JsonEncoding;
use serde::{Deserialize, Serialize};

/// Errors safe to show on the owner-facing forms.
#[derive(Clone, Debug, Deserialize, Serialize, thiserror::Error)]
pub enum AdminError {
    #[error("Invalid username or password.")]
    InvalidCredentials,
    #[error("Your session has expired. Sign in again.")]
    Unauthenticated,
    #[error("Every field is required.")]
    MissingField,
    #[error("No such project.")]
    ProjectNotFound,
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
