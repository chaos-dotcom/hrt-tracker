use chrono::{Local, TimeZone};
use js_sys::Date;
use leptos::window;
use leptos::*;
use leptos_router::{use_navigate, use_params_map, A};
use std::collections::HashMap;
use std::rc::Rc;

use crate::layout::page_layout;
use crate::store::use_store;
use crate::utils::{fmt_decimal, parse_date_or_now};
use hrt_shared::types::{InjectableEstradiols, SubVial, Vial};

const ESTER_OPTIONS: [InjectableEstradiols; 6] = [
    InjectableEstradiols::Benzoate,
    InjectableEstradiols::Cypionate,
    InjectableEstradiols::Enanthate,
    InjectableEstradiols::Undecylate,
    InjectableEstradiols::Valerate,
    InjectableEstradiols::PolyestradiolPhosphate,
];

fn ester_label(kind: &InjectableEstradiols) -> &'static str {
    match kind {
        InjectableEstradiols::Benzoate => "Estradiol Benzoate",
        InjectableEstradiols::Cypionate => "Estradiol Cypionate",
        InjectableEstradiols::Enanthate => "Estradiol Enanthate",
        InjectableEstradiols::Undecylate => "Estradiol Undecylate",
        InjectableEstradiols::Valerate => "Estradiol Valerate",
        InjectableEstradiols::PolyestradiolPhosphate => "Polyestradiol Phosphate",
    }
}

fn format_date(ms: i64) -> String {
    Local
        .timestamp_millis_opt(ms)
        .single()
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "".to_string())
}

fn format_datetime(ms: i64) -> String {
    Local
        .timestamp_millis_opt(ms)
        .single()
        .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_else(|| "".to_string())
}

fn parse_optional_date(value: &str) -> Option<i64> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(parse_date_or_now(trimmed))
    }
}

