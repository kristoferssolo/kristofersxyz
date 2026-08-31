//! The owner-facing server functions.
//!
//! Each one resolves the session, validates its input into domain types, and
//! answers with the refreshed portfolio, so the editor and every public
//! consumer read the same content. The generated request types are what the
//! pages bind their forms to.

mod projects;
mod session;
mod singletons;
#[cfg(feature = "ssr")]
mod ssr;

pub use projects::{CreateProject, MoveProject, SaveProject};
pub use session::{Login, Logout, SessionUser, current_user};
pub use singletons::{SaveContact, SaveProfile, SaveSite};
