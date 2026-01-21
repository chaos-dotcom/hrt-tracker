use leptos::*;

use crate::layout::page_layout;
use crate::store::use_store;
use crate::utils::{parse_date_or_now, parse_hormone_unit};
use hrt_shared::types::{
    Antiandrogens, DosageHistoryEntry, HormoneUnits, InjectableEstradiols, OralEstradiols,
    ProgesteroneRoutes, Progesterones,
};

#[component]
pub fn CreateDosage() -> impl IntoView {
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

        let unit = parse_hormone_unit(&unit_value.get()).unwrap_or(HormoneUnits::Mg);
        let note = if note_value.get().trim().is_empty() {
            None
        } else {
            Some(note_value.get())
        };
        let entry = match dosage_type.get().as_str() {
            "oralEstradiol" => DosageHistoryEntry::OralEstradiol {
                date,
                id: None,
                kind: OralEstradiols::Hemihydrate,
                dose,
                unit,
                pillQuantity: None,
                note,
            },
            "antiandrogen" => DosageHistoryEntry::Antiandrogen {
                date,
                id: None,
                kind: Antiandrogens::Spiro,
                dose,
                unit,
                note,
            },
            "progesterone" => DosageHistoryEntry::Progesterone {
                date,
                id: None,
                kind: Progesterones::Micronized,
                route: ProgesteroneRoutes::Oral,
                dose,
                unit,
                pillQuantity: None,
                note,
            },
            _ => DosageHistoryEntry::InjectableEstradiol {
                date,
                id: None,
                kind: InjectableEstradiols::Valerate,
                dose,
                unit,
                note,
                injectionSite: None,
                vialId: None,
                subVialId: None,
                syringeKind: None,
                needleLength: None,
                needleGauge: None,
                photos: None,
            },
        };

        store.data.update(|d| d.dosageHistory.push(entry));
        store.mark_dirty();

        dose_value.set("".to_string());
        medication_name.set("".to_string());
        note_value.set("".to_string());
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

                <label>"Medication type"</label>
                <select
                    on:change=move |ev| dosage_type.set(event_target_value(&ev))
                    prop:value=move || dosage_type.get()
                >
                    <option value="injectableEstradiol">"Injectable Estradiol"</option>
                    <option value="oralEstradiol">"Oral Estradiol"</option>
                    <option value="antiandrogen">"Antiandrogen"</option>
                    <option value="progesterone">"Progesterone"</option>
                </select>

                <label>"Medication name"</label>
                <input
                    type="text"
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
                <select
                    on:change=move |ev| unit_value.set(event_target_value(&ev))
                    prop:value=move || unit_value.get()
                >
                    <option value="mg">"mg"</option>
                </select>

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
            </form>
        }
        .into_view(),
    )
}