#[component]
pub fn VialsPage() -> impl IntoView {
    let store = use_store();
    let vials = move || store.data.get().vials.clone();
    let new_sub_numbers = create_rw_signal(HashMap::<String, String>::new());
    let new_sub_iu = create_rw_signal(HashMap::<String, String>::new());

    let update_sub_number = {
        let new_sub_numbers = new_sub_numbers;
        move |vial_id: String, value: String| {
            new_sub_numbers.update(|map| {
                map.insert(vial_id, value);
            });
        }
    };
    let update_sub_number = StoredValue::new(Rc::new(update_sub_number));

    let update_sub_iu = {
        let new_sub_iu = new_sub_iu;
        move |vial_id: String, value: String| {
            new_sub_iu.update(|map| {
                map.insert(vial_id, value);
            });
        }
    };
    let update_sub_iu = StoredValue::new(Rc::new(update_sub_iu));

    let add_sub_vial = {
        let store = store.clone();
        let new_sub_numbers = new_sub_numbers;
        let new_sub_iu = new_sub_iu;
        move |vial_id: String| {
            let reset_id = vial_id.clone();
            let value = new_sub_numbers
                .get()
                .get(&vial_id)
                .cloned()
                .unwrap_or_default();
            let iu_value = new_sub_iu.get().get(&vial_id).cloned().unwrap_or_default();
            if value.trim().is_empty() {
                return;
            }
            let initial_iu = iu_value
                .trim()
                .parse::<f64>()
                .ok()
                .filter(|v| v.is_finite() && *v >= 0.0);
            let created_at = Date::now() as i64;
            store.data.update(|data| {
                if let Some(vial) = data.vials.iter_mut().find(|v| v.id == vial_id) {
                    let sub = SubVial {
                        id: format!("sub-{}-{}", vial_id, created_at),
                        personalNumber: value.trim().to_string(),
                        createdAt: created_at,
                        notes: None,
                        initialIu: initial_iu,
                    };
                    vial.subVials.push(sub);
                }
            });
            store.mark_dirty();
            new_sub_numbers.update(|map| {
                map.insert(reset_id.clone(), String::new());
            });
            new_sub_iu.update(|map| {
                map.insert(reset_id, String::new());
            });
        }
    };
    let add_sub_vial = StoredValue::new(Rc::new(add_sub_vial));

    let delete_sub_vial = {
        let store = store.clone();
        move |vial_id: String, sub_id: String| {
            store.data.update(|data| {
                if let Some(vial) = data.vials.iter_mut().find(|v| v.id == vial_id) {
                    vial.subVials.retain(|sub| sub.id != sub_id);
                }
            });
            store.mark_dirty();
        }
    };
    let delete_sub_vial = StoredValue::new(Rc::new(delete_sub_vial));

    let update_sub_initial_iu = {
        let store = store.clone();
        move |vial_id: String, sub_id: String, value: String| {
            let parsed = value
                .trim()
                .parse::<f64>()
                .ok()
                .filter(|v| v.is_finite() && *v >= 0.0);
            store.data.update(|data| {
                if let Some(vial) = data.vials.iter_mut().find(|v| v.id == vial_id) {
                    if let Some(sub) = vial.subVials.iter_mut().find(|s| s.id == sub_id) {
                        sub.initialIu = parsed;
                    }
                }
            });
            store.mark_dirty();
        }
    };
    let update_sub_initial_iu = StoredValue::new(Rc::new(update_sub_initial_iu));

    let mark_spent = {
        let store = store.clone();
        move |vial_id: String| {
            let suggested = Local::now().format("%Y-%m-%d").to_string();
            let prompt_value = window()
                .prompt_with_message_and_default(
                    "Spent date (YYYY-MM-DD). Leave blank for today:",
                    &suggested,
                )
                .ok()
                .flatten();
            let spent_at = match prompt_value {
                Some(value) if !value.trim().is_empty() => parse_date_or_now(value.trim()),
                _ => Date::now() as i64,
            };
            store.data.update(|data| {
                if let Some(vial) = data.vials.iter_mut().find(|v| v.id == vial_id) {
                    vial.isSpent = Some(true);
                    vial.spentAt = Some(spent_at);
                }
            });
            store.mark_dirty();
        }
    };
    let mark_spent = StoredValue::new(Rc::new(mark_spent));

    let mark_active = {
        let store = store.clone();
        move |vial_id: String| {
            store.data.update(|data| {
                if let Some(vial) = data.vials.iter_mut().find(|v| v.id == vial_id) {
                    vial.isSpent = Some(false);
                    vial.spentAt = None;
                }
            });
            store.mark_dirty();
        }
    };
    let mark_active = StoredValue::new(Rc::new(mark_active));

    let delete_vial = {
        let store = store.clone();
        move |vial_id: String| {
            let confirmed = window()
                .confirm_with_message("Delete this vial?")
                .unwrap_or(false);
            if !confirmed {
                return;
            }
            store.data.update(|data| {
                data.vials.retain(|vial| vial.id != vial_id);
            });
            store.mark_dirty();
        }
    };
    let delete_vial = StoredValue::new(Rc::new(delete_vial));

    page_layout(
        "Vials",
        view! {
            <div class="view-layout">
                <div class="view-header">
                    <div>
                        <h2>"Vials"</h2>
                        <p class="muted">"Track vial metadata and sub-vials."</p>
                    </div>
                    <div class="header-actions">
                        <A href="/vials/create">"Create New Vial"</A>
                    </div>
                </div>
                <Show
                    when=move || !vials().is_empty()
                    fallback=move || view! { <p class="muted">"No vials yet."</p> }
                >
                    <div class="vial-grid">
                        <For
                            each=vials
                            key=|vial| vial.id.clone()
                            children=move |vial| {
                                let use_by = vial
                                    .useBy
                                    .map(format_date)
                                    .unwrap_or_else(|| "-".to_string());
                                let created = format_datetime(vial.createdAt);
                                let status = if vial.isSpent.unwrap_or(false) {
                                    "Spent"
                                } else {
                                    "Active"
                                };
                                let is_spent = vial.isSpent.unwrap_or(false);
                                let concentration = vial
                                    .concentrationMgPerMl
                                    .map(|v| format!("{:.2} mg/mL", v))
                                    .unwrap_or_else(|| "-".to_string());
                                let ester = vial.esterKind.clone().unwrap_or_else(|| "-".to_string());
                                let batch = vial.batchNumber.clone().unwrap_or_else(|| "-".to_string());
                                let source = vial.source.clone().unwrap_or_else(|| "-".to_string());
                                let suspension = vial
                                    .suspensionOil
                                    .clone()
                                    .unwrap_or_else(|| "-".to_string());
                                let other = vial
                                    .otherIngredients
                                    .clone()
                                    .unwrap_or_else(|| "-".to_string());
                                let vial_link = format!("/vials/{}", vial.id);
                                let vial_id = StoredValue::new(vial.id.clone());
                                let sub_vials = StoredValue::new(vial.subVials.clone());
                                let sub_input_value = move || {
                                    new_sub_numbers
                                        .get()
                                        .get(&vial_id.get_value())
                                        .cloned()
                                        .unwrap_or_default()
                                };
                                let sub_iu_input_value = move || {
                                    new_sub_iu
                                        .get()
                                        .get(&vial_id.get_value())
                                        .cloned()
                                        .unwrap_or_default()
                                };
                                view! {
                                    <div class="card vial-card">
                                        <div class="vial-header">
                                            <div>
                                                <div class="vial-title">{ester}</div>
                                                <div class="vial-meta">
                                                    <div>{format!("Batch: {}", batch)}</div>
                                                    <div>{format!("Source: {}", source)}</div>
                                                    <div>{format!("Use by: {}", use_by)}</div>
                                                    <div>{format!("Concentration: {}", concentration)}</div>
                                                    <div>{format!("Status: {}", status)}</div>
                                                    <div>{format!("Suspension oil: {}", suspension)}</div>
                                                    <div>{format!("Other ingredients: {}", other)}</div>
                                                    <div class="vial-created">{format!("Created {created}")}</div>
                                                </div>
                                            </div>
                                            <div class="vial-actions">
                                                <A class="vial-action" href=vial_link>
                                                    "Edit"
                                                </A>
                                                <Show
                                                    when=move || is_spent
                                                    fallback=move || view! {
                                                        <button
                                                            type="button"
                                                            class="vial-action warning"
                                                            on:click=move |_| {
                                                                let mark_spent = mark_spent.get_value();
                                                                mark_spent(vial_id.get_value());
                                                            }
                                                        >
                                                            "Mark Spent"
                                                        </button>
                                                    }
                                                >
                                                    <button
                                                        type="button"
                                                        class="vial-action success"
                                                        on:click=move |_| {
                                                            let mark_active = mark_active.get_value();
                                                            mark_active(vial_id.get_value());
                                                        }
                                                    >
                                                        "Mark Active"
                                                    </button>
                                                </Show>
                                                <button
                                                    type="button"
                                                    class="vial-action danger"
                                                    on:click=move |_| {
                                                        let delete_vial = delete_vial.get_value();
                                                        delete_vial(vial_id.get_value());
                                                    }
                                                >
                                                    "Delete"
                                                </button>
                                            </div>
                                        </div>
                                        <div class="subvial-inline">
                                            <div class="vial-subvial-title">"Sub-vials / Cartridges"</div>
                                            <Show
                                                when=move || !sub_vials.get_value().is_empty()
                                                fallback=move || view! { <div class="muted">"None"</div> }
                                            >
                                                <ul class="subvial-list">
                                                    <For
                                                        each=move || sub_vials.get_value()
                                                        key=|sub| sub.id.clone()
                                                        children=move |sub| {
                                                            let sub_id = sub.id.clone();
                                                            let created = format_date(sub.createdAt);
                                                            let initial_value = sub
                                                                .initialIu
                                                                .map(|v| fmt_decimal(v, 2))
                                                                .unwrap_or_default();
                                                            view! {
                                                                <li>
                                                                    <span>
                                                                        {format!("#{}", sub.personalNumber)}
                                                                        <span class="muted">{format!(" ({created})")}</span>
                                                                    </span>
                                                                    <label class="subvial-meta">
                                                                        "Starting IU"
                                                                        <input
                                                                            type="number"
                                                                            step="any"
                                                                            min="0"
                                                                            value=initial_value
                                                                            on:input={
                                                                                let sub_id = sub_id.clone();
                                                                                move |ev| {
                                                                                    let update_sub_initial_iu = update_sub_initial_iu.get_value();
                                                                                    update_sub_initial_iu(
                                                                                        vial_id.get_value(),
                                                                                        sub_id.clone(),
                                                                                        event_target_value(&ev),
                                                                                    )
                                                                                }
                                                                            }
                                                                        />
                                                                    </label>
                                                                    <button
                                                                        type="button"
                                                                        class="vial-action danger"
                                                                        on:click={
                                                                            let sub_id = sub_id.clone();
                                                                            move |_| {
                                                                                let delete_sub_vial = delete_sub_vial.get_value();
                                                                                delete_sub_vial(
                                                                                    vial_id.get_value(),
                                                                                    sub_id.clone(),
                                                                                )
                                                                            }
                                                                        }
                                                                    >
                                                                        "Delete"
                                                                    </button>
                                                                </li>
                                                            }
                                                        }
                                                    />
                                                </ul>
                                            </Show>
                                            <div class="subvial-add">
                                                <input
                                                    type="text"
                                                    placeholder="Add sub-vial number"
                                                    on:input=move |ev| {
                                                        let update_sub_number = update_sub_number.get_value();
                                                        update_sub_number(
                                                            vial_id.get_value(),
                                                            event_target_value(&ev),
                                                        )
                                                    }
                                                    prop:value=sub_input_value
                                                />
                                                <input
                                                    type="number"
                                                    step="any"
                                                    min="0"
                                                    placeholder="Starting IU"
                                                    on:input=move |ev| {
                                                        let update_sub_iu = update_sub_iu.get_value();
                                                        update_sub_iu(
                                                            vial_id.get_value(),
                                                            event_target_value(&ev),
                                                        )
                                                    }
                                                    prop:value=sub_iu_input_value
                                                />
                                                <button
                                                    type="button"
                                                    class="action-button"
                                                    on:click=move |_| {
                                                        let add_sub_vial = add_sub_vial.get_value();
                                                        add_sub_vial(vial_id.get_value());
                                                    }
                                                >
                                                    "Add"
                                                </button>
                                            </div>
                                        </div>
                                    </div>
                                }
                            }
                        />
                    </div>
                </Show>
            </div>
        }
        .into_view(),
    )
}

