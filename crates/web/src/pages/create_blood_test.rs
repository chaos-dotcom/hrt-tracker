use gloo_timers::callback::Timeout;
use js_sys::Date;
use leptos::*;
use leptos_router::A;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsValue;

use crate::layout::page_layout;
use crate::store::use_store;
use crate::utils::{hormone_unit_label, parse_hormone_unit};
use hrt_shared::types::{BloodTest, HormoneUnits};

#[derive(Clone, PartialEq)]
struct UnitOption {
    label: String,
}

const UNIT_OPTIONS: [HormoneUnits; 9] = [
    HormoneUnits::E2PgMl,
    HormoneUnits::E2PmolL,
    HormoneUnits::TNgDl,
    HormoneUnits::TNmolL,
    HormoneUnits::Mg,
    HormoneUnits::NgMl,
    HormoneUnits::MIuMl,
    HormoneUnits::MIuL,
    HormoneUnits::UL,
];

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
    if value.trim().is_empty() {
        return Date::now() as i64;
    }
    let parsed = Date::parse(value);
    if parsed.is_nan() {
        Date::now() as i64
    } else {
        parsed as i64
    }
}

fn parse_optional(value: &str) -> Option<f64> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    trimmed.parse::<f64>().ok().filter(|v| v.is_finite())
}

fn unit_or_default(value: &str, fallback: HormoneUnits) -> HormoneUnits {
    parse_hormone_unit(value).unwrap_or(fallback)
}

