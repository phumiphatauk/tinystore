//! UI router with Leptos integration

use axum::Router;
use leptos::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use tinystore_ui::app::App;
use tinystore_storage::FilesystemBackend;
use crate::state::AppState;

/// Create the UI router with Leptos SSR
pub async fn create_ui_router(app_state: AppState<FilesystemBackend>) -> Router {
    // Get Leptos configuration
    let conf = get_configuration(None).await.expect("Failed to get Leptos configuration");
    let leptos_options = conf.leptos_options;

    let routes = generate_route_list(App);

    Router::new()
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            move || {
                provide_context(app_state.clone());
            },
            App,
        )
        .with_state(leptos_options)
}