#[component]
pub fn VialsCreatePage() -> impl IntoView {
    let store = use_store();
    let navigate = use_navigate();
    let ester_kind = create_rw_signal(String::new());
    let custom_ester = create_rw_signal(String::new());
    let suspension_oil = create_rw_signal(String::new());
    let other_ingredients = create_rw_signal(String::new());
    let batch_number = create_rw_signal(String::new());
    let source = create_rw_signal(String::new());
    let concentration = create_rw_signal(String::new());
    let created_date = create_rw_signal(Local::now().format("%Y-%m-%d").to_string());
    let use_by = create_rw_signal(String::new());
    let first_sub_number = create_rw_signal(String::new());
    let first_sub_iu = create_rw_signal(String::new());

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let created = js_sys::Date::now() as i64;
        let entry_id = format!("vial-{}", created);
        let ester_value = if ester_kind.get() == "__other__" {
            custom_ester.get()
        } else {
            ester_kind.get()
        };
        let ester_value = ester_value.trim().to_string();
        let created_at = parse_date_or_now(&created_date.get());
        let use_by_ms = parse_optional_date(&use_by.get());
        let concentration_value = concentration
            .get()
            .trim()
            .parse::<f64>()
            .ok()
            .filter(|v| v.is_finite() && *v > 0.0);
        let first_sub_initial = first_sub_iu
            .get()
            .trim()
            .parse::<f64>()
            .ok()
            .filter(|v| v.is_finite() && *v >= 0.0);
        let mut sub_vials = Vec::new();
        if !first_sub_number.get().trim().is_empty() {
            let sub_created = Date::now() as i64;
            sub_vials.push(SubVial {
                id: format!("sub-{}-{}", entry_id, sub_created),
                personalNumber: first_sub_number.get().trim().to_string(),
                createdAt: sub_created,
                notes: None,
                initialIu: first_sub_initial,
            });
        }
        let entry = Vial {
            id: entry_id,
            esterKind: if ester_value.is_empty() {
                None
            } else {
                Some(ester_value)
            },
            suspensionOil: if suspension_oil.get().trim().is_empty() {
                None
            } else {
                Some(suspension_oil.get())
            },
            otherIngredients: if other_ingredients.get().trim().is_empty() {
                None
            } else {
                Some(other_ingredients.get())
            },
            batchNumber: if batch_number.get().trim().is_empty() {
                None
            } else {
                Some(batch_number.get())
            },
            source: if source.get().trim().is_empty() {
                None
            } else {
                Some(source.get())
            },
            concentrationMgPerMl: concentration_value,
            isSpent: Some(false),
            spentAt: None,
            useBy: use_by_ms,
            createdAt: created_at,
            subVials: sub_vials,
        };

        store.data.update(|data| {
            data.vials.push(entry);
        });
        store.mark_dirty();
        navigate("/vials", Default::default());
    };
    let on_submit = Rc::new(on_submit);

    page_layout(
        "Create Vial",
        view! {
            <form class="form-wide" on:submit={
                let on_submit = on_submit.clone();
                move |ev| on_submit(ev)
            }>
                <label>
                    "Ester kind"
                    <select
                        on:change=move |ev| ester_kind.set(event_target_value(&ev))
                        prop:value=move || ester_kind.get()
                    >
                        <option value="">"Select..."</option>
                        {ESTER_OPTIONS
                            .iter()
                            .map(|kind| {
                                let label = ester_label(kind).to_string();
                                view! { <option value=label.clone()>{label}</option> }
                            })
                            .collect_view()}
                        <option value="__other__">"Other..."</option>
                    </select>
                </label>
                <Show when=move || ester_kind.get() == "__other__">
                    <label>
                        "Custom ester"
                        <input
                            type="text"
                            placeholder="Custom ester"
                            on:input=move |ev| custom_ester.set(event_target_value(&ev))
                            prop:value=move || custom_ester.get()
                        />
                    </label>
                </Show>
                <label>
                    "Suspension oil"
                    <input
                        type="text"
                        placeholder="e.g., Castor oil, MCT"
                        on:input=move |ev| suspension_oil.set(event_target_value(&ev))
                        prop:value=move || suspension_oil.get()
                    />
                </label>
                <label>
                    "Other ingredients"
                    <input
                        type="text"
                        placeholder="e.g., Benzyl benzoate, benzyl alcohol"
                        on:input=move |ev| other_ingredients.set(event_target_value(&ev))
                        prop:value=move || other_ingredients.get()
                    />
                </label>
                <label>
                    "Batch number"
                    <input
                        type="text"
                        placeholder="Batch/lot #"
                        on:input=move |ev| batch_number.set(event_target_value(&ev))
                        prop:value=move || batch_number.get()
                    />
                </label>
                <label>
                    "Vial date"
                    <input
                        type="date"
                        on:input=move |ev| created_date.set(event_target_value(&ev))
                        prop:value=move || created_date.get()
                    />
                </label>
                <label>
                    "Use by date"
                    <input
                        type="date"
                        on:input=move |ev| use_by.set(event_target_value(&ev))
                        prop:value=move || use_by.get()
                    />
                </label>
                <label>
                    "Concentration (mg/mL)"
                    <input
                        type="number"
                        step="any"
                        min="0"
                        placeholder="e.g., 40"
                        on:input=move |ev| concentration.set(event_target_value(&ev))
                        prop:value=move || concentration.get()
                    />
                </label>
                <label>
                    "Manufacturer / Source"
                    <input
                        type="text"
                        placeholder="e.g., compounding pharmacy"
                        on:input=move |ev| source.set(event_target_value(&ev))
                        prop:value=move || source.get()
                    />
                </label>
                <label>
                    "First sub-vial/cartridge number (optional)"
                    <input
                        type="text"
                        placeholder="e.g., 1"
                        on:input=move |ev| first_sub_number.set(event_target_value(&ev))
                        prop:value=move || first_sub_number.get()
                    />
                </label>
                <label>
                    "Starting IU in first cartridge (optional)"
                    <input
                        type="number"
                        step="any"
                        min="0"
                        placeholder="e.g., 200"
                        on:input=move |ev| first_sub_iu.set(event_target_value(&ev))
                        prop:value=move || first_sub_iu.get()
                    />
                </label>
                <button type="submit">"Create"</button>
            </form>
        }
        .into_view(),
    )
}

