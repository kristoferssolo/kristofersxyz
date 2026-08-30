mod home;
mod not_found;
mod project;

pub use home::HomePage;
pub use not_found::{MissingPage, NotFoundPage, set_not_found};
pub use project::ProjectPage;
