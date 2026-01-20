use leptos::*;
use leptos_router::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

mod store;
use store::{use_store, StoreProvider};

#[cfg(not(target_arch = "wasm32"))]
use axum::routing::get;
#[cfg(not(target_arch = "wasm32"))]
use axum::Router;
#[cfg(not(target_arch = "wasm32"))]
use tower_http::services::ServeDir;

use chrono::{Local, TimeZone};
use hrt_shared::types::{DosageHistoryEntry, HormoneUnits, LengthUnit};
use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use std::rc::Rc;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <StoreProvider>
                <div class="app-shell">
                    <header class="top-bar">
                        <div class="brand">
                            <span class="brand-title">"HRT Tracker"</span>
                            <span class="brand-sub">"Dose, labs, and trends"</span>
                        </div>
                        <nav class="nav-links">
                            <A href="/" active_class="active">"Dashboard"</A>
                            <A href="/view" active_class="active">"View"</A>
                            <A href="/stats" active_class="active">"Stats"</A>
                            <A href="/estrannaise" active_class="active">"Estrannaise"</A>
                            <A href="/create/dosage" active_class="active">"New Dose"</A>
                            <A href="/create/blood-test" active_class="active">"New Blood Test"</A>
                            <A href="/create/measurement" active_class="active">"New Measurement"</A>
                            <A href="/calc" active_class="active">"Calculator"</A>
                            <A href="/vials" active_class="active">"Vials"</A>
                            <A href="/backup" active_class="active">"Backup"</A>
                        </nav>
                    </header>
                    <main class="main-content">
                        <Routes>
                            <Route path="/" view=Dashboard />
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
            <main>
                <nav>
                    <ul>
                        <li><A href="/">"Dashboard"</A></li>
                        <li><A href="/create/dosage">"New Dose"</A></li>
                        <li><A href="/create/blood-test">"New Blood Test"</A></li>
                        <li><A href="/create/measurement">"New Measurement"</A></li>
                        <li><A href="/view">"View"</A></li>
                        <li><A href="/stats">"Stats"</A></li>
                        <li><A href="/backup">"Backup"</A></li>
                        <li><A href="/calc">"Calculator"</A></li>
                        <li><A href="/vials">"Vials"</A></li>
                        <li><A href="/estrannaise">"Estrannaise"</A></li>
                    </ul>
                </nav>
                <Routes>
                    <Route path="/" view=Dashboard />
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
        </Router>
    }
}

fn page_layout(title: &'static str, body: View) -> impl IntoView {
    let store = use_store();
    let is_loading = store.is_loading;
    let is_saving = store.is_saving;
    let is_dirty = store.is_dirty;
    let last_saved = store.last_saved;
    let error = store.last_error;

    let status_text = move || {
        if is_saving.get() {
            "Saving...".to_string()
        } else if let Some(last) = last_saved.get() {
            let stamp = Local
                .timestamp_millis_opt(last)
                .single()
                .map(|d| d.format("%H:%M").to_string())
                .unwrap_or_else(|| "".to_string());
            if stamp.is_empty() {
                "Saved".to_string()
            } else {
                format!("Saved {stamp}")
            }
        } else if is_dirty.get() {
            "Unsaved changes".to_string()
        } else {
            "Ready".to_string()
        }
    };

fn page_shell(title: &'static str) -> impl IntoView {
    view! {
        <section>
            <header>
                <div class="page-title">
                    <h1>{title}</h1>
                    <span class="status-chip">{status_text}</span>
                </div>
                <button
                    on:click=move |_| store.save()
                    prop:disabled=move || is_saving.get()
                >
                    "Save"
                </button>
            </header>
            <Show when=move || is_loading.get()>
                <p>"Loading data..."</p>
            </Show>
            <Show when=move || error.get().is_some()>
                <p class="error">{move || error.get().unwrap_or_default()}</p>
            </Show>
            {body}
            <h1>{title}</h1>
            <p>"Placeholder for Rust UI rewrite."</p>
        </section>
    }
}

#[component]
fn Dashboard() -> impl IntoView {
    let store = use_store();
    let data = store.data;
    let dosage_count = move || data.get().dosageHistory.len();
    let blood_count = move || data.get().bloodTests.len();
    let measurement_count = move || data.get().measurements.len();
    let vial_count = move || data.get().vials.len();
    let notes_count = move || data.get().notes.len();

    let last_dose = move || {
        let data_value = data.get();
        let latest = data_value
            .dosageHistory
            .iter()
            .max_by_key(|entry| match entry {
                DosageHistoryEntry::InjectableEstradiol { date, .. }
                | DosageHistoryEntry::OralEstradiol { date, .. }
                | DosageHistoryEntry::Antiandrogen { date, .. }
                | DosageHistoryEntry::Progesterone { date, .. } => *date,
            });
        match latest {
            Some(DosageHistoryEntry::InjectableEstradiol {
                date,
                kind,
                dose,
                unit,
                ..
            }) => {
                let date_text = Local
                    .timestamp_millis_opt(*date)
                    .single()
                    .map(|d| d.format("%b %d").to_string())
                    .unwrap_or_else(|| "".to_string());
                format!("{} · {:.2} {:?} · {:?}", date_text, dose, unit, kind)
            }
            Some(DosageHistoryEntry::OralEstradiol {
                date,
                kind,
                dose,
                unit,
                ..
            }) => {
                let date_text = Local
                    .timestamp_millis_opt(*date)
                    .single()
                    .map(|d| d.format("%b %d").to_string())
                    .unwrap_or_else(|| "".to_string());
                format!("{} · {:.2} {:?} · {:?}", date_text, dose, unit, kind)
            }
            Some(DosageHistoryEntry::Antiandrogen {
                date,
                kind,
                dose,
                unit,
                ..
            }) => {
                let date_text = Local
                    .timestamp_millis_opt(*date)
                    .single()
                    .map(|d| d.format("%b %d").to_string())
                    .unwrap_or_else(|| "".to_string());
                format!("{} · {:.2} {:?} · {:?}", date_text, dose, unit, kind)
            }
            Some(DosageHistoryEntry::Progesterone {
                date,
                kind,
                dose,
                unit,
                ..
            }) => {
                let date_text = Local
                    .timestamp_millis_opt(*date)
                    .single()
                    .map(|d| d.format("%b %d").to_string())
                    .unwrap_or_else(|| "".to_string());
                format!("{} · {:.2} {:?} · {:?}", date_text, dose, unit, kind)
            }
            None => "No doses yet".to_string(),
        }
    };

    let last_blood = move || {
        data.get()
            .bloodTests
            .iter()
            .max_by_key(|entry| entry.date)
            .map(|entry| {
                let date_text = Local
                    .timestamp_millis_opt(entry.date)
                    .single()
                    .map(|d| d.format("%b %d").to_string())
                    .unwrap_or_else(|| "".to_string());
                let e2 = entry
                    .estradiolLevel
                    .map(|v| format!("{:.2}", v))
                    .unwrap_or_else(|| "-".to_string());
                let unit = entry
                    .estradiolUnit
                    .as_ref()
                    .map(|u| format!("{:?}", u))
                    .unwrap_or_default();
                format!("{} · {} {}", date_text, e2, unit)
            })
            .unwrap_or_else(|| "No blood tests yet".to_string())
    };

    let last_measurement = move || {
        data.get()
            .measurements
            .iter()
            .max_by_key(|entry| entry.date)
            .map(|entry| {
                let date_text = Local
                    .timestamp_millis_opt(entry.date)
                    .single()
                    .map(|d| d.format("%b %d").to_string())
                    .unwrap_or_else(|| "".to_string());
                let weight = entry
                    .weight
                    .map(|v| format!("{:.1}", v))
                    .unwrap_or_else(|| "-".to_string());
                let unit = entry
                    .weightUnit
                    .as_ref()
                    .map(|u| format!("{:?}", u))
                    .unwrap_or_else(|| "".to_string());
                format!("{} · {} {}", date_text, weight, unit)
            })
            .unwrap_or_else(|| "No measurements yet".to_string())
    };

    page_layout(
        "Dashboard",
        view! {
            <div class="card-grid">
                <div class="mini-card">
                    <h3>"Last Dose"</h3>
                    <p>{last_dose}</p>
                </div>
                <div class="mini-card">
                    <h3>"Last Blood Test"</h3>
                    <p>{last_blood}</p>
                </div>
                <div class="mini-card">
                    <h3>"Last Measurement"</h3>
                    <p>{last_measurement}</p>
                </div>
            </div>

            <div class="card-grid">
                <div class="mini-card">
                    <h3>"Totals"</h3>
                    <p>"Dose history: "{dosage_count}</p>
                    <p>"Blood tests: "{blood_count}</p>
                    <p>"Measurements: "{measurement_count}</p>
                    <p>"Vials: "{vial_count}</p>
                    <p>"Notes: "{notes_count}</p>
                </div>
                <div class="mini-card">
                    <h3>"Quick Actions"</h3>
                    <div class="primary-actions">
                        <A href="/create/dosage">"Add Dose"</A>
                        <A href="/create/blood-test">"Add Blood Test"</A>
                        <A href="/create/measurement">"Add Measurement"</A>
                    </div>
                </div>
            </div>
        }
        .into_view(),
    )
    page_shell("Dashboard")
}

#[component]
fn CreateDosage() -> impl IntoView {
    let store = use_store();
    let dosage_type = create_rw_signal("injectableEstradiol".to_string());
    let dose_value = create_rw_signal("".to_string());
    let unit_value = create_rw_signal("mg".to_string());
    let medication_name = create_rw_signal("".to_string());
    let note_value = create_rw_signal("".to_string());
    let date_value = create_rw_signal("".to_string());
    let error = create_rw_signal(None::<String>);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        error.set(None);
        let dose = dose_value.get().trim().parse::<f64>().ok();
        let date = parse_date_or_now(&date_value.get());
        let dose = match dose {
            Some(value) => value,
            None => {
                error.set(Some("Dose is required.".to_string()));
                return;
            }
        };

        if medication_name.get().trim().is_empty() {
            error.set(Some("Medication name is required.".to_string()));
            return;
        }

        let entry = serde_json::json!({
            "date": date,
            "medicationType": dosage_type.get(),
            "type": medication_name.get(),
            "dose": dose,
            "unit": unit_value.get(),
            "note": if note_value.get().trim().is_empty() {
                serde_json::Value::Null
            } else {
                serde_json::Value::String(note_value.get())
            }
        });

        let mut data = store.data.get();
        let mut history = serde_json::to_value(&data.dosageHistory)
            .ok()
            .and_then(|v| v.as_array().cloned())
            .unwrap_or_default();
        history.push(entry);
        data.dosageHistory =
            serde_json::from_value(serde_json::Value::Array(history)).unwrap_or_default();
        store.data.set(data);
        store.is_dirty.set(true);
        store.save();
        dose_value.set("".to_string());
        note_value.set("".to_string());
        medication_name.set("".to_string());
        date_value.set("".to_string());
    };

    page_layout(
        "Create Dosage",
        view! {
            <form on:submit=on_submit>
                <label>"Date"</label>
                <input
                    type="date"
                    on:input=move |ev| date_value.set(event_target_value(&ev))
                    prop:value=move || date_value.get()
                />

                <label>"Medication Type"</label>
                <select
                    on:change=move |ev| dosage_type.set(event_target_value(&ev))
                    prop:value=move || dosage_type.get()
                >
                    <option value="injectableEstradiol">"Injectable Estradiol"</option>
                    <option value="oralEstradiol">"Oral Estradiol"</option>
                    <option value="antiandrogen">"Antiandrogen"</option>
                    <option value="progesterone">"Progesterone"</option>
                </select>

                <label>"Medication Name"</label>
                <input
                    type="text"
                    placeholder="Estradiol Valerate"
                    on:input=move |ev| medication_name.set(event_target_value(&ev))
                    prop:value=move || medication_name.get()
                />

                <label>"Dose"</label>
                <input
                    type="number"
                    step="0.01"
                    on:input=move |ev| dose_value.set(event_target_value(&ev))
                    prop:value=move || dose_value.get()
                />

                <label>"Unit"</label>
                <input
                    type="text"
                    on:input=move |ev| unit_value.set(event_target_value(&ev))
                    prop:value=move || unit_value.get()
                />

                <label>"Notes"</label>
                <textarea
                    rows="3"
                    on:input=move |ev| note_value.set(event_target_value(&ev))
                    prop:value=move || note_value.get()
                ></textarea>

                <button type="submit">"Add Dose"</button>
                <Show when=move || error.get().is_some()>
                    <p class="error">{move || error.get().unwrap_or_default()}</p>
                </Show>
                <p class="muted">"This is a temporary form until the full Rust UI is rebuilt."</p>
            </form>
        }
        .into_view(),
    )
    page_shell("Create Dosage")
}

