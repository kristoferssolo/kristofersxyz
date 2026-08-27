//! The owner-facing content editor, built from Leptos routes and components.

use crate::domain::ProjectSlug;

mod components;
mod error;
mod pages;
mod server_functions;

pub use pages::{
    AuthenticatedAdmin, ContactEditorPage, DashboardPage, LoginPage, ProfileEditorPage,
    ProjectEditorPage, SiteEditorPage,
};

fn admin_path_for_slug(slug: &ProjectSlug) -> String {
    format!("/admin/project/{slug}")
}
