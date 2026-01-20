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
use gloo_events::EventListener;
use hrt_shared::estrannaise::e2_multidose_3c;
use hrt_shared::types::{
    BloodTest, DosageHistoryEntry, EstrannaiseModel, HormoneUnits, InjectableEstradiols,
    LengthUnit, Settings,
};
use leptos::window;
use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::HtmlCanvasElement;

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

    let x_axis_mode = create_rw_signal("date".to_string());
    let time_range_days = create_rw_signal(365_i64);
    let show_medications = create_rw_signal(true);
    let show_e2 = create_rw_signal(true);
    let show_t = create_rw_signal(true);
    let show_prog = create_rw_signal(false);
    let show_fsh = create_rw_signal(false);
    let show_lh = create_rw_signal(false);
    let show_prolactin = create_rw_signal(false);
    let show_shbg = create_rw_signal(false);
    let show_fai = create_rw_signal(false);
    let view_zoom = create_rw_signal(ViewZoom::default());
    let view_tooltip = create_rw_signal(None::<ChartTooltip>);

    let view_chart_state = create_memo({
        let settings = store.settings;
        move |_| {
            let data_value = data.get();
            let settings_value = settings.get();
            compute_view_chart_state(
                &data_value,
                &settings_value,
                &x_axis_mode.get(),
                time_range_days.get(),
                show_medications.get(),
                show_e2.get(),
                show_t.get(),
                show_prog.get(),
                show_fsh.get(),
                show_lh.get(),
                show_prolactin.get(),
                show_shbg.get(),
                show_fai.get(),
            )
        }
    });

    const VIEW_CANVAS_ID: &str = "view-chart-canvas";
    let view_drag = Rc::new(RefCell::new(None::<DragState>));

    let on_view_mouse_move = {
        let view_chart_state = view_chart_state.clone();
        let view_zoom = view_zoom;
        let view_drag = view_drag.clone();
        let view_tooltip = view_tooltip;
        move |ev: leptos::ev::MouseEvent| {
            let Some(canvas) = window()
                .document()
                .and_then(|doc| doc.get_element_by_id(VIEW_CANVAS_ID))
                .and_then(|el| el.dyn_into::<HtmlCanvasElement>().ok())
            else {
                return;
            };
            let rect = canvas.get_bounding_client_rect();
            let cursor_x = ev.client_x() as f64 - rect.left();
            let cursor_y = ev.client_y() as f64 - rect.top();
            let state = view_chart_state.get();
            let zoom = view_zoom.get();
            let x_min = zoom.x_min.unwrap_or(state.domain_min);
            let x_max = zoom.x_max.unwrap_or(state.domain_max);
            let padding = chart_padding();
            let (width, height, domain_span, y_span) = compute_chart_bounds(
                rect.width(),
                rect.height(),
                padding,
                x_min,
                x_max,
                state.y_min,
                state.y_max,
            );
            if let Some(drag) = view_drag.borrow().as_ref() {
                view_tooltip.set(None);
                let delta_px = cursor_x - drag.start_x;
                let span = x_max - x_min;
                let delta_domain = -(delta_px / width) * span;
                let next_min = drag.start_min + delta_domain;
                let next_max = drag.start_max + delta_domain;
                view_zoom.set(clamp_zoom(
                    state.domain_min,
                    state.domain_max,
                    next_min,
                    next_max,
                ));
            } else {
                let mut best = find_nearest_point(
                    &state.points,
                    x_min,
                    domain_span,
                    state.y_min,
                    y_span,
                    width,
                    height,
                    padding,
                    cursor_x,
                    cursor_y,
                );
                if let Some(candidate) = find_nearest_point(
                    &state.dosage_points,
                    x_min,
                    domain_span,
                    state.y_min,
                    y_span,
                    width,
                    height,
                    padding,
                    cursor_x,
                    cursor_y,
                ) {
                    let replace = best
                        .as_ref()
                        .map(|(_, dist)| *dist)
                        .unwrap_or(f64::INFINITY)
                        > candidate.1;
                    if replace {
                        best = Some(candidate);
                    }
                }
                if let Some((candidate, _)) = best.take() {
                    view_tooltip.set(Some(candidate));
                } else {
                    view_tooltip.set(None);
                }
            }
        }
    };

    let on_view_mouse_leave = {
        let view_drag = view_drag.clone();
        let view_tooltip = view_tooltip;
        move |_| {
            view_drag.replace(None);
            view_tooltip.set(None);
        }
    };

    let on_view_mouse_down = {
        let view_drag = view_drag.clone();
        let view_zoom = view_zoom;
        let view_chart_state = view_chart_state.clone();
        move |ev: leptos::ev::MouseEvent| {
            let Some(canvas) = window()
                .document()
                .and_then(|doc| doc.get_element_by_id(VIEW_CANVAS_ID))
                .and_then(|el| el.dyn_into::<HtmlCanvasElement>().ok())
            else {
                return;
            };
            let rect = canvas.get_bounding_client_rect();
            let cursor_x = ev.client_x() as f64 - rect.left();
            let state = view_chart_state.get();
            let zoom = view_zoom.get();
            let x_min = zoom.x_min.unwrap_or(state.domain_min);
            let x_max = zoom.x_max.unwrap_or(state.domain_max);
            view_drag.replace(Some(DragState {
                start_x: cursor_x,
                start_min: x_min,
                start_max: x_max,
            }));
        }
    };

    let on_view_mouse_up = {
        let view_drag = view_drag.clone();
        move |_| {
            view_drag.replace(None);
        }
    };

    let on_view_wheel = {
        let view_zoom = view_zoom;
        let view_chart_state = view_chart_state.clone();
        move |ev: leptos::ev::WheelEvent| {
            ev.prevent_default();
            let Some(canvas) = window()
                .document()
                .and_then(|doc| doc.get_element_by_id(VIEW_CANVAS_ID))
                .and_then(|el| el.dyn_into::<HtmlCanvasElement>().ok())
            else {
                return;
            };
            let rect = canvas.get_bounding_client_rect();
            let cursor_x = ev.client_x() as f64 - rect.left();
            let state = view_chart_state.get();
            let zoom = view_zoom.get();
            let x_min = zoom.x_min.unwrap_or(state.domain_min);
            let x_max = zoom.x_max.unwrap_or(state.domain_max);
            let padding = chart_padding();
            let (width, _, domain_span, _) = compute_chart_bounds(
                rect.width(),
                rect.height(),
                padding,
                x_min,
                x_max,
                state.y_min,
                state.y_max,
            );
            let cursor_ratio = ((cursor_x - padding.0) / width).clamp(0.0, 1.0);
            let zoom_factor = if ev.delta_y() < 0.0 { 0.85 } else { 1.15 };
            let new_span = (domain_span * zoom_factor).max(1.0);
            let center = x_min + domain_span * cursor_ratio;
            let new_min = center - new_span * cursor_ratio;
            let new_max = new_min + new_span;
            view_zoom.set(clamp_zoom(
                state.domain_min,
                state.domain_max,
                new_min,
                new_max,
            ));
        }
    };

    create_effect({
        let view_chart_state = view_chart_state.clone();
        let view_zoom = view_zoom;
        move |_| {
            let state = view_chart_state.get();
            if !state.has_data {
                return;
            }
            draw_view_chart(VIEW_CANVAS_ID, &state, view_zoom.get());
        }
    });

    let view_resize_listener: Rc<RefCell<Option<EventListener>>> = Rc::new(RefCell::new(None));
    create_effect({
        let view_chart_state = view_chart_state.clone();
        let view_zoom = view_zoom;
        let view_resize_listener = view_resize_listener.clone();
        move |_| {
            view_chart_state.get();
            let window = window();
            let listener = EventListener::new(&window, "resize", move |_| {
                let state = view_chart_state.get();
                if state.has_data {
                    draw_view_chart(VIEW_CANVAS_ID, &state, view_zoom.get());
                }
            });
            view_resize_listener.replace(Some(listener));
        }
    });

    let reset_view_zoom = {
        let view_zoom = view_zoom;
        move |_| view_zoom.set(ViewZoom::default())
    };

    let x_axis_days_disabled = move || view_chart_state.get().first_dose.is_none();
    let view_tooltip_value = move || view_tooltip.get();

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
            <div class="view-layout">
                <div class="view-header">
                    <div>
                        <h2>"HRT Tracking Data"</h2>
                        <p class="muted">
                            "This chart shows your hormone levels from blood tests along with your dosage history over time."
                        </p>
                    </div>
                    <div class="header-actions">
                        <A href="/create/measurement">"Add Measurement"</A>
                    </div>
                </div>

                <div class="card">
                    <h3>"Current Regimen"</h3>
                    <div class="view-summary">
                        <Show when=move || view_chart_state.get().first_dose.is_some()>
                            <p>
                                <strong>"Days since first dose: "</strong>
                                {move || view_chart_state.get().first_dose.map(|first| {
                                    let now = js_sys::Date::now() as i64;
                                    let diff = (now - first) / (24 * 60 * 60 * 1000);
                                    diff.max(0)
                                }).unwrap_or(0)}
                            </p>
                        </Show>
                        <Show when=move || store.data.get().injectableEstradiol.is_some()>
                            <p>
                                <strong>"Injectable Estradiol: "</strong>
                                {move || store
                                    .data
                                    .get()
                                    .injectableEstradiol
                                    .as_ref()
                                    .map(|cfg| format!("{:?}, {:.2} {:?} every {:.1} days", cfg.kind, cfg.dose, cfg.unit, cfg.frequency))
                                    .unwrap_or_default()}
                            </p>
                        </Show>
                        <Show when=move || store.data.get().oralEstradiol.is_some()>
                            <p>
                                <strong>"Oral Estradiol: "</strong>
                                {move || store
                                    .data
                                    .get()
                                    .oralEstradiol
                                    .as_ref()
                                    .map(|cfg| format!("{:?}, {:.2} {:?} every {:.1} days", cfg.kind, cfg.dose, cfg.unit, cfg.frequency))
                                    .unwrap_or_default()}
                            </p>
                        </Show>
                        <Show when=move || store.data.get().antiandrogen.is_some()>
                            <p>
                                <strong>"Antiandrogen: "</strong>
                                {move || store
                                    .data
                                    .get()
                                    .antiandrogen
                                    .as_ref()
                                    .map(|cfg| format!("{:?}, {:.2} {:?} every {:.1} days", cfg.kind, cfg.dose, cfg.unit, cfg.frequency))
                                    .unwrap_or_default()}
                            </p>
                        </Show>
                        <Show when=move || store.data.get().progesterone.is_some()>
                            <p>
                                <strong>"Progesterone: "</strong>
                                {move || store
                                    .data
                                    .get()
                                    .progesterone
                                    .as_ref()
                                    .map(|cfg| format!("{:?} ({:?}), {:.2} {:?} every {:.1} days", cfg.kind, cfg.route, cfg.dose, cfg.unit, cfg.frequency))
                                    .unwrap_or_default()}
                            </p>
                        </Show>
                        <Show when=move || store.data.get().injectableEstradiol.is_none()
                            && store.data.get().oralEstradiol.is_none()
                            && store.data.get().antiandrogen.is_none()
                            && store.data.get().progesterone.is_none()>
                            <p class="muted">"No regimen set up. You can set one on the dosage page."</p>
                        </Show>
                    </div>
                </div>

                <div class="view-chart-controls">
                    <div class="chart-toolbar">
                        <div class="chart-toolbar-group">
                            <span class="muted">"X-Axis:"</span>
                            <button
                                class:active=move || x_axis_mode.get() == "date"
                                on:click=move |_| x_axis_mode.set("date".to_string())
                            >
                                "Date"
                            </button>
                            <button
                                class:active=move || x_axis_mode.get() == "days"
                                on:click=move |_| x_axis_mode.set("days".to_string())
                                prop:disabled=move || x_axis_days_disabled()
                            >
                                "Days since first dose"
                            </button>
                        </div>
                        <div class="chart-toolbar-group">
                            <span class="muted">"Time Range:"</span>
                            <button class:active=move || time_range_days.get() == 30 on:click=move |_| time_range_days.set(30)>
                                "30 days"
                            </button>
                            <button class:active=move || time_range_days.get() == 90 on:click=move |_| time_range_days.set(90)>
                                "90 days"
                            </button>
                            <button class:active=move || time_range_days.get() == 180 on:click=move |_| time_range_days.set(180)>
                                "180 days"
                            </button>
                            <button class:active=move || time_range_days.get() == 365 on:click=move |_| time_range_days.set(365)>
                                "1 year"
                            </button>
                            <button class:active=move || time_range_days.get() == 9999 on:click=move |_| time_range_days.set(9999)>
                                "All"
                            </button>
                        </div>
                        <div class="chart-toolbar-group">
                            <button on:click=reset_view_zoom disabled=move || view_zoom.get().x_min.is_none()>
                                "Reset zoom"
                            </button>
                        </div>
                    </div>

                    <div class="chart-toolbar view-levels-group">
                        <span class="muted">"Show Levels:"</span>
                        <button class:active=move || show_e2.get() on:click=move |_| show_e2.set(!show_e2.get())>
                            "E2"
                        </button>
                        <button class:active=move || show_t.get() on:click=move |_| show_t.set(!show_t.get())>
                            "T"
                        </button>
                        <button class:active=move || show_prog.get() on:click=move |_| show_prog.set(!show_prog.get())>
                            "Prog"
                        </button>
                        <button class:active=move || show_fsh.get() on:click=move |_| show_fsh.set(!show_fsh.get())>
                            "FSH"
                        </button>
                        <button class:active=move || show_lh.get() on:click=move |_| show_lh.set(!show_lh.get())>
                            "LH"
                        </button>
                        <button
                            class:active=move || show_prolactin.get()
                            on:click=move |_| show_prolactin.set(!show_prolactin.get())
                        >
                            "Prolactin"
                        </button>
                        <button class:active=move || show_shbg.get() on:click=move |_| show_shbg.set(!show_shbg.get())>
                            "SHBG"
                        </button>
                        <button class:active=move || show_fai.get() on:click=move |_| show_fai.set(!show_fai.get())>
                            "FAI"
                        </button>
                    </div>

                    <div class="chart-toolbar view-dosage-group">
                        <span class="muted">"Show Dosages:"</span>
                        <button class:active=move || show_medications.get() on:click=move |_| show_medications.set(!show_medications.get())>
                            {move || if show_medications.get() { "Medication Dosages (on)" } else { "Medication Dosages" }}
                        </button>
                    </div>
                </div>

                <div class="chart-card chart-interactive">
                    <Show when=move || view_chart_state.get().has_data fallback=move || view! {
                        <div class="empty-state">"No data available for the selected time range."</div>
                    }>
                        <canvas
                            id=VIEW_CANVAS_ID
                            width="900"
                            height="420"
                            on:mousemove=on_view_mouse_move.clone()
                            on:mouseleave=on_view_mouse_leave.clone()
                            on:mousedown=on_view_mouse_down.clone()
                            on:mouseup=on_view_mouse_up.clone()
                            on:wheel=on_view_wheel.clone()
                        ></canvas>
                        <Show when=move || view_tooltip_value().is_some()>
                            <div
                                class="chart-tooltip"
                                style=move || {
                                    view_tooltip_value()
                                        .map(|tip| format!("left: {:.0}px; top: {:.0}px;", tip.x + 12.0, tip.y + 12.0))
                                        .unwrap_or_default()
                                }
                            >
                                {move || view_tooltip_value().map(|tip| tip.text).unwrap_or_default()}
                            </div>
                        </Show>
                    </Show>
                    <div class="chart-note muted">
                        <p>"* Dosage values are scaled for visibility on the chart."</p>
                        <p>"* Hover over data points for details."</p>
                        <Show when=move || !store.data.get().bloodTests.is_empty()>
                            <p>"* Hormone measurements are normalized to display units for charting; hover shows recorded units."</p>
                        </Show>
                    </div>
                </div>
            </div>

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
            <div class="chart-card">
                <div class="empty-state">"Conversion chart not available in Rust UI yet."</div>
            </div>
        }
        .into_view(),
    )
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

    let render_vial = {
        let vial_store = vial_store.clone();
        let sub_label = sub_label.clone();
        let sub_notes = sub_notes.clone();
        Rc::new(move |entry: hrt_shared::types::Vial| {
            let created = Local
                .timestamp_millis_opt(entry.createdAt)
                .single()
                .map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| "".to_string());
            let use_by = entry
                .useBy
                .and_then(|value| {
                    Local
                        .timestamp_millis_opt(value)
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
                            on:click={
                                let entry_id = entry_id.clone();
                                let store_toggle = store_toggle.clone();
                                move |_| {
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

    let rendered_vial = {
        let render_vial = render_vial.clone();
        let vial = vial.clone();
        create_memo(move |_| vial().map(|entry| (render_vial)(entry)))
    };

    page_layout(
        "Vial Detail",
        view! {
            <Show
                when=move || rendered_vial.get().is_some()
                fallback=move || view! { <div class="empty-state">"Vial not found."</div> }
            >
                {move || rendered_vial.get().unwrap_or_else(|| view! {}.into_view())}
            </Show>
        }
        .into_view(),
    )
}

#[component]
fn EstrannaisePage() -> impl IntoView {
    let store = use_store();
    let x_axis_mode = create_rw_signal("date".to_string());
    let forecast_enabled = create_rw_signal(true);
    let forecast_weeks = create_rw_signal(8_i64);
    let forecast_dose_override = create_rw_signal(String::new());
    let forecast_frequency_override = create_rw_signal(String::new());
    let estrannaise_zoom = create_rw_signal(ViewZoom::default());
    let estrannaise_tooltip = create_rw_signal(None::<ChartTooltip>);

    let estrannaise_series = create_memo({
        let settings = store.settings;
        move |_| {
            let data_value = store.data.get();
            let settings_value = settings.get();
            let dose_override = forecast_dose_override.get().trim().parse::<f64>().ok();
            let freq_override = forecast_frequency_override.get().trim().parse::<f64>().ok();
            compute_estrannaise_series(
                &data_value,
                &settings_value,
                &x_axis_mode.get(),
                forecast_enabled.get(),
                forecast_weeks.get(),
                dose_override,
                freq_override,
            )
        }
    });

    const ESTRANNAISE_CANVAS_ID: &str = "estrannaise-chart-canvas";
    let estrannaise_drag = Rc::new(RefCell::new(None::<DragState>));

    let on_mouse_move = {
        let estrannaise_series = estrannaise_series.clone();
        let estrannaise_zoom = estrannaise_zoom;
        let estrannaise_tooltip = estrannaise_tooltip;
        let estrannaise_drag = estrannaise_drag.clone();
        move |ev: leptos::ev::MouseEvent| {
            let Some(canvas) = window()
                .document()
                .and_then(|doc| doc.get_element_by_id(ESTRANNAISE_CANVAS_ID))
                .and_then(|el| el.dyn_into::<HtmlCanvasElement>().ok())
            else {
                return;
            };
            let rect = canvas.get_bounding_client_rect();
            let cursor_x = ev.client_x() as f64 - rect.left();
            let cursor_y = ev.client_y() as f64 - rect.top();
            let series = estrannaise_series.get();
            let zoom = estrannaise_zoom.get();
            let x_min = zoom.x_min.unwrap_or(series.domain_min);
            let x_max = zoom.x_max.unwrap_or(series.domain_max);
            let padding = chart_padding();
            let (width, height, domain_span, y_span) = compute_chart_bounds(
                rect.width(),
                rect.height(),
                padding,
                x_min,
                x_max,
                series.y_min,
                series.y_max,
            );
            let mut best: Option<(ChartTooltip, f64)> = None;
            for set in [&series.blended, &series.stepped, &series.blood] {
                if let Some(candidate) = find_nearest_estrannaise_point(
                    set,
                    x_min,
                    domain_span,
                    series.y_min,
                    y_span,
                    width,
                    height,
                    padding,
                    cursor_x,
                    cursor_y,
                ) {
                    if best
                        .as_ref()
                        .map(|(_, dist)| *dist)
                        .unwrap_or(f64::INFINITY)
                        > candidate.1
                    {
                        best = Some(candidate);
                    }
                }
            }
            if let Some(drag) = estrannaise_drag.borrow().as_ref() {
                estrannaise_tooltip.set(None);
                let delta_px = cursor_x - drag.start_x;
                let span = x_max - x_min;
                let delta_domain = -(delta_px / width) * span;
                let next_min = drag.start_min + delta_domain;
                let next_max = drag.start_max + delta_domain;
                estrannaise_zoom.set(clamp_zoom(
                    series.domain_min,
                    series.domain_max,
                    next_min,
                    next_max,
                ));
            } else {
                estrannaise_tooltip.set(best.map(|(tip, _)| tip));
            }
        }
    };

    let on_mouse_leave = {
        let estrannaise_drag = estrannaise_drag.clone();
        let estrannaise_tooltip = estrannaise_tooltip;
        move |_| {
            estrannaise_drag.replace(None);
            estrannaise_tooltip.set(None);
        }
    };

    let on_mouse_down = {
        let estrannaise_drag = estrannaise_drag.clone();
        let estrannaise_zoom = estrannaise_zoom;
        let estrannaise_series = estrannaise_series.clone();
        move |ev: leptos::ev::MouseEvent| {
            let Some(canvas) = window()
                .document()
                .and_then(|doc| doc.get_element_by_id(ESTRANNAISE_CANVAS_ID))
                .and_then(|el| el.dyn_into::<HtmlCanvasElement>().ok())
            else {
                return;
            };
            let rect = canvas.get_bounding_client_rect();
            let cursor_x = ev.client_x() as f64 - rect.left();
            let series = estrannaise_series.get();
            let zoom = estrannaise_zoom.get();
            let x_min = zoom.x_min.unwrap_or(series.domain_min);
            let x_max = zoom.x_max.unwrap_or(series.domain_max);
            estrannaise_drag.replace(Some(DragState {
                start_x: cursor_x,
                start_min: x_min,
                start_max: x_max,
            }));
        }
    };

    let on_mouse_up = {
        let estrannaise_drag = estrannaise_drag.clone();
        move |_| {
            estrannaise_drag.replace(None);
        }
    };

    let on_wheel = {
        let estrannaise_zoom = estrannaise_zoom;
        let estrannaise_series = estrannaise_series.clone();
        move |ev: leptos::ev::WheelEvent| {
            ev.prevent_default();
            let Some(canvas) = window()
                .document()
                .and_then(|doc| doc.get_element_by_id(ESTRANNAISE_CANVAS_ID))
                .and_then(|el| el.dyn_into::<HtmlCanvasElement>().ok())
            else {
                return;
            };
            let rect = canvas.get_bounding_client_rect();
            let cursor_x = ev.client_x() as f64 - rect.left();
            let series = estrannaise_series.get();
            let zoom = estrannaise_zoom.get();
            let x_min = zoom.x_min.unwrap_or(series.domain_min);
            let x_max = zoom.x_max.unwrap_or(series.domain_max);
            let padding = chart_padding();
            let (width, _, domain_span, _) = compute_chart_bounds(
                rect.width(),
                rect.height(),
                padding,
                x_min,
                x_max,
                series.y_min,
                series.y_max,
            );
            let cursor_ratio = ((cursor_x - padding.0) / width).clamp(0.0, 1.0);
            let zoom_factor = if ev.delta_y() < 0.0 { 0.85 } else { 1.15 };
            let new_span = (domain_span * zoom_factor).max(1.0);
            let center = x_min + domain_span * cursor_ratio;
            let new_min = center - new_span * cursor_ratio;
            let new_max = new_min + new_span;
            estrannaise_zoom.set(clamp_zoom(
                series.domain_min,
                series.domain_max,
                new_min,
                new_max,
            ));
        }
    };

    create_effect({
        let estrannaise_series = estrannaise_series.clone();
        let estrannaise_zoom = estrannaise_zoom;
        move |_| {
            let series = estrannaise_series.get();
            if series.blended.is_empty() && series.stepped.is_empty() && series.blood.is_empty() {
                return;
            }
            draw_estrannaise_chart(ESTRANNAISE_CANVAS_ID, &series, estrannaise_zoom.get());
        }
    });

    let resize_listener: Rc<RefCell<Option<EventListener>>> = Rc::new(RefCell::new(None));
    create_effect({
        let estrannaise_series = estrannaise_series.clone();
        let estrannaise_zoom = estrannaise_zoom;
        let resize_listener = resize_listener.clone();
        move |_| {
            estrannaise_series.get();
            let window = window();
            let listener = EventListener::new(&window, "resize", move |_| {
                let series = estrannaise_series.get();
                if series.blended.is_empty() && series.stepped.is_empty() && series.blood.is_empty()
                {
                    return;
                }
                draw_estrannaise_chart(ESTRANNAISE_CANVAS_ID, &series, estrannaise_zoom.get());
            });
            resize_listener.replace(Some(listener));
        }
    });

    let reset_zoom = {
        let estrannaise_zoom = estrannaise_zoom;
        move |_| estrannaise_zoom.set(ViewZoom::default())
    };

    let tooltip_value = move || estrannaise_tooltip.get();

    page_layout(
        "Estrannaise",
        view! {
            <div class="view-layout">
                <div class="view-header">
                    <div>
                        <h2>"Estrannaise"</h2>
                        <p class="muted">
                            "Estrannaise-style modeling with blended vs. step fudge factors, plus forecasted schedule windows."
                        </p>
                    </div>
                </div>

                <div class="chart-toolbar">
                    <div class="chart-toolbar-group">
                        <span class="muted">"X-Axis:"</span>
                        <button
                            class:active=move || x_axis_mode.get() == "date"
                            on:click=move |_| x_axis_mode.set("date".to_string())
                        >
                            "Date"
                        </button>
                        <button
                            class:active=move || x_axis_mode.get() == "days"
                            on:click=move |_| x_axis_mode.set("days".to_string())
                            prop:disabled=move || estrannaise_series.get().first_dose.is_none()
                        >
                            "Days since first dose"
                        </button>
                    </div>
                    <div class="chart-toolbar-group">
                        <label class="muted">"Forecast"</label>
                        <input
                            type="checkbox"
                            on:change=move |ev| forecast_enabled.set(event_target_checked(&ev))
                            prop:checked=move || forecast_enabled.get()
                        />
                    </div>
                    <div class="chart-toolbar-group">
                        <label class="muted">"Weeks"</label>
                        <select on:change=move |ev| forecast_weeks.set(event_target_value(&ev).parse::<i64>().unwrap_or(8))>
                            <option value="4" selected=move || forecast_weeks.get() == 4>"4"</option>
                            <option value="6" selected=move || forecast_weeks.get() == 6>"6"</option>
                            <option value="8" selected=move || forecast_weeks.get() == 8>"8"</option>
                        </select>
                    </div>
                    <div class="chart-toolbar-group">
                        <label class="muted">"Dose"</label>
                        <input
                            type="number"
                            step="0.1"
                            min="0"
                            class="chart-input"
                            placeholder="auto"
                            on:input=move |ev| forecast_dose_override.set(event_target_value(&ev))
                            prop:value=move || forecast_dose_override.get()
                        />
                    </div>
                    <div class="chart-toolbar-group">
                        <label class="muted">"Every (days)"</label>
                        <input
                            type="number"
                            step="1"
                            min="1"
                            class="chart-input"
                            placeholder="auto"
                            on:input=move |ev| forecast_frequency_override.set(event_target_value(&ev))
                            prop:value=move || forecast_frequency_override.get()
                        />
                    </div>
                    <div class="chart-toolbar-group">
                        <button on:click=reset_zoom disabled=move || estrannaise_zoom.get().x_min.is_none()>
                            "Reset zoom"
                        </button>
                    </div>
                </div>

                <div class="chart-card chart-interactive">
                    <Show
                        when=move || !estrannaise_series.get().blended.is_empty()
                            || !estrannaise_series.get().stepped.is_empty()
                            || !estrannaise_series.get().blood.is_empty()
                        fallback=move || view! {
                            <div class="empty-state">
                                <p>"No Estrannaise data available."</p>
                                <p class="muted">"Add injectable estradiol history and blood tests to see model lines."</p>
                            </div>
                        }
                    >
                        <canvas
                            id=ESTRANNAISE_CANVAS_ID
                            width="900"
                            height="420"
                            on:mousemove=on_mouse_move.clone()
                            on:mouseleave=on_mouse_leave.clone()
                            on:mousedown=on_mouse_down.clone()
                            on:mouseup=on_mouse_up.clone()
                            on:wheel=on_wheel.clone()
                        ></canvas>
                        <Show when=move || tooltip_value().is_some()>
                            <div
                                class="chart-tooltip"
                                style=move || {
                                    tooltip_value()
                                        .map(|tip| format!("left: {:.0}px; top: {:.0}px;", tip.x + 12.0, tip.y + 12.0))
                                        .unwrap_or_default()
                                }
                            >
                                {move || tooltip_value().map(|tip| tip.text).unwrap_or_default()}
                            </div>
                        </Show>
                    </Show>
                    <div class="chart-note muted">
                        <p>"* Blue line blends fudge factor between blood tests."</p>
                        <p>"* Pink dashed line steps to each test's fudge factor."</p>
                        <p>"* Orange points show measured E2 in display units."</p>
                        <p>"* Shaded region is forecasted schedule window."</p>
                    </div>
                </div>
            </div>
        }
        .into_view(),
    )
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

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct ViewZoom {
    x_min: Option<f64>,
    x_max: Option<f64>,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct ChartTooltip {
    text: String,
    x: f64,
    y: f64,
}

#[derive(Clone, Debug, PartialEq)]
struct ViewChartPoint {
    x: f64,
    y: f64,
    label: String,
    color: RGBColor,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct ViewChartState {
    domain_min: f64,
    domain_max: f64,
    y_min: f64,
    y_max: f64,
    x_label: String,
    y_label: String,
    points: Vec<ViewChartPoint>,
    dosage_points: Vec<ViewChartPoint>,
    first_dose: Option<i64>,
    use_days: bool,
    has_data: bool,
}

#[derive(Clone, Debug, PartialEq)]
struct EstrannaisePoint {
    x: f64,
    y: f64,
    label: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct EstrannaiseSeries {
    blended: Vec<EstrannaisePoint>,
    stepped: Vec<EstrannaisePoint>,
    blood: Vec<EstrannaisePoint>,
    forecast: Option<(f64, f64)>,
    domain_min: f64,
    domain_max: f64,
    y_min: f64,
    y_max: f64,
    x_label: String,
    y_label: String,
    first_dose: Option<i64>,
    use_days: bool,
}

#[derive(Clone, Copy, Debug)]
struct DragState {
    start_x: f64,
    start_min: f64,
    start_max: f64,
}

const CHART_MARGIN: f64 = 18.0;
const CHART_X_LABEL: f64 = 42.0;
const CHART_Y_LABEL: f64 = 52.0;

fn chart_padding() -> (f64, f64, f64, f64) {
    (
        CHART_MARGIN + CHART_Y_LABEL,
        CHART_MARGIN,
        CHART_MARGIN,
        CHART_MARGIN + CHART_X_LABEL,
    )
}

fn clamp_zoom(domain_min: f64, domain_max: f64, new_min: f64, new_max: f64) -> ViewZoom {
    let full_span = domain_max - domain_min;
    let span = (new_max - new_min).max(1.0);
    if span >= full_span * 0.98 {
        return ViewZoom::default();
    }
    let mut min_val = new_min;
    let mut max_val = new_min + span;
    if min_val < domain_min {
        min_val = domain_min;
        max_val = domain_min + span;
    }
    if max_val > domain_max {
        max_val = domain_max;
        min_val = domain_max - span;
    }
    ViewZoom {
        x_min: Some(min_val),
        x_max: Some(max_val),
    }
}

fn hormone_unit_label(unit: &HormoneUnits) -> &'static str {
    match unit {
        HormoneUnits::E2PmolL => "pmol/L",
        HormoneUnits::E2PgMl => "pg/mL",
        HormoneUnits::TNgDl => "ng/dL",
        HormoneUnits::TNmolL => "nmol/L",
        HormoneUnits::Mg => "mg",
        HormoneUnits::NgMl => "ng/mL",
        HormoneUnits::MIuMl => "mIU/mL",
        HormoneUnits::MIuL => "mIU/L",
        HormoneUnits::UL => "U/L",
    }
}

fn convert_estradiol_to_display(
    value: f64,
    unit: &HormoneUnits,
    display_unit: &HormoneUnits,
) -> f64 {
    if display_unit == &HormoneUnits::E2PmolL {
        if unit == &HormoneUnits::E2PmolL {
            value
        } else {
            value * 3.6713
        }
    } else if unit == &HormoneUnits::E2PmolL {
        value / 3.6713
    } else {
        value
    }
}

fn estradiol_conversion_factor(display_unit: &HormoneUnits) -> f64 {
    if display_unit == &HormoneUnits::E2PmolL {
        3.6713
    } else {
        1.0
    }
}

fn convert_testosterone_to_ng_dl(value: f64, unit: &HormoneUnits) -> f64 {
    if unit == &HormoneUnits::TNmolL {
        value * 28.818
    } else {
        value
    }
}

fn convert_fsh_to_miu_ml(value: f64, unit: &HormoneUnits) -> f64 {
    match unit {
        HormoneUnits::MIuL => value / 1000.0,
        HormoneUnits::UL => value,
        _ => value,
    }
}

fn convert_lh_to_miu_ml(value: f64, unit: &HormoneUnits) -> f64 {
    match unit {
        HormoneUnits::MIuL => value / 1000.0,
        HormoneUnits::UL => value,
        _ => value,
    }
}

fn convert_progesterone_to_ng_ml(value: f64, unit: &HormoneUnits) -> f64 {
    if unit == &HormoneUnits::TNmolL {
        value * 0.31
    } else {
        value
    }
}

fn fmt_date_label(date_ms: i64, axis_mode: &str, first_dose: Option<i64>) -> String {
    const DAY_MS: i64 = 24 * 60 * 60 * 1000;
    if axis_mode == "days" {
        if let Some(first) = first_dose {
            let days = (date_ms - first) as f64 / DAY_MS as f64;
            return format!("Day {:.1}", days);
        }
    }
    Local
        .timestamp_millis_opt(date_ms)
        .single()
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| date_ms.to_string())
}

fn map_estrannaise_model(kind: &InjectableEstradiols) -> Option<EstrannaiseModel> {
    match kind {
        InjectableEstradiols::Benzoate => Some(EstrannaiseModel::EbIm),
        InjectableEstradiols::Valerate => Some(EstrannaiseModel::EvIm),
        InjectableEstradiols::Enanthate => Some(EstrannaiseModel::EEnIm),
        InjectableEstradiols::Cypionate => Some(EstrannaiseModel::EcIm),
        InjectableEstradiols::Undecylate => Some(EstrannaiseModel::EUnIm),
        InjectableEstradiols::PolyestradiolPhosphate => None,
    }
}

fn extract_fudge_series(tests: &[BloodTest]) -> Vec<(i64, f64)> {
    let mut series: Vec<(i64, f64)> = tests
        .iter()
        .filter_map(|t| t.fudgeFactor.map(|value| (t.date, value)))
        .collect();
    series.sort_by_key(|(date, _)| *date);
    if series.is_empty() {
        series.push((js_sys::Date::now() as i64, 1.0));
    }
    series
}

fn blend_fudge(series: &[(i64, f64)], target: i64) -> f64 {
    if series.is_empty() {
        return 1.0;
    }
    if target <= series[0].0 {
        return series[0].1;
    }
    let last = series[series.len() - 1];
    if target >= last.0 {
        return last.1;
    }
    for window in series.windows(2) {
        let (prev_date, prev_val) = window[0];
        let (next_date, next_val) = window[1];
        if target <= next_date {
            let span = (next_date - prev_date) as f64;
            if span <= 0.0 {
                return prev_val;
            }
            let ratio = (target - prev_date) as f64 / span;
            return prev_val + (next_val - prev_val) * ratio;
        }
    }
    last.1
}

fn step_fudge(series: &[(i64, f64)], target: i64) -> f64 {
    if series.is_empty() {
        return 1.0;
    }
    if target <= series[0].0 {
        return series[0].1;
    }
    for window in series.windows(2) {
        let (_prev_date, prev_val) = window[0];
        let (next_date, _) = window[1];
        if target < next_date {
            return prev_val;
        }
    }
    series[series.len() - 1].1
}

fn compute_view_chart_state(
    data: &hrt_shared::types::HrtData,
    settings: &Settings,
    axis_mode: &str,
    time_range_days: i64,
    show_medications: bool,
    show_e2: bool,
    show_t: bool,
    show_prog: bool,
    show_fsh: bool,
    show_lh: bool,
    show_prolactin: bool,
    show_shbg: bool,
    show_fai: bool,
) -> ViewChartState {
    let now = js_sys::Date::now() as i64;
    let start_time = now - time_range_days * 24 * 60 * 60 * 1000;
    let display_unit = settings
        .displayEstradiolUnit
        .clone()
        .unwrap_or(HormoneUnits::E2PmolL);
    let first_dose = data
        .dosageHistory
        .iter()
        .map(|d| match d {
            DosageHistoryEntry::InjectableEstradiol { date, .. }
            | DosageHistoryEntry::OralEstradiol { date, .. }
            | DosageHistoryEntry::Antiandrogen { date, .. }
            | DosageHistoryEntry::Progesterone { date, .. } => *date,
        })
        .min();

    let use_days = axis_mode == "days" && first_dose.is_some();
    let x_label = if use_days {
        "Days since first dose".to_string()
    } else {
        "Date".to_string()
    };

    let mut points = Vec::new();
    let mut all_values = Vec::new();
    let mut has_data = false;

    for test in data.bloodTests.iter().filter(|t| t.date >= start_time) {
        let x = if use_days {
            (test.date - first_dose.unwrap_or(test.date)) as f64 / (24.0 * 60.0 * 60.0 * 1000.0)
        } else {
            test.date as f64
        };
        let date_label = fmt_date_label(test.date, axis_mode, first_dose);
        let date_short = if use_days {
            date_label.clone()
        } else {
            Local
                .timestamp_millis_opt(test.date)
                .single()
                .map(|d| d.format("%b %d").to_string())
                .unwrap_or_else(|| date_label.clone())
        };
        if show_e2 {
            if let Some(value) = test.estradiolLevel {
                let raw_unit = test.estradiolUnit.clone().unwrap_or(HormoneUnits::E2PgMl);
                let plot_val = convert_estradiol_to_display(value, &raw_unit, &display_unit);
                let unit_label = hormone_unit_label(&display_unit);
                let tooltip = if raw_unit != display_unit {
                    format!(
                        "Estradiol: {:.2} {} -> {:.2} {} ({})",
                        value,
                        hormone_unit_label(&raw_unit),
                        plot_val,
                        unit_label,
                        date_short
                    )
                } else {
                    format!("Estradiol: {:.2} {} ({})", plot_val, unit_label, date_short)
                };
                points.push(ViewChartPoint {
                    x,
                    y: plot_val,
                    label: tooltip,
                    color: RGBColor(70, 130, 180),
                });
                all_values.push(plot_val);
                has_data = true;
            }
        }
        if show_t {
            if let Some(value) = test.testLevel {
                let raw_unit = test.testUnit.clone().unwrap_or(HormoneUnits::TNgDl);
                let plot_val = convert_testosterone_to_ng_dl(value, &raw_unit);
                let unit_label = "ng/dL";
                let tooltip = if raw_unit != HormoneUnits::TNgDl {
                    format!(
                        "Testosterone: {:.2} {} -> {:.2} {} ({})",
                        value,
                        hormone_unit_label(&raw_unit),
                        plot_val,
                        unit_label,
                        date_short
                    )
                } else {
                    format!(
                        "Testosterone: {:.2} {} ({})",
                        plot_val, unit_label, date_short
                    )
                };
                points.push(ViewChartPoint {
                    x,
                    y: plot_val,
                    label: tooltip,
                    color: RGBColor(220, 20, 60),
                });
                all_values.push(plot_val);
                has_data = true;
            }
        }
        if show_prog {
            if let Some(value) = test.progesteroneLevel {
                let raw_unit = test.progesteroneUnit.clone().unwrap_or(HormoneUnits::NgMl);
                let plot_val = convert_progesterone_to_ng_ml(value, &raw_unit);
                let tooltip = format!("Progesterone: {:.2} ng/mL ({})", plot_val, date_short);
                points.push(ViewChartPoint {
                    x,
                    y: plot_val,
                    label: tooltip,
                    color: RGBColor(148, 0, 211),
                });
                all_values.push(plot_val);
                has_data = true;
            }
        }
        if show_fsh {
            if let Some(value) = test.fshLevel {
                let raw_unit = test.fshUnit.clone().unwrap_or(HormoneUnits::MIuMl);
                let plot_val = convert_fsh_to_miu_ml(value, &raw_unit);
                let tooltip = format!("FSH: {:.2} mIU/mL ({})", plot_val, date_short);
                points.push(ViewChartPoint {
                    x,
                    y: plot_val,
                    label: tooltip,
                    color: RGBColor(34, 139, 34),
                });
                all_values.push(plot_val);
                has_data = true;
            }
        }
        if show_lh {
            if let Some(value) = test.lhLevel {
                let raw_unit = test.lhUnit.clone().unwrap_or(HormoneUnits::MIuMl);
                let plot_val = convert_lh_to_miu_ml(value, &raw_unit);
                let tooltip = format!("LH: {:.2} mIU/mL ({})", plot_val, date_short);
                points.push(ViewChartPoint {
                    x,
                    y: plot_val,
                    label: tooltip,
                    color: RGBColor(0, 139, 139),
                });
                all_values.push(plot_val);
                has_data = true;
            }
        }
        if show_prolactin {
            if let Some(value) = test.prolactinLevel {
                let raw_unit = test.prolactinUnit.clone().unwrap_or(HormoneUnits::NgMl);
                let unit_label = hormone_unit_label(&raw_unit);
                let tooltip = format!("Prolactin: {:.2} {} ({})", value, unit_label, date_short);
                points.push(ViewChartPoint {
                    x,
                    y: value,
                    label: tooltip,
                    color: RGBColor(139, 69, 19),
                });
                all_values.push(value);
                has_data = true;
            }
        }
        if show_shbg {
            if let Some(value) = test.shbgLevel {
                let raw_unit = test.shbgUnit.clone().unwrap_or(HormoneUnits::TNmolL);
                let unit_label = hormone_unit_label(&raw_unit);
                let tooltip = format!("SHBG: {:.2} {} ({})", value, unit_label, date_short);
                points.push(ViewChartPoint {
                    x,
                    y: value,
                    label: tooltip,
                    color: RGBColor(255, 20, 147),
                });
                all_values.push(value);
                has_data = true;
            }
        }
        if show_fai {
            if let Some(value) = test.freeAndrogenIndex {
                let tooltip = format!("FAI: {:.2} ({})", value, date_short);
                points.push(ViewChartPoint {
                    x,
                    y: value,
                    label: tooltip,
                    color: RGBColor(0, 0, 0),
                });
                all_values.push(value);
                has_data = true;
            }
        }
    }

    let mut dosage_points = Vec::new();
    if show_medications {
        for dose in data.dosageHistory.iter() {
            let date = match dose {
                DosageHistoryEntry::InjectableEstradiol { date, .. }
                | DosageHistoryEntry::OralEstradiol { date, .. }
                | DosageHistoryEntry::Antiandrogen { date, .. }
                | DosageHistoryEntry::Progesterone { date, .. } => *date,
            };
            if date < start_time {
                continue;
            }
            let (label, value, color) = match dose {
                DosageHistoryEntry::InjectableEstradiol {
                    kind, dose, unit, ..
                } => (
                    format!(
                        "Injection: {:?}, {:.2} {}",
                        kind,
                        dose,
                        hormone_unit_label(unit)
                    ),
                    (*dose * 20.0).min(300.0),
                    RGBColor(0, 114, 178),
                ),
                DosageHistoryEntry::OralEstradiol {
                    kind, dose, unit, ..
                } => (
                    format!(
                        "Oral E: {:?}, {:.2} {}",
                        kind,
                        dose,
                        hormone_unit_label(unit)
                    ),
                    (*dose * 10.0).min(200.0),
                    RGBColor(46, 139, 87),
                ),
                DosageHistoryEntry::Antiandrogen {
                    kind, dose, unit, ..
                } => (
                    format!("AA: {:?}, {:.2} {}", kind, dose, hormone_unit_label(unit)),
                    (*dose * 10.0).min(200.0),
                    RGBColor(255, 140, 0),
                ),
                DosageHistoryEntry::Progesterone {
                    kind, dose, unit, ..
                } => (
                    format!(
                        "Progesterone: {:?}, {:.2} {}",
                        kind,
                        dose,
                        hormone_unit_label(unit)
                    ),
                    (*dose).min(400.0),
                    RGBColor(255, 215, 0),
                ),
            };
            let x = if use_days {
                (date - first_dose.unwrap_or(date)) as f64 / (24.0 * 60.0 * 60.0 * 1000.0)
            } else {
                date as f64
            };
            let date_label = fmt_date_label(date, axis_mode, first_dose);
            dosage_points.push(ViewChartPoint {
                x,
                y: value,
                label: format!("{} ({})", label, date_label),
                color,
            });

            all_values.push(value);
            has_data = true;
        }
    }

    if all_values.is_empty() {
        all_values.push(0.0);
        all_values.push(1.0);
    }
    let mut y_min = all_values.iter().cloned().fold(f64::INFINITY, f64::min);
    let mut y_max = all_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    if (y_min - y_max).abs() < f64::EPSILON {
        y_min = (y_min - 1.0).max(0.0);
        y_max += 1.0;
    } else {
        let pad = (y_max - y_min) * 0.08;
        y_min = (y_min - pad).max(0.0);
        y_max += pad;
    }

    let mut x_values: Vec<f64> = points.iter().map(|p| p.x).collect();
    x_values.extend(dosage_points.iter().map(|p| p.x));

    let (domain_min, domain_max) = if x_values.is_empty() {
        if use_days {
            (0.0, 30.0)
        } else {
            (start_time as f64, now as f64)
        }
    } else {
        let mut min_x = x_values.iter().cloned().fold(f64::INFINITY, f64::min);
        let mut max_x = x_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        if (min_x - max_x).abs() < f64::EPSILON {
            min_x -= 1.0;
            max_x += 1.0;
        }
        (min_x, max_x)
    };

    ViewChartState {
        domain_min,
        domain_max,
        y_min,
        y_max,
        x_label,
        y_label: "Levels".to_string(),
        points,
        dosage_points,
        first_dose,
        use_days,
        has_data,
    }
}

fn compute_estrannaise_series(
    data: &hrt_shared::types::HrtData,
    settings: &Settings,
    axis_mode: &str,
    forecast_enabled: bool,
    forecast_weeks: i64,
    forecast_dose_override: Option<f64>,
    forecast_freq_override: Option<f64>,
) -> EstrannaiseSeries {
    let display_unit = settings
        .displayEstradiolUnit
        .clone()
        .unwrap_or(HormoneUnits::E2PmolL);
    let conversion = estradiol_conversion_factor(&display_unit);
    let dose_history: Vec<_> = data
        .dosageHistory
        .iter()
        .filter_map(|d| match d {
            DosageHistoryEntry::InjectableEstradiol {
                date,
                kind,
                dose,
                unit: _,
                ..
            } => Some((*date, kind.clone(), *dose)),
            _ => None,
        })
        .collect();
    let mut dose_history = dose_history;
    dose_history.sort_by_key(|(date, _, _)| *date);

    let earliest_date = if !dose_history.is_empty() {
        Some(dose_history.first().map(|(date, _, _)| *date).unwrap())
    } else {
        data.bloodTests.iter().map(|t| t.date).min()
    };
    let Some(first_dose) = earliest_date else {
        return EstrannaiseSeries::default();
    };

    let last_dose = dose_history
        .last()
        .map(|(date, _, _)| *date)
        .unwrap_or(first_dose);
    let start_date = first_dose;
    let base_end = (last_dose + 30 * 24 * 60 * 60 * 1000).max(js_sys::Date::now() as i64);
    let forecast_weeks = forecast_weeks.clamp(4, 8);
    let forecast_end =
        base_end.max(js_sys::Date::now() as i64 + forecast_weeks * 7 * 24 * 60 * 60 * 1000);

    let schedule = data.injectableEstradiol.clone();
    let forecast_start = js_sys::Date::now() as i64;
    let forecast_start_date = schedule
        .as_ref()
        .and_then(|s| s.nextDoseDate)
        .unwrap_or(forecast_start)
        .max(forecast_start);
    let forecast_dose = forecast_dose_override.or_else(|| schedule.as_ref().map(|s| s.dose));
    let forecast_freq = forecast_freq_override.or_else(|| schedule.as_ref().map(|s| s.frequency));
    let forecast_type = schedule
        .as_ref()
        .map(|s| s.kind.clone())
        .or_else(|| dose_history.last().map(|(_, kind, _)| kind.clone()));

    let mut forecast_doses = Vec::new();
    if forecast_enabled {
        if let (Some(dose), Some(freq), Some(kind)) = (forecast_dose, forecast_freq, forecast_type)
        {
            let mut t = forecast_start_date;
            while t <= forecast_end {
                forecast_doses.push((t, kind.clone(), dose));
                t += (freq * 24.0 * 60.0 * 60.0 * 1000.0) as i64;
            }
        }
    }

    let mut all_doses = dose_history.clone();
    all_doses.extend(forecast_doses.clone());
    all_doses.sort_by_key(|(date, _, _)| *date);

    let series = extract_fudge_series(&data.bloodTests);
    let step_ms = 6 * 60 * 60 * 1000;
    let mut blended = Vec::new();
    let mut stepped = Vec::new();
    let mut y_values = Vec::new();

    let mut time_map = Vec::new();
    let mut dose_map = Vec::new();
    let mut model_map = Vec::new();
    for (date, kind, dose) in &all_doses {
        if let Some(model) = map_estrannaise_model(kind) {
            time_map.push((*date - start_date) as f64 / (24.0 * 60.0 * 60.0 * 1000.0));
            dose_map.push(*dose);
            model_map.push(model);
        }
    }

    if !model_map.is_empty() {
        let mut t = start_date;
        while t <= forecast_end {
            let day_value = (t - start_date) as f64 / (24.0 * 60.0 * 60.0 * 1000.0);
            let blended_fudge = blend_fudge(&series, t);
            let step_fudge = step_fudge(&series, t);
            let blended_val = e2_multidose_3c(
                day_value,
                &dose_map,
                &time_map,
                &model_map,
                blended_fudge * conversion,
                false,
            );
            let stepped_val = e2_multidose_3c(
                day_value,
                &dose_map,
                &time_map,
                &model_map,
                step_fudge * conversion,
                false,
            );
            let x = if axis_mode == "days" {
                day_value
            } else {
                t as f64
            };
            let label = fmt_date_label(t, axis_mode, Some(start_date));
            blended.push(EstrannaisePoint {
                x,
                y: blended_val,
                label: format!("Blended: {:.1} ({})", blended_val, label),
            });
            stepped.push(EstrannaisePoint {
                x,
                y: stepped_val,
                label: format!("Step: {:.1} ({})", stepped_val, label),
            });
            y_values.push(blended_val);
            y_values.push(stepped_val);
            t += step_ms;
        }
    }

    let blood: Vec<EstrannaisePoint> = data
        .bloodTests
        .iter()
        .filter_map(|test| {
            test.estradiolLevel
                .map(|value| (test.date, value, test.estradiolUnit.clone()))
        })
        .map(|(date, value, unit)| {
            let raw_unit = unit.unwrap_or(HormoneUnits::E2PgMl);
            let plot_val = convert_estradiol_to_display(value, &raw_unit, &display_unit);
            let x = if axis_mode == "days" {
                (date - first_dose) as f64 / (24.0 * 60.0 * 60.0 * 1000.0)
            } else {
                date as f64
            };
            let label = fmt_date_label(date, axis_mode, Some(first_dose));
            y_values.push(plot_val);
            EstrannaisePoint {
                x,
                y: plot_val,
                label: format!("Blood test: {:.1} ({})", plot_val, label),
            }
        })
        .collect();

    let mut y_min = y_values.iter().cloned().fold(f64::INFINITY, f64::min);
    let mut y_max = y_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    if (y_min - y_max).abs() < f64::EPSILON {
        y_min = y_min - 1.0;
        y_max += 1.0;
    } else {
        let pad = (y_max - y_min) * 0.08;
        y_min -= pad;
        y_max += pad;
    }

    let domain_min = if axis_mode == "days" {
        0.0
    } else {
        start_date as f64
    };
    let domain_max = if axis_mode == "days" {
        ((forecast_end - start_date) as f64 / (24.0 * 60.0 * 60.0 * 1000.0)).max(30.0)
    } else {
        forecast_end as f64
    };

    let forecast = if forecast_enabled {
        let start_x = if axis_mode == "days" {
            (forecast_start - start_date) as f64 / (24.0 * 60.0 * 60.0 * 1000.0)
        } else {
            forecast_start as f64
        };
        let end_x = if axis_mode == "days" {
            (forecast_end - start_date) as f64 / (24.0 * 60.0 * 60.0 * 1000.0)
        } else {
            forecast_end as f64
        };
        Some((start_x, end_x))
    } else {
        None
    };

    EstrannaiseSeries {
        blended,
        stepped,
        blood,
        forecast,
        domain_min,
        domain_max,
        y_min,
        y_max,
        x_label: if axis_mode == "days" {
            "Days since first dose".to_string()
        } else {
            "Date".to_string()
        },
        y_label: format!("E2 ({})", hormone_unit_label(&display_unit)),
        first_dose: Some(first_dose),
        use_days: axis_mode == "days",
    }
}

fn compute_chart_bounds(
    client_width: f64,
    client_height: f64,
    padding: (f64, f64, f64, f64),
    domain_min: f64,
    domain_max: f64,
    y_min: f64,
    y_max: f64,
) -> (f64, f64, f64, f64) {
    let (left, top, right, bottom) = padding;
    let width = (client_width - left - right).max(1.0);
    let height = (client_height - top - bottom).max(1.0);
    let domain_span = (domain_max - domain_min).abs().max(1.0);
    let y_span = (y_max - y_min).abs().max(1.0);
    (width, height, domain_span, y_span)
}

fn data_to_canvas_x(x: f64, domain_min: f64, domain_span: f64, width: f64, left: f64) -> f64 {
    left + ((x - domain_min) / domain_span) * width
}

fn data_to_canvas_y(y: f64, y_min: f64, y_span: f64, height: f64, top: f64) -> f64 {
    top + height - ((y - y_min) / y_span) * height
}

fn find_nearest_point(
    points: &[ViewChartPoint],
    domain_min: f64,
    domain_span: f64,
    y_min: f64,
    y_span: f64,
    width: f64,
    height: f64,
    padding: (f64, f64, f64, f64),
    cursor_x: f64,
    cursor_y: f64,
) -> Option<(ChartTooltip, f64)> {
    let (left, top, _, _) = padding;
    let mut best: Option<(f64, &ViewChartPoint, f64, f64)> = None;
    for point in points {
        let px = data_to_canvas_x(point.x, domain_min, domain_span, width, left);
        let py = data_to_canvas_y(point.y, y_min, y_span, height, top);
        let dx = px - cursor_x;
        let dy = py - cursor_y;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist < 18.0 {
            match best {
                Some((best_dist, _, _, _)) if dist >= best_dist => {}
                _ => best = Some((dist, point, px, py)),
            }
        }
    }
    best.map(|(dist, point, px, py)| {
        (
            ChartTooltip {
                text: point.label.clone(),
                x: px,
                y: py,
            },
            dist,
        )
    })
}

fn find_nearest_estrannaise_point(
    points: &[EstrannaisePoint],
    domain_min: f64,
    domain_span: f64,
    y_min: f64,
    y_span: f64,
    width: f64,
    height: f64,
    padding: (f64, f64, f64, f64),
    cursor_x: f64,
    cursor_y: f64,
) -> Option<(ChartTooltip, f64)> {
    let (left, top, _, _) = padding;
    let mut best: Option<(f64, &EstrannaisePoint, f64, f64)> = None;
    for point in points {
        let px = data_to_canvas_x(point.x, domain_min, domain_span, width, left);
        let py = data_to_canvas_y(point.y, y_min, y_span, height, top);
        let dx = px - cursor_x;
        let dy = py - cursor_y;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist < 18.0 {
            match best {
                Some((best_dist, _, _, _)) if dist >= best_dist => {}
                _ => best = Some((dist, point, px, py)),
            }
        }
    }
    best.map(|(dist, point, px, py)| {
        (
            ChartTooltip {
                text: point.label.clone(),
                x: px,
                y: py,
            },
            dist,
        )
    })
}

fn draw_view_chart(canvas_id: &str, state: &ViewChartState, zoom: ViewZoom) {
    let Some(canvas) = window()
        .document()
        .and_then(|doc| doc.get_element_by_id(canvas_id))
        .and_then(|el| el.dyn_into::<HtmlCanvasElement>().ok())
    else {
        return;
    };
    let rect = canvas.get_bounding_client_rect();
    let width = rect.width().max(320.0);
    let height = rect.height().max(280.0);
    let dpr = window().device_pixel_ratio();
    canvas.set_width((width * dpr) as u32);
    canvas.set_height((height * dpr) as u32);

    let backend = CanvasBackend::with_canvas_object(canvas)
        .expect("canvas backend")
        .into_drawing_area();
    backend.fill(&RGBColor(15, 17, 26)).ok();

    let x_min = zoom.x_min.unwrap_or(state.domain_min);
    let x_max = zoom.x_max.unwrap_or(state.domain_max);
    let mut chart = match ChartBuilder::on(&backend)
        .margin(CHART_MARGIN as i32)
        .x_label_area_size(CHART_X_LABEL as i32)
        .y_label_area_size(CHART_Y_LABEL as i32)
        .build_cartesian_2d(x_min..x_max, state.y_min..state.y_max)
    {
        Ok(chart) => chart,
        Err(_) => return,
    };

    chart
        .configure_mesh()
        .disable_mesh()
        .label_style(
            ("Quicksand", 12)
                .into_font()
                .color(&RGBColor(180, 167, 198)),
        )
        .axis_style(&RGBColor(80, 70, 100))
        .x_desc(state.x_label.clone())
        .y_desc(state.y_label.clone())
        .draw()
        .ok();

    let mut line_map: std::collections::HashMap<RGBColor, Vec<(f64, f64)>> =
        std::collections::HashMap::new();
    for point in &state.points {
        line_map
            .entry(point.color)
            .or_default()
            .push((point.x, point.y));
    }
    for (color, mut series) in line_map {
        series.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        chart.draw_series(LineSeries::new(series, &color)).ok();
    }

    for point in &state.points {
        chart
            .draw_series(std::iter::once(Circle::new(
                (point.x, point.y),
                4,
                point.color.filled(),
            )))
            .ok();
    }

    for point in &state.dosage_points {
        chart
            .draw_series(std::iter::once(TriangleMarker::new(
                (point.x, point.y),
                6,
                point.color.filled(),
            )))
            .ok();
    }

    backend.present().ok();
}

fn draw_estrannaise_chart(canvas_id: &str, series: &EstrannaiseSeries, zoom: ViewZoom) {
    let Some(canvas) = window()
        .document()
        .and_then(|doc| doc.get_element_by_id(canvas_id))
        .and_then(|el| el.dyn_into::<HtmlCanvasElement>().ok())
    else {
        return;
    };
    let rect = canvas.get_bounding_client_rect();
    let width = rect.width().max(320.0);
    let height = rect.height().max(280.0);
    let dpr = window().device_pixel_ratio();
    canvas.set_width((width * dpr) as u32);
    canvas.set_height((height * dpr) as u32);

    let backend = CanvasBackend::with_canvas_object(canvas)
        .expect("canvas backend")
        .into_drawing_area();
    backend.fill(&RGBColor(15, 17, 26)).ok();

    let x_min = zoom.x_min.unwrap_or(series.domain_min);
    let x_max = zoom.x_max.unwrap_or(series.domain_max);
    let mut chart = match ChartBuilder::on(&backend)
        .margin(CHART_MARGIN as i32)
        .x_label_area_size(CHART_X_LABEL as i32)
        .y_label_area_size(CHART_Y_LABEL as i32)
        .build_cartesian_2d(x_min..x_max, series.y_min..series.y_max)
    {
        Ok(chart) => chart,
        Err(_) => return,
    };

    chart
        .configure_mesh()
        .disable_mesh()
        .label_style(
            ("Quicksand", 12)
                .into_font()
                .color(&RGBColor(180, 167, 198)),
        )
        .axis_style(&RGBColor(80, 70, 100))
        .x_desc(series.x_label.clone())
        .y_desc(series.y_label.clone())
        .draw()
        .ok();

    if let Some((start, end)) = series.forecast {
        chart
            .draw_series(std::iter::once(Rectangle::new(
                [(start, series.y_min), (end, series.y_max)],
                RGBAColor(246, 193, 119, 0.12).filled(),
            )))
            .ok();
    }

    if !series.blended.is_empty() {
        let line = series.blended.iter().map(|p| (p.x, p.y));
        chart
            .draw_series(LineSeries::new(line, &RGBColor(46, 134, 171)))
            .ok();
    }
    if !series.stepped.is_empty() {
        let line = series.stepped.iter().map(|p| (p.x, p.y));
        chart
            .draw_series(LineSeries::new(line, &RGBColor(162, 59, 114)))
            .ok();
    }
    for point in &series.blood {
        chart
            .draw_series(std::iter::once(Circle::new(
                (point.x, point.y),
                4,
                RGBColor(255, 165, 0).filled(),
            )))
            .ok();
    }

    backend.present().ok();
}
