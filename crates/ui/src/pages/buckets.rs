//! Buckets page

use leptos::*;
use leptos_router::*;
use tinystore_shared::BucketInfo;
use crate::api::client;

#[component]
pub fn Buckets() -> impl IntoView {
    let (buckets, set_buckets) = create_signal(Vec::<BucketInfo>::new());
    let (error, set_error) = create_signal(None::<String>);
    let (loading, set_loading) = create_signal(true);
    let (show_create_modal, set_show_create_modal) = create_signal(false);
    let (new_bucket_name, set_new_bucket_name) = create_signal(String::new());

    // Load buckets
    let load_buckets = move || {
        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);

            match client::fetch_buckets().await {
                Ok(response) => {
                    set_buckets.set(response.buckets);
                }
                Err(e) => {
                    set_error.set(Some(format!("Failed to load buckets: {}", e)));
                }
            }

            set_loading.set(false);
        });
    };

    // Initial load
    create_effect(move |_| {
        load_buckets();
    });

    // Create bucket handler
    let on_create_bucket = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        let name = new_bucket_name.get();
        if name.is_empty() {
            set_error.set(Some("Bucket name cannot be empty".to_string()));
            return;
        }

        spawn_local(async move {
            set_error.set(None);

            match client::create_bucket(name.clone()).await {
                Ok(_) => {
                    set_show_create_modal.set(false);
                    set_new_bucket_name.set(String::new());
                    load_buckets();
                }
                Err(e) => {
                    set_error.set(Some(format!("Failed to create bucket: {}", e)));
                }
            }
        });
    };

    // Delete bucket handler
    let delete_bucket = move |bucket_name: String| {
        spawn_local(async move {
            set_error.set(None);

            match client::delete_bucket(&bucket_name).await {
                Ok(_) => {
                    load_buckets();
                }
                Err(e) => {
                    set_error.set(Some(format!("Failed to delete bucket: {}", e)));
                }
            }
        });
    };

    view! {
        <div class="buckets-page">
            <div class="page-header">
                <div>
                    <h1 class="page-title">"Buckets"</h1>
                    <p class="page-subtitle">"Manage your S3 buckets"</p>
                </div>
                <button
                    class="btn btn-primary"
                    on:click=move |_| set_show_create_modal.set(true)
                >
                    "+ Create Bucket"
                </button>
            </div>

            {move || error.get().map(|err| view! {
                <div class="alert alert-error">
                    <span class="alert-icon">"⚠️"</span>
                    <span>{err}</span>
                </div>
            })}

            {move || if loading.get() {
                view! {
                    <div class="loading">
                        <div class="spinner"></div>
                        <p>"Loading buckets..."</p>
                    </div>
                }.into_view()
            } else if buckets.get().is_empty() {
                view! {
                    <div class="empty-state">
                        <div class="empty-icon">"🪣"</div>
                        <h3>"No buckets yet"</h3>
                        <p>"Create your first bucket to get started"</p>
                        <button
                            class="btn btn-primary"
                            on:click=move |_| set_show_create_modal.set(true)
                        >
                            "Create Bucket"
                        </button>
                    </div>
                }.into_view()
            } else {
                view! {
                    <div class="buckets-grid">
                        {move || buckets.get().into_iter().map(|bucket| {
                            let bucket_name = bucket.name.clone();
                            let bucket_name_for_link = bucket.name.clone();
                            let bucket_name_for_delete = bucket.name.clone();

                            view! {
                                <div class="bucket-card">
                                    <div class="bucket-header">
                                        <h3 class="bucket-name">
                                            <A href=format!("/ui/buckets/{}", bucket_name)>
                                                {bucket_name.clone()}
                                            </A>
                                        </h3>
                                    </div>
                                    <div class="bucket-body">
                                        <div class="bucket-info">
                                            <span class="info-label">"Created:"</span>
                                            <span class="info-value">
                                                {bucket.creation_date.format("%Y-%m-%d %H:%M:%S").to_string()}
                                            </span>
                                        </div>
                                    </div>
                                    <div class="bucket-actions">
                                        <A
                                            href=format!("/ui/buckets/{}", bucket_name_for_link)
                                            class="btn btn-sm btn-primary"
                                        >
                                            "View Objects"
                                        </A>
                                        <button
                                            class="btn btn-sm btn-danger"
                                            on:click=move |_| {
                                                if web_sys::window()
                                                    .and_then(|w| w.confirm_with_message(
                                                        &format!("Are you sure you want to delete bucket '{}'?", bucket_name_for_delete)
                                                    ).ok())
                                                    .unwrap_or(false)
                                                {
                                                    delete_bucket(bucket_name_for_delete.clone());
                                                }
                                            }
                                        >
                                            "Delete"
                                        </button>
                                    </div>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                }.into_view()
            }}

            // Create bucket modal
            {move || if show_create_modal.get() {
                view! {
                    <div class="modal-overlay" on:click=move |_| set_show_create_modal.set(false)>
                        <div class="modal" on:click=move |e| e.stop_propagation()>
                            <div class="modal-header">
                                <h2>"Create New Bucket"</h2>
                                <button
                                    class="modal-close"
                                    on:click=move |_| set_show_create_modal.set(false)
                                >
                                    "×"
                                </button>
                            </div>
                            <form on:submit=on_create_bucket>
                                <div class="modal-body">
                                    <div class="form-group">
                                        <label for="bucket-name">"Bucket Name"</label>
                                        <input
                                            type="text"
                                            id="bucket-name"
                                            class="form-input"
                                            placeholder="my-bucket"
                                            value=move || new_bucket_name.get()
                                            on:input=move |ev| set_new_bucket_name.set(event_target_value(&ev))
                                        />
                                        <p class="form-help">
                                            "Bucket names must be lowercase, 3-63 characters, and can contain letters, numbers, and hyphens."
                                        </p>
                                    </div>
                                </div>
                                <div class="modal-footer">
                                    <button
                                        type="button"
                                        class="btn btn-secondary"
                                        on:click=move |_| set_show_create_modal.set(false)
                                    >
                                        "Cancel"
                                    </button>
                                    <button type="submit" class="btn btn-primary">
                                        "Create"
                                    </button>
                                </div>
                            </form>
                        </div>
                    </div>
                }.into_view()
            } else {
                view! { <div></div> }.into_view()
            }}
        </div>
    }
}