#[component]
fn CreateBloodTest() -> impl IntoView {
    let store = use_store();
    let estradiol_value = create_rw_signal("".to_string());
    let estradiol_unit = create_rw_signal("pg/mL".to_string());
    let test_value = create_rw_signal("".to_string());
    let test_unit = create_rw_signal("ng/dL".to_string());
    let notes = create_rw_signal("".to_string());
    let date_value = create_rw_signal("".to_string());
    let error = create_rw_signal(None::<String>);
    let show_error = move || error.get().unwrap_or_default();

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        error.set(None);
        let date = parse_date_or_now(&date_value.get());
        let estradiol_level = estradiol_value.get().trim().parse::<f64>().ok();
        let test_level = test_value.get().trim().parse::<f64>().ok();

        if estradiol_level.is_none() && test_level.is_none() {
            error.set(Some("Provide at least one hormone value.".to_string()));
            return;
        }

        let entry = serde_json::json!({
            "date": date,
            "estradiolLevel": estradiol_level,
            "estradiolUnit": estradiol_unit.get(),
            "testLevel": test_level,
            "testUnit": test_unit.get(),
            "notes": if notes.get().trim().is_empty() {
                serde_json::Value::Null
            } else {
                serde_json::Value::String(notes.get())
            }
        });

        let mut data = store.data.get();
        let mut list = serde_json::to_value(&data.bloodTests)
            .ok()
            .and_then(|v| v.as_array().cloned())
            .unwrap_or_default();
        list.push(entry);
        data.bloodTests =
            serde_json::from_value(serde_json::Value::Array(list)).unwrap_or_default();
        store.data.set(data);
        store.is_dirty.set(true);
        store.save();
        estradiol_value.set("".to_string());
        test_value.set("".to_string());
        notes.set("".to_string());
        date_value.set("".to_string());
    };

    page_layout(
        "Create Blood Test",
        view! {
            <form on:submit=on_submit>
                <label>"Date"</label>
                <input
                    type="date"
                    on:input=move |ev| date_value.set(event_target_value(&ev))
                    prop:value=move || date_value.get()
                />

                <label>"Estradiol"</label>
                <div class="inline">
                    <input
                        type="number"
                        step="0.01"
                        on:input=move |ev| estradiol_value.set(event_target_value(&ev))
                        prop:value=move || estradiol_value.get()
                    />
                    <select
                        on:change=move |ev| estradiol_unit.set(event_target_value(&ev))
                        prop:value=move || estradiol_unit.get()
                    >
                        <option value="pg/mL">"pg/mL"</option>
                        <option value="pmol/L">"pmol/L"</option>
                    </select>
                </div>

                <label>"Testosterone"</label>
                <div class="inline">
                    <input
                        type="number"
                        step="0.01"
                        on:input=move |ev| test_value.set(event_target_value(&ev))
                        prop:value=move || test_value.get()
                    />
                    <select
                        on:change=move |ev| test_unit.set(event_target_value(&ev))
                        prop:value=move || test_unit.get()
                    >
                        <option value="ng/dL">"ng/dL"</option>
                        <option value="nmol/L">"nmol/L"</option>
                    </select>
                </div>

                <label>"Notes"</label>
                <textarea
                    rows="3"
                    on:input=move |ev| notes.set(event_target_value(&ev))
                    prop:value=move || notes.get()
                ></textarea>

                <button type="submit">"Add Blood Test"</button>
                <Show when=move || error.get().is_some()>
                    <p class="error">{show_error}</p>
                </Show>
                <p class="muted">"This is a temporary form until the full Rust UI is rebuilt."</p>
            </form>
        }
        .into_view(),
    )
    page_shell("Create Blood Test")
}

