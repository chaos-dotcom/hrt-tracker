use leptos::*;

use crate::layout::page_layout;
use hrt_shared::types::Hormone;

#[component]
pub fn CalcPage() -> impl IntoView {
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
        let output = hrt_shared::convert::convert_hormone(parsed, Hormone::Estradiol, &from, &to)
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