#[component]
pub fn CreateBloodTest() -> impl IntoView {
    let store = use_store();
    let unit_options = UNIT_OPTIONS
        .iter()
        .map(|unit| UnitOption {
            label: hormone_unit_label(unit).to_string(),
        })
        .collect::<Vec<_>>();

    let test_date_time = create_rw_signal(to_local_input_value(Date::now() as i64));
    let estradiol_level = create_rw_signal("0".to_string());
    let estradiol_unit = create_rw_signal(hormone_unit_label(&HormoneUnits::E2PgMl).to_string());
    let estrannaise_number = create_rw_signal("0".to_string());
    let estrannaise_unit = create_rw_signal(hormone_unit_label(&HormoneUnits::E2PgMl).to_string());
    let test_level = create_rw_signal("0".to_string());
    let test_unit = create_rw_signal(hormone_unit_label(&HormoneUnits::TNgDl).to_string());
    let progesterone_level = create_rw_signal("0".to_string());
    let progesterone_unit = create_rw_signal(hormone_unit_label(&HormoneUnits::NgMl).to_string());
    let fsh_level = create_rw_signal("0".to_string());
    let fsh_unit = create_rw_signal(hormone_unit_label(&HormoneUnits::MIuMl).to_string());
    let lh_level = create_rw_signal("0".to_string());
    let lh_unit = create_rw_signal(hormone_unit_label(&HormoneUnits::MIuMl).to_string());
    let prolactin_level = create_rw_signal("0".to_string());
    let prolactin_unit = create_rw_signal(hormone_unit_label(&HormoneUnits::NgMl).to_string());
    let shbg_level = create_rw_signal("0".to_string());
    let shbg_unit = create_rw_signal(hormone_unit_label(&HormoneUnits::TNmolL).to_string());
    let free_androgen_index = create_rw_signal("0".to_string());
    let notes = create_rw_signal(String::new());
    let show_feedback = create_rw_signal(false);
    let feedback_timeout: Rc<RefCell<Option<Timeout>>> = Rc::new(RefCell::new(None));

    let on_submit = {
        let feedback_timeout = feedback_timeout.clone();
        move |ev: leptos::ev::SubmitEvent| {
            ev.prevent_default();
            let date = parse_datetime_local(&test_date_time.get());

            let estradiol_value = parse_optional(&estradiol_level.get());
            let estrannaise_value = parse_optional(&estrannaise_number.get());
            let test_value = parse_optional(&test_level.get());
            let progesterone_value = parse_optional(&progesterone_level.get());
            let fsh_value = parse_optional(&fsh_level.get());
            let lh_value = parse_optional(&lh_level.get());
            let prolactin_value = parse_optional(&prolactin_level.get());
            let shbg_value = parse_optional(&shbg_level.get());
            let free_androgen_value = parse_optional(&free_androgen_index.get());

            let estradiol_unit_value =
                unit_or_default(&estradiol_unit.get(), HormoneUnits::E2PgMl);
            let test_unit_value = unit_or_default(&test_unit.get(), HormoneUnits::TNgDl);
            let progesterone_unit_value =
                unit_or_default(&progesterone_unit.get(), HormoneUnits::NgMl);
            let fsh_unit_value = unit_or_default(&fsh_unit.get(), HormoneUnits::MIuMl);
            let lh_unit_value = unit_or_default(&lh_unit.get(), HormoneUnits::MIuMl);
            let prolactin_unit_value = unit_or_default(&prolactin_unit.get(), HormoneUnits::NgMl);
            let shbg_unit_value = unit_or_default(&shbg_unit.get(), HormoneUnits::TNmolL);

            let estrannaise_unit_value =
                unit_or_default(&estrannaise_unit.get(), HormoneUnits::E2PgMl);
            let measured_e2 = estradiol_value.map(|value| {
                if estradiol_unit_value == HormoneUnits::E2PmolL {
                    value / 3.671
                } else {
                    value
                }
            });
            let predicted_e2 = estrannaise_value.map(|value| {
                if estrannaise_unit_value == HormoneUnits::E2PmolL {
                    value / 3.671
                } else {
                    value
                }
            });
            let fudge_factor = match (measured_e2, predicted_e2) {
                (Some(measured), Some(predicted))
                    if predicted.is_finite() && predicted > 0.0 && measured.is_finite() =>
                {
                    Some((measured / predicted * 1000.0).round() / 1000.0)
                }
                _ => None,
            };

            let entry = BloodTest {
                date,
                estradiolLevel: estradiol_value,
                testLevel: test_value,
                estradiolUnit: Some(estradiol_unit_value),
                testUnit: Some(test_unit_value),
                progesteroneLevel: progesterone_value,
                progesteroneUnit: Some(progesterone_unit_value),
                fshLevel: fsh_value,
                fshUnit: Some(fsh_unit_value),
                lhLevel: lh_value,
                lhUnit: Some(lh_unit_value),
                prolactinLevel: prolactin_value,
                prolactinUnit: Some(prolactin_unit_value),
                shbgLevel: shbg_value,
                shbgUnit: Some(shbg_unit_value),
                freeAndrogenIndex: free_androgen_value,
                estrannaiseNumber: predicted_e2,
                fudgeFactor: fudge_factor,
                notes: if notes.get().trim().is_empty() {
                    None
                } else {
                    Some(notes.get())
                },
                estrogenType: None,
            };

            store.data.update(|d| d.bloodTests.push(entry));
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

    let render_unit_options = move || {
        unit_options
            .clone()
            .into_iter()
            .map(|option| {
                let label = option.label.clone();
                view! { <option value=label.clone()>{label}</option> }
            })
            .collect_view()
    };

    page_layout(
        "Create Blood Test",
        view! {
            <div class="view-layout">
                <div class="view-header">
                    <div>
                        <h2>"Create blood test entry"</h2>
                        <p class="muted">"Record labs and estrannaise data in one place."</p>
                    </div>
                    <div class="header-actions">
                        <A href="/backup">"View all tests"</A>
                    </div>
                </div>

                <form class="form-wide" on:submit=on_submit>
                    <label>
                        "Test date / time"
                        <input
                            type="datetime-local"
                            on:input=move |ev| test_date_time.set(event_target_value(&ev))
                            prop:value=move || test_date_time.get()
                        />
                    </label>

                    <div class="form-section">
                        <h3>"Hormone levels"</h3>

                        <div class="inline-equal">
                            <label>
                                "Estradiol level"
                                <input
                                    type="number"
                                    step="any"
                                    on:input=move |ev| estradiol_level.set(event_target_value(&ev))
                                    prop:value=move || estradiol_level.get()
                                />
                            </label>
                            <label>
                                "Estradiol unit"
                                <select
                                    on:change=move |ev| estradiol_unit.set(event_target_value(&ev))
                                    prop:value=move || estradiol_unit.get()
                                >
                                    {render_unit_options()}
                                </select>
                            </label>
                        </div>

                        <div class="inline-equal">
                            <label>
                                "Estrannaise predicted E2"
                                <input
                                    type="number"
                                    step="any"
                                    on:input=move |ev| estrannaise_number.set(event_target_value(&ev))
                                    prop:value=move || estrannaise_number.get()
                                />
                            </label>
                            <label>
                                "Predicted unit"
                                <select
                                    on:change=move |ev| estrannaise_unit.set(event_target_value(&ev))
                                    prop:value=move || estrannaise_unit.get()
                                >
                                    <option value="pg/mL">"pg/mL"</option>
                                    <option value="pmol/L">"pmol/L"</option>
                                </select>
                            </label>
                        </div>

                        <div class="inline-equal">
                            <label>
                                "Testosterone level"
                                <input
                                    type="number"
                                    step="any"
                                    on:input=move |ev| test_level.set(event_target_value(&ev))
                                    prop:value=move || test_level.get()
                                />
                            </label>
                            <label>
                                "Testosterone unit"
                                <select
                                    on:change=move |ev| test_unit.set(event_target_value(&ev))
                                    prop:value=move || test_unit.get()
                                >
                                    {render_unit_options()}
                                </select>
                            </label>
                        </div>

                        <div class="inline-equal">
                            <label>
                                "Progesterone level"
                                <input
                                    type="number"
                                    step="any"
                                    on:input=move |ev| progesterone_level.set(event_target_value(&ev))
                                    prop:value=move || progesterone_level.get()
                                />
                            </label>
                            <label>
                                "Progesterone unit"
                                <select
                                    on:change=move |ev| progesterone_unit.set(event_target_value(&ev))
                                    prop:value=move || progesterone_unit.get()
                                >
                                    {render_unit_options()}
                                </select>
                            </label>
                        </div>

                        <div class="inline-equal">
                            <label>
                                "FSH level"
                                <input
                                    type="number"
                                    step="any"
                                    on:input=move |ev| fsh_level.set(event_target_value(&ev))
                                    prop:value=move || fsh_level.get()
                                />
                            </label>
                            <label>
                                "FSH unit"
                                <select
                                    on:change=move |ev| fsh_unit.set(event_target_value(&ev))
                                    prop:value=move || fsh_unit.get()
                                >
                                    {render_unit_options()}
                                </select>
                            </label>
                        </div>

                        <div class="inline-equal">
                            <label>
                                "LH level"
                                <input
                                    type="number"
                                    step="any"
                                    on:input=move |ev| lh_level.set(event_target_value(&ev))
                                    prop:value=move || lh_level.get()
                                />
                            </label>
                            <label>
                                "LH unit"
                                <select
                                    on:change=move |ev| lh_unit.set(event_target_value(&ev))
                                    prop:value=move || lh_unit.get()
                                >
                                    {render_unit_options()}
                                </select>
                            </label>
                        </div>

                        <div class="inline-equal">
                            <label>
                                "Prolactin level"
                                <input
                                    type="number"
                                    step="any"
                                    on:input=move |ev| prolactin_level.set(event_target_value(&ev))
                                    prop:value=move || prolactin_level.get()
                                />
                            </label>
                            <label>
                                "Prolactin unit"
                                <select
                                    on:change=move |ev| prolactin_unit.set(event_target_value(&ev))
                                    prop:value=move || prolactin_unit.get()
                                >
                                    {render_unit_options()}
                                </select>
                            </label>
                        </div>

                        <div class="inline-equal">
                            <label>
                                "SHBG level"
                                <input
                                    type="number"
                                    step="any"
                                    on:input=move |ev| shbg_level.set(event_target_value(&ev))
                                    prop:value=move || shbg_level.get()
                                />
                            </label>
                            <label>
                                "SHBG unit"
                                <select
                                    on:change=move |ev| shbg_unit.set(event_target_value(&ev))
                                    prop:value=move || shbg_unit.get()
                                >
                                    {render_unit_options()}
                                </select>
                            </label>
                        </div>

                        <div class="inline-equal">
                            <label>
                                "Free Androgen Index"
                                <input
                                    type="number"
                                    step="any"
                                    on:input=move |ev| free_androgen_index.set(event_target_value(&ev))
                                    prop:value=move || free_androgen_index.get()
                                />
                            </label>
                            <div></div>
                        </div>
                    </div>

                    <label>
                        "Notes"
                        <textarea
                            rows="3"
                            placeholder="notes..."
                            on:input=move |ev| notes.set(event_target_value(&ev))
                            prop:value=move || notes.get()
                        ></textarea>
                    </label>

                    <div class="form-actions">
                        <button type="submit">"Create test"</button>
                        <Show when=move || show_feedback.get()>
                            <p class="muted">"Blood test added!"</p>
                        </Show>
                    </div>
                </form>
            </div>
        }
        .into_view(),
    )
}