#[component]
fn CreateMeasurement() -> impl IntoView {
    let store = use_store();
    let weight = create_rw_signal("".to_string());
    let weight_unit = create_rw_signal("kg".to_string());
    let waist = create_rw_signal("".to_string());
    let hip = create_rw_signal("".to_string());
    let unit = create_rw_signal("cm".to_string());
    let date_value = create_rw_signal("".to_string());
    let error = create_rw_signal(None::<String>);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        error.set(None);
        let date = parse_date_or_now(&date_value.get());
        let weight_val = weight.get().trim().parse::<f64>().ok();
        let waist_val = waist.get().trim().parse::<f64>().ok();
        let hip_val = hip.get().trim().parse::<f64>().ok();

        if weight_val.is_none() && waist_val.is_none() && hip_val.is_none() {
            error.set(Some("Provide at least one measurement.".to_string()));
            return;
        }

        let entry = serde_json::json!({
            "date": date,
            "weight": weight_val,
            "weightUnit": weight_unit.get(),
            "waist": waist_val,
            "hip": hip_val,
            "bodyMeasurementUnit": unit.get()
        });

        let mut data = store.data.get();
        let mut list = serde_json::to_value(&data.measurements)
            .ok()
            .and_then(|v| v.as_array().cloned())
            .unwrap_or_default();
        list.push(entry);
        data.measurements =
            serde_json::from_value(serde_json::Value::Array(list)).unwrap_or_default();
        store.data.set(data);
        store.is_dirty.set(true);
        store.save();
        weight.set("".to_string());
        waist.set("".to_string());
        hip.set("".to_string());
        date_value.set("".to_string());
    };

    page_layout(
        "Create Measurement",
        view! {
            <form on:submit=on_submit>
                <label>"Date"</label>
                <input
                    type="date"
                    on:input=move |ev| date_value.set(event_target_value(&ev))
                    prop:value=move || date_value.get()
                />

                <label>"Weight"</label>
                <div class="inline">
                    <input
                        type="number"
                        step="0.01"
                        on:input=move |ev| weight.set(event_target_value(&ev))
                        prop:value=move || weight.get()
                    />
                    <input
                        type="text"
                        on:input=move |ev| weight_unit.set(event_target_value(&ev))
                        prop:value=move || weight_unit.get()
                    />
                </div>

                <label>"Waist"</label>
                <input
                    type="number"
                    step="0.1"
                    on:input=move |ev| waist.set(event_target_value(&ev))
                    prop:value=move || waist.get()
                />

                <label>"Hip"</label>
                <input
                    type="number"
                    step="0.1"
                    on:input=move |ev| hip.set(event_target_value(&ev))
                    prop:value=move || hip.get()
                />

                <label>"Measurement Unit"</label>
                <input
                    type="text"
                    on:input=move |ev| unit.set(event_target_value(&ev))
                    prop:value=move || unit.get()
                />

                <button type="submit">"Add Measurement"</button>
                <Show when=move || error.get().is_some()>
                    <p class="error">{move || error.get().unwrap_or_default()}</p>
                </Show>
                <p class="muted">"This is a temporary form until the full Rust UI is rebuilt."</p>
            </form>
        }
        .into_view(),
    )
    page_shell("Create Measurement")
}

