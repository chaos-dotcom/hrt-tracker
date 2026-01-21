use leptos::*;

use crate::layout::page_layout;
use crate::store::use_store;
use crate::utils::parse_date_or_now;

#[component]
pub fn CreateBloodTest() -> impl IntoView {
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
        store.mark_dirty();
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
