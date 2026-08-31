//! The project tables: reading them into the content model, writing an edit or
//! a new Project, and moving one through the public order.
//!
//! Every write runs in one transaction, so a failure leaves the stored order
//! and the stored child rows exactly as they were.

mod load;
mod order;
mod write;

pub use load::load;
pub use order::move_to;
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
