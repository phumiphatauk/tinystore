//! Sidebar component

use leptos::*;
use leptos_router::*;

#[component]
pub fn Sidebar() -> impl IntoView {
    view! {
        <aside class="sidebar">
            <div class="sidebar-menu">
                <p class="menu-label">"Navigation"</p>
                <ul class="menu-list">
                    <li>
                        <A href="/ui" class="sidebar-link">
                            <span class="icon">"📊"</span>
                            <span>"Dashboard"</span>
                        </A>
                    </li>
                    <li>
                        <A href="/ui/buckets" class="sidebar-link">
                            <span class="icon">"🪣"</span>
                            <span>"Buckets"</span>
                        </A>
                    </li>
                    <li>
                        <A href="/ui/settings" class="sidebar-link">
                            <span class="icon">"⚙️"</span>
                            <span>"Settings"</span>
                        </A>
                    </li>
                </ul>
            </div>
        </aside>
    }
}
