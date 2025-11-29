//! Settings page

use leptos::*;
use tinystore_shared::api::{CredentialInfo, CreateCredentialRequest};
use crate::api::client;

#[component]
pub fn Settings() -> impl IntoView {
    let (credentials, set_credentials) = create_signal(Vec::<CredentialInfo>::new());
    let (error, set_error) = create_signal(None::<String>);
    let (loading, set_loading) = create_signal(true);
    let (show_create_modal, set_show_create_modal) = create_signal(false);
    let (new_access_key, set_new_access_key) = create_signal(String::new());
    let (new_secret_key, set_new_secret_key) = create_signal(String::new());
    let (is_admin, set_is_admin) = create_signal(false);

    // Load credentials
    let load_credentials = move || {
        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);

            match client::fetch_credentials().await {
                Ok(creds) => {
                    set_credentials.set(creds);
                }
                Err(e) => {
                    set_error.set(Some(format!("Failed to load credentials: {}", e)));
                }
            }

            set_loading.set(false);
        });
    };

    // Initial load
    create_effect(move |_| {
        load_credentials();
    });

    // Create credential handler
    let on_create_credential = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        let access_key = new_access_key.get();
        let secret_key = new_secret_key.get();

        if access_key.is_empty() || secret_key.is_empty() {
            set_error.set(Some("Access key and secret key cannot be empty".to_string()));
            return;
        }

        let req = CreateCredentialRequest {
            access_key: access_key.clone(),
            secret_key: secret_key.clone(),
            is_admin: is_admin.get(),
        };

        spawn_local(async move {
            set_error.set(None);

            match client::create_credential(req).await {
                Ok(_) => {
                    set_show_create_modal.set(false);
                    set_new_access_key.set(String::new());
                    set_new_secret_key.set(String::new());
                    set_is_admin.set(false);
                    load_credentials();
                }
                Err(e) => {
                    set_error.set(Some(format!("Failed to create credential: {}", e)));
                }
            }
        });
    };

    // Delete credential handler
    let delete_credential = move |id: String| {
        spawn_local(async move {
            set_error.set(None);

            match client::delete_credential(&id).await {
                Ok(_) => {
                    load_credentials();
                }
                Err(e) => {
                    set_error.set(Some(format!("Failed to delete credential: {}", e)));
                }
            }
        });
    };

    view! {
        <div class="settings-page">
            <div class="page-header">
                <div>
                    <h1 class="page-title">"Settings"</h1>
                    <p class="page-subtitle">"Manage access credentials"</p>
                </div>
                <button
                    class="btn btn-primary"
                    on:click=move |_| set_show_create_modal.set(true)
                >
                    "+ Create Credential"
                </button>
            </div>

            {move || error.get().map(|err| view! {
                <div class="alert alert-error">
                    <span class="alert-icon">"⚠️"</span>
                    <span>{err}</span>
                </div>
            })}

            <div class="section">
                <h2 class="section-title">"Access Credentials"</h2>
                <p class="section-description">
                    "Manage AWS-style access keys and secret keys for S3 API authentication."
                </p>

                {move || if loading.get() {
                    view! {
                        <div class="loading">
                            <div class="spinner"></div>
                            <p>"Loading credentials..."</p>
                        </div>
                    }.into_view()
                } else if credentials.get().is_empty() {
                    view! {
                        <div class="empty-state">
                            <div class="empty-icon">"🔑"</div>
                            <h3>"No credentials yet"</h3>
                            <p>"Create your first credential to get started"</p>
                            <button
                                class="btn btn-primary"
                                on:click=move |_| set_show_create_modal.set(true)
                            >
                                "Create Credential"
                            </button>
                        </div>
                    }.into_view()
                } else {
                    view! {
                        <div class="credentials-table">
                            <table>
                                <thead>
                                    <tr>
                                        <th>"Access Key"</th>
                                        <th>"Admin"</th>
                                        <th>"Created"</th>
                                        <th>"Actions"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {move || credentials.get().into_iter().map(|cred| {
                                        let cred_id = cred.id.clone();

                                        view! {
                                            <tr>
                                                <td class="access-key">{cred.access_key}</td>
                                                <td>
                                                    {if cred.is_admin {
                                                        view! { <span class="badge badge-admin">"Admin"</span> }.into_view()
                                                    } else {
                                                        view! { <span class="badge badge-user">"User"</span> }.into_view()
                                                    }}
                                                </td>
                                                <td>{cred.created_at.format("%Y-%m-%d %H:%M:%S").to_string()}</td>
                                                <td class="actions">
                                                    <button
                                                        class="btn btn-sm btn-danger"
                                                        on:click=move |_| {
                                                            if web_sys::window()
                                                                .and_then(|w| w.confirm_with_message(
                                                                    "Are you sure you want to delete this credential?"
                                                                ).ok())
                                                                .unwrap_or(false)
                                                            {
                                                                delete_credential(cred_id.clone());
                                                            }
                                                        }
                                                    >
                                                        "Delete"
                                                    </button>
                                                </td>
                                            </tr>
                                        }
                                    }).collect::<Vec<_>>()}
                                </tbody>
                            </table>
                        </div>
                    }.into_view()
                }}
            </div>

            // Create credential modal
            {move || if show_create_modal.get() {
                view! {
                    <div class="modal-overlay" on:click=move |_| set_show_create_modal.set(false)>
                        <div class="modal" on:click=move |e| e.stop_propagation()>
                            <div class="modal-header">
                                <h2>"Create New Credential"</h2>
                                <button
                                    class="modal-close"
                                    on:click=move |_| set_show_create_modal.set(false)
                                >
                                    "×"
                                </button>
                            </div>
                            <form on:submit=on_create_credential>
                                <div class="modal-body">
                                    <div class="form-group">
                                        <label for="access-key">"Access Key"</label>
                                        <input
                                            type="text"
                                            id="access-key"
                                            class="form-input"
                                            placeholder="my-access-key"
                                            value=move || new_access_key.get()
                                            on:input=move |ev| set_new_access_key.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div class="form-group">
                                        <label for="secret-key">"Secret Key"</label>
                                        <input
                                            type="password"
                                            id="secret-key"
                                            class="form-input"
                                            placeholder="my-secret-key"
                                            value=move || new_secret_key.get()
                                            on:input=move |ev| set_new_secret_key.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div class="form-group">
                                        <label class="checkbox-label">
                                            <input
                                                type="checkbox"
                                                checked=move || is_admin.get()
                                                on:change=move |ev| set_is_admin.set(event_target_checked(&ev))
                                            />
                                            <span>"Admin privileges"</span>
                                        </label>
                                        <p class="form-help">
                                            "Admin credentials have full access to all buckets and operations."
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