#[component]
fn ViewPage() -> impl IntoView {
    let store = use_store();
    let data = store.data;
    let rows = move || data.get().dosageHistory.clone();
    let blood_rows = move || data.get().bloodTests.clone();
    let measurement_rows = move || data.get().measurements.clone();
    let editing_key = create_rw_signal(None::<String>);
    let editing_date = create_rw_signal(String::new());
    let editing_dose = create_rw_signal(String::new());
    let editing_note = create_rw_signal(String::new());
    let confirm_delete = create_rw_signal(None::<String>);
    let confirm_title = create_rw_signal(String::new());
    let confirm_action = create_rw_signal(None::<Rc<dyn Fn()>>);

    let edit_blood_date = create_rw_signal(None::<i64>);
    let edit_blood_date_text = create_rw_signal(String::new());
    let edit_blood_e2 = create_rw_signal(String::new());
    let edit_blood_e2_unit = create_rw_signal(String::new());
    let edit_blood_t = create_rw_signal(String::new());
    let edit_blood_t_unit = create_rw_signal(String::new());
    let edit_blood_notes = create_rw_signal(String::new());

    let edit_measurement_date = create_rw_signal(None::<i64>);
    let edit_measurement_date_text = create_rw_signal(String::new());
    let edit_measurement_weight = create_rw_signal(String::new());
    let edit_measurement_waist = create_rw_signal(String::new());
    let edit_measurement_hip = create_rw_signal(String::new());
    let edit_measurement_unit = create_rw_signal(String::new());

    let entry_matches = |entry: &DosageHistoryEntry, key: &str| match entry {
        DosageHistoryEntry::InjectableEstradiol { date, id, .. }
        | DosageHistoryEntry::OralEstradiol { date, id, .. }
        | DosageHistoryEntry::Antiandrogen { date, id, .. }
        | DosageHistoryEntry::Progesterone { date, id, .. } => id
            .as_ref()
            .map(|v| v == key)
            .unwrap_or_else(|| date.to_string() == key),
    };

    let on_save_edit = Rc::new({
        let store_edit = store.clone();
        move || {
            let key = match editing_key.get() {
                Some(value) => value,
                None => return,
            };
            let dose_value = match editing_dose.get().trim().parse::<f64>().ok() {
                Some(value) => value,
                None => return,
            };
            let date_value = parse_date_or_now(&editing_date.get());
            let note_text = editing_note.get();
            let note_value = if note_text.trim().is_empty() {
                None
            } else {
                Some(note_text.clone())
            };

            store_edit.data.update(|d| {
                for entry in &mut d.dosageHistory {
                    if entry_matches(entry, &key) {
                        match entry {
                            DosageHistoryEntry::InjectableEstradiol {
                                date, dose, note, ..
                            }
                            | DosageHistoryEntry::OralEstradiol {
                                date, dose, note, ..
                            }
                            | DosageHistoryEntry::Antiandrogen {
                                date, dose, note, ..
                            }
                            | DosageHistoryEntry::Progesterone {
                                date, dose, note, ..
                            } => {
                                *date = date_value;
                                *dose = dose_value;
                                *note = note_value.clone();
                            }
                        }
                    }
                }
            });
            store_edit.is_dirty.set(true);
            store_edit.save();
            editing_key.set(None);
        }
    });

    let on_cancel_edit = move |_| editing_key.set(None);

    let store_blood_modal = store.clone();
    let store_measure_modal = store.clone();

    page_layout(
        "View",
        view! {
            <section>
                <h2>"Dosage History"</h2>
                <Show
                    when=move || !rows().is_empty()
                    fallback=move || view! { <div class="empty-state">"No dosage history yet."</div> }
                >
                    <table class="table">
                        <thead>
                            <tr>
                                <th>"Date"</th>
                                <th>"Medication"</th>
                                <th>"Dose"</th>
                                <th>"Unit"</th>
                                <th>"Actions"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <For
                                each=rows
                                key=|entry| match entry {
                                    DosageHistoryEntry::InjectableEstradiol { date, id, .. } => {
                                        id.clone().unwrap_or_else(|| date.to_string())
                                    }
                                    DosageHistoryEntry::OralEstradiol { date, id, .. } => {
                                        id.clone().unwrap_or_else(|| date.to_string())
                                    }
                                    DosageHistoryEntry::Antiandrogen { date, id, .. } => {
                                        id.clone().unwrap_or_else(|| date.to_string())
                                    }
                                    DosageHistoryEntry::Progesterone { date, id, .. } => {
                                        id.clone().unwrap_or_else(|| date.to_string())
                                    }
                                }
                                children=move |entry| {
                                    let (date, name, dose, unit, id_opt, note) = match entry {
                                        DosageHistoryEntry::InjectableEstradiol {
                                            date,
                                            id,
                                            kind,
                                            dose,
                                            unit,
                                            note,
                                            ..
                                        } => (date, format!("{:?}", kind), dose, format!("{:?}", unit), id, note),
                                        DosageHistoryEntry::OralEstradiol {
                                            date,
                                            id,
                                            kind,
                                            dose,
                                            unit,
                                            note,
                                            ..
                                        } => (date, format!("{:?}", kind), dose, format!("{:?}", unit), id, note),
                                        DosageHistoryEntry::Antiandrogen {
                                            date,
                                            id,
                                            kind,
                                            dose,
                                            unit,
                                            note,
                                            ..
                                        } => (date, format!("{:?}", kind), dose, format!("{:?}", unit), id, note),
                                        DosageHistoryEntry::Progesterone {
                                            date,
                                            id,
                                            kind,
                                            dose,
                                            unit,
                                            note,
                                            ..
                                        } => (date, format!("{:?}", kind), dose, format!("{:?}", unit), id, note),
                                    };
                                    let date_text = Local
                                        .timestamp_millis_opt(date)
                                        .single()
                                        .map(|d| d.format("%Y-%m-%d").to_string())
                                        .unwrap_or_else(|| date.to_string());
                                    let entry_key = id_opt.clone().unwrap_or_else(|| date.to_string());
                                    let note_value = note.clone().unwrap_or_default();
                                    let on_delete = {
                                        let store = use_store();
                                        let entry_key = entry_key.clone();
                                        let confirm_delete = confirm_delete;
                                        let confirm_title = confirm_title;
                                        let confirm_action = confirm_action;
                                        move |_: leptos::ev::MouseEvent| {
                                            confirm_title.set("Delete dosage entry?".to_string());
                                            confirm_delete.set(Some(entry_key.clone()));
                                            let store = store.clone();
                                            let entry_key = entry_key.clone();
                                            confirm_action.set(Some(Rc::new(move || {
                                                store.data.update(|d| {
                                                    d.dosageHistory.retain(|item| !entry_matches(item, &entry_key));
                                                });
                                                store.is_dirty.set(true);
                                                store.save();
                                            })));
                                        }
                                    };
                                let on_edit = {
                                    let entry_key = entry_key.clone();
                                    let editing_key = editing_key;
                                    let editing_date = editing_date;
                                    let editing_dose = editing_dose;
                                    let editing_note = editing_note;
                                    let note_value = note_value.clone();
                                    let date_text_edit = date_text.clone();
                                    move |_: leptos::ev::MouseEvent| {
                                        editing_key.set(Some(entry_key.clone()));
                                        editing_date.set(date_text_edit.clone());
                                        editing_dose.set(format!("{:.2}", dose));
                                        editing_note.set(note_value.clone());
                                    }
                                };


                                    view! {
                                        <tr>
                                            <td>{date_text}</td>
                                            <td>{name}</td>
                                            <td>{format!("{:.2}", dose)}</td>
                                            <td>{unit}</td>
                                            <td>
                                                <button type="button" on:click=on_edit>"Edit"</button>
                                                <button type="button" on:click=on_delete>"Delete"</button>
                                            </td>
                                        </tr>
                                        <tr>
                                            <td colspan="5" class="muted">{note_value}</td>
                                        </tr>
                                    }
                                }
                            />
                        </tbody>
                    </table>
                </Show>
            </section>

            <section>
                <h2>"Blood Tests"</h2>
                <Show
                    when=move || !blood_rows().is_empty()
                    fallback=move || view! { <div class="empty-state">"No blood tests yet."</div> }
                >
                    <table class="table">
                        <thead>
                            <tr>
                                <th>"Date"</th>
                                <th>"Estradiol"</th>
                                <th>"Testosterone"</th>
                                <th>"Notes"</th>
                                <th>"Actions"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <For
                                each=blood_rows
                                key=|entry| entry.date
                                children=move |entry| {
                                    let date_text = Local
                                        .timestamp_millis_opt(entry.date)
                                        .single()
                                        .map(|d| d.format("%Y-%m-%d").to_string())
                                        .unwrap_or_else(|| entry.date.to_string());
                                    let e2 = entry
                                        .estradiolLevel
                                        .map(|v| format!("{:.2}", v))
                                        .unwrap_or_else(|| "-".to_string());
                                    let e2_unit = entry
                                        .estradiolUnit
                                        .as_ref()
                                        .map(|u| format!("{:?}", u))
                                        .unwrap_or_else(|| "".to_string());
                                    let t = entry
                                        .testLevel
                                        .map(|v| format!("{:.2}", v))
                                        .unwrap_or_else(|| "-".to_string());
                                    let t_unit = entry
                                        .testUnit
                                        .as_ref()
                                        .map(|u| format!("{:?}", u))
                                        .unwrap_or_else(|| "".to_string());
                                    let on_delete = {
                                        let store = use_store();
                                        let date = entry.date;
                                        let confirm_delete = confirm_delete;
                                        let confirm_title = confirm_title;
                                        let confirm_action = confirm_action;
                                        move |_: leptos::ev::MouseEvent| {
                                            confirm_title.set("Delete blood test?".to_string());
                                            confirm_delete.set(Some(date.to_string()));
                                            let store = store.clone();
                                            confirm_action.set(Some(Rc::new(move || {
                                                store
                                                    .data
                                                    .update(|d| d.bloodTests.retain(|b| b.date != date));
                                                store.is_dirty.set(true);
                                                store.save();
                                            })));
                                        }
                                    };
                                    let on_edit = {
                                        let date = entry.date;
                                        let edit_blood_date = edit_blood_date;
                                        let edit_blood_date_text = edit_blood_date_text;
                                        let edit_blood_e2 = edit_blood_e2;
                                        let edit_blood_e2_unit = edit_blood_e2_unit;
                                        let edit_blood_t = edit_blood_t;
                                        let edit_blood_t_unit = edit_blood_t_unit;
                                        let edit_blood_notes = edit_blood_notes;
                                        let e2_unit_value = entry
                                            .estradiolUnit
                                            .as_ref()
                                            .map(|u| format!("{:?}", u))
                                            .unwrap_or_else(|| "".to_string());
                                        let t_unit_value = entry
                                            .testUnit
                                            .as_ref()
                                            .map(|u| format!("{:?}", u))
                                            .unwrap_or_else(|| "".to_string());
                                        let e2_value = entry
                                            .estradiolLevel
                                            .map(|v| format!("{:.2}", v))
                                            .unwrap_or_default();
                                        let t_value = entry
                                            .testLevel
                                            .map(|v| format!("{:.2}", v))
                                            .unwrap_or_default();
                                        let notes_value = entry.notes.clone().unwrap_or_default();
                                        let date_text_edit = date_text.clone();
                                        move |_: leptos::ev::MouseEvent| {
                                            edit_blood_date.set(Some(date));
                                            edit_blood_date_text.set(date_text_edit.clone());
                                            edit_blood_e2.set(e2_value.clone());
                                            edit_blood_e2_unit.set(e2_unit_value.clone());
                                            edit_blood_t.set(t_value.clone());
                                            edit_blood_t_unit.set(t_unit_value.clone());
                                            edit_blood_notes.set(notes_value.clone());
                                        }
                                    };
                                    view! {
                                        <tr>
                                            <td>{date_text}</td>
                                            <td>{format!("{} {}", e2, e2_unit)}</td>
                                            <td>{format!("{} {}", t, t_unit)}</td>
                                            <td>{entry.notes.unwrap_or_default()}</td>
                                            <td>
                                                <button type="button" on:click=on_edit>"Edit"</button>
                                                <button type="button" on:click=on_delete>"Delete"</button>
                                            </td>
                                        </tr>
                                    }
                                }
                            />
                        </tbody>
                    </table>
                </Show>
            </section>

            <section>
                <h2>"Measurements"</h2>
                <Show
                    when=move || !measurement_rows().is_empty()
                    fallback=move || view! { <div class="empty-state">"No measurements yet."</div> }
                >
                    <table class="table">
                        <thead>
                            <tr>
                                <th>"Date"</th>
                                <th>"Weight"</th>
                                <th>"Waist"</th>
                                <th>"Hip"</th>
                                <th>"Actions"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <For
                                each=measurement_rows
                                key=|entry| entry.date
                                children=move |entry| {
                                    let date_text = Local
                                        .timestamp_millis_opt(entry.date)
                                        .single()
                                        .map(|d| d.format("%Y-%m-%d").to_string())
                                        .unwrap_or_else(|| entry.date.to_string());
                                    let weight = entry
                                        .weight
                                        .map(|v| format!("{:.2}", v))
                                        .unwrap_or_else(|| "-".to_string());
                                    let weight_unit = entry
                                        .weightUnit
                                        .map(|u| format!("{:?}", u))
                                        .unwrap_or_else(|| "".to_string());
                                    let waist = entry
                                        .waist
                                        .map(|v| format!("{:.1}", v))
                                        .unwrap_or_else(|| "-".to_string());
                                    let hip = entry
                                        .hip
                                        .map(|v| format!("{:.1}", v))
                                        .unwrap_or_else(|| "-".to_string());
                                    let unit = entry
                                        .bodyMeasurementUnit
                                        .as_ref()
                                        .map(|u| format!("{:?}", u))
                                        .unwrap_or_else(|| "".to_string());
                                    let on_delete = {
                                        let store = use_store();
                                        let date = entry.date;
                                        let confirm_delete = confirm_delete;
                                        let confirm_title = confirm_title;
                                        let confirm_action = confirm_action;
                                        move |_: leptos::ev::MouseEvent| {
                                            confirm_title.set("Delete measurement?".to_string());
                                            confirm_delete.set(Some(date.to_string()));
                                            let store = store.clone();
                                            confirm_action.set(Some(Rc::new(move || {
                                                store
                                                    .data
                                                    .update(|d| d.measurements.retain(|m| m.date != date));
                                                store.is_dirty.set(true);
                                                store.save();
                                            })));
                                        }
                                    };
                                let on_edit = {
                                    let date = entry.date;
                                    let edit_measurement_date = edit_measurement_date;
                                    let edit_measurement_date_text = edit_measurement_date_text;
                                    let edit_measurement_weight = edit_measurement_weight;
                                    let edit_measurement_waist = edit_measurement_waist;
                                    let edit_measurement_hip = edit_measurement_hip;
                                    let edit_measurement_unit = edit_measurement_unit;
                                    let weight_value = entry
                                        .weight
                                        .map(|v| format!("{:.2}", v))
                                        .unwrap_or_default();
                                    let waist_value = entry
                                        .waist
                                        .map(|v| format!("{:.1}", v))
                                        .unwrap_or_default();
                                    let hip_value = entry
                                        .hip
                                        .map(|v| format!("{:.1}", v))
                                        .unwrap_or_default();
                                    let unit_value = entry
                                        .bodyMeasurementUnit
                                        .as_ref()
                                        .map(|u| format!("{:?}", u))
                                        .unwrap_or_default();
                                    let date_text_edit = date_text.clone();
                                    move |_| {
                                        edit_measurement_date.set(Some(date));
                                        edit_measurement_date_text.set(date_text_edit.clone());
                                        edit_measurement_weight.set(weight_value.clone());
                                        edit_measurement_waist.set(waist_value.clone());
                                        edit_measurement_hip.set(hip_value.clone());
                                        edit_measurement_unit.set(unit_value.clone());
                                    }
                                };
                                    view! {
                                        <tr>
                                            <td>{date_text}</td>
                                            <td>{format!("{} {}", weight, weight_unit)}</td>
                                            <td>{format!("{} {}", waist, unit)}</td>
                                            <td>{format!("{} {}", hip, unit)}</td>
                                            <td>
                                                <button type="button" on:click=on_edit>"Edit"</button>
                                                <button type="button" on:click=on_delete>"Delete"</button>
                                            </td>
                                        </tr>
                                    }
                                }
                            />
                        </tbody>
                    </table>
                </Show>
            </section>

            <Show when=move || editing_key.get().is_some()>
                <div class="modal-backdrop" on:click=move |_| editing_key.set(None)>
                    <div class="modal" on:click=move |ev| ev.stop_propagation()>
                        <h3>"Edit Dosage Entry"</h3>
                        <label>"Date"</label>
                        <input
                            type="date"
                            on:input=move |ev| editing_date.set(event_target_value(&ev))
                            prop:value=move || editing_date.get()
                        />
                        <label>"Dose"</label>
                        <input
                            type="number"
                            step="0.01"
                            on:input=move |ev| editing_dose.set(event_target_value(&ev))
                            prop:value=move || editing_dose.get()
                        />
                        <label>"Notes"</label>
                        <textarea
                            rows="3"
                            on:input=move |ev| editing_note.set(event_target_value(&ev))
                            prop:value=move || editing_note.get()
                        ></textarea>
                        <div class="modal-actions">
                            <button type="button" on:click={
                                let on_save_edit = on_save_edit.clone();
                                move |_| (on_save_edit)()
                            }>
                                "Save"
                            </button>
                            <button type="button" on:click=on_cancel_edit>"Cancel"</button>
                        </div>
                    </div>
                </div>
            </Show>

            <Show when=move || confirm_delete.get().is_some()>
                <div class="modal-backdrop" on:click=move |_| confirm_delete.set(None)>
                    <div class="modal" on:click=move |ev| ev.stop_propagation()>
                        <h3>{move || confirm_title.get()}</h3>
                        <p>"This action cannot be undone."</p>
                        <div class="modal-actions">
                            <button type="button" on:click={
                                let confirm_action = confirm_action.clone();
                                let confirm_delete = confirm_delete;
                                move |_| {
                                    if let Some(action) = confirm_action.get() {
                                        action();
                                    }
                                    confirm_delete.set(None);
                                }
                            }>
                                "Delete"
                            </button>
                            <button type="button" on:click={
                                let confirm_delete = confirm_delete;
                                move |_| confirm_delete.set(None)
                            }>
                                "Cancel"
                            </button>
                        </div>
                    </div>
                </div>
            </Show>

            <Show when=move || edit_blood_date.get().is_some()>
                <div class="modal-backdrop" on:click=move |_| edit_blood_date.set(None)>
                    <div class="modal" on:click=move |ev| ev.stop_propagation()>
                        <h3>"Edit Blood Test"</h3>
                        <label>"Date"</label>
                        <input
                            type="date"
                            on:input=move |ev| edit_blood_date_text.set(event_target_value(&ev))
                            prop:value=move || edit_blood_date_text.get()
                        />
                        <label>"Estradiol"</label>
                        <div class="inline">
                            <input
                                type="number"
                                step="0.01"
                                on:input=move |ev| edit_blood_e2.set(event_target_value(&ev))
                                prop:value=move || edit_blood_e2.get()
                            />
                            <select
                                on:change=move |ev| edit_blood_e2_unit.set(event_target_value(&ev))
                                prop:value=move || edit_blood_e2_unit.get()
                            >
                                <option value="pg/mL">"pg/mL"</option>
                                <option value="pmol/L">"pmol/L"</option>
                            </select>
                        </div>
                        <label>"Testosterone"</label>
                        <div class="inline">
                            <input
                                type="number"
                                step="0.01"
                                on:input=move |ev| edit_blood_t.set(event_target_value(&ev))
                                prop:value=move || edit_blood_t.get()
                            />
                            <select
                                on:change=move |ev| edit_blood_t_unit.set(event_target_value(&ev))
                                prop:value=move || edit_blood_t_unit.get()
                            >
                                <option value="ng/dL">"ng/dL"</option>
                                <option value="nmol/L">"nmol/L"</option>
                            </select>
                        </div>
                        <label>"Notes"</label>
                        <textarea
                            rows="3"
                            on:input=move |ev| edit_blood_notes.set(event_target_value(&ev))
                            prop:value=move || edit_blood_notes.get()
                        ></textarea>
                        <div class="modal-actions">
                            <button type="button" on:click={
                                let store = store_blood_modal.clone();
                                move |_: leptos::ev::MouseEvent| {
                                    let date = match edit_blood_date.get() {
                                        Some(value) => value,
                                        None => return,
                                    };
                                    let new_date = parse_date_or_now(&edit_blood_date_text.get());
                                    let e2_value = edit_blood_e2.get().trim().parse::<f64>().ok();
                                    let t_value = edit_blood_t.get().trim().parse::<f64>().ok();
                                    let e2_unit = edit_blood_e2_unit.get();
                                    let t_unit = edit_blood_t_unit.get();
                                    let notes = edit_blood_notes.get();
                                    store.data.update(|d| {
                                        for entry in &mut d.bloodTests {
                                            if entry.date == date {
                                                entry.date = new_date;
                                                entry.estradiolLevel = e2_value;
                                                entry.testLevel = t_value;
                                                entry.estradiolUnit = parse_hormone_unit(&e2_unit);
                                                entry.testUnit = parse_hormone_unit(&t_unit);
                                                entry.notes = if notes.trim().is_empty() {
                                                    None
                                                } else {
                                                    Some(notes.clone())
                                                };
                                            }
                                        }
                                    });
                                    store.is_dirty.set(true);
                                    store.save();
                                    edit_blood_date.set(None);
                                }
                            }>
                                "Save"
                            </button>
                            <button type="button" on:click={
                                let edit_blood_date = edit_blood_date;
                                move |_: leptos::ev::MouseEvent| edit_blood_date.set(None)
                            }>
                                "Cancel"
                            </button>
                        </div>
                    </div>
                </div>
            </Show>

            <Show when=move || edit_measurement_date.get().is_some()>
                <div class="modal-backdrop" on:click=move |_| edit_measurement_date.set(None)>
                    <div class="modal" on:click=move |ev| ev.stop_propagation()>
                        <h3>"Edit Measurement"</h3>
                        <label>"Date"</label>
                        <input
                            type="date"
                            on:input=move |ev| edit_measurement_date_text.set(event_target_value(&ev))
                            prop:value=move || edit_measurement_date_text.get()
                        />
                        <label>"Weight"</label>
                        <input
                            type="number"
                            step="0.01"
                            on:input=move |ev| edit_measurement_weight.set(event_target_value(&ev))
                            prop:value=move || edit_measurement_weight.get()
                        />
                        <label>"Waist"</label>
                        <input
                            type="number"
                            step="0.1"
                            on:input=move |ev| edit_measurement_waist.set(event_target_value(&ev))
                            prop:value=move || edit_measurement_waist.get()
                        />
                        <label>"Hip"</label>
                        <input
                            type="number"
                            step="0.1"
                            on:input=move |ev| edit_measurement_hip.set(event_target_value(&ev))
                            prop:value=move || edit_measurement_hip.get()
                        />
                        <label>"Unit"</label>
                        <input
                            type="text"
                            on:input=move |ev| edit_measurement_unit.set(event_target_value(&ev))
                            prop:value=move || edit_measurement_unit.get()
                        />
                        <div class="modal-actions">
                            <button type="button" on:click={
                                let store = store_measure_modal.clone();
                                move |_: leptos::ev::MouseEvent| {
                                    let date = match edit_measurement_date.get() {
                                        Some(value) => value,
                                        None => return,
                                    };
                                    let new_date = parse_date_or_now(&edit_measurement_date_text.get());
                                    let weight = edit_measurement_weight.get().trim().parse::<f64>().ok();
                                    let waist = edit_measurement_waist.get().trim().parse::<f64>().ok();
                                    let hip = edit_measurement_hip.get().trim().parse::<f64>().ok();
                                    let unit = edit_measurement_unit.get();
                                    store.data.update(|d| {
                                        for entry in &mut d.measurements {
                                            if entry.date == date {
                                                entry.date = new_date;
                                                entry.weight = weight;
                                                entry.waist = waist;
                                                entry.hip = hip;
                                                entry.bodyMeasurementUnit = parse_length_unit(&unit);
                                            }
                                        }
                                    });
                                    store.is_dirty.set(true);
                                    store.save();
                                    edit_measurement_date.set(None);
                                }
                            }>
                                "Save"
                            </button>
                            <button type="button" on:click={
                                let edit_measurement_date = edit_measurement_date;
                                move |_: leptos::ev::MouseEvent| edit_measurement_date.set(None)
                            }>
                                "Cancel"
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
        }
        .into_view(),
    )
    page_shell("View")
}

