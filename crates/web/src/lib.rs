use leptos::*;
use leptos_router::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

mod charts;
mod estrannaise;
mod layout;
mod pages;
mod store;
mod utils;

#[cfg(not(target_arch = "wasm32"))]
mod server;

use pages::{
    BackupPage, CalcPage, CreateBloodTest, CreateDosage, CreateMeasurement, EstrannaisePage,
    StatsPage, VialsCreatePage, VialsDetailPage, VialsPage, ViewPage,
};
use store::StoreProvider;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <StoreProvider>
                <div class="app-shell">
                    <header class="top-bar">
                        <div class="brand">
                            <span class="brand-title">"HRT Tracker"</span>
                            <span class="brand-sub">"Get Absolutely Estrogen'd Idiot"</span>
                        </div>
                        <nav class="nav-links">
                            <A href="/view" active_class="active">"View"</A>
                            <A href="/stats" active_class="active">"Stats"</A>
                            <A href="/estrannaise" active_class="active">"Estrannaise"</A>
                            <A href="/create/dosage" active_class="active">"New Dose"</A>
                            <A href="/create/blood-test" active_class="active">"New Blood Test"</A>
                            <A href="/create/measurement" active_class="active">"New Measurement"</A>
                            <A href="/calc" active_class="active">"Calculator"</A>
                            <A href="/vials" active_class="active">"Vials"</A>
                            <A href="/backup" active_class="active">"Settings & Backup"</A>
                        </nav>
                    </header>
                    <main class="main-content">
                        <Routes>
                            <Route path="/" view=ViewPage />
                            <Route path="/create/dosage" view=CreateDosage />
                            <Route path="/create/blood-test" view=CreateBloodTest />
                            <Route path="/create/measurement" view=CreateMeasurement />
                            <Route path="/view" view=ViewPage />
                            <Route path="/stats" view=StatsPage />
                            <Route path="/backup" view=BackupPage />
                            <Route path="/calc" view=CalcPage />
                            <Route path="/vials" view=VialsPage />
                            <Route path="/vials/create" view=VialsCreatePage />
                            <Route path="/vials/:id" view=VialsDetailPage />
                            <Route path="/estrannaise" view=EstrannaisePage />
                        </Routes>
                    </main>
                </div>
            </StoreProvider>
        </Router>
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn mount_app() {
    mount_to_body(App);
}

#[cfg(not(target_arch = "wasm32"))]
pub use server::serve;
