//! Objects page (bucket browser)

use leptos::*;
use leptos_router::*;

#[component]
pub fn Objects() -> impl IntoView {
    let params = use_params_map();
    let bucket_name = move || {
        params.with(|params| {
            params.get("name").cloned().unwrap_or_default()
        })
    };

    view! {
        <div class="objects">
            <h1>"Objects in bucket: " {bucket_name}</h1>
            <p>"TODO: Implement in Step 7-8"</p>
        </div>
    }
}
