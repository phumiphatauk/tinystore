//! Main application component and routing

use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::pages::{Dashboard, Buckets, Objects, Settings, Login};
use crate::components::Layout;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/assets/style.css"/>
        <Title text="TinyStore"/>

        <Router>
            <Routes>
                // Login page without layout
                <Route path="/ui/login" view=Login/>

                // Main app routes with layout
                <Route path="/ui" view=move || view! {
                    <Layout>
                        <Dashboard/>
                    </Layout>
                }/>
                <Route path="/ui/buckets" view=move || view! {
                    <Layout>
                        <Buckets/>
                    </Layout>
                }/>
                <Route path="/ui/buckets/:name" view=move || view! {
                    <Layout>
                        <Objects/>
                    </Layout>
                }/>
                <Route path="/ui/settings" view=move || view! {
                    <Layout>
                        <Settings/>
                    </Layout>
                }/>
            </Routes>
        </Router>
    }
}
