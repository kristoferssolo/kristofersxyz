mod owner;
mod project;

#[cfg(feature = "ssr")]
pub use owner::OwnerId;
pub use owner::{Username, UsernameError};
pub use project::{
    Project, ProjectDescription, ProjectDescriptionError, ProjectLink, ProjectSlug,
    ProjectSlugError,
};
