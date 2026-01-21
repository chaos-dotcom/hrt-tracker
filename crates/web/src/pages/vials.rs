use chrono::{Local, TimeZone};
use leptos::*;
use leptos_router::{use_params_map, A};
use std::rc::Rc;

use crate::layout::page_layout;
use crate::store::use_store;
use crate::utils::parse_date_or_now;

#[component]
pub fn VialsPage() -> impl IntoView {
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
pub fn VialsCreatePage() -> impl IntoView {
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
        store.mark_dirty();

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
pub fn VialsDetailPage() -> impl IntoView {
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
                                    store_toggle.mark_dirty();
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
                            store_subvial.mark_dirty();
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
