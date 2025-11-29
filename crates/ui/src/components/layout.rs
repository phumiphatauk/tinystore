//! Main layout component

use leptos::*;
use crate::components::{Navbar, Sidebar};

#[component]
pub fn Layout(children: Children) -> impl IntoView {
    view! {
        <div class="app-container">
            <Navbar/>
            <div class="main-container">
                <Sidebar/>
                <main class="content-area">
                    {children()}
                </main>
            </div>
        </div>
    }
}
