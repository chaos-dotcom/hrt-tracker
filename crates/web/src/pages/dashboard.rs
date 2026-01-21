use chrono::{Local, TimeZone};
use leptos::*;
use leptos_router::A;

use crate::layout::page_layout;
use crate::store::use_store;
use hrt_shared::types::DosageHistoryEntry;

#[component]
pub fn Dashboard() -> impl IntoView {
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
