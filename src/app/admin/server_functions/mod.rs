//! The owner-facing server functions.
//!
//! Each one resolves the session, validates its input into domain types, and
//! answers with the refreshed portfolio, so the editor and every public
//! consumer read the same content. The generated request types are what the
//! pages bind their forms to.

mod projects;
mod screenshots;
mod session;
mod singletons;
#[cfg(feature = "ssr")]
mod ssr;
#[cfg(feature = "ssr")]
mod upload;

pub use projects::{CreateProject, MoveProject, SaveProject};
pub use screenshots::{
    DeleteProjectScreenshot, MoveProjectScreenshot, SaveScreenshotDetails,
    upload_project_screenshot,
};
pub use session::{Login, Logout, SessionUser, current_user};
pub use singletons::{SaveContact, SaveProfile, SaveSite};
