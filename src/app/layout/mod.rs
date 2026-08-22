//! Persistent chrome shared by every portfolio page.

mod blank_page;
mod sidebar;
mod status_bar;

pub use blank_page::BlankPage;
pub use status_bar::{StatusBarState, StatusLocation};
