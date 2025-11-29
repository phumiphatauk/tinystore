//! Navigation bar component

use leptos::*;
use leptos_router::*;

#[component]
pub fn Navbar() -> impl IntoView {
    view! {
        <nav class="navbar">
            <div class="navbar-brand">
                <A href="/ui" class="navbar-item">
                    <h1 class="title is-4">"TinyStore"</h1>
                </A>
            </div>
            <div class="navbar-menu">
                <div class="navbar-start">
                    <A href="/ui" class="navbar-item">"Dashboard"</A>
                    <A href="/ui/buckets" class="navbar-item">"Buckets"</A>
                    <A href="/ui/settings" class="navbar-item">"Settings"</A>
                </div>
                <div class="navbar-end">
                    <div class="navbar-item">
                        <span class="tag is-info">"v0.1.0"</span>
                    </div>
                </div>
            </div>
        </nav>
    }
}
