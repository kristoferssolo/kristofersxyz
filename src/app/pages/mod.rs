mod home;
mod not_found;
mod project;

pub use home::HomePage;
pub use not_found::NotFoundPage;
pub use project::ProjectPage;

/// Statusline amber, the only colour on the site besides greys.
const AMBER: &str = "#e2a340";
