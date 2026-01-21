use leptos::*;

use crate::layout::page_layout;
use crate::store::use_store;
use crate::utils::parse_date_or_now;

#[component]
pub fn CreateMeasurement() -> impl IntoView {
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
        store.mark_dirty();
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
                        step="0.1"
                        on:input=move |ev| weight.set(event_target_value(&ev))
                        prop:value=move || weight.get()
                    />
                    <select
                        on:change=move |ev| weight_unit.set(event_target_value(&ev))
                        prop:value=move || weight_unit.get()
                    >
                        <option value="kg">"kg"</option>
                        <option value="lbs">"lbs"</option>
                    </select>
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
                <label>"Body measurement unit"</label>
                <select
                    on:change=move |ev| unit.set(event_target_value(&ev))
                    prop:value=move || unit.get()
                >
                    <option value="cm">"cm"</option>
                    <option value="in">"in"</option>
                </select>
                <button type="submit">"Add Measurement"</button>
                <Show when=move || error.get().is_some()>
                    <p class="error">{move || error.get().unwrap_or_default()}</p>
                </Show>
            </form>
        }
        .into_view(),
    )
}