#[component]
pub fn VialsDetailPage() -> impl IntoView {
    let store = use_store();
    let navigate = use_navigate();
    let params = use_params_map();
    let vial_id = move || params.with(|p| p.get("id").cloned().unwrap_or_else(|| "".into()));
    let vial = move || {
        let id = vial_id();
        store.data.get().vials.into_iter().find(|v| v.id == id)
    };
    let ester_kind = create_rw_signal(String::new());
    let custom_ester = create_rw_signal(String::new());
    let suspension_oil = create_rw_signal(String::new());
    let other_ingredients = create_rw_signal(String::new());
    let batch_number = create_rw_signal(String::new());
    let source = create_rw_signal(String::new());
    let concentration = create_rw_signal(String::new());
    let created_date = create_rw_signal(String::new());
    let use_by = create_rw_signal(String::new());
    let is_spent = create_rw_signal(false);
    let spent_date = create_rw_signal(String::new());

    create_effect(move |_| {
        if let Some(entry) = vial() {
            ester_kind.set(entry.esterKind.clone().unwrap_or_default());
            suspension_oil.set(entry.suspensionOil.unwrap_or_default());
            other_ingredients.set(entry.otherIngredients.unwrap_or_default());
            batch_number.set(entry.batchNumber.unwrap_or_default());
            source.set(entry.source.unwrap_or_default());
            concentration.set(
                entry
                    .concentrationMgPerMl
                    .map(|v| format!("{:.2}", v))
                    .unwrap_or_default(),
            );
            created_date.set(format_date(entry.createdAt));
            use_by.set(entry.useBy.map(format_date).unwrap_or_default());
            is_spent.set(entry.isSpent.unwrap_or(false));
            spent_date.set(entry.spentAt.map(format_date).unwrap_or_default());
            custom_ester.set(String::new());
        }
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let id = vial_id();
        let Some(existing) = vial() else {
            return;
        };
        let ester_value = if ester_kind.get() == "__other__" {
            custom_ester.get()
        } else {
            ester_kind.get()
        };
        let ester_value = ester_value.trim().to_string();
        let created_value = created_date.get();
        let created_at = if created_value.trim().is_empty() {
            existing.createdAt
        } else {
            parse_date_or_now(&created_value)
        };
        let use_by_ms = parse_optional_date(&use_by.get());
        let concentration_value = concentration
            .get()
            .trim()
            .parse::<f64>()
            .ok()
            .filter(|v| v.is_finite() && *v > 0.0);
        let spent_at = if is_spent.get() {
            let spent_value = spent_date.get();
            let value = if spent_value.trim().is_empty() {
                existing.spentAt.unwrap_or_else(|| Date::now() as i64)
            } else {
                parse_date_or_now(&spent_value)
            };
            Some(value)
        } else {
            None
        };
        store.data.update(|data| {
            if let Some(target) = data.vials.iter_mut().find(|v| v.id == id) {
                target.esterKind = if ester_value.is_empty() {
                    None
                } else {
                    Some(ester_value)
                };
                target.suspensionOil = if suspension_oil.get().trim().is_empty() {
                    None
                } else {
                    Some(suspension_oil.get())
                };
                target.otherIngredients = if other_ingredients.get().trim().is_empty() {
                    None
                } else {
                    Some(other_ingredients.get())
                };
                target.batchNumber = if batch_number.get().trim().is_empty() {
                    None
                } else {
                    Some(batch_number.get())
                };
                target.source = if source.get().trim().is_empty() {
                    None
                } else {
                    Some(source.get())
                };
                target.concentrationMgPerMl = concentration_value;
                target.createdAt = created_at;
                target.useBy = use_by_ms;
                target.isSpent = Some(is_spent.get());
                target.spentAt = spent_at;
            }
        });
        store.mark_dirty();
        navigate("/vials", Default::default());
    };

    page_layout(
        "Edit Vial",
        view! {
            <Show
                when=move || vial().is_some()
                fallback=move || view! { <div class="empty-state">"Vial not found."</div> }
            >
                <form class="form-wide" on:submit={
                    let on_submit = on_submit.clone();
                    move |ev| on_submit(ev)
                }>
                    <label>
                        "Ester kind"
                        <select
                            on:change=move |ev| ester_kind.set(event_target_value(&ev))
                            prop:value=move || ester_kind.get()
                        >
                            <option value="">"Select..."</option>
                            {ESTER_OPTIONS
                                .iter()
                                .map(|kind| {
                                    let label = ester_label(kind).to_string();
                                    view! { <option value=label.clone()>{label}</option> }
                                })
                                .collect_view()}
                            <option value="__other__">"Other..."</option>
                        </select>
                    </label>
                    <Show when=move || ester_kind.get() == "__other__">
                        <label>
                            "Custom ester"
                            <input
                                type="text"
                                placeholder="Custom ester"
                                on:input=move |ev| custom_ester.set(event_target_value(&ev))
                                prop:value=move || custom_ester.get()
                            />
                        </label>
                    </Show>
                    <label>
                        "Concentration (mg/mL)"
                        <input
                            type="number"
                            step="any"
                            min="0"
                            on:input=move |ev| concentration.set(event_target_value(&ev))
                            prop:value=move || concentration.get()
                        />
                    </label>
                    <label>
                        "Suspension oil"
                        <input
                            type="text"
                            on:input=move |ev| suspension_oil.set(event_target_value(&ev))
                            prop:value=move || suspension_oil.get()
                        />
                    </label>
                    <label>
                        "Other ingredients"
                        <input
                            type="text"
                            on:input=move |ev| other_ingredients.set(event_target_value(&ev))
                            prop:value=move || other_ingredients.get()
                        />
                    </label>
                    <label>
                        "Batch number"
                        <input
                            type="text"
                            on:input=move |ev| batch_number.set(event_target_value(&ev))
                            prop:value=move || batch_number.get()
                        />
                    </label>
                    <label>
                        "Vial date"
                        <input
                            type="date"
                            on:input=move |ev| created_date.set(event_target_value(&ev))
                            prop:value=move || created_date.get()
                        />
                    </label>
                    <label>
                        "Use by date"
                        <input
                            type="date"
                            on:input=move |ev| use_by.set(event_target_value(&ev))
                            prop:value=move || use_by.get()
                        />
                    </label>
                    <label>
                        "Manufacturer / Source"
                        <input
                            type="text"
                            on:input=move |ev| source.set(event_target_value(&ev))
                            prop:value=move || source.get()
                        />
                    </label>
                    <label class="inline">
                        <span>"Spent"</span>
                        <input
                            type="checkbox"
                            on:change=move |ev| is_spent.set(event_target_checked(&ev))
                            prop:checked=move || is_spent.get()
                        />
                    </label>
                    <Show when=move || is_spent.get()>
                        <label>
                            "Spent date"
                            <input
                                type="date"
                                on:input=move |ev| spent_date.set(event_target_value(&ev))
                                prop:value=move || spent_date.get()
                            />
                        </label>
                    </Show>
                    <button type="submit">"Save"</button>
                </form>
            </Show>
        }
        .into_view(),
    )
}