#[component]
fn StatsPage() -> impl IntoView {
    let store = use_store();
    let settings = store.settings;
    let display_unit = move || settings.get().displayEstradiolUnit;
    let ics_secret = move || settings.get().icsSecret.unwrap_or_default();
    let auto_backfill = move || settings.get().enableAutoBackfill;

    let store_toggle = store.clone();
    let on_toggle_backfill = move |ev: leptos::ev::Event| {
        let enabled = event_target_checked(&ev);
        store_toggle
            .settings
            .update(|s| s.enableAutoBackfill = enabled);
        store_toggle.is_dirty.set(true);
        store_toggle.save();
    };

    let store_secret = store.clone();
    let on_secret_input = move |ev: leptos::ev::Event| {
        let value = event_target_value(&ev);
        store_secret.settings.update(|s| {
            s.icsSecret = if value.trim().is_empty() {
                None
            } else {
                Some(value)
            };
        });
        store_secret.is_dirty.set(true);
        store_secret.save();
    };

    let store_unit = store.clone();
    let on_display_unit_change = move |ev: leptos::ev::Event| {
        let value = event_target_value(&ev);

        let unit = match value.as_str() {
            "pmol/L" => Some(HormoneUnits::E2PmolL),
            "pg/mL" => Some(HormoneUnits::E2PgMl),
            _ => None,
        };
        store_unit
            .settings
            .update(|s| s.displayEstradiolUnit = unit);
        store_unit.is_dirty.set(true);
        store_unit.save();
    };

    page_layout(
        "Settings",
        view! {
            <form>
                <label>
                    <input
                        type="checkbox"
                        on:change=on_toggle_backfill
                        prop:checked=move || auto_backfill()
                    />
                    " Enable auto backfill"
                </label>

                <label>
                        <input
                            type="checkbox"
                            on:change={
                                let store = store.clone();
                                move |ev| {
                                    let enabled = event_target_checked(&ev);
                                    store.settings.update(|s| s.enableBloodTestSchedule = Some(enabled));
                                    store.is_dirty.set(true);
                                    store.save();
                                }
                            }
                            prop:checked=move || store.settings.get().enableBloodTestSchedule.unwrap_or(false)
                        />

                    " Enable blood test schedule"
                </label>

                <label>"Blood test interval (months)"</label>
                        <input
                            type="number"
                            step="1"
                            min="1"
                            on:input={
                                let store = store.clone();
                                move |ev| {
                                    let value = event_target_value(&ev);
                                    let parsed = value.parse::<f64>().ok();
                                    store.settings.update(|s| s.bloodTestIntervalMonths = parsed);
                                    store.is_dirty.set(true);
                                    store.save();
                                }
                            }
                            prop:value=move || store
                                .settings
                                .get()
                                .bloodTestIntervalMonths
                                .map(|v| v.to_string())
                                .unwrap_or_else(|| "".to_string())
                        />


                <label>"ICS Secret"</label>
                <input
                    type="text"
                    placeholder="Optional secret"
                    on:input=on_secret_input
                    prop:value=move || ics_secret()
                />

                <label>"Display Estradiol Unit"</label>
                <select on:change=on_display_unit_change>
                    <option value="pmol/L" selected=move || display_unit() == Some(HormoneUnits::E2PmolL)>
                        "pmol/L"
                    </option>
                    <option value="pg/mL" selected=move || display_unit() == Some(HormoneUnits::E2PgMl)>
                        "pg/mL"
                    </option>
                </select>

                <button
                    type="button"
                    on:click={
                        let store = store.clone();
                        move |_| store.save()
                    }
                    prop:disabled=move || store.is_saving.get()
                >
                    "Save Settings"
                </button>
            </form>
        }
        .into_view(),
    )
    page_shell("Stats")
}

