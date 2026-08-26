//! The owner-facing content editor, built from Leptos routes and components.

mod components;
mod error;
mod pages;
mod server_functions;

pub use pages::{
    AuthenticatedAdmin, ContactEditorPage, DashboardPage, LoginPage, ProfileEditorPage,
    ProjectEditorPage, SiteEditorPage,
};
