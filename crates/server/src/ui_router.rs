//! UI router with Leptos integration

use axum::Router;
use leptos::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use tinystore_ui::app::App;
use tinystore_storage::FilesystemBackend;
use crate::state::AppState;

/// Create the UI router with Leptos SSR
pub fn create_ui_router(app_state: AppState<FilesystemBackend>) -> Router {
    // Build Leptos options manually
    let leptos_options = LeptosOptions::builder()
        .output_name("tinystore")
        .site_pkg_dir("pkg")
        .site_root("public")
        .build();

    let routes = generate_route_list(App);

    // Provide app state to Leptos context
    let leptos_options_clone = leptos_options.clone();

    Router::new()
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            move || {
                provide_context(app_state.clone());
            },
            App,
        )
        .with_state(leptos_options_clone)
}
