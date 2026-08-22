//! Persistent chrome shared by every portfolio page.

mod blank_page;
mod help_panel;
mod notice_view;
mod sidebar;
mod status_bar;

pub use blank_page::BlankPage;
pub use status_bar::{StatusBarState, StatusLocation};