#[component]
fn BackupPage() -> impl IntoView {
    let store = use_store();
    let data = move || store.data.get();
    let settings = move || store.settings.get();
    let export_text = move || {
        let payload = serde_json::json!({
            "data": data(),
            "settings": settings(),
        });
        serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
    };

    page_layout(
        "Backup",
        view! {
            <p class="muted">"Export your full data + settings bundle for safekeeping."</p>
            <textarea rows="12" readonly prop:value=export_text></textarea>
            <div class="primary-actions">
                <button
                    type="button"
                    on:click={
                        let store = store.clone();
                        move |_| store.save()
                    }
                >
                    "Save to Disk"
                </button>
            </div>
        }
        .into_view(),
    )
    page_shell("Backup")
}

#[component]
fn CalcPage() -> impl IntoView {
    let value = create_rw_signal("".to_string());
    let from_unit = create_rw_signal("pg/mL".to_string());
    let to_unit = create_rw_signal("pmol/L".to_string());
    let result = create_rw_signal(None::<String>);

    let on_convert = move |_| {
        let parsed = value.get().trim().parse::<f64>().ok();
        let parsed = match parsed {
            Some(value) => value,
            None => {
                result.set(Some("Enter a valid value.".to_string()));
                return;
            }
        };
        let from = from_unit.get();
        let to = to_unit.get();
        let output = hrt_shared::convert::convert_hormone(
            parsed,
            hrt_shared::types::Hormone::Estradiol,
            &from,
            &to,
        )
        .map(|v| format!("{:.3} {}", v, to))
        .unwrap_or_else(|err| err);
        result.set(Some(output));
    };

    page_layout(
        "Calculator",
        view! {
            <form>
                <label>"Value"</label>
                <input
                    type="number"
                    step="0.01"
                    on:input=move |ev| value.set(event_target_value(&ev))
                    prop:value=move || value.get()
                />

                <label>"From"</label>
                <select on:change=move |ev| from_unit.set(event_target_value(&ev))>
                    <option value="pg/mL">"pg/mL"</option>
                    <option value="pmol/L">"pmol/L"</option>
                    <option value="ng/dL">"ng/dL"</option>
                    <option value="nmol/L">"nmol/L"</option>
                </select>

                <label>"To"</label>
                <select on:change=move |ev| to_unit.set(event_target_value(&ev))>
                    <option value="pmol/L">"pmol/L"</option>
                    <option value="pg/mL">"pg/mL"</option>
                    <option value="ng/dL">"ng/dL"</option>
                    <option value="nmol/L">"nmol/L"</option>
                </select>

                <button type="button" on:click=on_convert>"Convert"</button>
                <Show when=move || result.get().is_some()>
                    <p class="muted">{move || result.get().unwrap_or_default()}</p>
                </Show>
            </form>
            <PlaceholderChart title="Conversion Curve" />
        }
        .into_view(),
    )
    page_shell("Calculator")
}

#[component]
fn VialsPage() -> impl IntoView {
    let store = use_store();
    let vials = move || store.data.get().vials;

    page_layout(
        "Vials",
        view! {
            <Show
                when=move || !vials().is_empty()
                fallback=move || view! { <div class="empty-state">"No vials recorded yet."</div> }
            >
                <table class="table">
                    <thead>
                        <tr>
                            <th>"Batch"</th>
                            <th>"Ester"</th>
                            <th>"Concentration"</th>
                            <th>"Use by"</th>
                            <th>"Status"</th>
                        </tr>
                    </thead>
                    <tbody>
                        <For
                            each=vials
                            key=|vial| vial.id.clone()
                            children=move |vial| {
                                let use_by = vial
                                    .useBy
                                    .and_then(|value| {
                                        Local.timestamp_millis_opt(value)
                                            .single()
                                            .map(|d| d.format("%Y-%m-%d").to_string())
                                    })
                                    .unwrap_or_else(|| "-".to_string());
                                let status = if vial.isSpent.unwrap_or(false) {
                                    "Spent"
                                } else {
                                    "Active"
                                };
                                let concentration = vial
                                    .concentrationMgPerMl
                                    .map(|v| format!("{:.2} mg/mL", v))
                                    .unwrap_or_else(|| "-".to_string());
                                view! {
                                    <tr>
                                        <td><A href=format!("/vials/{}", vial.id.clone())>{vial.batchNumber.clone().unwrap_or_else(|| "-".to_string())}</A></td>
                                        <td>{vial.esterKind.clone().unwrap_or_else(|| "-".to_string())}</td>
                                        <td>{concentration}</td>
                                        <td>{use_by}</td>
                                        <td>{status}</td>
                                    </tr>
                                }
                            }
                        />
                    </tbody>
                </table>
                <div class="primary-actions">
                    <A href="/vials/create">"Add Vial"</A>
                </div>
            </Show>
        }
        .into_view(),
    )
    page_shell("Vials")
}

