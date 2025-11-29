//! TinyStore Web UI (Leptos)

pub mod app;
pub mod pages;
pub mod components;
pub mod api;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::App;
    console_error_panic_hook::set_once();
    leptos::mount_to_body(App);
}
