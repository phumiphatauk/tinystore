//! Objects page (bucket browser)

use leptos::*;
use leptos_router::*;
use tinystore_shared::ObjectInfo;
use crate::api::client;
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, FileReader};

#[component]
pub fn Objects() -> impl IntoView {
    let params = use_params_map();
    let bucket_name = move || {
        params.with(|params| {
            params.get("name").cloned().unwrap_or_default()
        })
    };

    let (objects, set_objects) = create_signal(Vec::<ObjectInfo>::new());
    let (error, set_error) = create_signal(None::<String>);
    let (loading, set_loading) = create_signal(true);
    let (show_upload_modal, set_show_upload_modal) = create_signal(false);
    let (upload_file, set_upload_file) = create_signal(None::<(String, Vec<u8>)>);

    // Load objects
    let load_objects = move || {
        let bucket = bucket_name();
        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);

            match client::fetch_objects(&bucket, None).await {
                Ok(response) => {
                    set_objects.set(response.objects);
                }
                Err(e) => {
                    set_error.set(Some(format!("Failed to load objects: {}", e)));
                }
            }

            set_loading.set(false);
        });
    };

    // Initial load
    create_effect(move |_| {
        load_objects();
    });

    // File upload handler
    let on_file_selected = move |ev: leptos::ev::Event| {
        let target = ev.target().unwrap();
        let input = target.dyn_into::<HtmlInputElement>().unwrap();

        if let Some(files) = input.files() {
            if let Some(file) = files.get(0) {
                let file_name = file.name();
                let file_reader = FileReader::new().unwrap();
                let file_reader_clone = file_reader.clone();

                let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move |_event: web_sys::ProgressEvent| {
                    if let Ok(result) = file_reader_clone.result() {
                        if let Some(array_buffer) = result.dyn_ref::<js_sys::ArrayBuffer>() {
                            let uint8_array = js_sys::Uint8Array::new(array_buffer);
                            let data = uint8_array.to_vec();
                            set_upload_file.set(Some((file_name.clone(), data)));
                        }
                    }
                }) as Box<dyn FnMut(_)>);

                file_reader.set_onload(Some(closure.as_ref().unchecked_ref()));
                closure.forget();

                let _ = file_reader.read_as_array_buffer(&file);
            }
        }
    };

    // Upload handler
    let on_upload = move |_| {
        if let Some((file_name, data)) = upload_file.get() {
            let bucket = bucket_name();

            spawn_local(async move {
                set_error.set(None);

                match client::upload_object(&bucket, &file_name, data).await {
                    Ok(_) => {
                        set_show_upload_modal.set(false);
                        set_upload_file.set(None);
                        load_objects();
                    }
                    Err(e) => {
                        set_error.set(Some(format!("Failed to upload object: {}", e)));
                    }
                }
            });
        }
    };

    // Delete object handler
    let delete_object = move |key: String| {
        let bucket = bucket_name();

        spawn_local(async move {
            set_error.set(None);

            match client::delete_object(&bucket, &key).await {
                Ok(_) => {
                    load_objects();
                }
                Err(e) => {
                    set_error.set(Some(format!("Failed to delete object: {}", e)));
                }
            }
        });
    };

    view! {
        <div class="objects-page">
            <div class="page-header">
                <div>
                    <div class="breadcrumb">
                        <A href="/ui/buckets">"Buckets"</A>
                        <span>" / "</span>
                        <span>{move || bucket_name()}</span>
                    </div>
                    <h1 class="page-title">"Objects in " {move || bucket_name()}</h1>
                </div>
                <button
                    class="btn btn-primary"
                    on:click=move |_| set_show_upload_modal.set(true)
                >
                    "+ Upload Object"
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
                        <p>"Loading objects..."</p>
                    </div>
                }.into_view()
            } else if objects.get().is_empty() {
                view! {
                    <div class="empty-state">
                        <div class="empty-icon">"📄"</div>
                        <h3>"No objects yet"</h3>
                        <p>"Upload your first object to get started"</p>
                        <button
                            class="btn btn-primary"
                            on:click=move |_| set_show_upload_modal.set(true)
                        >
                            "Upload Object"
                        </button>
                    </div>
                }.into_view()
            } else {
                view! {
                    <div class="objects-table">
                        <table>
                            <thead>
                                <tr>
                                    <th>"Name"</th>
                                    <th>"Size"</th>
                                    <th>"Last Modified"</th>
                                    <th>"Storage Class"</th>
                                    <th>"Actions"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {move || objects.get().into_iter().map(|object| {
                                    let key = object.key.clone();
                                    let key_for_delete = object.key.clone();
                                    let bucket = bucket_name();

                                    view! {
                                        <tr>
                                            <td class="object-name">{key.clone()}</td>
                                            <td>{format_bytes(object.size)}</td>
                                            <td>{object.last_modified.format("%Y-%m-%d %H:%M:%S").to_string()}</td>
                                            <td>
                                                <span class="badge">{object.storage_class}</span>
                                            </td>
                                            <td class="actions">
                                                <a
                                                    href=format!("/api/buckets/{}/objects/{}", bucket, key)
                                                    download
                                                    class="btn btn-sm btn-secondary"
                                                >
                                                    "Download"
                                                </a>
                                                <button
                                                    class="btn btn-sm btn-danger"
                                                    on:click=move |_| {
                                                        if web_sys::window()
                                                            .and_then(|w| w.confirm_with_message(
                                                                &format!("Are you sure you want to delete '{}'?", key_for_delete)
                                                            ).ok())
                                                            .unwrap_or(false)
                                                        {
                                                            delete_object(key_for_delete.clone());
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

            // Upload modal
            {move || if show_upload_modal.get() {
                view! {
                    <div class="modal-overlay" on:click=move |_| set_show_upload_modal.set(false)>
                        <div class="modal" on:click=move |e| e.stop_propagation()>
                            <div class="modal-header">
                                <h2>"Upload Object"</h2>
                                <button
                                    class="modal-close"
                                    on:click=move |_| set_show_upload_modal.set(false)
                                >
                                    "×"
                                </button>
                            </div>
                            <div class="modal-body">
                                <div class="form-group">
                                    <label for="file-input">"Select File"</label>
                                    <input
                                        type="file"
                                        id="file-input"
                                        class="form-input"
                                        on:change=on_file_selected
                                    />
                                    {move || upload_file.get().map(|(name, data)| view! {
                                        <p class="form-help">
                                            "Selected: " {name.clone()} " (" {format_bytes(data.len() as u64)} ")"
                                        </p>
                                    })}
                                </div>
                            </div>
                            <div class="modal-footer">
                                <button
                                    type="button"
                                    class="btn btn-secondary"
                                    on:click=move |_| set_show_upload_modal.set(false)
                                >
                                    "Cancel"
                                </button>
                                <button
                                    type="button"
                                    class="btn btn-primary"
                                    disabled=move || upload_file.get().is_none()
                                    on:click=on_upload
                                >
                                    "Upload"
                                </button>
                            </div>
                        </div>
                    </div>
                }.into_view()
            } else {
                view! { <div></div> }.into_view()
            }}
        </div>
    }
}

/// Format bytes into human-readable format
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];

    if bytes == 0 {
        return "0 B".to_string();
    }

    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}