#[component]
fn VialsCreatePage() -> impl IntoView {
    let store = use_store();
    let batch_number = create_rw_signal("".to_string());
    let ester_kind = create_rw_signal("".to_string());
    let concentration = create_rw_signal("".to_string());
    let use_by = create_rw_signal("".to_string());
    let error = create_rw_signal(None::<String>);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        error.set(None);
        if batch_number.get().trim().is_empty() {
            error.set(Some("Batch number is required.".to_string()));
            return;
        }

        let created = js_sys::Date::now() as i64;
        let use_by_ms = parse_date_or_now(&use_by.get());
        let concentration_value = concentration.get().trim().parse::<f64>().ok();
        let entry = hrt_shared::types::Vial {
            id: format!("vial-{}", created),
            esterKind: if ester_kind.get().trim().is_empty() {
                None
            } else {
                Some(ester_kind.get())
            },
            suspensionOil: None,
            otherIngredients: None,
            batchNumber: Some(batch_number.get()),
            source: None,
            concentrationMgPerMl: concentration_value,
            isSpent: Some(false),
            spentAt: None,
            useBy: Some(use_by_ms),
            createdAt: created,
            subVials: vec![],
        };

        let mut data = store.data.get();
        data.vials.push(entry);
        store.data.set(data);
        store.is_dirty.set(true);
        store.save();
        batch_number.set("".to_string());
        ester_kind.set("".to_string());
        concentration.set("".to_string());
        use_by.set("".to_string());
    };

    page_layout(
        "Create Vial",
        view! {
            <form on:submit=on_submit>
                <label>"Batch Number"</label>
                <input
                    type="text"
                    on:input=move |ev| batch_number.set(event_target_value(&ev))
                    prop:value=move || batch_number.get()
                />

                <label>"Ester"</label>
                <input
                    type="text"
                    on:input=move |ev| ester_kind.set(event_target_value(&ev))
                    prop:value=move || ester_kind.get()
                />

                <label>"Concentration (mg/mL)"</label>
                <input
                    type="number"
                    step="0.01"
                    on:input=move |ev| concentration.set(event_target_value(&ev))
                    prop:value=move || concentration.get()
                />

                <label>"Use By"</label>
                <input
                    type="date"
                    on:input=move |ev| use_by.set(event_target_value(&ev))
                    prop:value=move || use_by.get()
                />

                <button type="submit">"Add Vial"</button>
                <Show when=move || error.get().is_some()>
                    <p class="error">{move || error.get().unwrap_or_default()}</p>
                </Show>
            </form>
        }
        .into_view(),
    )
    page_shell("Create Vial")
}

#[component]
fn VialsDetailPage() -> impl IntoView {
    let store = use_store();
    let params = use_params_map();
    let vial_id = move || params.with(|p| p.get("id").cloned().unwrap_or_else(|| "".into()));
    let vial = move || {
        let id = vial_id();
        store.data.get().vials.into_iter().find(|v| v.id == id)
    };
    let vial_store = store.clone();
    let sub_label = create_rw_signal("".to_string());
    let sub_notes = create_rw_signal("".to_string());

    let render_vial = move |entry: hrt_shared::types::Vial| {
        let created = Local
            .timestamp_millis_opt(entry.createdAt)
            .single()
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "".to_string());
        let use_by = entry
            .useBy
            .and_then(|value| {
                Local.timestamp_millis_opt(value)
                    .single()
                    .map(|d| d.format("%Y-%m-%d").to_string())
            })
            .unwrap_or_else(|| "-".to_string());
        let spent_label = if entry.isSpent.unwrap_or(false) {
            "Spent"
        } else {
            "Active"
        };
        let entry_id = entry.id.clone();
        let store_toggle = vial_store.clone();
        let store_subvial = vial_store.clone();
        let is_spent = entry.isSpent.unwrap_or(false);
        let title = entry.batchNumber.clone().unwrap_or_else(|| "Vial".to_string());
        let ester = entry.esterKind.clone().unwrap_or_else(|| "-".to_string());
        let concentration = entry
            .concentrationMgPerMl
            .map(|v| format!("{:.2} mg/mL", v))
            .unwrap_or_else(|| "-".to_string());
        let sub_vials = create_rw_signal(entry.subVials);

        view! {
            <section>
                <h2>{title}</h2>
                <p>{"Ester: "}{ester}</p>
                <p>{"Concentration: "}{concentration}</p>
                <p>{"Created: "}{created}</p>
                <p>{"Use by: "}{use_by}</p>
                <p>{"Status: "}{spent_label}</p>

                <div class="primary-actions">
                    <button
                        type="button"
                        on:click=move |_| {
                            store_toggle.data.update(|d| {
                                if let Some(target) = d.vials.iter_mut().find(|v| v.id == entry_id) {
                                    let next = !target.isSpent.unwrap_or(false);
                                    target.isSpent = Some(next);
                                    target.spentAt = if next { Some(js_sys::Date::now() as i64) } else { None };
                                }
                            });
                            store_toggle.is_dirty.set(true);
                            store_toggle.save();
                        }
                    >
                        {if is_spent { "Mark Active" } else { "Mark Spent" }}
                    </button>
                </div>

                <form class="subvial-form" on:submit={
                    let store_subvial = store_subvial.clone();
                    let entry_id = entry_id.clone();
                    let sub_vials = sub_vials.clone();
                    move |ev| {
                        ev.prevent_default();
                        let label = sub_label.get();
                        let notes = sub_notes.get();
                        if label.trim().is_empty() {
                            return;
                        }
                        let mut next_list = sub_vials.get();
                        store_subvial.data.update(|d| {
                            if let Some(target) = d.vials.iter_mut().find(|v| v.id == entry_id) {
                                let stamp = js_sys::Date::now() as i64;
                                let new_sub = hrt_shared::types::SubVial {
                                    id: format!("sub-{}-{}", entry_id, stamp),
                                    personalNumber: label.trim().to_string(),
                                    createdAt: stamp,
                                    notes: if notes.trim().is_empty() { None } else { Some(notes.clone()) },
                                };
                                target.subVials.push(new_sub.clone());
                                next_list.push(new_sub);
                            }
                        });
                        sub_vials.set(next_list);
                        store_subvial.is_dirty.set(true);
                        store_subvial.save();
                        sub_label.set("".to_string());
                        sub_notes.set("".to_string());
                    }
                }>
                    <label>"New Sub-Vial Label"</label>
                    <input
                        type="text"
                        placeholder="SUB-1"
                        on:input=move |ev| sub_label.set(event_target_value(&ev))
                        prop:value=move || sub_label.get()
                    />
                    <label>"Notes"</label>
                    <input
                        type="text"
                        placeholder="Optional"
                        on:input=move |ev| sub_notes.set(event_target_value(&ev))
                        prop:value=move || sub_notes.get()
                    />
                    <button type="submit">"Add Sub-Vial"</button>
                </form>

                <Show
                    when=move || !sub_vials.get().is_empty()
                    fallback=move || view! { <div class="empty-state">"No sub-vials yet."</div> }
                >
                    <table class="table">
                        <thead>
                            <tr>
                                <th>"Label"</th>
                                <th>"Created"</th>
                                <th>"Notes"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <For
                                each=move || sub_vials.get()
                                key=|sub| sub.id.clone()
                                children=move |sub| {
                                    let created = Local
                                        .timestamp_millis_opt(sub.createdAt)
                                        .single()
                                        .map(|d| d.format("%Y-%m-%d").to_string())
                                        .unwrap_or_else(|| "".to_string());
                                    view! {
                                        <tr>
                                            <td>{sub.personalNumber}</td>
                                            <td>{created}</td>
                                            <td>{sub.notes.unwrap_or_default()}</td>
                                        </tr>
                                    }
                                }
                            />
                        </tbody>
                    </table>
                </Show>
            </section>
        }
        .into_view()
    };
            let entry_id = entry.id.clone();
            let store_toggle = vial_store.clone();
            let store_subvial = vial_store.clone();
            let is_spent = entry.isSpent.unwrap_or(false);
            let title = entry
                .batchNumber
                .clone()
                .unwrap_or_else(|| "Vial".to_string());
            let ester = entry.esterKind.clone().unwrap_or_else(|| "-".to_string());
            let concentration = entry
                .concentrationMgPerMl
                .map(|v| format!("{:.2} mg/mL", v))
                .unwrap_or_else(|| "-".to_string());
            let sub_vials = create_rw_signal(entry.subVials);

            view! {
                <section>
                    <h2>{title}</h2>
                    <p>{"Ester: "}{ester}</p>
                    <p>{"Concentration: "}{concentration}</p>
                    <p>{"Created: "}{created}</p>
                    <p>{"Use by: "}{use_by}</p>
                    <p>{"Status: "}{spent_label}</p>

                    <div class="primary-actions">
                        <button
                            type="button"
                            on:click=move |_| {
                                store_toggle.data.update(|d| {
                                    if let Some(target) = d.vials.iter_mut().find(|v| v.id == entry_id) {
                                        let next = !target.isSpent.unwrap_or(false);
                                        target.isSpent = Some(next);
                                        target.spentAt = if next { Some(js_sys::Date::now() as i64) } else { None };
                                    }
                                });
                                store_toggle.is_dirty.set(true);
                                store_toggle.save();
                            }
                        >
                            {if is_spent { "Mark Active" } else { "Mark Spent" }}
                        </button>
                    </div>

                    <form class="subvial-form" on:submit={
                        let store_subvial = store_subvial.clone();
                        let entry_id = entry_id.clone();
                        let sub_vials = sub_vials.clone();
                        move |ev| {
                            ev.prevent_default();
                            let label = sub_label.get();
                            let notes = sub_notes.get();
                            if label.trim().is_empty() {
                                return;
                            }
                            let mut next_list = sub_vials.get();
                            store_subvial.data.update(|d| {
                                if let Some(target) = d.vials.iter_mut().find(|v| v.id == entry_id) {
                                    let stamp = js_sys::Date::now() as i64;
                                    let new_sub = hrt_shared::types::SubVial {
                                        id: format!("sub-{}-{}", entry_id, stamp),
                                        personalNumber: label.trim().to_string(),
                                        createdAt: stamp,
                                        notes: if notes.trim().is_empty() { None } else { Some(notes.clone()) },
                                    };
                                    target.subVials.push(new_sub.clone());
                                    next_list.push(new_sub);
                                }
                            });
                            sub_vials.set(next_list);
                            store_subvial.is_dirty.set(true);
                            store_subvial.save();
                            sub_label.set("".to_string());
                            sub_notes.set("".to_string());
                        }
                    }>
                        <label>"New Sub-Vial Label"</label>
                        <input
                            type="text"
                            placeholder="SUB-1"
                            on:input=move |ev| sub_label.set(event_target_value(&ev))
                            prop:value=move || sub_label.get()
                        />
                        <label>"Notes"</label>
                        <input
                            type="text"
                            placeholder="Optional"
                            on:input=move |ev| sub_notes.set(event_target_value(&ev))
                            prop:value=move || sub_notes.get()
                        />
                        <button type="submit">"Add Sub-Vial"</button>
                    </form>

                    <Show
                        when=move || !sub_vials.get().is_empty()
                        fallback=move || view! { <div class="empty-state">"No sub-vials yet."</div> }
                    >
                        <table class="table">
                            <thead>
                                <tr>
                                    <th>"Label"</th>
                                    <th>"Created"</th>
                                    <th>"Notes"</th>
                                </tr>
                            </thead>
                            <tbody>
                                <For
                                    each=move || sub_vials.get()
                                    key=|sub| sub.id.clone()
                                    children=move |sub| {
                                        let created = Local
                                            .timestamp_millis_opt(sub.createdAt)
                                            .single()
                                            .map(|d| d.format("%Y-%m-%d").to_string())
                                            .unwrap_or_else(|| "".to_string());
                                        view! {
                                            <tr>
                                                <td>{sub.personalNumber}</td>
                                                <td>{created}</td>
                                                <td>{sub.notes.unwrap_or_default()}</td>
                                            </tr>
                                        }
                                    }
                                />
                            </tbody>
                        </table>
                    </Show>
                </section>
            }
            .into_view()
        })
    };

    page_layout(
        "Vial Detail",
        view! {
            <Show
                when=move || vial().is_some()
                fallback=move || view! { <div class="empty-state">"Vial not found."</div> }
            >
                {move || vial().map(render_vial).unwrap_or_else(|| view! {}.into_view())}
            </Show>
        }
        .into_view(),
    )
    let id = move || params.with(|p| p.get("id").cloned().unwrap_or_else(|| "?".into()));
    view! {
        <section>
            <h1>"Vial Detail"</h1>
            <p>"Vial id: "{id}</p>
            <p>"Placeholder for Rust UI rewrite."</p>
        </section>
    }
}

