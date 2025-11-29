//! Main application component and routing

use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::pages::{Dashboard, Buckets, Objects, Settings, Login};

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/tinystore.css"/>
        <Title text="TinyStore"/>

        <Router>
            <main>
                <Routes>
                    <Route path="/ui/login" view=Login/>
                    <Route path="/ui" view=Dashboard/>
                    <Route path="/ui/buckets" view=Buckets/>
                    <Route path="/ui/buckets/:name" view=Objects/>
                    <Route path="/ui/settings" view=Settings/>
                </Routes>
            </main>
        </Router>
    }
}
