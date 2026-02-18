use gloo_timers::callback::Timeout;
use js_sys::Date;
use leptos::*;
use leptos_router::A;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsValue;

use crate::layout::page_layout;
use crate::store::use_store;
use crate::utils::{parse_decimal, parse_length_unit};
use hrt_shared::types::{Measurement, WeightUnit};

const BC_LOCATIONS: [&str; 8] = [
    "US or CA",
    "UK",
    "EU under EN 13402",
    "FR, BE, or ES",
    "Australia or New Zealand",
    "US or CA with underbust +4",
    "UK with underbust +4",
    "UK using dress code",
];

const BC_SIZES: [&[i32]; 8] = [
    &[
        28, 30, 32, 34, 36, 38, 40, 42, 44, 46, 48, 50, 52, 54, 56, 58, 60,
    ],
    &[
        28, 30, 32, 34, 36, 38, 40, 42, 44, 46, 48, 50, 52, 54, 56, 58, 60,
    ],
    &[
        60, 65, 70, 75, 80, 85, 90, 95, 100, 105, 110, 115, 120, 125, 130, 135, 140,
    ],
    &[
        75, 80, 85, 90, 95, 100, 105, 110, 115, 120, 125, 130, 135, 140, 145, 150, 155,
    ],
    &[
        6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32, 34, 36, 38,
    ],
    &[
        32, 34, 36, 38, 40, 42, 44, 46, 48, 50, 52, 54, 56, 58, 60, 62, 64,
    ],
    &[
        32, 34, 36, 38, 40, 42, 44, 46, 48, 50, 52, 54, 56, 58, 60, 62, 64,
    ],
    &[
        4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32, 34, 36,
    ],
];

const BC_US_CUP: [&str; 9] = ["AA", "A", "B", "C", "D", "DD/E", "DDD/F", "DDDD/G", "H"];
const BC_AZ_CUP: [&str; 9] = ["AA", "A", "B", "C", "D", "DD", "E", "F", "G"];
const BC_UK_CUP: [&str; 9] = ["AA", "A", "B", "C", "D", "DD", "E", "F", "FF"];
const BC_EU_CUP: [&str; 9] = ["AA", "A", "B", "C", "D", "E", "F", "H", "I"];

#[derive(Clone, PartialEq)]
struct BraConversion {
    location: &'static str,
    size: i32,
    cup: &'static str,
}

#[derive(Clone, PartialEq)]
struct BraCalculation {
    location: &'static str,
    size: i32,
    cup: &'static str,
    conversions: Vec<BraConversion>,
}

fn bra_system_index(value: &str) -> usize {
    match value {
        "us" => 0,
        "uk" => 1,
        "eu" => 2,
        "fr" => 3,
        "au" => 4,
        "us-plus4" => 5,
        "uk-plus4" => 6,
        "uk-dress" => 7,
        _ => 1,
    }
}

fn cup_set_for_system(index: usize) -> &'static [&'static str] {
    match index {
        0 | 5 => &BC_US_CUP,
        1 | 6 | 7 => &BC_UK_CUP,
        4 => &BC_AZ_CUP,
        _ => &BC_EU_CUP,
    }
}

fn closest_size_index(value: f64, sizes: &[i32]) -> usize {
    let mut best_index = 0usize;
    let mut best_diff = f64::MAX;
    for (idx, size) in sizes.iter().enumerate() {
        let diff = (value - *size as f64).abs();
        if diff < best_diff {
            best_diff = diff;
            best_index = idx;
        }
    }
    best_index
}

fn to_inches(value: f64, unit: &str) -> f64 {
    if unit == "cm" {
        value / 2.54
    } else {
        value
    }
}

fn to_centimeters(value: f64, unit: &str) -> f64 {
    if unit == "cm" {
        value
    } else {
        value * 2.54
    }
}

fn calculate_bra_size(
    underbust: f64,
    bust: f64,
    unit: &str,
    system: &str,
) -> Option<BraCalculation> {
    if !underbust.is_finite() || !bust.is_finite() || underbust <= 0.0 || bust <= 0.0 {
        return None;
    }
    let system_index = bra_system_index(system);
    let mut underbust_in = to_inches(underbust, unit);
    let bust_in = to_inches(bust, unit);
    if system == "us-plus4" || system == "uk-plus4" {
        underbust_in += 4.0;
    }
    let size_value = if system_index == 2 || system_index == 3 {
        to_centimeters(underbust, unit)
    } else {
        underbust_in
    };
    let size_index = closest_size_index(size_value, BC_SIZES[system_index]);
    let diff_in = (bust_in - underbust_in).max(0.0);
    let mut cup_index = diff_in.round() as isize;
    if cup_index < 0 {
        cup_index = 0;
    }
    let base_cups = cup_set_for_system(system_index);
    let max_cup = base_cups.len().saturating_sub(1) as isize;
    if cup_index > max_cup {
        cup_index = max_cup;
    }
    let cup_index = cup_index as usize;
    let base_size = BC_SIZES[system_index][size_index];
    let base_cup = base_cups[cup_index];
    let mut conversions = Vec::new();
    for target in 0..BC_SIZES.len() {
        if target == system_index {
            continue;
        }
        let target_size = BC_SIZES[target][size_index];
        let target_cup = cup_set_for_system(target)[cup_index];
        conversions.push(BraConversion {
            location: BC_LOCATIONS[target],
            size: target_size,
            cup: target_cup,
        });
    }
    Some(BraCalculation {
        location: BC_LOCATIONS[system_index],
        size: base_size,
        cup: base_cup,
        conversions,
    })
}