#[component]
fn EstrannaisePage() -> impl IntoView {
    page_layout(
        "Estrannaise",
        view! {
            <p class="muted">"Charting comes in Phase 6. These controls mirror the original inputs."</p>
            <form>
                <label>"Model"</label>
                <select>
                    <option value="EB im">"EB im"</option>
                    <option value="EV im">"EV im"</option>
                    <option value="EEn im">"EEn im"</option>
                    <option value="EC im">"EC im"</option>
                    <option value="EUn im">"EUn im"</option>
                    <option value="EUn casubq">"EUn casubq"</option>
                    <option value="patch tw">"patch tw"</option>
                    <option value="patch ow">"patch ow"</option>
                </select>

                <label>"Dose (mg)"</label>
                <input type="number" step="0.1" />

                <label>"Every (days)"</label>
                <input type="number" step="1" />

                <label>"Step fudge factor"</label>
                <input type="number" step="0.1" />

                <label>"Weeks"</label>
                <input type="number" step="1" />
            </form>
            <PlaceholderChart title="Estrannaise Preview" />
        }
        .into_view(),
    )
    page_shell("Estrannaise")
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn mount_app() {
    mount_to_body(App);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn serve() {
    let addr = std::env::var("HRT_WEB_ADDR").unwrap_or_else(|_| "127.0.0.1:4100".to_string());

    let app = Router::new()
        .nest_service("/pkg", ServeDir::new("target/site/pkg"))
        .route("/", get(index_handler))
        .fallback(get(index_handler));

    println!("Web UI listening on http://{addr}");
    let runtime = tokio::runtime::Runtime::new().expect("Failed to start runtime");
    runtime.block_on(async move {
        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .expect("Failed to bind web server");
        axum::serve(listener, app).await.expect("web server error");
    });
}

#[cfg(not(target_arch = "wasm32"))]
async fn index_handler() -> axum::response::Html<String> {
    axum::response::Html(read_index())
}

#[cfg(not(target_arch = "wasm32"))]
fn read_index() -> String {
    let candidates = ["target/site/index.html", "crates/web/index.html"];
    for path in candidates {
        if let Ok(contents) = std::fs::read_to_string(path) {
            return contents;
        }
    }
    "Missing index.html".to_string()
}

fn parse_date_or_now(value: &str) -> i64 {
    if value.trim().is_empty() {
        return js_sys::Date::now() as i64;
    }
    let parts: Vec<i64> = value
        .split('-')
        .filter_map(|v| v.parse::<i64>().ok())
        .collect();
    if parts.len() != 3 {
        return js_sys::Date::now() as i64;
    }
    let (year, month, day) = (parts[0], parts[1], parts[2]);
    if year <= 0 || month == 0 || day == 0 {
        return js_sys::Date::now() as i64;
    }
    let date = js_sys::Date::new_with_year_month_day(year as u32, (month - 1) as i32, day as i32);
    date.get_time() as i64
}

fn parse_hormone_unit(value: &str) -> Option<HormoneUnits> {
    match value {
        "pg/mL" => Some(HormoneUnits::E2PgMl),
        "pmol/L" => Some(HormoneUnits::E2PmolL),
        "ng/dL" => Some(HormoneUnits::TNgDl),
        "nmol/L" => Some(HormoneUnits::TNmolL),
        _ => None,
    }
}

fn parse_length_unit(value: &str) -> Option<LengthUnit> {
    match value {
        "cm" => Some(LengthUnit::CM),
        "in" => Some(LengthUnit::IN),
        _ => None,
    }
}

#[component]
fn PlaceholderChart(title: &'static str) -> impl IntoView {
    let canvas_id = format!("chart-{}", title.to_lowercase().replace(' ', "-"));
    let canvas_draw_id = canvas_id.clone();

    let draw = move || {
        let backend = CanvasBackend::new(&canvas_draw_id)
            .expect("canvas backend")
            .into_drawing_area();
        backend.fill(&RGBColor(15, 17, 26)).ok();

        let chart = ChartBuilder::on(&backend)
            .margin(16)
            .caption(title, ("Quicksand", 18))
            .x_label_area_size(24)
            .y_label_area_size(32)
            .build_cartesian_2d(0..10, 0..10)
            .ok();
        if let Some(mut chart) = chart {
            chart
                .configure_mesh()
                .disable_mesh()
                .label_style(
                    ("Quicksand", 12)
                        .into_font()
                        .color(&RGBColor(180, 167, 198)),
                )
                .axis_style(&RGBColor(80, 70, 100))
                .draw()
                .ok();
            chart
                .draw_series(LineSeries::new(
                    (0..10).map(|x| (x, (x * x) % 10)),
                    &RGBColor(243, 154, 181),
                ))
                .ok();
        }
        backend.present().ok();
    };

    view! {
        <div class="chart-card">
            <canvas id=canvas_id width="640" height="280" on:load=move |_| draw()></canvas>
        </div>
    }
}