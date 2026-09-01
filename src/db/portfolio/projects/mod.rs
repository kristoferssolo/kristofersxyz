//! The project tables: reading them into the content model, writing an edit or
//! a new Project, moving one through the public order, and storing the Project
//! Screenshots that carry its visual evidence.
//!
//! Every write runs in one transaction, so a failure leaves the stored order
//! and the stored child rows exactly as they were.

mod load;
mod order;
mod screenshots;
mod write;

pub use load::load;
pub use order::move_to;
pub use screenshots::{
    StoredScreenshot, append, image, move_within_project, remove as delete_project_screenshot,
    set_details,
};
pub use write::{create, set};

/// Why a new Project could not be stored.
#[derive(Debug, thiserror::Error)]
pub enum CreateError {
    #[error("a project already uses this slug")]
    DuplicateSlug,
    #[error("the project could not be stored")]
    Transaction(#[from] sqlx::Error),
}

/// Why a Project could not be moved.
#[derive(Debug, thiserror::Error)]
pub enum MoveError {
    #[error("no project has this slug")]
    UnknownProject,
    #[error("the project cannot move that way")]
    InvalidMovement,
    #[error("the move could not be stored")]
    Transaction(#[from] sqlx::Error),
}

/// Why a Project Screenshot could not be stored, changed, or read.
#[derive(Debug, thiserror::Error)]
pub enum ScreenshotError {
    #[error("no project has this slug")]
    UnknownProject,
    #[error("no screenshot has this id")]
    UnknownScreenshot,
    #[error("the screenshot cannot move that way")]
    InvalidMovement,
    #[error("the stored screenshot is unreadable")]
    Corrupt,
    #[error("the screenshot could not be stored")]
    Transaction(#[from] sqlx::Error),
}
