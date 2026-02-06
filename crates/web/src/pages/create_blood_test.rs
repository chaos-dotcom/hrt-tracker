use gloo_timers::callback::Timeout;
use js_sys::{Date, Object, Reflect};
use leptos::*;
use leptos_router::A;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{FileReader, HtmlInputElement};

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

#[derive(Clone, Debug)]
struct OcrValue {
    value: String,
    unit: Option<String>,
}

#[derive(Clone, Debug, Default)]
struct OcrExtraction {
    estradiol: Option<OcrValue>,
    testosterone: Option<OcrValue>,
    progesterone: Option<OcrValue>,
    fsh: Option<OcrValue>,
    lh: Option<OcrValue>,
    prolactin: Option<OcrValue>,
    shbg: Option<OcrValue>,
    fai: Option<OcrValue>,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = Tesseract, js_name = recognize)]
    fn tesseract_recognize(image: JsValue, lang: &str, options: JsValue) -> js_sys::Promise;
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

fn parse_ocr_number(token: &str) -> Option<String> {
    let trimmed = token.trim();
    if trimmed.is_empty() {
        return None;
    }
    let trimmed = trimmed.trim_matches(|c: char| c == '<' || c == '>' || c == '=');
    let mut cleaned = String::new();
    for c in trimmed.chars() {
        if c.is_ascii_digit() {
            cleaned.push(c);
        } else if c == '.' {
            cleaned.push(c);
        } else if c == ',' {
            cleaned.push('.');
        }
    }
    if cleaned.ends_with('.') {
        cleaned.pop();
    }
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned)
    }
}

fn normalize_ocr_unit(token: &str) -> Option<String> {
    let trimmed = token.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut cleaned = trimmed
        .trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '/' && c != '|' && c != '\\')
        .to_lowercase();
    cleaned = cleaned.replace('|', "/").replace('\\', "/");
    cleaned = cleaned.replace("/i", "/l").replace("/1", "/l");
    cleaned = cleaned.replace("mlu", "miu");
    match cleaned.as_str() {
        "pmol/l" => Some("pmol/L".to_string()),
        "pg/ml" => Some("pg/mL".to_string()),
        "ng/dl" => Some("ng/dL".to_string()),
        "nmol/l" => Some("nmol/L".to_string()),
        "ng/ml" => Some("ng/mL".to_string()),
        "miu/l" => Some("mIU/L".to_string()),
        "miu/ml" => Some("mIU/mL".to_string()),
        "u/l" | "iu/l" | "ui/l" => Some("U/L".to_string()),
        _ => None,
    }
}

fn extract_ocr_value(text: &str, labels: &[&str]) -> Option<OcrValue> {
    let lower = text.to_lowercase();
    for label in labels {
        if let Some(idx) = lower.find(label) {
            let start = idx + label.len();
            let end = (start + 220).min(lower.len());
            let window = &lower[start..end];
            let tokens: Vec<&str> = window.split_whitespace().collect();
            for (index, token) in tokens.iter().enumerate() {
                if let Some(value) = parse_ocr_number(token) {
                    let mut unit = None;
                    for candidate in tokens.iter().skip(index + 1).take(4) {
                        if let Some(normalized) = normalize_ocr_unit(candidate) {
                            unit = Some(normalized);
                            break;
                        }
                    }
                    return Some(OcrValue { value, unit });
                }
            }
        }
    }
    None
}

