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

#[cfg(test)]
mod tests {
    fn assert_no_numeric_input_type(file_name: &str, source: &str) {
        assert!(
            !source.contains("type=\"number\""),
            "{file_name} still contains type=\"number\" which blocks decimal typing in some browsers/locales"
        );
    }

    #[test]
    fn decimal_entry_pages_do_not_use_number_input_type() {
        let pages = [
            ("pages/backup.rs", include_str!("pages/backup.rs")),
            ("pages/calc.rs", include_str!("pages/calc.rs")),
            (
                "pages/create_blood_test.rs",
                include_str!("pages/create_blood_test.rs"),
            ),
            (
                "pages/create_dosage.rs",
                include_str!("pages/create_dosage.rs"),
            ),
            (
                "pages/create_measurement.rs",
                include_str!("pages/create_measurement.rs"),
            ),
            ("pages/estrannaise.rs", include_str!("pages/estrannaise.rs")),
            ("pages/vials.rs", include_str!("pages/vials.rs")),
            ("pages/view.rs", include_str!("pages/view.rs")),
        ];

        for (file_name, source) in pages {
            assert_no_numeric_input_type(file_name, source);
        }
    }
}
