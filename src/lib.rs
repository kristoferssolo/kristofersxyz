#![recursion_limit = "512"]

pub mod app;
#[cfg(feature = "ssr")]
pub mod configuration;
#[cfg(feature = "ssr")]
pub mod db;
pub mod domain;
pub mod errors;
#[cfg(feature = "ssr")]
pub mod router;
#[cfg(feature = "ssr")]
pub mod startup;
#[cfg(feature = "ssr")]
pub mod telemetry;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::App;

    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