fn to_local_input_value(ms: i64) -> String {
    let date = Date::new(&JsValue::from_f64(ms as f64));
    let year = date.get_full_year();
    let month = date.get_month() + 1;
    let day = date.get_date();
    let hour = date.get_hours();
    let minute = date.get_minutes();
    format!(
        "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}",
        year = year,
        month = month,
        day = day,
        hour = hour,
        minute = minute
    )
}

fn parse_datetime_local(value: &str) -> i64 {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Date::now() as i64;
    }
    let parsed = Date::parse(trimmed);
    if parsed.is_nan() {
        Date::now() as i64
    } else {
        parsed as i64
    }
}

fn parse_optional(value: &str) -> Option<f64> {
    parse_decimal(value)
}

fn parse_weight_unit(value: &str) -> Option<WeightUnit> {
    match value {
        "kg" => Some(WeightUnit::KG),
        "lbs" => Some(WeightUnit::LBS),
        _ => None,
    }
}

#[component]
pub fn CreateMeasurement() -> impl IntoView {
    let store = use_store();
    let measurement_date_time = create_rw_signal(to_local_input_value(Date::now() as i64));
    let weight = create_rw_signal(String::new());
    let weight_unit = create_rw_signal("kg".to_string());
    let height = create_rw_signal(String::new());
    let height_unit = create_rw_signal("cm".to_string());
    let underbust = create_rw_signal(String::new());
    let bust = create_rw_signal(String::new());
    let bideltoid = create_rw_signal(String::new());
    let waist = create_rw_signal(String::new());
    let hip = create_rw_signal(String::new());
    let body_unit = create_rw_signal("cm".to_string());
    let show_feedback = create_rw_signal(false);
    let feedback_timeout: Rc<RefCell<Option<Timeout>>> = Rc::new(RefCell::new(None));

    let bra_calculation = create_memo(move |_| {
        let underbust_val = parse_optional(&underbust.get());
        let bust_val = parse_optional(&bust.get());
        let Some(underbust_val) = underbust_val else {
            return None;
        };
        let Some(bust_val) = bust_val else {
            return None;
        };
        let system = store
            .settings
            .get()
            .braSizeSystem
            .clone()
            .unwrap_or_else(|| "uk".to_string());
        calculate_bra_size(underbust_val, bust_val, &body_unit.get(), &system)
    });

    let on_submit = {
        let feedback_timeout = feedback_timeout.clone();
        move |ev: leptos::ev::SubmitEvent| {
            ev.prevent_default();
            let date = parse_datetime_local(&measurement_date_time.get());
            let weight_val = parse_optional(&weight.get());
            let height_val = parse_optional(&height.get());
            let underbust_val = parse_optional(&underbust.get());
            let bust_val = parse_optional(&bust.get());
            let bideltoid_val = parse_optional(&bideltoid.get());
            let waist_val = parse_optional(&waist.get());
            let hip_val = parse_optional(&hip.get());
            let weight_unit_val = parse_weight_unit(&weight_unit.get());
            let height_unit_val = parse_length_unit(&height_unit.get());
            let body_unit_val = parse_length_unit(&body_unit.get());
            let system = store
                .settings
                .get()
                .braSizeSystem
                .clone()
                .unwrap_or_else(|| "uk".to_string());
            let bra_value = match (underbust_val, bust_val) {
                (Some(underbust_val), Some(bust_val)) => {
                    calculate_bra_size(underbust_val, bust_val, &body_unit.get(), &system)
                        .map(|calc| format!("{}{}", calc.size, calc.cup))
                }
                _ => None,
            };

            let entry = Measurement {
                date,
                weight: weight_val,
                weightUnit: weight_unit_val,
                height: height_val,
                heightUnit: height_unit_val,
                underbust: underbust_val,
                bust: bust_val,
                bideltoid: bideltoid_val,
                waist: waist_val,
                hip: hip_val,
                bodyMeasurementUnit: body_unit_val,
                braSize: bra_value,
            };

            store.data.update(|data| {
                data.measurements.push(entry);
            });
            store.mark_dirty();

            show_feedback.set(true);
            if let Some(existing) = feedback_timeout.borrow_mut().take() {
                drop(existing);
            }
            let show_feedback = show_feedback.clone();
            *feedback_timeout.borrow_mut() = Some(Timeout::new(3000, move || {
                show_feedback.set(false);
            }));
        }
    };

    page_layout(
        "Create Measurement Entry",
        view! {
            <div class="view-layout">
                <div class="view-header">
                    <div>
                        <h2>"Create Measurement Entry"</h2>
                        <p class="muted">"Track body measurements over time."</p>
                    </div>
                    <div class="header-actions">
                        <A href="/view">"View History"</A>
                    </div>
                </div>
                <form class="form-wide" on:submit=on_submit>
                    <label>
                        "Measurement Date / Time"
                        <input
                            type="datetime-local"
                            on:input=move |ev| measurement_date_time.set(event_target_value(&ev))
                            prop:value=move || measurement_date_time.get()
                        />
                    </label>

                    <div class="inline-equal">
                        <label>
                            "Weight"
                            <div class="inline">
                                <input
                                    type="text"
                                    step="any"
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
                        </label>
                        <label>
                            "Height"
                            <div class="inline">
                                <input
                                    type="text"
                                    step="any"
                                    on:input=move |ev| height.set(event_target_value(&ev))
                                    prop:value=move || height.get()
                                />
                                <select
                                    on:change=move |ev| height_unit.set(event_target_value(&ev))
                                    prop:value=move || height_unit.get()
                                >
                                    <option value="cm">"cm"</option>
                                    <option value="in">"in"</option>
                                </select>
                            </div>
                        </label>
                    </div>

                    <div class="form-section">
                        <div class="form-section-header">
                            <h3>"Body Measurements"</h3>
                            <select
                                on:change=move |ev| body_unit.set(event_target_value(&ev))
                                prop:value=move || body_unit.get()
                            >
                                <option value="cm">"cm"</option>
                                <option value="in">"in"</option>
                            </select>
                        </div>
                        <div class="measurement-grid">
                            <input
                                type="text"
                                step="any"
                                placeholder="Underbust"
                                on:input=move |ev| underbust.set(event_target_value(&ev))
                                prop:value=move || underbust.get()
                            />
                            <input
                                type="text"
                                step="any"
                                placeholder="Bust"
                                on:input=move |ev| bust.set(event_target_value(&ev))
                                prop:value=move || bust.get()
                            />
                            <input
                                type="text"
                                step="any"
                                placeholder="Bideltoid (shoulder)"
                                on:input=move |ev| bideltoid.set(event_target_value(&ev))
                                prop:value=move || bideltoid.get()
                            />
                            <input
                                type="text"
                                step="any"
                                placeholder="Waist"
                                on:input=move |ev| waist.set(event_target_value(&ev))
                                prop:value=move || waist.get()
                            />
                            <input
                                type="text"
                                step="any"
                                placeholder="Hip"
                                on:input=move |ev| hip.set(event_target_value(&ev))
                                prop:value=move || hip.get()
                            />
                        </div>
                    </div>

                    <Show when=move || bra_calculation.get().is_some()>
                        {move || {
                            let calc = bra_calculation.get().unwrap();
                            let location = calc.location;
                            let size = calc.size;
                            let cup = calc.cup;
                            let conversions = StoredValue::new(calc.conversions.clone());
                            view! {
                                <div class="bra-result">
                                    <div class="bra-result-header">
                                        <h3>"Calculated Bra Size"</h3>
                                        <span class="muted">{format!("Based on {location}")}</span>
                                    </div>
                                    <div class="bra-result-main">
                                        <span class="bra-result-size">{format!("{size}{cup}")}</span>
                                        <span class="muted">{format!("({location})")}</span>
                                    </div>
                                    <table class="bra-table">
                                        <thead>
                                            <tr>
                                                <th>"Location"</th>
                                                <th>"Size"</th>
                                                <th>"Cup"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            <For
                                                each=move || conversions.get_value()
                                                key=|item| item.location
                                                children=move |item| {
                                                    view! {
                                                        <tr>
                                                            <td>{item.location}</td>
                                                            <td>{item.size}</td>
                                                            <td>{item.cup}</td>
                                                        </tr>
                                                    }
                                                }
                                            />
                                        </tbody>
                                    </table>
                                </div>
                            }
                        }}
                    </Show>

                    <div class="form-actions">
                        <button type="submit">"Create Measurement"</button>
                        <Show when=move || show_feedback.get()>
                            <p class="muted">"Measurement added!"</p>
                        </Show>
                    </div>
                </form>
            </div>
        }
        .into_view(),
    )
}