fn extract_ocr_values(text: &str) -> OcrExtraction {
    let cleaned = text.replace('\r', "\n");
    OcrExtraction {
        estradiol: extract_ocr_value(&cleaned, &["oestradiol", "estradiol"]),
        testosterone: extract_ocr_value(&cleaned, &["testosterone"]),
        progesterone: extract_ocr_value(&cleaned, &["progesterone"]),
        fsh: extract_ocr_value(
            &cleaned,
            &[
                "follicle stimulating hormone",
                "follicle-stimulating hormone",
                "fsh",
            ],
        ),
        lh: extract_ocr_value(
            &cleaned,
            &["luteinising hormone", "luteinizing hormone", "lh"],
        ),
        prolactin: extract_ocr_value(&cleaned, &["prolactin"]),
        shbg: extract_ocr_value(&cleaned, &["sex hormone binding globulin", "shbg"]),
        fai: extract_ocr_value(&cleaned, &["free androgen index", "fai"]),
    }
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
    let default_e2_unit = store
        .settings
        .get()
        .displayEstradiolUnit
        .unwrap_or(HormoneUnits::E2PmolL);
    let estradiol_unit = create_rw_signal(hormone_unit_label(&default_e2_unit).to_string());
    let estrannaise_number = create_rw_signal("0".to_string());
    let estrannaise_unit = create_rw_signal(hormone_unit_label(&HormoneUnits::E2PgMl).to_string());
    let test_level = create_rw_signal("0".to_string());
    let test_unit = create_rw_signal(hormone_unit_label(&HormoneUnits::TNmolL).to_string());
    let progesterone_level = create_rw_signal("0".to_string());
    let progesterone_unit = create_rw_signal(hormone_unit_label(&HormoneUnits::TNmolL).to_string());
    let fsh_level = create_rw_signal("0".to_string());
    let fsh_unit = create_rw_signal(hormone_unit_label(&HormoneUnits::UL).to_string());
    let lh_level = create_rw_signal("0".to_string());
    let lh_unit = create_rw_signal(hormone_unit_label(&HormoneUnits::UL).to_string());
    let prolactin_level = create_rw_signal("0".to_string());
    let prolactin_unit = create_rw_signal(hormone_unit_label(&HormoneUnits::MIuL).to_string());
    let shbg_level = create_rw_signal("0".to_string());
    let shbg_unit = create_rw_signal(hormone_unit_label(&HormoneUnits::TNmolL).to_string());
    let free_androgen_index = create_rw_signal("0".to_string());
    let notes = create_rw_signal(String::new());
    let ocr_busy = create_rw_signal(false);
    let ocr_status = create_rw_signal(String::new());
    let ocr_error = create_rw_signal(None::<String>);
    let ocr_input_ref = create_node_ref::<HtmlInputElement>();
    let show_feedback = create_rw_signal(false);
    let feedback_timeout: Rc<RefCell<Option<Timeout>>> = Rc::new(RefCell::new(None));

    let open_ocr_picker = {
        let ocr_input_ref = ocr_input_ref;
        move |_| {
            if ocr_busy.get() {
                return;
            }
            if let Some(input) = ocr_input_ref.get() {
                input.click();
            }
        }
    };

    let on_ocr_change = {
        let estradiol_level = estradiol_level;
        let estradiol_unit = estradiol_unit;
        let test_level = test_level;
        let test_unit = test_unit;
        let progesterone_level = progesterone_level;
        let progesterone_unit = progesterone_unit;
        let fsh_level = fsh_level;
        let fsh_unit = fsh_unit;
        let lh_level = lh_level;
        let lh_unit = lh_unit;
        let prolactin_level = prolactin_level;
        let prolactin_unit = prolactin_unit;
        let shbg_level = shbg_level;
        let shbg_unit = shbg_unit;
        let free_androgen_index = free_androgen_index;
        move |ev: leptos::ev::Event| {
            if ocr_busy.get() {
                return;
            }
            let input: HtmlInputElement = event_target(&ev);
            let Some(files) = input.files() else {
                return;
            };
            let Some(file) = files.get(0) else {
                return;
            };
            let Ok(reader) = FileReader::new() else {
                ocr_error.set(Some("Could not read the image file.".to_string()));
                ocr_status.set(String::new());
                return;
            };
            let input_clone = input.clone();
            let input_reset = input.clone();
            let reader_clone = reader.clone();
            ocr_busy.set(true);
            ocr_error.set(None);
            ocr_status.set("Reading image...".to_string());
            let onloadend = Closure::wrap(Box::new(move |_ev: web_sys::Event| {
                let data_url = reader_clone
                    .result()
                    .ok()
                    .and_then(|value| value.as_string())
                    .unwrap_or_default();
                if data_url.trim().is_empty() {
                    ocr_error.set(Some("Failed to read image data.".to_string()));
                    ocr_status.set(String::new());
                    ocr_busy.set(false);
                    input_clone.set_value("");
                    return;
                }
                let tesseract_loaded =
                    Reflect::has(&js_sys::global(), &JsValue::from_str("Tesseract"))
                        .unwrap_or(false);
                if !tesseract_loaded {
                    ocr_error.set(Some("OCR library did not load. Try refreshing.".to_string()));
                    ocr_status.set(String::new());
                    ocr_busy.set(false);
                    input_clone.set_value("");
                    return;
                }
                ocr_status.set("Running OCR...".to_string());
                spawn_local(async move {
                    let options = Object::new();
                    let _ = Reflect::set(
                        &options,
                        &JsValue::from_str("workerPath"),
                        &JsValue::from_str("/ocr/worker.min.js"),
                    );
                    let _ = Reflect::set(
                        &options,
                        &JsValue::from_str("corePath"),
                        &JsValue::from_str("/ocr/tesseract-core.wasm.js"),
                    );
                    let _ = Reflect::set(
                        &options,
                        &JsValue::from_str("langPath"),
                        &JsValue::from_str("/ocr"),
                    );
                    let result = JsFuture::from(tesseract_recognize(
                        JsValue::from_str(&data_url),
                        "eng",
                        options.into(),
                    ))
                    .await;
                    let value = match result {
                        Ok(value) => value,
                        Err(_) => {
                            ocr_error.set(Some("OCR failed to process the image.".to_string()));
                            ocr_status.set(String::new());
                            ocr_busy.set(false);
                            input_clone.set_value("");
                            return;
                        }
                    };
                    let text = Reflect::get(&value, &JsValue::from_str("data"))
                        .ok()
                        .and_then(|data| Reflect::get(&data, &JsValue::from_str("text")).ok())
                        .and_then(|value| value.as_string())
                        .unwrap_or_default();
                    if text.trim().is_empty() {
                        ocr_error.set(Some("OCR did not find any text.".to_string()));
                        ocr_status.set(String::new());
                        ocr_busy.set(false);
                        input_clone.set_value("");
                        return;
                    }
                    let extracted = extract_ocr_values(&text);
                    let mut filled = 0;
                    if let Some(value) = extracted.estradiol {
                        estradiol_level.set(value.value);
                        if let Some(unit) = value.unit {
                            estradiol_unit.set(unit);
                        }
                        filled += 1;
                    }
                    if let Some(value) = extracted.testosterone {
                        test_level.set(value.value);
                        if let Some(unit) = value.unit {
                            test_unit.set(unit);
                        }
                        filled += 1;
                    }
                    if let Some(value) = extracted.progesterone {
                        progesterone_level.set(value.value);
                        if let Some(unit) = value.unit {
                            progesterone_unit.set(unit);
                        }
                        filled += 1;
                    }
                    if let Some(value) = extracted.fsh {
                        fsh_level.set(value.value);
                        if let Some(unit) = value.unit {
                            fsh_unit.set(unit);
                        }
                        filled += 1;
                    }
                    if let Some(value) = extracted.lh {
                        lh_level.set(value.value);
                        if let Some(unit) = value.unit {
                            lh_unit.set(unit);
                        }
                        filled += 1;
                    }
                    if let Some(value) = extracted.prolactin {
                        prolactin_level.set(value.value);
                        if let Some(unit) = value.unit {
                            prolactin_unit.set(unit);
                        }
                        filled += 1;
                    }
                    if let Some(value) = extracted.shbg {
                        shbg_level.set(value.value);
                        if let Some(unit) = value.unit {
                            shbg_unit.set(unit);
                        }
                        filled += 1;
                    }
                    if let Some(value) = extracted.fai {
                        free_androgen_index.set(value.value);
                        filled += 1;
                    }
                    if filled == 0 {
                        ocr_error.set(Some("OCR ran, but no lab values were found.".to_string()));
                        ocr_status.set(String::new());
                    } else {
                        ocr_status.set(format!(
                            "Autofilled {filled} fields from OCR. Review before saving."
                        ));
                    }
                    ocr_busy.set(false);
                    input_clone.set_value("");
                });
            }) as Box<dyn FnMut(_)>);
            reader.set_onloadend(Some(onloadend.as_ref().unchecked_ref()));
            if reader.read_as_data_url(&file).is_err() {
                ocr_error.set(Some("Failed to read the image file.".to_string()));
                ocr_status.set(String::new());
                ocr_busy.set(false);
                input_reset.set_value("");
                return;
            }
            onloadend.forget();
        }
    };

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

            let default_e2_unit = store
                .settings
                .get()
                .displayEstradiolUnit
                .unwrap_or(HormoneUnits::E2PmolL);
            let estradiol_unit_value = unit_or_default(&estradiol_unit.get(), default_e2_unit);
            let test_unit_value = unit_or_default(&test_unit.get(), HormoneUnits::TNmolL);
            let progesterone_unit_value =
                unit_or_default(&progesterone_unit.get(), HormoneUnits::TNmolL);
            let fsh_unit_value = unit_or_default(&fsh_unit.get(), HormoneUnits::UL);
            let lh_unit_value = unit_or_default(&lh_unit.get(), HormoneUnits::UL);
            let prolactin_unit_value = unit_or_default(&prolactin_unit.get(), HormoneUnits::MIuL);
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

            store.data.update(|d| {
                if let Some(existing) = d.bloodTests.iter_mut().find(|item| item.date == date) {
                    *existing = entry;
                } else {
                    d.bloodTests.push(entry);
                }
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
                        <h3>"Import from screenshot"</h3>
                        <div class="inline-equal">
                            <div>
                                <input
                                    type="file"
                                    accept="image/*"
                                    node_ref=ocr_input_ref
                                    on:change=on_ocr_change
                                    class="hidden-input"
                                />
                                <button
                                    type="button"
                                    on:click=open_ocr_picker
                                    prop:disabled=move || ocr_busy.get()
                                >
                                    {move || if ocr_busy.get() { "Running OCR..." } else { "Upload lab screenshot" }}
                                </button>
                                <p class="muted">"PNG/JPEG/WEBP · Cropped results work best."</p>
                            </div>
                            <div>
                                <Show when=move || !ocr_status.get().is_empty()>
                                    <p class="muted">{move || ocr_status.get()}</p>
                                </Show>
                                <Show when=move || ocr_error.get().is_some()>
                                    <p class="muted">{move || ocr_error.get().unwrap_or_default()}</p>
                                </Show>
                            </div>
                        </div>
                    </div>

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
