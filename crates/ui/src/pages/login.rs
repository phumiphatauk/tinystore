//! Login page

use leptos::*;
use leptos_router::*;

#[component]
pub fn Login() -> impl IntoView {
    let (access_key, set_access_key) = create_signal(String::new());
    let (secret_key, set_secret_key) = create_signal(String::new());
    let (error, set_error) = create_signal(None::<String>);
    let navigate = use_navigate();

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        let access = access_key.get();
        let secret = secret_key.get();

        if access.is_empty() || secret.is_empty() {
            set_error.set(Some("Please enter both access key and secret key".to_string()));
            return;
        }

        // Store credentials in local storage
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.set_item("tinystore_access_key", &access);
                let _ = storage.set_item("tinystore_secret_key", &secret);
            }
        }

        // Navigate to dashboard
        navigate("/ui", Default::default());
    };

    view! {
        <div class="login-container">
            <div class="login-box">
                <div class="login-header">
                    <h1 class="login-title">"TinyStore"</h1>
                    <p class="login-subtitle">"S3-Compatible Object Storage"</p>
                </div>

                <form on:submit=on_submit class="login-form">
                    <div class="form-group">
                        <label for="access-key">"Access Key"</label>
                        <input
                            type="text"
                            id="access-key"
                            class="form-input"
                            placeholder="Enter your access key"
                            value=move || access_key.get()
                            on:input=move |ev| set_access_key.set(event_target_value(&ev))
                        />
                    </div>

                    <div class="form-group">
                        <label for="secret-key">"Secret Key"</label>
                        <input
                            type="password"
                            id="secret-key"
                            class="form-input"
                            placeholder="Enter your secret key"
                            value=move || secret_key.get()
                            on:input=move |ev| set_secret_key.set(event_target_value(&ev))
                        />
                    </div>

                    {move || error.get().map(|err| view! {
                        <div class="error-message">{err}</div>
                    })}

                    <button type="submit" class="btn btn-primary btn-block">
                        "Sign In"
                    </button>
                </form>

                <div class="login-footer">
                    <p class="text-muted">
                        "Default credentials: access_key = tinystore, secret_key = tinystore123"
                    </p>
                </div>
            </div>
        </div>
    }
}
