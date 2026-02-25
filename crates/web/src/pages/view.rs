use gloo_events::EventListener;
use gloo_net::http::Request;
use js_sys::{Date, Math};
use leptos::window;
use leptos::*;
use leptos_router::A;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::{FormData, HtmlCanvasElement, HtmlInputElement};

mod helpers;
mod scheduling;
mod types;

use self::helpers::{
    bloodtest_pdf_url, dosage_entry_date, dosage_entry_matches_key, dosage_photo_view,
    hormone_unit_labels, injection_site_from_label, injection_site_label, length_unit_label,
    measurement_key, measurement_matches_target, parse_date_only, parse_datetime_local,
    parse_optional_num, parse_weight_unit, progesterone_route_label, syringe_kind_label,
    to_local_input_value, update_photo_note, weight_unit_label,
};
use self::scheduling::{generate_estrannaise_url, get_next_scheduled_candidate};
use self::types::{
    RegimenKey, UploadResponse, DAY_MS, INJECTION_SITE_OPTIONS, SYRINGE_KIND_OPTIONS,
};

use crate::charts::view::{compute_view_chart_state, draw_view_chart, find_nearest_point};
use crate::charts::{
    chart_padding, clamp_zoom, compute_chart_bounds, ChartTooltip, DragState, ViewZoom,
};
use crate::layout::page_layout;
use crate::store::use_store;
use crate::utils::{
    compute_fudge_factor, fmt_blood_value, fmt_date_label, fmt_decimal, format_injectable_dose,
    hormone_unit_label, injectable_dose_from_iu, parse_hormone_unit, parse_length_unit,
};
use hrt_shared::logic::{predict_e2_pg_ml, snap_to_next_injection_boundary};
use hrt_shared::types::{
    DiaryEntry, DosageHistoryEntry, DosagePhoto, HormoneUnits, HrtData, ProgesteroneRoutes,
};

#[component]
pub fn ViewPage() -> impl IntoView {
    let store = use_store();
    let data = store.data;
    let rows = move || {
        let mut entries = data.get().dosageHistory.clone();
        entries.sort_by(|a, b| dosage_entry_date(b).cmp(&dosage_entry_date(a)));
        entries
    };
    let editing_key = create_rw_signal(None::<String>);
    let editing_med_type = create_rw_signal(String::new());
    let editing_entry_id = create_rw_signal(String::new());
    let editing_date = create_rw_signal(String::new());
    let editing_dose = create_rw_signal(String::new());
    let editing_dose_in_iu = create_rw_signal(false);
    let editing_unit = create_rw_signal(String::new());
    let editing_route = create_rw_signal(String::new());
    let editing_pill_qty = create_rw_signal(String::new());
    let editing_note = create_rw_signal(String::new());
    let editing_injection_site = create_rw_signal(String::new());
    let editing_vial_id = create_rw_signal(String::new());
    let editing_sub_vial_id = create_rw_signal(String::new());
    let editing_syringe_kind = create_rw_signal(String::new());
    let editing_needle_length = create_rw_signal(String::new());
    let editing_needle_gauge = create_rw_signal(String::new());
    let editing_bonus = create_rw_signal(false);
    let upload_busy = create_rw_signal(false);
    let photo_input_ref: NodeRef<html::Input> = create_node_ref();
    let confirm_delete = create_rw_signal(None::<String>);
    let confirm_title = create_rw_signal(String::new());
    let confirm_action = create_rw_signal(None::<Rc<dyn Fn()>>);

    let edit_blood_date = create_rw_signal(None::<i64>);
    let edit_blood_date_text = create_rw_signal(String::new());
    let edit_blood_e2 = create_rw_signal(String::new());
    let edit_blood_e2_unit = create_rw_signal(String::new());
    let edit_blood_estrannaise = create_rw_signal(String::new());
    let edit_blood_estrannaise_unit = create_rw_signal(String::new());
    let edit_blood_t = create_rw_signal(String::new());
    let edit_blood_t_unit = create_rw_signal(String::new());
    let edit_blood_prog = create_rw_signal(String::new());
    let edit_blood_prog_unit = create_rw_signal(String::new());
    let edit_blood_fsh = create_rw_signal(String::new());
    let edit_blood_fsh_unit = create_rw_signal(String::new());
    let edit_blood_lh = create_rw_signal(String::new());
    let edit_blood_lh_unit = create_rw_signal(String::new());
    let edit_blood_prolactin = create_rw_signal(String::new());
    let edit_blood_prolactin_unit = create_rw_signal(String::new());
    let edit_blood_shbg = create_rw_signal(String::new());
    let edit_blood_shbg_unit = create_rw_signal(String::new());
    let edit_blood_fai = create_rw_signal(String::new());
    let edit_blood_notes = create_rw_signal(String::new());
    let edit_blood_pdf_files = create_rw_signal(Vec::<String>::new());

    let edit_measurement_id = create_rw_signal(None::<String>);
    let edit_measurement_date = create_rw_signal(None::<i64>);
    let edit_measurement_date_text = create_rw_signal(String::new());
    let edit_measurement_weight = create_rw_signal(String::new());
    let edit_measurement_weight_unit = create_rw_signal(String::new());
    let edit_measurement_height = create_rw_signal(String::new());
    let edit_measurement_height_unit = create_rw_signal(String::new());
    let edit_measurement_underbust = create_rw_signal(String::new());
    let edit_measurement_bust = create_rw_signal(String::new());
    let edit_measurement_bideltoid = create_rw_signal(String::new());
    let edit_measurement_waist = create_rw_signal(String::new());
    let edit_measurement_hip = create_rw_signal(String::new());
    let edit_measurement_unit = create_rw_signal(String::new());
    let edit_measurement_bra_size = create_rw_signal(String::new());

    let note_title = create_rw_signal(String::new());
    let note_content = create_rw_signal(String::new());
    let note_date = create_rw_signal({
        let now = Date::new_0();
        format!(
            "{:04}-{:02}-{:02}",
            now.get_full_year(),
            now.get_month() + 1,
            now.get_date()
        )
    });
    let editing_note_id = create_rw_signal(None::<String>);
    let editing_note_title = create_rw_signal(String::new());
    let editing_note_content = create_rw_signal(String::new());
    let editing_note_date = create_rw_signal(String::new());

    let x_axis_mode = create_rw_signal("date".to_string());
    let time_range_days = create_rw_signal(365_i64);
    let show_medications = create_rw_signal(true);
    let show_e2 = create_rw_signal(true);
    let show_t = create_rw_signal(true);
    let show_prog = create_rw_signal(false);
    let show_fsh = create_rw_signal(false);
    let show_lh = create_rw_signal(false);
    let show_prolactin = create_rw_signal(false);
    let show_shbg = create_rw_signal(false);
    let show_fai = create_rw_signal(false);
    let view_zoom = create_rw_signal(ViewZoom::default());
    let view_tooltip = create_rw_signal(None::<ChartTooltip>);

    let view_chart_state = create_memo({
        let settings = store.settings;
        move |_| {
            let data_value = data.get();
            let settings_value = settings.get();
            compute_view_chart_state(
                &data_value,
                &settings_value,
                &x_axis_mode.get(),
                time_range_days.get(),
                show_medications.get(),
                show_e2.get(),
                show_t.get(),
                show_prog.get(),
                show_fsh.get(),
                show_lh.get(),
                show_prolactin.get(),
                show_shbg.get(),
                show_fai.get(),
            )
        }
    });

    let first_dose_date =
        create_memo(move |_| data.get().dosageHistory.iter().map(dosage_entry_date).min());

    let days_since_first_dose = create_memo(move |_| {
        first_dose_date.get().map(|first| {
            let now = Date::now() as i64;
            ((now - first).abs() / DAY_MS) as i64
        })
    });

    let latest_fudge_factor = create_memo(move |_| {
        let mut tests: Vec<_> = data
            .get()
            .bloodTests
            .iter()
            .filter_map(|test| test.fudgeFactor.map(|value| (test.date, value)))
            .collect();
        tests.sort_by_key(|(date, _)| *date);
        tests.last().map(|(_, value)| *value)
    });

    let estrannaise_url = create_memo(move |_| {
        let data_value = data.get();
        generate_estrannaise_url(&data_value, latest_fudge_factor.get())
    });

    let has_any_regimen = create_memo(move |_| {
        let data_value = data.get();
        data_value.injectableEstradiol.is_some()
            || data_value.oralEstradiol.is_some()
            || data_value.antiandrogen.is_some()
            || data_value.progesterone.is_some()
    });

    let next_scheduled_candidate = create_memo(move |_| {
        let data_value = data.get();
        let use_iu = store.settings.get().displayInjectableInIU.unwrap_or(false);
        get_next_scheduled_candidate(&data_value, use_iu)
    });

    let sorted_notes = create_memo(move |_| {
        let mut notes = data.get().notes.clone();
        notes.sort_by(|a, b| b.date.cmp(&a.date));
        notes
    });

    let sorted_blood_tests = create_memo(move |_| {
        let mut tests = data.get().bloodTests.clone();
        tests.sort_by(|a, b| b.date.cmp(&a.date));
        tests
    });

    let sorted_measurements = create_memo(move |_| {
        let mut items = data.get().measurements.clone();
        items.sort_by(|a, b| b.date.cmp(&a.date));
        items
    });

    let sorted_dosages = create_memo(move |_| rows());

    let pill_total = move || {
        let dose = parse_optional_num(&editing_dose.get()).unwrap_or(0.0);
        let qty = parse_optional_num(&editing_pill_qty.get()).unwrap_or(0.0);
        if dose.is_finite() && qty.is_finite() {
            dose * qty
        } else {
            0.0
        }
    };

    let editing_dose_as_mg = create_memo({
        let store = store.clone();
        move |_| {
            if editing_med_type.get() != "injectableEstradiol" {
                return None;
            }
            let dose_value = parse_optional_num(&editing_dose.get())?;
            if dose_value <= 0.0 {
                return None;
            }
            if !editing_dose_in_iu.get() {
                return Some(dose_value);
            }
            let data_value = store.data.get();
            let schedule_vial_id = data_value
                .injectableEstradiol
                .as_ref()
                .and_then(|cfg| cfg.vialId.as_ref());
            let selected_vial = editing_vial_id.get();
            let selected_vial_id = if selected_vial.trim().is_empty() {
                None
            } else {
                Some(&selected_vial)
            };
            injectable_dose_from_iu(&data_value, dose_value, selected_vial_id, schedule_vial_id)
        }
    });

    create_effect({
        let view_zoom = view_zoom;
        move |_| {
            x_axis_mode.get();
            view_zoom.set(ViewZoom::default());
        }
    });

    let add_note = {
        let store = store.clone();
        move |_| {
            let content = note_content.get().trim().to_string();
            if content.is_empty() {
                return;
            }
            let title = note_title.get().trim().to_string();
            let date_ms = parse_date_only(&note_date.get());
            let id = format!(
                "note-{}-{}",
                Date::now() as i64,
                (Math::random() * 1_000_000.0) as i64
            );
            store.data.update(|data| {
                data.notes.insert(
                    0,
                    DiaryEntry {
                        id,
                        date: date_ms,
                        title: if title.is_empty() { None } else { Some(title) },
                        content,
                    },
                );
            });
            store.mark_dirty();
            store.save();
            note_title.set(String::new());
            note_content.set(String::new());
            let now = Date::new_0();
            note_date.set(format!(
                "{:04}-{:02}-{:02}",
                now.get_full_year(),
                now.get_month() + 1,
                now.get_date()
            ));
        }
    };

    let delete_note = StoredValue::new(Rc::new({
        let store = store.clone();
        move |id: String| {
            store.data.update(|data| {
                data.notes.retain(|note| note.id != id);
            });
            store.mark_dirty();
            store.save();
        }
    }));

    let start_edit_note = {
        move |note: DiaryEntry| {
            editing_note_id.set(Some(note.id.clone()));
            editing_note_title.set(note.title.unwrap_or_default());
            editing_note_content.set(note.content);
            editing_note_date.set(to_local_input_value(note.date)[0..10].to_string());
        }
    };

    let cancel_edit_note = move |_| editing_note_id.set(None);

    let save_edit_note = StoredValue::new(Rc::new({
        let store = store.clone();
        move || {
            let Some(id) = editing_note_id.get() else {
                return;
            };
            let title = editing_note_title.get().trim().to_string();
            let content = editing_note_content.get().trim().to_string();
            if content.is_empty() {
                return;
            }
            let date_ms = parse_date_only(&editing_note_date.get());
            store.data.update(|data| {
                for note in &mut data.notes {
                    if note.id == id {
                        note.title = if title.is_empty() {
                            None
                        } else {
                            Some(title.clone())
                        };
                        note.content = content.clone();
                        note.date = date_ms;
                    }
                }
            });
            store.mark_dirty();
            store.save();
            editing_note_id.set(None);
        }
    }));

    let record_next_dose_now = {
        let store = store.clone();
        move |_| {
            let candidate = next_scheduled_candidate.get();
            let Some(candidate) = candidate else {
                return;
            };
            let now = Date::now() as i64;
            store.data.update(|data| match candidate.med_type {
                RegimenKey::InjectableEstradiol => {
                    if let Some(cfg) = data.injectableEstradiol.as_mut() {
                        let record = DosageHistoryEntry::InjectableEstradiol {
                            date: now,
                            id: None,
                            kind: cfg.kind.clone(),
                            dose: cfg.dose,
                            unit: cfg.unit.clone(),
                            note: None,
                            bonusDose: None,
                            injectionSite: None,
                            vialId: cfg.vialId.clone(),
                            subVialId: cfg.subVialId.clone(),
                            syringeKind: cfg.syringeKind.clone(),
                            needleLength: cfg.needleLength.clone(),
                            needleGauge: cfg.needleGauge.clone(),
                            photos: None,
                        };
                        data.dosageHistory.push(record);
                        if cfg.frequency.is_finite() && cfg.frequency > 0.0 {
                            cfg.nextDoseDate = Some(now + (cfg.frequency * DAY_MS as f64) as i64);
                        }
                    }
                }
                RegimenKey::OralEstradiol => {
                    if let Some(cfg) = data.oralEstradiol.as_ref() {
                        let record = DosageHistoryEntry::OralEstradiol {
                            date: now,
                            id: None,
                            kind: cfg.kind.clone(),
                            dose: cfg.dose,
                            unit: cfg.unit.clone(),
                            pillQuantity: Some(1.0),
                            note: None,
                        };
                        data.dosageHistory.push(record);
                    }
                }
                RegimenKey::Antiandrogen => {
                    if let Some(cfg) = data.antiandrogen.as_ref() {
                        let record = DosageHistoryEntry::Antiandrogen {
                            date: now,
                            id: None,
                            kind: cfg.kind.clone(),
                            dose: cfg.dose,
                            unit: cfg.unit.clone(),
                            note: None,
                        };
                        data.dosageHistory.push(record);
                    }
                }
                RegimenKey::Progesterone => {
                    if let Some(cfg) = data.progesterone.as_ref() {
                        let record = DosageHistoryEntry::Progesterone {
                            date: now,
                            id: None,
                            kind: cfg.kind.clone(),
                            route: cfg.route.clone(),
                            dose: cfg.dose,
                            unit: cfg.unit.clone(),
                            pillQuantity: Some(1.0),
                            note: None,
                        };
                        data.dosageHistory.push(record);
                    }
                }
            });
            store.mark_dirty();
            store.save();
        }
    };

    const VIEW_CANVAS_ID: &str = "view-chart-canvas";
    let view_drag = Rc::new(RefCell::new(None::<DragState>));

    let on_view_mouse_move = {
        let view_chart_state = view_chart_state.clone();
        let view_zoom = view_zoom;
        let view_drag = view_drag.clone();
        let view_tooltip = view_tooltip;
        move |ev: leptos::ev::MouseEvent| {
            let Some(canvas) = window()
                .document()
                .and_then(|doc| doc.get_element_by_id(VIEW_CANVAS_ID))
                .and_then(|el| el.dyn_into::<HtmlCanvasElement>().ok())
            else {
                return;
            };
            let rect = canvas.get_bounding_client_rect();
            let cursor_x = ev.client_x() as f64 - rect.left();
            let cursor_y = ev.client_y() as f64 - rect.top();
            let state = view_chart_state.get();
            let zoom = view_zoom.get();
            let x_min = zoom.x_min.unwrap_or(state.domain_min);
            let x_max = zoom.x_max.unwrap_or(state.domain_max);
            let padding = chart_padding();
            let (width, height, domain_span, y_span) = compute_chart_bounds(
                rect.width(),
                rect.height(),
                padding,
                x_min,
                x_max,
                state.y_min,
                state.y_max,
            );
            if let Some(drag) = view_drag.borrow().as_ref() {
                view_tooltip.set(None);
                let delta_px = cursor_x - drag.start_x;
                let span = x_max - x_min;
                let delta_domain = -(delta_px / width) * span;
                let next_min = drag.start_min + delta_domain;
                let next_max = drag.start_max + delta_domain;
                view_zoom.set(clamp_zoom(
                    state.domain_min,
                    state.domain_max,
                    next_min,
                    next_max,
                ));
            } else {
                let mut best = find_nearest_point(
                    &state.points,
                    x_min,
                    domain_span,
                    state.y_min,
                    y_span,
                    width,
                    height,
                    padding,
                    cursor_x,
                    cursor_y,
                );
                if let Some(candidate) = find_nearest_point(
                    &state.dosage_points,
                    x_min,
                    domain_span,
                    state.y_min,
                    y_span,
                    width,
                    height,
                    padding,
                    cursor_x,
                    cursor_y,
                ) {
                    let replace = best
                        .as_ref()
                        .map(|(_, dist)| *dist)
                        .unwrap_or(f64::INFINITY)
                        > candidate.1;
                    if replace {
                        best = Some(candidate);
                    }
                }
                if let Some((candidate, _)) = best.take() {
                    view_tooltip.set(Some(candidate));
                } else {
                    view_tooltip.set(None);
                }
            }
        }
    };

    let on_view_mouse_leave = {
        let view_drag = view_drag.clone();
        let view_tooltip = view_tooltip;
        move |_| {
            view_drag.replace(None);
            view_tooltip.set(None);
        }
    };

    let on_view_mouse_down = {
        let view_drag = view_drag.clone();
        let view_zoom = view_zoom;
        let view_chart_state = view_chart_state.clone();
        move |ev: leptos::ev::MouseEvent| {
            let Some(canvas) = window()
                .document()
                .and_then(|doc| doc.get_element_by_id(VIEW_CANVAS_ID))
                .and_then(|el| el.dyn_into::<HtmlCanvasElement>().ok())
            else {
                return;
            };
            let rect = canvas.get_bounding_client_rect();
            let cursor_x = ev.client_x() as f64 - rect.left();
            let state = view_chart_state.get();
            let zoom = view_zoom.get();
            let x_min = zoom.x_min.unwrap_or(state.domain_min);
            let x_max = zoom.x_max.unwrap_or(state.domain_max);
            view_drag.replace(Some(DragState {
                start_x: cursor_x,
                start_min: x_min,
                start_max: x_max,
            }));
        }
    };

    let on_view_mouse_up = {
        let view_drag = view_drag.clone();
        move |_| {
            view_drag.replace(None);
        }
    };

    let on_view_wheel = {
        let view_zoom = view_zoom;
        let view_chart_state = view_chart_state.clone();
        move |ev: leptos::ev::WheelEvent| {
            ev.prevent_default();
            let Some(canvas) = window()
                .document()
                .and_then(|doc| doc.get_element_by_id(VIEW_CANVAS_ID))
                .and_then(|el| el.dyn_into::<HtmlCanvasElement>().ok())
            else {
                return;
            };
            let rect = canvas.get_bounding_client_rect();
            let cursor_x = ev.client_x() as f64 - rect.left();
            let state = view_chart_state.get();
            let zoom = view_zoom.get();
            let x_min = zoom.x_min.unwrap_or(state.domain_min);
            let x_max = zoom.x_max.unwrap_or(state.domain_max);
            let padding = chart_padding();
            let (width, _, domain_span, _) = compute_chart_bounds(
                rect.width(),
                rect.height(),
                padding,
                x_min,
                x_max,
                state.y_min,
                state.y_max,
            );
            let cursor_ratio = ((cursor_x - padding.0) / width).clamp(0.0, 1.0);
            let zoom_factor = if ev.delta_y() < 0.0 { 0.85 } else { 1.15 };
            let new_span = (domain_span * zoom_factor).max(1.0);
            let center = x_min + domain_span * cursor_ratio;
            let new_min = center - new_span * cursor_ratio;
            let new_max = new_min + new_span;
            view_zoom.set(clamp_zoom(
                state.domain_min,
                state.domain_max,
                new_min,
                new_max,
            ));
        }
    };

    create_effect({
        let view_chart_state = view_chart_state.clone();
        let view_zoom = view_zoom;
        move |_| {
            let state = view_chart_state.get();
            if !state.has_data {
                return;
            }
            draw_view_chart(VIEW_CANVAS_ID, &state, view_zoom.get());
        }
    });

    let view_resize_listener: Rc<RefCell<Option<EventListener>>> = Rc::new(RefCell::new(None));
    create_effect({
        let view_chart_state = view_chart_state.clone();
        let view_zoom = view_zoom;
        let view_resize_listener = view_resize_listener.clone();
        move |_| {
            view_chart_state.get();
            let window = window();
            let listener = EventListener::new(&window, "resize", move |_| {
                let state = view_chart_state.get();
                if state.has_data {
                    draw_view_chart(VIEW_CANVAS_ID, &state, view_zoom.get());
                }
            });
            view_resize_listener.replace(Some(listener));
        }
    });

    let reset_view_zoom = {
        let view_zoom = view_zoom;
        move |_| view_zoom.set(ViewZoom::default())
    };

    let x_axis_days_disabled = move || view_chart_state.get().first_dose.is_none();
    let view_tooltip_value = move || view_tooltip.get();
    let next_candidate_label = move || {
        next_scheduled_candidate
            .get()
            .map(|c| c.label)
            .unwrap_or_default()
    };
    let is_injectable = move || editing_med_type.get() == "injectableEstradiol";
    let is_prog = move || editing_med_type.get() == "progesterone";
    let is_pill = move || {
        let med = editing_med_type.get();
        med == "oralEstradiol" || med == "progesterone"
    };
    let editing_med_label = move || match editing_med_type.get().as_str() {
        "injectableEstradiol" => "Injectable Estradiol",
        "oralEstradiol" => "Oral Estradiol",
        "antiandrogen" => "Antiandrogen",
        "progesterone" => "Progesterone",
        _ => "",
    };

    let editing_photos = create_memo(move |_| {
        let key = editing_entry_id.get();
        if key.trim().is_empty() {
            return Vec::new();
        }
        let mut output = Vec::new();
        for entry in &data.get().dosageHistory {
            if dosage_entry_matches_key(entry, &key) {
                if let DosageHistoryEntry::InjectableEstradiol { photos, .. } = entry {
                    if let Some(items) = photos {
                        output.extend(items.iter().map(dosage_photo_view));
                    }
                }
            }
        }
        output
    });

    let on_save_edit = Rc::new({
        let store_edit = store.clone();
        move || {
            let key = match editing_key.get() {
                Some(value) => value,
                None => return,
            };
            let dose_input_value = parse_optional_num(&editing_dose.get()).unwrap_or(0.0);
            let dose_is_iu = editing_dose_in_iu.get();
            let date_value = parse_datetime_local(&editing_date.get());
            let note_text = editing_note.get();
            let note_value = if note_text.trim().is_empty() {
                None
            } else {
                Some(note_text.clone())
            };
            let unit_value = if editing_med_type.get() == "injectableEstradiol" {
                HormoneUnits::Mg
            } else {
                parse_hormone_unit(&editing_unit.get()).unwrap_or(HormoneUnits::Mg)
            };
            let pill_qty = parse_optional_num(&editing_pill_qty.get()).filter(|v| *v > 0.0);
            let route_value =
                if editing_route.get() == progesterone_route_label(&ProgesteroneRoutes::Boofed) {
                    ProgesteroneRoutes::Boofed
                } else {
                    ProgesteroneRoutes::Oral
                };

            store_edit.data.update(|d| {
                let schedule_vial_id_for_iu = d
                    .injectableEstradiol
                    .as_ref()
                    .and_then(|cfg| cfg.vialId.clone());
                let mut iu_conversion_data = HrtData::default();
                iu_conversion_data.vials = d.vials.clone();
                for entry in &mut d.dosageHistory {
                    if dosage_entry_matches_key(entry, &key) {
                        match entry {
                            DosageHistoryEntry::InjectableEstradiol {
                                date,
                                dose,
                                unit,
                                note,
                                bonusDose,
                                injectionSite,
                                vialId,
                                subVialId,
                                syringeKind,
                                needleLength,
                                needleGauge,
                                ..
                            } => {
                                *date = date_value;
                                if dose_is_iu && unit_value == HormoneUnits::Mg {
                                    let selected_vial = editing_vial_id.get();
                                    let selected_vial_id = if selected_vial.trim().is_empty() {
                                        None
                                    } else {
                                        Some(&selected_vial)
                                    };
                                    if let Some(converted) = injectable_dose_from_iu(
                                        &iu_conversion_data,
                                        dose_input_value,
                                        selected_vial_id,
                                        schedule_vial_id_for_iu.as_ref(),
                                    ) {
                                        *dose = converted;
                                    }
                                } else {
                                    *dose = dose_input_value;
                                }
                                *unit = unit_value.clone();
                                *note = note_value.clone();
                                *bonusDose = if editing_bonus.get() {
                                    Some(true)
                                } else {
                                    None
                                };
                                *injectionSite =
                                    injection_site_from_label(&editing_injection_site.get());
                                *vialId = if editing_vial_id.get().trim().is_empty() {
                                    None
                                } else {
                                    Some(editing_vial_id.get())
                                };
                                *subVialId = if editing_sub_vial_id.get().trim().is_empty() {
                                    None
                                } else {
                                    Some(editing_sub_vial_id.get())
                                };
                                *syringeKind = if editing_syringe_kind.get().trim().is_empty() {
                                    None
                                } else {
                                    Some(editing_syringe_kind.get())
                                };
                                *needleLength = if editing_needle_length.get().trim().is_empty() {
                                    None
                                } else {
                                    Some(editing_needle_length.get())
                                };
                                *needleGauge = if editing_needle_gauge.get().trim().is_empty() {
                                    None
                                } else {
                                    Some(editing_needle_gauge.get())
                                };
                            }
                            DosageHistoryEntry::OralEstradiol {
                                date,
                                dose,
                                unit,
                                note,
                                pillQuantity,
                                ..
                            } => {
                                *date = date_value;
                                *dose = dose_input_value;
                                *unit = unit_value.clone();
                                *note = note_value.clone();
                                *pillQuantity = pill_qty;
                            }
                            DosageHistoryEntry::Antiandrogen {
                                date,
                                dose,
                                unit,
                                note,
                                ..
                            } => {
                                *date = date_value;
                                *dose = dose_input_value;
                                *unit = unit_value.clone();
                                *note = note_value.clone();
                            }
                            DosageHistoryEntry::Progesterone {
                                date,
                                dose,
                                unit,
                                note,
                                pillQuantity,
                                route,
                                ..
                            } => {
                                *date = date_value;
                                *dose = dose_input_value;
                                *unit = unit_value.clone();
                                *note = note_value.clone();
                                *pillQuantity = pill_qty;
                                *route = route_value.clone();
                            }
                        }
                    }
                }
            });
            store_edit.mark_dirty();
            store_edit.save();
            editing_key.set(None);
        }
    });

    let on_cancel_edit = move |_| editing_key.set(None);

    create_effect(move |_| {
        let selected_vial = editing_vial_id.get();
        if selected_vial.trim().is_empty() {
            editing_sub_vial_id.set(String::new());
            return;
        }
        let data_value = data.get();
        let Some(vial) = data_value.vials.iter().find(|v| v.id == selected_vial) else {
            editing_sub_vial_id.set(String::new());
            return;
        };
        let selected_sub = editing_sub_vial_id.get();
        if !vial.subVials.iter().any(|s| s.id == selected_sub) {
            editing_sub_vial_id.set(String::new());
        }
    });

    let open_photo_picker = {
        let photo_input_ref = photo_input_ref;
        move |_| {
            if upload_busy.get() {
                return;
            }
            if editing_med_type.get() != "injectableEstradiol" {
                return;
            }
            if editing_entry_id.get().trim().is_empty() {
                return;
            }
            if let Some(input) = photo_input_ref.get() {
                input.click();
            }
        }
    };

    let on_photo_change = StoredValue::new(Rc::new({
        let upload_busy = upload_busy;
        let editing_entry_id = editing_entry_id;
        let store = store.clone();
        move |ev: leptos::ev::Event| {
            if upload_busy.get() {
                return;
            }
            if editing_med_type.get() != "injectableEstradiol" {
                return;
            }
            let input: HtmlInputElement = event_target(&ev);
            let Some(files) = input.files() else {
                return;
            };
            let entry_id = editing_entry_id.get();
            if entry_id.trim().is_empty() {
                return;
            }
            let input_clone = input.clone();
            let file_list: Vec<_> = (0..files.length())
                .filter_map(|idx| files.get(idx))
                .collect();
            if file_list.is_empty() {
                return;
            }
            upload_busy.set(true);
            let store = store.clone();
            spawn_local(async move {
                for file in file_list {
                    let form = match FormData::new() {
                        Ok(form) => form,
                        Err(_) => continue,
                    };
                    if form
                        .append_with_blob_and_filename("file", &file, &file.name())
                        .is_err()
                    {
                        continue;
                    }
                    let request =
                        match Request::post(&format!("/api/dosage-photo/{entry_id}")).body(form) {
                            Ok(request) => request,
                            Err(_) => continue,
                        };
                    let Ok(response) = request.send().await else {
                        continue;
                    };
                    if !response.ok() {
                        continue;
                    }
                    let Ok(payload) = response.json::<UploadResponse>().await else {
                        continue;
                    };
                    if payload.filenames.is_empty() {
                        continue;
                    }
                    store.data.update(|data| {
                        for entry in &mut data.dosageHistory {
                            if dosage_entry_matches_key(entry, &entry_id) {
                                if let DosageHistoryEntry::InjectableEstradiol { photos, .. } =
                                    entry
                                {
                                    let list = photos.get_or_insert_with(Vec::new);
                                    for filename in &payload.filenames {
                                        list.push(DosagePhoto::Legacy(filename.clone()));
                                    }
                                }
                            }
                        }
                    });
                    store.mark_dirty();
                    store.save();
                }
                input_clone.set_value("");
                upload_busy.set(false);
            });
        }
    }));

    let on_photo_delete = StoredValue::new(Rc::new({
        let store = store.clone();
        let editing_entry_id = editing_entry_id;
        move |filename: String| {
            let entry_id = editing_entry_id.get();
            if entry_id.trim().is_empty() {
                return;
            }
            let store = store.clone();
            spawn_local(async move {
                let request = Request::delete(&format!("/api/dosage-photo/{entry_id}/{filename}"));
                let Ok(response) = request.send().await else {
                    return;
                };
                if !response.ok() {
                    return;
                }
                store.data.update(|data| {
                    for entry in &mut data.dosageHistory {
                        if dosage_entry_matches_key(entry, &entry_id) {
                            if let DosageHistoryEntry::InjectableEstradiol { photos, .. } = entry {
                                if let Some(list) = photos.as_mut() {
                                    list.retain(|photo| match photo {
                                        DosagePhoto::Legacy(file) => file != &filename,
                                        DosagePhoto::Entry { file, .. } => file != &filename,
                                    });
                                    if list.is_empty() {
                                        *photos = None;
                                    }
                                }
                            }
                        }
                    }
                });
                store.mark_dirty();
                store.save();
            });
        }
    }));

    let on_photo_note_update = StoredValue::new(Rc::new({
        let store = store.clone();
        let editing_entry_id = editing_entry_id;
        move |filename: String, note: String| {
            let entry_id = editing_entry_id.get();
            if entry_id.trim().is_empty() {
                return;
            }
            store.data.update(|data| {
                for entry in &mut data.dosageHistory {
                    if dosage_entry_matches_key(entry, &entry_id) {
                        if let DosageHistoryEntry::InjectableEstradiol { photos, .. } = entry {
                            if let Some(list) = photos.as_mut() {
                                for photo in list.iter_mut() {
                                    let matches = match photo {
                                        DosagePhoto::Legacy(file) => file == &filename,
                                        DosagePhoto::Entry { file, .. } => file == &filename,
                                    };
                                    if matches {
                                        update_photo_note(photo, note.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            });
            store.mark_dirty();
        }
    }));

    let store_blood_modal = store.clone();
    let store_measure_modal = store.clone();

    page_layout(
        "View",
        view! {
            <div class="view-layout">
                <div class="view-header">
                    <div>
                        <h2>"HRT Tracking Data"</h2>
                        <p class="muted">
                            "This chart shows your hormone levels from blood tests along with your dosage history over time."
                        </p>
                    </div>
                    <div class="header-actions">
                        <A href="/create/measurement">"Add Measurement"</A>
                    </div>
                </div>

                <div class="card">
                    <div class="card-header">
                        <h3>"Current Regimen"</h3>
                        <div class="card-actions">
                            <Show when=move || estrannaise_url.get().is_some()>
                                <a
                                    href=move || estrannaise_url.get().unwrap_or_default()
                                    target="_blank"
                                    rel="noopener noreferrer"
                                >
                                    "View on Estrannaise"
                                </a>
                            </Show>
                            <button
                                type="button"
                                on:click=record_next_dose_now
                                prop:disabled=move || !has_any_regimen.get()
                                prop:title=move || {
                                    let label = next_candidate_label();
                                    if label.is_empty() {
                                        "".to_string()
                                    } else {
                                        format!("Record: {label}")
                                    }
                                }
                            >
                                "Record next dose now"
                            </button>
                            <A href="/edit/schedule">"Edit Schedule"</A>
                        </div>
                    </div>
                    <div class="view-summary">
                        <Show when=move || days_since_first_dose.get().is_some()>
                            <p>
                                <strong>"Days since first dose: "</strong>
                                {move || days_since_first_dose.get().unwrap_or(0)}
                            </p>
                        </Show>
                        <Show when=move || latest_fudge_factor.get().is_some()>
                            <p>
                                <strong>"Estrannaise fudge factor: "</strong>
                                {move || latest_fudge_factor.get().unwrap_or(0.0)}
                            </p>
                        </Show>
                        <Show when=move || store.data.get().injectableEstradiol.is_some()>
                            <p>
                                <strong>"Injectable Estradiol: "</strong>
                                {move || {
                                    let data_value = store.data.get();
                                    let use_iu = store.settings.get().displayInjectableInIU.unwrap_or(false);
                                    data_value
                                        .injectableEstradiol
                                        .as_ref()
                                        .map(|cfg| {
                                            let dose_label = format_injectable_dose(
                                                &data_value,
                                                cfg.dose,
                                                &cfg.unit,
                                                cfg.vialId.as_ref(),
                                                cfg.vialId.as_ref(),
                                                use_iu,
                                            );
                                            format!(
                                                "{:?}, {dose_label} every {:.1} days",
                                                cfg.kind, cfg.frequency
                                            )
                                        })
                                        .unwrap_or_default()
                                }}
                            </p>
                        </Show>
                        <Show when=move || store.data.get().oralEstradiol.is_some()>
                            <p>
                                <strong>"Oral Estradiol: "</strong>
                                {move || store
                                    .data
                                    .get()
                                    .oralEstradiol
                                    .as_ref()
                                    .map(|cfg| format!("{:?}, {:.2} {:?} every {:.1} days", cfg.kind, cfg.dose, cfg.unit, cfg.frequency))
                                    .unwrap_or_default()}
                            </p>
                        </Show>
                        <Show when=move || store.data.get().antiandrogen.is_some()>
                            <p>
                                <strong>"Antiandrogen: "</strong>
                                {move || store
                                    .data
                                    .get()
                                    .antiandrogen
                                    .as_ref()
                                    .map(|cfg| format!("{:?}, {:.2} {:?} every {:.1} days", cfg.kind, cfg.dose, cfg.unit, cfg.frequency))
                                    .unwrap_or_default()}
                            </p>
                        </Show>
                        <Show when=move || store.data.get().progesterone.is_some()>
                            <p>
                                <strong>"Progesterone: "</strong>
                                {move || store
                                    .data
                                    .get()
                                    .progesterone
                                    .as_ref()
                                    .map(|cfg| format!("{:?} ({:?}), {:.2} {:?} every {:.1} days", cfg.kind, cfg.route, cfg.dose, cfg.unit, cfg.frequency))
                                    .unwrap_or_default()}
                            </p>
                        </Show>
                        <Show when=move || store.data.get().injectableEstradiol.is_none()
                            && store.data.get().oralEstradiol.is_none()
                            && store.data.get().antiandrogen.is_none()
                            && store.data.get().progesterone.is_none()>
                            <p class="muted">"No regimen set up. You can set one on the dosage page."</p>
                        </Show>
                    </div>
                </div>

                <div class="view-chart-controls">
                    <div class="chart-toolbar">
                        <div class="chart-toolbar-group">
                            <span class="muted">"X-Axis:"</span>
                            <button
                                class:active=move || x_axis_mode.get() == "date"
                                on:click=move |_| x_axis_mode.set("date".to_string())
                            >
                                "Date"
                            </button>
                            <button
                                class:active=move || x_axis_mode.get() == "days"
                                on:click=move |_| x_axis_mode.set("days".to_string())
                                prop:disabled=move || x_axis_days_disabled()
                            >
                                "Days since first dose"
                            </button>
                        </div>
                        <div class="chart-toolbar-group">
                            <span class="muted">"Time Range:"</span>
                            <button class:active=move || time_range_days.get() == 30 on:click=move |_| time_range_days.set(30)>
                                "30 days"
                            </button>
                            <button class:active=move || time_range_days.get() == 90 on:click=move |_| time_range_days.set(90)>
                                "90 days"
                            </button>
                            <button class:active=move || time_range_days.get() == 180 on:click=move |_| time_range_days.set(180)>
                                "180 days"
                            </button>
                            <button class:active=move || time_range_days.get() == 365 on:click=move |_| time_range_days.set(365)>
                                "1 year"
                            </button>
                            <button class:active=move || time_range_days.get() == 9999 on:click=move |_| time_range_days.set(9999)>
                                "All"
                            </button>
                        </div>
                        <div class="chart-toolbar-group">
                            <button on:click=reset_view_zoom disabled=move || view_zoom.get().x_min.is_none()>
                                "Reset zoom"
                            </button>
                        </div>
                    </div>

                    <div class="chart-toolbar view-levels-group">
                        <span class="muted">"Show Levels:"</span>
                        <button class:active=move || show_e2.get() on:click=move |_| show_e2.set(!show_e2.get())>
                            "E2"
                        </button>
                        <button class:active=move || show_t.get() on:click=move |_| show_t.set(!show_t.get())>
                            "T"
                        </button>
                        <button class:active=move || show_prog.get() on:click=move |_| show_prog.set(!show_prog.get())>
                            "Prog"
                        </button>
                        <button class:active=move || show_fsh.get() on:click=move |_| show_fsh.set(!show_fsh.get())>
                            "FSH"
                        </button>
                        <button class:active=move || show_lh.get() on:click=move |_| show_lh.set(!show_lh.get())>
                            "LH"
                        </button>
                        <button
                            class:active=move || show_prolactin.get()
                            on:click=move |_| show_prolactin.set(!show_prolactin.get())
                        >
                            "Prolactin"
                        </button>
                        <button class:active=move || show_shbg.get() on:click=move |_| show_shbg.set(!show_shbg.get())>
                            "SHBG"
                        </button>
                        <button class:active=move || show_fai.get() on:click=move |_| show_fai.set(!show_fai.get())>
                            "FAI"
                        </button>
                    </div>

                    <div class="chart-toolbar view-dosage-group">
                        <span class="muted">"Show Dosages:"</span>
                        <button class:active=move || show_medications.get() on:click=move |_| show_medications.set(!show_medications.get())>
                            {move || if show_medications.get() { "Medication Dosages (on)" } else { "Medication Dosages" }}
                        </button>
                    </div>
                </div>

                <div class="chart-card chart-interactive">
                    <Show when=move || view_chart_state.get().has_data fallback=move || view! {
                        <div class="empty-state">"No data available for the selected time range."</div>
                    }>
                        <canvas
                            id=VIEW_CANVAS_ID
                            width="900"
                            height="420"
                            on:mousemove=on_view_mouse_move.clone()
                            on:mouseleave=on_view_mouse_leave.clone()
                            on:mousedown=on_view_mouse_down.clone()
                            on:mouseup=on_view_mouse_up.clone()
                            on:wheel=on_view_wheel.clone()
                        ></canvas>
                        <Show when=move || view_tooltip_value().is_some()>
                            <div
                                class="chart-tooltip"
                                style=move || {
                                    view_tooltip_value()
                                        .map(|tip| format!("left: {:.0}px; top: {:.0}px;", tip.x + 12.0, tip.y + 12.0))
                                        .unwrap_or_default()
                                }
                            >
                                {move || view_tooltip_value().map(|tip| tip.text).unwrap_or_default()}
                            </div>
                        </Show>
                    </Show>
                    <div class="chart-note muted">
                        <p>"* Dosage values are scaled for visibility on the chart."</p>
                        <p>"* Hover over data points for details."</p>
                        <Show when=move || !store.data.get().bloodTests.is_empty()>
                            <p>"* Hormone measurements are normalized to display units for charting; hover shows recorded units."</p>
                        </Show>
                    </div>
                </div>
            </div>

            <div class="card">
                <h3>"Diary / Notes"</h3>
                <div class="note-form">
                    <div class="note-row">
                        <input
                            type="date"
                            on:input=move |ev| note_date.set(event_target_value(&ev))
                            prop:value=move || note_date.get()
                        />
                        <input
                            type="text"
                            placeholder="Title (optional)"
                            on:input=move |ev| note_title.set(event_target_value(&ev))
                            prop:value=move || note_title.get()
                        />
                    </div>
                    <textarea
                        rows="3"
                        placeholder="Write a note..."
                        on:input=move |ev| note_content.set(event_target_value(&ev))
                        prop:value=move || note_content.get()
                    ></textarea>
                    <div class="form-actions">
                        <button
                            type="button"
                            on:click=add_note
                            prop:disabled=move || note_content.get().trim().is_empty()
                        >
                            "Add Note"
                        </button>
                    </div>
                </div>

                <div class="note-list">
                    <Show
                        when=move || !sorted_notes.get().is_empty()
                        fallback=move || view! { <p class="muted">"No notes yet."</p> }
                    >
                        <ul class="note-items">
                            <For
                                each=move || sorted_notes.get()
                                key=|note| note.id.clone()
                                children=move |note| {
                                    let note_id = note.id.clone();
                                    let note_id_for_edit = note_id.clone();
                                    let note_title = StoredValue::new(note.title.clone().unwrap_or_default());
                                    let note_content = StoredValue::new(note.content.clone());
                                    let note_has_title = !note_title.get_value().is_empty();
                                    let is_editing = move || editing_note_id.get().as_ref() == Some(&note_id_for_edit);
                                    let date_label = move || fmt_date_label(note.date, &x_axis_mode.get(), first_dose_date.get());
                                    let edit_note = StoredValue::new(note.clone());
                                    let note_id_value = StoredValue::new(note_id.clone());
                                    let save_edit_note = save_edit_note;
                                    view! {
                                        <li class="note-item">
                                            <Show when=is_editing fallback=move || view! {
                                                <div class="note-display">
                                                    <div>
                                                        <div class="note-date">{date_label}</div>
                                                        <Show when=move || note_has_title>
                                                            <div class="note-title">{note_title.get_value()}</div>
                                                        </Show>
                                                        <div class="note-content">{note_content.get_value()}</div>
                                                    </div>
                                                    <div class="note-actions">
                                                        <button
                                                            type="button"
                                                            class="action-button"
                                                            on:click=move |_| start_edit_note(edit_note.get_value())
                                                        >
                                                            "Edit"
                                                        </button>
                                                        <button
                                                            type="button"
                                                            class="action-button"
                                                            on:click=move |_| {
                                                            let delete_note = delete_note.get_value();
                                                            delete_note(note_id_value.get_value());
                                                        }>
                                                            "Delete"
                                                        </button>
                                                    </div>
                                                </div>
                                            }>
                                                <div class="note-edit">
                                                    <div class="note-row">
                                                        <input
                                                            type="date"
                                                            on:input=move |ev| editing_note_date.set(event_target_value(&ev))
                                                            prop:value=move || editing_note_date.get()
                                                        />
                                                        <input
                                                            type="text"
                                                            placeholder="Title (optional)"
                                                            on:input=move |ev| editing_note_title.set(event_target_value(&ev))
                                                            prop:value=move || editing_note_title.get()
                                                        />
                                                    </div>
                                                    <textarea
                                                        rows="4"
                                                        on:input=move |ev| editing_note_content.set(event_target_value(&ev))
                                                        prop:value=move || editing_note_content.get()
                                                    ></textarea>
                                                    <div class="note-actions">
                                                        <button type="button" on:click=move |_| {
                                                            let save_edit_note = save_edit_note.get_value();
                                                            save_edit_note();
                                                        }>
                                                            "Save"
                                                        </button>
                                                        <button type="button" on:click=cancel_edit_note>"Cancel"</button>
                                                    </div>
                                                </div>
                                            </Show>
                                        </li>
                                    }
                                }
                            />
                        </ul>
                    </Show>
                </div>
            </div>

            <div class="view-history-grid">
                <div class="card">
                    <h3>"Blood Test History"</h3>
                    <div class="history-scroll">
                        <Show
                            when=move || !sorted_blood_tests.get().is_empty()
                            fallback=move || view! { <p class="muted">"No blood test data available."</p> }
                        >
                            <ul class="history-list">
                                <For
                                    each=move || sorted_blood_tests.get()
                                    key=|entry| entry.date
                                    children=move |entry| {
                                        let entry_date = entry.date;
                                        let e2_default_label = {
                                            let unit = store
                                                .settings
                                                .get()
                                                .displayEstradiolUnit
                                                .unwrap_or(HormoneUnits::E2PmolL);
                                            hormone_unit_label(&unit)
                                        };
                                        let date_label =
                                            move || fmt_date_label(entry_date, &x_axis_mode.get(), first_dose_date.get());
                                        let on_edit = {
                                            let entry = entry.clone();
                                            move |_| {
                                                let date_text = to_local_input_value(entry.date);
                                                edit_blood_date.set(Some(entry.date));
                                                edit_blood_date_text.set(date_text);
                                                edit_blood_e2.set(entry.estradiolLevel.map(fmt_blood_value).unwrap_or_default());
                                                let e2_default = store
                                                    .settings
                                                    .get()
                                                    .displayEstradiolUnit
                                                    .unwrap_or(HormoneUnits::E2PmolL);
                                                edit_blood_e2_unit.set(
                                                    entry
                                                        .estradiolUnit
                                                        .as_ref()
                                                        .map(|u| hormone_unit_label(u).to_string())
                                                        .unwrap_or_else(|| hormone_unit_label(&e2_default).to_string()),
                                                );
                                                edit_blood_estrannaise.set(entry.estrannaiseNumber.map(fmt_blood_value).unwrap_or_default());
                                                edit_blood_estrannaise_unit.set("pg/mL".to_string());
                                                edit_blood_t.set(entry.testLevel.map(fmt_blood_value).unwrap_or_default());
                                                edit_blood_t_unit.set(
                                                    entry
                                                        .testUnit
                                                        .as_ref()
                                                        .map(|u| hormone_unit_label(u).to_string())
                                                        .unwrap_or_else(|| "nmol/L".to_string()),
                                                );
                                                edit_blood_prog.set(entry.progesteroneLevel.map(fmt_blood_value).unwrap_or_default());
                                                edit_blood_prog_unit.set(
                                                    entry
                                                        .progesteroneUnit
                                                        .as_ref()
                                                        .map(|u| hormone_unit_label(u).to_string())
                                                        .unwrap_or_else(|| "nmol/L".to_string()),
                                                );
                                                edit_blood_fsh.set(entry.fshLevel.map(fmt_blood_value).unwrap_or_default());
                                                edit_blood_fsh_unit.set(
                                                    entry
                                                        .fshUnit
                                                        .as_ref()
                                                        .map(|u| hormone_unit_label(u).to_string())
                                                        .unwrap_or_else(|| "U/L".to_string()),
                                                );
                                                edit_blood_lh.set(entry.lhLevel.map(fmt_blood_value).unwrap_or_default());
                                                edit_blood_lh_unit.set(
                                                    entry
                                                        .lhUnit
                                                        .as_ref()
                                                        .map(|u| hormone_unit_label(u).to_string())
                                                        .unwrap_or_else(|| "U/L".to_string()),
                                                );
                                                edit_blood_prolactin.set(entry.prolactinLevel.map(fmt_blood_value).unwrap_or_default());
                                                edit_blood_prolactin_unit.set(
                                                    entry
                                                        .prolactinUnit
                                                        .as_ref()
                                                        .map(|u| hormone_unit_label(u).to_string())
                                                        .unwrap_or_else(|| "mIU/L".to_string()),
                                                );
                                                edit_blood_shbg.set(entry.shbgLevel.map(fmt_blood_value).unwrap_or_default());
                                                edit_blood_shbg_unit.set(entry.shbgUnit.as_ref().map(|u| hormone_unit_label(u).to_string()).unwrap_or_else(|| "nmol/L".to_string()));
                                                edit_blood_fai.set(entry.freeAndrogenIndex.map(fmt_blood_value).unwrap_or_default());
                                                edit_blood_notes.set(entry.notes.clone().unwrap_or_default());
                                                edit_blood_pdf_files.set(
                                                    entry.pdfFiles.clone().unwrap_or_default(),
                                                );
                                            }
                                        };
                                        let pdf_files = StoredValue::new(entry.pdfFiles.clone().unwrap_or_default());
                                        view! {
                                            <li class="history-item">
                                                <div>
                                                    <div class="history-date">{date_label}</div>
                                                    <div class="history-meta history-meta-inline">
                                                        <Show when=move || entry.estradiolLevel.is_some()>
                                                            <span>{format!(
                                                                "E2: {} {}",
                                                                fmt_blood_value(entry.estradiolLevel.unwrap_or_default()),
                                                                entry
                                                                    .estradiolUnit
                                                                    .as_ref()
                                                                    .map(|u| hormone_unit_label(u))
                                                                    .unwrap_or(e2_default_label)
                                                            )}</span>
                                                        </Show>
                                                        <Show when=move || entry.testLevel.is_some()>
                                                            <span>{format!(
                                                                "T: {} {}",
                                                                fmt_blood_value(entry.testLevel.unwrap_or_default()),
                                                                entry
                                                                    .testUnit
                                                                    .as_ref()
                                                                    .map(|u| hormone_unit_label(u))
                                                                    .unwrap_or("nmol/L")
                                                            )}</span>
                                                        </Show>
                                                        <Show when=move || entry.progesteroneLevel.is_some()>
                                                            <span>{format!(
                                                                "Prog: {} {}",
                                                                fmt_blood_value(entry.progesteroneLevel.unwrap_or_default()),
                                                                entry
                                                                    .progesteroneUnit
                                                                    .as_ref()
                                                                    .map(|u| hormone_unit_label(u))
                                                                    .unwrap_or("nmol/L")
                                                            )}</span>
                                                        </Show>
                                                        <Show when=move || entry.fshLevel.is_some()>
                                                            <span>{format!(
                                                                "FSH: {} {}",
                                                                fmt_blood_value(entry.fshLevel.unwrap_or_default()),
                                                                entry
                                                                    .fshUnit
                                                                    .as_ref()
                                                                    .map(|u| hormone_unit_label(u))
                                                                    .unwrap_or("U/L")
                                                            )}</span>
                                                        </Show>
                                                        <Show when=move || entry.lhLevel.is_some()>
                                                            <span>{format!(
                                                                "LH: {} {}",
                                                                fmt_blood_value(entry.lhLevel.unwrap_or_default()),
                                                                entry
                                                                    .lhUnit
                                                                    .as_ref()
                                                                    .map(|u| hormone_unit_label(u))
                                                                    .unwrap_or("U/L")
                                                            )}</span>
                                                        </Show>
                                                        <Show when=move || entry.prolactinLevel.is_some()>
                                                            <span>{format!(
                                                                "PRL: {} {}",
                                                                fmt_blood_value(entry.prolactinLevel.unwrap_or_default()),
                                                                entry
                                                                    .prolactinUnit
                                                                    .as_ref()
                                                                    .map(|u| hormone_unit_label(u))
                                                                    .unwrap_or("mIU/L")
                                                            )}</span>
                                                        </Show>
                                                        <Show when=move || entry.shbgLevel.is_some()>
                                                            <span>{format!(
                                                                "SHBG: {} {}",
                                                                fmt_blood_value(entry.shbgLevel.unwrap_or_default()),
                                                                entry
                                                                    .shbgUnit
                                                                    .as_ref()
                                                                    .map(|u| hormone_unit_label(u))
                                                                    .unwrap_or("nmol/L")
                                                            )}</span>
                                                        </Show>
                                                        <Show when=move || entry.freeAndrogenIndex.is_some()>
                                                            <span>{format!(
                                                                "FAI: {}",
                                                                fmt_blood_value(entry.freeAndrogenIndex.unwrap_or_default())
                                                            )}</span>
                                                        </Show>
                                                        <Show when=move || entry.fudgeFactor.is_some()>
                                                            <span>{format!("FF: {:.3}", entry.fudgeFactor.unwrap_or_default())}</span>
                                                        </Show>
                                                        <Show when=move || !pdf_files.get_value().is_empty()>
                                                            <span>{format!("PDFs: {}", pdf_files.get_value().len())}</span>
                                                        </Show>
                                                    </div>
                                                    <Show when=move || !pdf_files.get_value().is_empty()>
                                                        <div class="history-meta history-meta-inline">
                                                            {pdf_files
                                                                .get_value()
                                                                .into_iter()
                                                                .map(|filename| {
                                                                    let href = bloodtest_pdf_url(&filename);
                                                                    view! {
                                                                        <a href=href target="_blank" rel="noopener noreferrer">
                                                                            {filename}
                                                                        </a>
                                                                    }
                                                                })
                                                                .collect_view()}
                                                        </div>
                                                    </Show>
                                                </div>
                                                <button type="button" class="action-button" on:click=on_edit>
                                                    "Edit"
                                                </button>
                                            </li>
                                        }
                                    }
                                />
                            </ul>
                        </Show>
                    </div>
                </div>

                <div class="card">
                    <h3>"Measurement History"</h3>
                    <div class="history-scroll">
                        <Show
                            when=move || !sorted_measurements.get().is_empty()
                            fallback=move || view! { <p class="muted">"No measurement data available."</p> }
                        >
                            <ul class="history-list">
                                <For
                                    each=move || sorted_measurements.get()
                                    key=|entry| measurement_key(entry)
                                    children=move |entry| {
                                        let entry_date = entry.date;
                                        let date_label =
                                            move || fmt_date_label(entry_date, &x_axis_mode.get(), first_dose_date.get());
                                        let body_unit = StoredValue::new(entry.bodyMeasurementUnit.clone());
                                        let bra_size = StoredValue::new(entry.braSize.clone());
                                        let on_edit = {
                                            let entry = entry.clone();
                                            move |_| {
                                                edit_measurement_id.set(entry.id.clone());
                                                edit_measurement_date.set(Some(entry.date));
                                                edit_measurement_date_text.set(to_local_input_value(entry.date));
                                                edit_measurement_weight.set(entry.weight.map(|v| format!("{:.2}", v)).unwrap_or_default());
                                                edit_measurement_weight_unit.set(entry.weightUnit.as_ref().map(|u| weight_unit_label(u).to_string()).unwrap_or_else(|| "kg".to_string()));
                                                edit_measurement_height.set(entry.height.map(|v| format!("{:.2}", v)).unwrap_or_default());
                                                edit_measurement_height_unit.set(entry.heightUnit.as_ref().map(|u| length_unit_label(u).to_string()).unwrap_or_else(|| "cm".to_string()));
                                                edit_measurement_underbust.set(entry.underbust.map(|v| format!("{:.2}", v)).unwrap_or_default());
                                                edit_measurement_bust.set(entry.bust.map(|v| format!("{:.2}", v)).unwrap_or_default());
                                                edit_measurement_bideltoid.set(entry.bideltoid.map(|v| format!("{:.2}", v)).unwrap_or_default());
                                                edit_measurement_waist.set(entry.waist.map(|v| format!("{:.2}", v)).unwrap_or_default());
                                                edit_measurement_hip.set(entry.hip.map(|v| format!("{:.2}", v)).unwrap_or_default());
                                                edit_measurement_unit.set(entry.bodyMeasurementUnit.as_ref().map(|u| length_unit_label(u).to_string()).unwrap_or_else(|| "cm".to_string()));
                                                edit_measurement_bra_size.set(entry.braSize.clone().unwrap_or_default());
                                            }
                                        };
                                        view! {
                                            <li class="history-item">
                                                <div>
                                                    <div class="history-date">{date_label}</div>
                                                    <div class="history-meta history-meta-inline">
                                                        <Show when=move || entry.weight.is_some()>
                                                            <span>{format!(
                                                                "Weight: {:.1} {}",
                                                                entry.weight.unwrap_or_default(),
                                                                entry
                                                                    .weightUnit
                                                                    .as_ref()
                                                                    .map(weight_unit_label)
                                                                    .unwrap_or("kg")
                                                            )}</span>
                                                        </Show>
                                                        <Show when=move || entry.height.is_some()>
                                                            <span>{format!(
                                                                "Height: {:.1} {}",
                                                                entry.height.unwrap_or_default(),
                                                                entry
                                                                    .heightUnit
                                                                    .as_ref()
                                                                    .map(length_unit_label)
                                                                    .unwrap_or("cm")
                                                            )}</span>
                                                        </Show>
                                                        <Show when=move || bra_size.get_value().is_some()>
                                                            <span>{format!(
                                                                "Bra: {}",
                                                                bra_size.get_value().unwrap_or_default()
                                                            )}</span>
                                                        </Show>
                                                        <Show when=move || entry.underbust.is_some()>
                                                            <span>{format!(
                                                                "Underbust: {:.1} {}",
                                                                entry.underbust.unwrap_or_default(),
                                                                body_unit
                                                                    .get_value()
                                                                    .as_ref()
                                                                    .map(length_unit_label)
                                                                    .unwrap_or("cm")
                                                            )}</span>
                                                        </Show>
                                                        <Show when=move || entry.bust.is_some()>
                                                            <span>{format!(
                                                                "Bust: {:.1} {}",
                                                                entry.bust.unwrap_or_default(),
                                                                body_unit
                                                                    .get_value()
                                                                    .as_ref()
                                                                    .map(length_unit_label)
                                                                    .unwrap_or("cm")
                                                            )}</span>
                                                        </Show>
                                                        <Show when=move || entry.waist.is_some()>
                                                            <span>{format!(
                                                                "Waist: {:.1} {}",
                                                                entry.waist.unwrap_or_default(),
                                                                body_unit
                                                                    .get_value()
                                                                    .as_ref()
                                                                    .map(length_unit_label)
                                                                    .unwrap_or("cm")
                                                            )}</span>
                                                        </Show>
                                                        <Show when=move || entry.hip.is_some()>
                                                            <span>{format!(
                                                                "Hip: {:.1} {}",
                                                                entry.hip.unwrap_or_default(),
                                                                body_unit
                                                                    .get_value()
                                                                    .as_ref()
                                                                    .map(length_unit_label)
                                                                    .unwrap_or("cm")
                                                            )}</span>
                                                        </Show>
                                                        <Show when=move || entry.bideltoid.is_some()>
                                                            <span>{format!(
                                                                "Shoulder: {:.1} {}",
                                                                entry.bideltoid.unwrap_or_default(),
                                                                body_unit
                                                                    .get_value()
                                                                    .as_ref()
                                                                    .map(length_unit_label)
                                                                    .unwrap_or("cm")
                                                            )}</span>
                                                        </Show>
                                                    </div>
                                                </div>
                                                <button type="button" class="action-button" on:click=on_edit>
                                                    "Edit"
                                                </button>
                                            </li>
                                        }
                                    }
                                />
                            </ul>
                        </Show>
                    </div>
                </div>

                <div class="card">
                    <h3>"Medication Dosage History"</h3>
                    <div class="history-scroll">
                        <Show
                            when=move || !sorted_dosages.get().is_empty()
                            fallback=move || view! { <p class="muted">"No dosage data available."</p> }
                        >
                            <ul class="history-list">
                                <For
                                    each=move || sorted_dosages.get()
                                    key=|entry| match entry {
                                        DosageHistoryEntry::InjectableEstradiol { date, id, .. }
                                        | DosageHistoryEntry::OralEstradiol { date, id, .. }
                                        | DosageHistoryEntry::Antiandrogen { date, id, .. }
                                        | DosageHistoryEntry::Progesterone { date, id, .. } => id
                                            .clone()
                                            .unwrap_or_else(|| date.to_string()),
                                    }
                                    children=move |entry| {
                                        let entry_date = dosage_entry_date(&entry);
                                        let date_label =
                                            move || fmt_date_label(entry_date, &x_axis_mode.get(), first_dose_date.get());
                                        let entry_key = match &entry {
                                            DosageHistoryEntry::InjectableEstradiol { date, id, .. }
                                            | DosageHistoryEntry::OralEstradiol { date, id, .. }
                                            | DosageHistoryEntry::Antiandrogen { date, id, .. }
                                            | DosageHistoryEntry::Progesterone { date, id, .. } => {
                                                id.clone().unwrap_or_else(|| date.to_string())
                                            }
                                        };
                                        let on_delete = {
                                            let store = use_store();
                                            let entry_key = entry_key.clone();
                                            let confirm_delete = confirm_delete;
                                            let confirm_title = confirm_title;
                                            let confirm_action = confirm_action;
                                            move |_| {
                                                confirm_title.set("Delete dosage entry?".to_string());
                                                confirm_delete.set(Some(entry_key.clone()));
                                                let store = store.clone();
                                                let entry_key = entry_key.clone();
                                                confirm_action.set(Some(Rc::new(move || {
                                                    store.data.update(|d| {
                                                        d.dosageHistory
                                                            .retain(|item| !dosage_entry_matches_key(item, &entry_key));
                                                    });
                                                    store.mark_dirty();
                                                    store.save();
                                                })));
                                            }
                                        };
                                        let on_edit = {
                                            let entry_key = entry_key.clone();
                                            let entry = entry.clone();
                                            let store = use_store();
                                            move |_| {
                                                let mut resolved_key = entry_key.clone();
                                                store.data.update(|data| {
                                                    for item in &mut data.dosageHistory {
                                                        if dosage_entry_matches_key(item, &entry_key) {
                                                            match item {
                                                                DosageHistoryEntry::InjectableEstradiol { id, date, .. }
                                                                | DosageHistoryEntry::OralEstradiol { id, date, .. }
                                                                | DosageHistoryEntry::Antiandrogen { id, date, .. }
                                                                | DosageHistoryEntry::Progesterone { id, date, .. } => {
                                                                    if id.is_none() {
                                                                        *id = Some(format!(
                                                                            "dose-{}-{}",
                                                                            date,
                                                                            (Math::random() * 1_000_000.0) as i64
                                                                        ));
                                                                    }
                                                                    if let Some(existing) = id.clone() {
                                                                        resolved_key = existing;
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                });
                                                editing_key.set(Some(resolved_key.clone()));
                                                editing_entry_id.set(resolved_key.clone());
                                                let date_text = to_local_input_value(dosage_entry_date(&entry));
                                                editing_date.set(date_text);
                                                editing_dose_in_iu.set(false);
                                                editing_dose.set(match &entry {
                                                    DosageHistoryEntry::InjectableEstradiol {
                                                        dose, ..
                                                    } => fmt_decimal(*dose, 3),
                                                    DosageHistoryEntry::OralEstradiol { dose, .. }
                                                    | DosageHistoryEntry::Antiandrogen { dose, .. }
                                                    | DosageHistoryEntry::Progesterone { dose, .. } => {
                                                        fmt_decimal(*dose, 3)
                                                    }
                                                });
                                                editing_unit.set(match &entry {
                                                    DosageHistoryEntry::InjectableEstradiol { .. } => {
                                                        hormone_unit_label(&HormoneUnits::Mg).to_string()
                                                    }
                                                    DosageHistoryEntry::OralEstradiol { unit, .. }
                                                    | DosageHistoryEntry::Antiandrogen { unit, .. }
                                                    | DosageHistoryEntry::Progesterone { unit, .. } => {
                                                        hormone_unit_label(unit).to_string()
                                                    }
                                                });
                                                editing_note.set(match &entry {
                                                    DosageHistoryEntry::InjectableEstradiol { note, .. }
                                                    | DosageHistoryEntry::OralEstradiol { note, .. }
                                                    | DosageHistoryEntry::Antiandrogen { note, .. }
                                                    | DosageHistoryEntry::Progesterone { note, .. } => {
                                                        note.clone().unwrap_or_default()
                                                    }
                                                });
                                                editing_med_type.set(match &entry {
                                                    DosageHistoryEntry::InjectableEstradiol { .. } => {
                                                        "injectableEstradiol".to_string()
                                                    }
                                                    DosageHistoryEntry::OralEstradiol { .. } => {
                                                        "oralEstradiol".to_string()
                                                    }
                                                    DosageHistoryEntry::Antiandrogen { .. } => {
                                                        "antiandrogen".to_string()
                                                    }
                                                    DosageHistoryEntry::Progesterone { .. } => {
                                                        "progesterone".to_string()
                                                    }
                                                });
                                                editing_route.set(progesterone_route_label(&ProgesteroneRoutes::Oral).to_string());
                                                editing_pill_qty.set(String::new());
                                                editing_injection_site.set(String::new());
                                                editing_vial_id.set(String::new());
                                                editing_sub_vial_id.set(String::new());
                                                editing_syringe_kind.set(String::new());
                                                editing_needle_length.set(String::new());
                                                editing_needle_gauge.set(String::new());
                                                editing_bonus.set(false);
                                                match &entry {
                                                    DosageHistoryEntry::InjectableEstradiol {
                                                        bonusDose,
                                                        injectionSite,
                                                        vialId,
                                                        subVialId,
                                                        syringeKind,
                                                        needleLength,
                                                        needleGauge,
                                                        ..
                                                    } => {
                                                        editing_bonus.set(bonusDose.unwrap_or(false));
                                                        editing_injection_site.set(
                                                            injectionSite
                                                                .as_ref()
                                                                .map(injection_site_label)
                                                                .unwrap_or("")
                                                                .to_string(),
                                                        );
                                                        editing_vial_id.set(vialId.clone().unwrap_or_default());
                                                        editing_sub_vial_id.set(subVialId.clone().unwrap_or_default());
                                                        editing_syringe_kind.set(syringeKind.clone().unwrap_or_default());
                                                        editing_needle_length.set(needleLength.clone().unwrap_or_default());
                                                        editing_needle_gauge.set(needleGauge.clone().unwrap_or_default());
                                                    }
                                                    DosageHistoryEntry::OralEstradiol { pillQuantity, .. } => {
                                                        editing_pill_qty.set(pillQuantity.map(|v| v.to_string()).unwrap_or_default());
                                                    }
                                                    DosageHistoryEntry::Progesterone { pillQuantity, route, .. } => {
                                                        editing_pill_qty.set(pillQuantity.map(|v| v.to_string()).unwrap_or_default());
                                                        editing_route.set(progesterone_route_label(&route).to_string());
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        };
                                        let (summary, details, meta) = match &entry {
                                            DosageHistoryEntry::InjectableEstradiol { kind, dose, unit, bonusDose, injectionSite, vialId, subVialId, syringeKind, needleLength, needleGauge, note, .. } => {
                                                let use_iu = store.settings.get().displayInjectableInIU.unwrap_or(false);
                                                let data_value = store.data.get();
                                                let schedule_vial_id = data_value
                                                    .injectableEstradiol
                                                    .as_ref()
                                                    .and_then(|cfg| cfg.vialId.as_ref());
                                                let dose_label = format_injectable_dose(
                                                    &data_value,
                                                    *dose,
                                                    unit,
                                                    vialId.as_ref(),
                                                    schedule_vial_id,
                                                    use_iu,
                                                );
                                                let summary = format!("Injection · {:?} · {dose_label}", kind);
                                                let mut details = Vec::new();
                                                if bonusDose.unwrap_or(false) {
                                                    details.push("Bonus dose".to_string());
                                                }
                                                if let Some(site) = injectionSite {
                                                    details.push(format!("Site: {}", injection_site_label(site)));
                                                }
                                                let mut meta = Vec::new();
                                                if let Some(vial_id) = vialId {
                                                    if let Some(vial) = store
                                                        .data
                                                        .get()
                                                        .vials
                                                        .iter()
                                                        .find(|v| v.id == *vial_id)
                                                    {
                                                        let mut label = format!(
                                                            "Vial: {}",
                                                            vial.esterKind.clone().unwrap_or_else(|| "—".to_string())
                                                        );
                                                        if let Some(batch) = &vial.batchNumber {
                                                            label.push_str(&format!(" · {}", batch));
                                                        }
                                                        if let Some(source) = &vial.source {
                                                            label.push_str(&format!(" · {}", source));
                                                        }
                                                        if let Some(sub_id) = subVialId {
                                                            if let Some(sub) = vial.subVials.iter().find(|s| s.id == *sub_id) {
                                                                label.push_str(&format!(" · sub-vial #{}", sub.personalNumber));
                                                            }
                                                        }
                                                        meta.push(label);
                                                    }
                                                }
                                                if syringeKind.is_some() || needleLength.is_some() {
                                                    meta.push(format!(
                                                        "Syringe: {}{}",
                                                        syringeKind.clone().unwrap_or_else(|| "—".to_string()),
                                                        needleLength
                                                            .as_ref()
                                                            .map(|len| format!(" · Needle: {len}"))
                                                            .unwrap_or_default()
                                                    ));
                                                }
                                                if let Some(gauge) = needleGauge {
                                                    meta.push(format!("Gauge: {gauge}"));
                                                }
                                                if let Some(note) = note {
                                                    details.push(format!("Note: {note}"));
                                                }
                                                (summary, details, meta)
                                            }
                                            DosageHistoryEntry::OralEstradiol { kind, dose, unit, pillQuantity, note, .. } => {
                                                let qty = pillQuantity.unwrap_or(1.0);
                                                let total = if *unit == HormoneUnits::Mg { dose * qty } else { *dose };
                                                let summary = format!("Oral E · {:?}", kind);
                                                let details = vec![format!("{dose:.2} {:?}/pill × {qty:.1} = {total:.2} mg", unit)];
                                                let mut meta = Vec::new();
                                                if let Some(note) = note {
                                                    meta.push(format!("Note: {note}"));
                                                }
                                                (summary, details, meta)
                                            }
                                            DosageHistoryEntry::Progesterone { kind, route, dose, unit, pillQuantity, note, .. } => {
                                                let qty = pillQuantity.unwrap_or(1.0);
                                                let total = if *unit == HormoneUnits::Mg { dose * qty } else { *dose };
                                                let summary = format!("Progesterone ({:?}) · {:?}", route, kind);
                                                let details = vec![format!("{dose:.2} {:?}/pill × {qty:.1} = {total:.2} mg", unit)];
                                                let mut meta = Vec::new();
                                                if let Some(note) = note {
                                                    meta.push(format!("Note: {note}"));
                                                }
                                                (summary, details, meta)
                                            }
                                            DosageHistoryEntry::Antiandrogen { kind, dose, unit, note, .. } => {
                                                let summary = format!("AA · {:?}", kind);
                                                let details = vec![format!("{dose:.2} {:?}", unit)];
                                                let mut meta = Vec::new();
                                                if let Some(note) = note {
                                                    meta.push(format!("Note: {note}"));
                                                }
                                                (summary, details, meta)
                                            }
                                        };
                                        let detail_lines = StoredValue::new(details.clone());
                                        let meta_lines = StoredValue::new(meta.clone());
                                        view! {
                                            <li class="history-item">
                                                <div>
                                                    <div class="history-date">{date_label}</div>
                                                    <div class="history-meta">
                                                        <span>{summary}</span>
                                                        {detail_lines
                                                            .get_value()
                                                            .into_iter()
                                                            .map(|line| view! { <span>{line}</span> })
                                                            .collect_view()}
                                                    </div>
                                                    <Show when=move || !meta_lines.get_value().is_empty()>
                                                        <div class="history-submeta">
                                                            {meta_lines
                                                                .get_value()
                                                                .into_iter()
                                                                .map(|line| view! { <div>{line}</div> })
                                                                .collect_view()}
                                                        </div>
                                                    </Show>
                                                </div>
                                                <div class="history-actions">
                                                    <button type="button" class="action-button" on:click=on_edit>
                                                        "Edit"
                                                    </button>
                                                    <button type="button" class="action-button" on:click=on_delete>
                                                        "Delete"
                                                    </button>
                                                </div>
                                            </li>
                                        }
                                    }
                                />
                            </ul>
                        </Show>
                    </div>
                </div>
            </div>

            <Show when=move || editing_key.get().is_some()>
                <div class="modal-backdrop" on:click=move |_| editing_key.set(None)>
                    <div class="modal" on:click=move |ev| ev.stop_propagation()>
                        <h3>"Edit Dosage Entry"</h3>
                        <Show when=move || !editing_med_label().is_empty()>
                            <p>
                                <strong>"Medication: "</strong>
                                {move || editing_med_label()}
                            </p>
                        </Show>
                        <label>"Date / Time"</label>
                        <input
                            type="datetime-local"
                            on:input=move |ev| editing_date.set(event_target_value(&ev))
                            prop:value=move || editing_date.get()
                        />
                        <div class="inline-equal">
                            <label>
                                {move || {
                                    if editing_med_type.get() != "injectableEstradiol" {
                                        "Dose"
                                    } else if editing_dose_in_iu.get() {
                                        "Dose (IU)"
                                    } else {
                                        "Dose (mg)"
                                    }
                                }}
                                <input
                                    type="text"
                                    step="any"
                                    on:input=move |ev| editing_dose.set(event_target_value(&ev))
                                    prop:value=move || editing_dose.get()
                                />
                            </label>
                            <Show when=move || editing_med_type.get() != "injectableEstradiol">
                                <label>
                                    "Unit"
                                    <select
                                        on:change=move |ev| {
                                            let value = event_target_value(&ev);
                                            if parse_hormone_unit(&value) != Some(HormoneUnits::Mg) {
                                                editing_dose_in_iu.set(false);
                                            }
                                            editing_unit.set(value);
                                        }
                                        prop:value=move || editing_unit.get()
                                    >
                                        {hormone_unit_labels()
                                            .into_iter()
                                            .map(|label| {
                                                let display = label.clone();
                                                view! { <option value=display.clone()>{display}</option> }
                                            })
                                            .collect_view()}
                                    </select>
                                </label>
                            </Show>
                        </div>
                        <Show
                            when=move || {
                                editing_med_type.get() == "injectableEstradiol" && editing_dose_in_iu.get()
                            }
                        >
                            <p class="muted">
                                "Stored as "
                                <strong>
                                    {move || {
                                        editing_dose_as_mg
                                            .get()
                                            .map(|dose| format!("{} mg", fmt_decimal(dose, 3)))
                                            .unwrap_or_else(|| "—".to_string())
                                    }}
                                </strong>
                            </p>
                        </Show>
                        <Show
                            when=move || {
                                editing_med_type.get() == "injectableEstradiol"
                                    && store.settings.get().displayInjectableInIU.unwrap_or(false)
                                    && !editing_dose_in_iu.get()
                            }
                        >
                            <p class="muted">
                                "Select a vial with concentration to edit this dose in IU."
                            </p>
                        </Show>
                        <Show when=is_prog>
                            <label>
                                "Route"
                                <select
                                    on:change=move |ev| editing_route.set(event_target_value(&ev))
                                    prop:value=move || editing_route.get()
                                >
                                    {[ProgesteroneRoutes::Oral, ProgesteroneRoutes::Boofed]
                                        .iter()
                                        .map(|route| {
                                            let label = progesterone_route_label(route);
                                            view! { <option value=label>{label}</option> }
                                        })
                                        .collect_view()}
                                </select>
                            </label>
                        </Show>
                        <Show when=is_pill>
                            <label>
                                "Pill quantity"
                                <input
                                    type="text"
                                    min="1"
                                    step="any"
                                    on:input=move |ev| editing_pill_qty.set(event_target_value(&ev))
                                    prop:value=move || editing_pill_qty.get()
                                />
                            </label>
                            <p class="muted">
                                {move || {
                                    let dose = parse_optional_num(&editing_dose.get()).unwrap_or(0.0);
                                    let qty = parse_optional_num(&editing_pill_qty.get()).unwrap_or(0.0);
                                    format!("Total = {dose:.2} mg/pill × {qty:.1} = {:.2} mg", pill_total())
                                }}
                            </p>
                        </Show>
                        <label>"Note (optional)"</label>
                        <textarea
                            rows="3"
                            on:input=move |ev| editing_note.set(event_target_value(&ev))
                            prop:value=move || editing_note.get()
                        ></textarea>
                        <Show when=is_injectable>
                            <label class="toggle toggle-wide">
                                <input
                                    type="checkbox"
                                    on:change=move |ev| editing_bonus.set(event_target_checked(&ev))
                                    prop:checked=move || editing_bonus.get()
                                />
                                <span class="toggle-track" aria-hidden="true"></span>
                                <span class="toggle-label">"Bonus dose (doesn't move schedule)"</span>
                            </label>
                            <label>"Injection site (optional)"</label>
                            <select
                                on:change=move |ev| editing_injection_site.set(event_target_value(&ev))
                                prop:value=move || editing_injection_site.get()
                            >
                                <option value="">"Select injection site"</option>
                                {INJECTION_SITE_OPTIONS
                                    .iter()
                                    .map(|site| {
                                        let label = injection_site_label(site);
                                        view! { <option value=label>{label}</option> }
                                    })
                                    .collect_view()}
                            </select>
                            <label>"Vial (optional)"</label>
                            <div class="inline">
                                <select
                                    on:change=move |ev| editing_vial_id.set(event_target_value(&ev))
                                    prop:value=move || editing_vial_id.get()
                                >
                                    <option value="">"None"</option>
                                    {move || {
                                        let selected = editing_vial_id.get();
                                        store
                                            .data
                                            .get()
                                            .vials
                                            .iter()
                                            .filter(|vial| !vial.isSpent.unwrap_or(false) || vial.id == selected)
                                            .map(|vial| {
                                                let mut label = vial
                                                    .esterKind
                                                    .clone()
                                                    .unwrap_or_else(|| "Unknown ester".to_string());
                                                if let Some(batch) = &vial.batchNumber {
                                                    label.push_str(&format!(" · {batch}"));
                                                }
                                                if let Some(source) = &vial.source {
                                                    label.push_str(&format!(" · {source}"));
                                                }
                                                let id = vial.id.clone();
                                                view! { <option value=id>{label}</option> }
                                            })
                                            .collect_view()
                                    }}
                                </select>
                                <A class="pill-button" href="/vials/create">"New..."</A>
                            </div>
                            <Show when=move || {
                                let selected = editing_vial_id.get();
                                if selected.trim().is_empty() {
                                    return false;
                                }
                                store
                                    .data
                                    .get()
                                    .vials
                                    .iter()
                                    .find(|vial| vial.id == selected)
                                    .map(|vial| !vial.subVials.is_empty())
                                    .unwrap_or(false)
                            }>
                                <label>"Sub-vial / cartridge (optional)"</label>
                                <select
                                    on:change=move |ev| editing_sub_vial_id.set(event_target_value(&ev))
                                    prop:value=move || editing_sub_vial_id.get()
                                >
                                    <option value="">"None"</option>
                                    {move || {
                                        let selected = editing_vial_id.get();
                                        store
                                            .data
                                            .get()
                                            .vials
                                            .iter()
                                            .find(|vial| vial.id == selected)
                                            .map(|vial| {
                                                vial.subVials
                                                    .iter()
                                                    .map(|sub| {
                                                        let label = format!("#{}", sub.personalNumber);
                                                        let id = sub.id.clone();
                                                        view! { <option value=id>{label}</option> }
                                                    })
                                                    .collect_view()
                                            })
                                            .unwrap_or_default()
                                    }}
                                </select>
                            </Show>
                            <label>"Syringe kind (optional)"</label>
                            <select
                                on:change=move |ev| editing_syringe_kind.set(event_target_value(&ev))
                                prop:value=move || editing_syringe_kind.get()
                            >
                                <option value="">"None"</option>
                                {SYRINGE_KIND_OPTIONS
                                    .iter()
                                    .map(|kind| {
                                        let label = syringe_kind_label(kind);
                                        view! { <option value=label>{label}</option> }
                                    })
                                    .collect_view()}
                            </select>
                            <label>"Needle length (optional)"</label>
                            <input
                                type="text"
                                placeholder="e.g., 4mm or 1\""
                                on:input=move |ev| editing_needle_length.set(event_target_value(&ev))
                                prop:value=move || editing_needle_length.get()
                            />
                            <label>"Needle gauge (optional)"</label>
                            <input
                                type="text"
                                placeholder="e.g., 32g or 30G"
                                on:input=move |ev| editing_needle_gauge.set(event_target_value(&ev))
                                prop:value=move || editing_needle_gauge.get()
                            />
                            <label>"Photos (optional)"</label>
                            <Show when=move || !editing_photos.get().is_empty()>
                                <div class="photo-grid">
                                    <For
                                        each=move || editing_photos.get()
                                        key=|photo| photo.file.clone()
                                        children=move |photo| {
                                            let filename = photo.file.clone();
                                            let note = photo.note.clone();
                                            let on_delete = {
                                                let filename = filename.clone();
                                                let on_photo_delete = on_photo_delete.get_value();
                                                move |_| on_photo_delete(filename.clone())
                                            };
                                            let on_note = {
                                                let filename = filename.clone();
                                                let on_photo_note_update = on_photo_note_update.get_value();
                                                move |ev| {
                                                    on_photo_note_update(filename.clone(), event_target_value(&ev))
                                                }
                                            };
                                            view! {
                                                <div class="photo-card">
                                                    <img
                                                        src=move || {
                                                            let entry_id = editing_entry_id.get();
                                                            format!("/api/dosage-photo/{entry_id}/{filename}")
                                                        }
                                                        alt="injection site"
                                                    />
                                                    <button type="button" class="photo-delete" on:click=on_delete>
                                                        "Delete"
                                                    </button>
                                                    <input
                                                        type="text"
                                                        placeholder="Add a note..."
                                                        on:input=on_note
                                                        prop:value=note
                                                    />
                                                </div>
                                            }
                                        }
                                    />
                                </div>
                            </Show>
                            <div class="photo-actions">
                                <input
                                    type="file"
                                    accept="image/*"
                                    multiple
                                    node_ref=photo_input_ref
                                    on:change=move |ev| {
                                        let on_photo_change = on_photo_change.get_value();
                                        on_photo_change(ev);
                                    }
                                    prop:disabled=move || upload_busy.get()
                                    class="hidden-input"
                                />
                                <button
                                    type="button"
                                    on:click=open_photo_picker
                                    prop:disabled=move || {
                                        upload_busy.get()
                                            || editing_entry_id.get().trim().is_empty()
                                    }
                                >
                                    {move || if upload_busy.get() { "Uploading..." } else { "Add Photos" }}
                                </button>
                                <span class="muted">"JPEG/PNG/WEBP/HEIC · Multiple files allowed"</span>
                            </div>
                        </Show>
                        <div class="modal-actions">
                            <button type="button" on:click={
                                let on_save_edit = on_save_edit.clone();
                                move |_| (on_save_edit)()
                            }>
                                "Save"
                            </button>
                            <button type="button" on:click=on_cancel_edit>"Cancel"</button>
                        </div>
                    </div>
                </div>
            </Show>

            <Show when=move || confirm_delete.get().is_some()>
                <div class="modal-backdrop modal-backdrop-top" on:click=move |_| confirm_delete.set(None)>
                    <div class="modal" on:click=move |ev| ev.stop_propagation()>
                        <h3>{move || confirm_title.get()}</h3>
                        <p>"This action cannot be undone."</p>
                        <div class="modal-actions">
                            <button type="button" on:click={
                                let confirm_action = confirm_action.clone();
                                let confirm_delete = confirm_delete;
                                move |_| {
                                    if let Some(action) = confirm_action.get() {
                                        action();
                                    }
                                    confirm_delete.set(None);
                                }
                            }>
                                "Delete"
                            </button>
                            <button type="button" on:click={
                                let confirm_delete = confirm_delete;
                                move |_| confirm_delete.set(None)
                            }>
                                "Cancel"
                            </button>
                        </div>
                    </div>
                </div>
            </Show>

            <Show when=move || edit_blood_date.get().is_some()>
                <div class="modal-backdrop" on:click=move |_| edit_blood_date.set(None)>
                    <div class="modal" on:click=move |ev| ev.stop_propagation()>
                        <h3>"Edit Blood Test"</h3>
                        <label>"Date / Time"</label>
                        <input
                            type="datetime-local"
                            on:input=move |ev| edit_blood_date_text.set(event_target_value(&ev))
                            prop:value=move || edit_blood_date_text.get()
                        />
                        <div class="inline-equal">
                            <label>
                                "Estradiol level"
                                <input
                                    type="text"
                                    step="any"
                                    on:input=move |ev| edit_blood_e2.set(event_target_value(&ev))
                                    prop:value=move || edit_blood_e2.get()
                                />
                            </label>
                            <label>
                                "Estradiol unit"
                                <select
                                    on:change=move |ev| edit_blood_e2_unit.set(event_target_value(&ev))
                                    prop:value=move || edit_blood_e2_unit.get()
                                >
                                    {hormone_unit_labels()
                                        .into_iter()
                                        .map(|label| {
                                            let display = label.clone();
                                            view! { <option value=display.clone()>{display}</option> }
                                        })
                                        .collect_view()}
                                </select>
                            </label>
                        </div>
                        <div class="inline-equal">
                            <label>
                                "Estrannaise predicted E2"
                                <input
                                    type="text"
                                    step="any"
                                    on:input=move |ev| edit_blood_estrannaise.set(event_target_value(&ev))
                                    prop:value=move || edit_blood_estrannaise.get()
                                />
                            </label>
                            <label>
                                "Predicted unit"
                                <select
                                    on:change=move |ev| edit_blood_estrannaise_unit.set(event_target_value(&ev))
                                    prop:value=move || edit_blood_estrannaise_unit.get()
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
                                    type="text"
                                    step="any"
                                    on:input=move |ev| edit_blood_t.set(event_target_value(&ev))
                                    prop:value=move || edit_blood_t.get()
                                />
                            </label>
                            <label>
                                "Testosterone unit"
                                <select
                                    on:change=move |ev| edit_blood_t_unit.set(event_target_value(&ev))
                                    prop:value=move || edit_blood_t_unit.get()
                                >
                                    {hormone_unit_labels()
                                        .into_iter()
                                        .map(|label| {
                                            let display = label.clone();
                                            view! { <option value=display.clone()>{display}</option> }
                                        })
                                        .collect_view()}
                                </select>
                            </label>
                        </div>
                        <div class="inline-equal">
                            <label>
                                "Progesterone level"
                                <input
                                    type="text"
                                    step="any"
                                    on:input=move |ev| edit_blood_prog.set(event_target_value(&ev))
                                    prop:value=move || edit_blood_prog.get()
                                />
                            </label>
                            <label>
                                "Progesterone unit"
                                <select
                                    on:change=move |ev| edit_blood_prog_unit.set(event_target_value(&ev))
                                    prop:value=move || edit_blood_prog_unit.get()
                                >
                                    {hormone_unit_labels()
                                        .into_iter()
                                        .map(|label| {
                                            let display = label.clone();
                                            view! { <option value=display.clone()>{display}</option> }
                                        })
                                        .collect_view()}
                                </select>
                            </label>
                        </div>
                        <div class="inline-equal">
                            <label>
                                "FSH level"
                                <input
                                    type="text"
                                    step="any"
                                    on:input=move |ev| edit_blood_fsh.set(event_target_value(&ev))
                                    prop:value=move || edit_blood_fsh.get()
                                />
                            </label>
                            <label>
                                "FSH unit"
                                <select
                                    on:change=move |ev| edit_blood_fsh_unit.set(event_target_value(&ev))
                                    prop:value=move || edit_blood_fsh_unit.get()
                                >
                                    {hormone_unit_labels()
                                        .into_iter()
                                        .map(|label| {
                                            let display = label.clone();
                                            view! { <option value=display.clone()>{display}</option> }
                                        })
                                        .collect_view()}
                                </select>
                            </label>
                        </div>
                        <div class="inline-equal">
                            <label>
                                "LH level"
                                <input
                                    type="text"
                                    step="any"
                                    on:input=move |ev| edit_blood_lh.set(event_target_value(&ev))
                                    prop:value=move || edit_blood_lh.get()
                                />
                            </label>
                            <label>
                                "LH unit"
                                <select
                                    on:change=move |ev| edit_blood_lh_unit.set(event_target_value(&ev))
                                    prop:value=move || edit_blood_lh_unit.get()
                                >
                                    {hormone_unit_labels()
                                        .into_iter()
                                        .map(|label| {
                                            let display = label.clone();
                                            view! { <option value=display.clone()>{display}</option> }
                                        })
                                        .collect_view()}
                                </select>
                            </label>
                        </div>
                        <div class="inline-equal">
                            <label>
                                "Prolactin level"
                                <input
                                    type="text"
                                    step="any"
                                    on:input=move |ev| edit_blood_prolactin.set(event_target_value(&ev))
                                    prop:value=move || edit_blood_prolactin.get()
                                />
                            </label>
                            <label>
                                "Prolactin unit"
                                <select
                                    on:change=move |ev| edit_blood_prolactin_unit.set(event_target_value(&ev))
                                    prop:value=move || edit_blood_prolactin_unit.get()
                                >
                                    {hormone_unit_labels()
                                        .into_iter()
                                        .map(|label| {
                                            let display = label.clone();
                                            view! { <option value=display.clone()>{display}</option> }
                                        })
                                        .collect_view()}
                                </select>
                            </label>
                        </div>
                        <div class="inline-equal">
                            <label>
                                "SHBG level"
                                <input
                                    type="text"
                                    step="any"
                                    on:input=move |ev| edit_blood_shbg.set(event_target_value(&ev))
                                    prop:value=move || edit_blood_shbg.get()
                                />
                            </label>
                            <label>
                                "SHBG unit"
                                <select
                                    on:change=move |ev| edit_blood_shbg_unit.set(event_target_value(&ev))
                                    prop:value=move || edit_blood_shbg_unit.get()
                                >
                                    {hormone_unit_labels()
                                        .into_iter()
                                        .map(|label| {
                                            let display = label.clone();
                                            view! { <option value=display.clone()>{display}</option> }
                                        })
                                        .collect_view()}
                                </select>
                            </label>
                        </div>
                        <div class="inline-equal">
                            <label>
                                "Free Androgen Index"
                                <input
                                    type="text"
                                    step="any"
                                    on:input=move |ev| edit_blood_fai.set(event_target_value(&ev))
                                    prop:value=move || edit_blood_fai.get()
                                />
                            </label>
                            <div></div>
                        </div>
                        <label>"Notes"</label>
                        <textarea
                            rows="3"
                            on:input=move |ev| edit_blood_notes.set(event_target_value(&ev))
                            prop:value=move || edit_blood_notes.get()
                        ></textarea>
                        <Show when=move || !edit_blood_pdf_files.get().is_empty()>
                            <label>"Attached report PDFs"</label>
                            <ul class="history-list">
                                <For
                                    each=move || edit_blood_pdf_files.get()
                                    key=|filename| filename.clone()
                                    children=move |filename| {
                                        let open_href = bloodtest_pdf_url(&filename);
                                        let remove_name = filename.clone();
                                        view! {
                                            <li class="history-item">
                                                <a href=open_href target="_blank" rel="noopener noreferrer">
                                                    {filename}
                                                </a>
                                                <button
                                                    type="button"
                                                    class="action-button"
                                                    on:click=move |_| {
                                                        edit_blood_pdf_files.update(|files| {
                                                            files.retain(|item| item != &remove_name);
                                                        });
                                                    }
                                                >
                                                    "Remove"
                                                </button>
                                            </li>
                                        }
                                    }
                                />
                            </ul>
                        </Show>
                        <div class="modal-actions">
                            <button type="button" on:click={
                                let store = store_blood_modal.clone();
                                let confirm_delete = confirm_delete;
                                let confirm_title = confirm_title;
                                let confirm_action = confirm_action;
                                let edit_blood_date = edit_blood_date;
                                move |_: leptos::ev::MouseEvent| {
                                    let date = match edit_blood_date.get() {
                                        Some(value) => value,
                                        None => return,
                                    };
                                    let files_to_delete = store
                                        .data
                                        .get()
                                        .bloodTests
                                        .iter()
                                        .find(|entry| entry.date == date)
                                        .and_then(|entry| entry.pdfFiles.clone())
                                        .unwrap_or_default();
                                    confirm_title.set("Delete blood test?".to_string());
                                    confirm_delete.set(Some(format!("blood-{date}")));
                                    let store = store.clone();
                                    confirm_action.set(Some(Rc::new(move || {
                                        store.data.update(|d| {
                                            d.bloodTests.retain(|entry| entry.date != date);
                                        });
                                        if !files_to_delete.is_empty() {
                                            let files_to_delete = files_to_delete.clone();
                                            spawn_local(async move {
                                                for filename in files_to_delete {
                                                    let _ = Request::delete(&bloodtest_pdf_url(&filename))
                                                        .send()
                                                        .await;
                                                }
                                            });
                                        }
                                        store.mark_dirty();
                                        store.save();
                                        edit_blood_date.set(None);
                                    })));
                                }
                            }>
                                "Delete"
                            </button>
                            <button type="button" on:click={
                                let store = store_blood_modal.clone();
                                move |_: leptos::ev::MouseEvent| {
                                    let date = match edit_blood_date.get() {
                                        Some(value) => value,
                                        None => return,
                                    };
                                    let new_date = parse_datetime_local(&edit_blood_date_text.get());
                                    let snapped_date = snap_to_next_injection_boundary(&store.data.get(), new_date);
                                    let e2_value = parse_optional_num(&edit_blood_e2.get());
                                    let estrannaise_value =
                                        parse_optional_num(&edit_blood_estrannaise.get());
                                    let t_value = parse_optional_num(&edit_blood_t.get());
                                    let prog_value = parse_optional_num(&edit_blood_prog.get());
                                    let fsh_value = parse_optional_num(&edit_blood_fsh.get());
                                    let lh_value = parse_optional_num(&edit_blood_lh.get());
                                    let prolactin_value = parse_optional_num(&edit_blood_prolactin.get());
                                    let shbg_value = parse_optional_num(&edit_blood_shbg.get());
                                    let fai_value = parse_optional_num(&edit_blood_fai.get());
                                    let e2_default = store
                                        .settings
                                        .get()
                                        .displayEstradiolUnit
                                        .unwrap_or(HormoneUnits::E2PmolL);
                                    let e2_unit =
                                        parse_hormone_unit(&edit_blood_e2_unit.get())
                                            .unwrap_or(e2_default);
                                    let estrannaise_unit =
                                        parse_hormone_unit(&edit_blood_estrannaise_unit.get())
                                            .unwrap_or(HormoneUnits::E2PgMl);
                                    let t_unit = parse_hormone_unit(&edit_blood_t_unit.get())
                                        .unwrap_or(HormoneUnits::TNmolL);
                                    let prog_unit =
                                        parse_hormone_unit(&edit_blood_prog_unit.get())
                                            .unwrap_or(HormoneUnits::TNmolL);
                                    let fsh_unit =
                                        parse_hormone_unit(&edit_blood_fsh_unit.get())
                                            .unwrap_or(HormoneUnits::UL);
                                    let lh_unit =
                                        parse_hormone_unit(&edit_blood_lh_unit.get())
                                            .unwrap_or(HormoneUnits::UL);
                                    let prolactin_unit =
                                        parse_hormone_unit(&edit_blood_prolactin_unit.get())
                                            .unwrap_or(HormoneUnits::MIuL);
                                    let shbg_unit =
                                        parse_hormone_unit(&edit_blood_shbg_unit.get())
                                            .unwrap_or(HormoneUnits::TNmolL);
                                    let notes = edit_blood_notes.get();
                                    let updated_pdf_files = edit_blood_pdf_files.get();
                                    let measured_e2 = e2_value.map(|value| {
                                        if e2_unit == HormoneUnits::E2PmolL {
                                            value / 3.671
                                        } else {
                                            value
                                        }
                                    });
                                    let predicted_input = estrannaise_value.map(|value| {
                                        if estrannaise_unit == HormoneUnits::E2PmolL {
                                            value / 3.671
                                        } else {
                                            value
                                        }
                                    });
                                    let predicted_model = predict_e2_pg_ml(&store.data.get(), snapped_date);
                                    let fudge_factor =
                                        compute_fudge_factor(measured_e2, predicted_model.or(predicted_input));
                                    let mut removed_pdf_files = Vec::new();
                                    store.data.update(|d| {
                                        for entry in &mut d.bloodTests {
                                            if entry.date == date {
                                                let previous_pdf_files =
                                                    entry.pdfFiles.clone().unwrap_or_default();
                                                removed_pdf_files = previous_pdf_files
                                                    .iter()
                                                    .filter(|name| {
                                                        !updated_pdf_files
                                                            .iter()
                                                            .any(|candidate| candidate == *name)
                                                    })
                                                    .cloned()
                                                    .collect();
                                                entry.date = snapped_date;
                                                entry.estradiolLevel = e2_value;
                                                entry.testLevel = t_value;
                                                entry.estradiolUnit = Some(e2_unit.clone());
                                                entry.testUnit = Some(t_unit.clone());
                                                entry.progesteroneLevel = prog_value;
                                                entry.progesteroneUnit = Some(prog_unit.clone());
                                                entry.fshLevel = fsh_value;
                                                entry.fshUnit = Some(fsh_unit.clone());
                                                entry.lhLevel = lh_value;
                                                entry.lhUnit = Some(lh_unit.clone());
                                                entry.prolactinLevel = prolactin_value;
                                                entry.prolactinUnit = Some(prolactin_unit.clone());
                                                entry.shbgLevel = shbg_value;
                                                entry.shbgUnit = Some(shbg_unit.clone());
                                                entry.freeAndrogenIndex = fai_value;
                                                entry.estrannaiseNumber = predicted_input;
                                                entry.fudgeFactor = fudge_factor;
                                                entry.notes = if notes.trim().is_empty() {
                                                    None
                                                } else {
                                                    Some(notes.clone())
                                                };
                                                entry.pdfFiles = if updated_pdf_files.is_empty() {
                                                    None
                                                } else {
                                                    Some(updated_pdf_files.clone())
                                                };
                                            }
                                        }
                                    });
                                    if !removed_pdf_files.is_empty() {
                                        spawn_local(async move {
                                            for filename in removed_pdf_files {
                                                let _ = Request::delete(&bloodtest_pdf_url(&filename))
                                                    .send()
                                                    .await;
                                            }
                                        });
                                    }
                                    store.mark_dirty();
                                    store.save();
                                    edit_blood_date.set(None);
                                }
                            }>
                                "Save"
                            </button>
                            <button type="button" on:click={
                                let edit_blood_date = edit_blood_date;
                                move |_: leptos::ev::MouseEvent| edit_blood_date.set(None)
                            }>
                                "Cancel"
                            </button>
                        </div>
                    </div>
                </div>
            </Show>

            <Show when=move || edit_measurement_id.get().is_some() || edit_measurement_date.get().is_some()>
                <div class="modal-backdrop" on:click=move |_| {
                    edit_measurement_id.set(None);
                    edit_measurement_date.set(None);
                }>
                    <div class="modal" on:click=move |ev| ev.stop_propagation()>
                        <h3>"Edit Measurement"</h3>
                        <label>"Date / Time"</label>
                        <input
                            type="datetime-local"
                            on:input=move |ev| edit_measurement_date_text.set(event_target_value(&ev))
                            prop:value=move || edit_measurement_date_text.get()
                        />
                        <div class="inline-equal">
                            <label>
                                "Weight"
                                <input
                                    type="text"
                                    step="any"
                                    on:input=move |ev| edit_measurement_weight.set(event_target_value(&ev))
                                    prop:value=move || edit_measurement_weight.get()
                                />
                            </label>
                            <label>
                                "Weight unit"
                                <select
                                    on:change=move |ev| {
                                        edit_measurement_weight_unit.set(event_target_value(&ev))
                                    }
                                    prop:value=move || edit_measurement_weight_unit.get()
                                >
                                    {["kg", "lbs"]
                                        .iter()
                                        .map(|label| {
                                            let display = (*label).to_string();
                                            view! { <option value=display.clone()>{display}</option> }
                                        })
                                        .collect_view()}
                                </select>
                            </label>
                        </div>
                        <div class="inline-equal">
                            <label>
                                "Height"
                                <input
                                    type="text"
                                    step="any"
                                    on:input=move |ev| edit_measurement_height.set(event_target_value(&ev))
                                    prop:value=move || edit_measurement_height.get()
                                />
                            </label>
                            <label>
                                "Height unit"
                                <select
                                    on:change=move |ev| {
                                        edit_measurement_height_unit.set(event_target_value(&ev))
                                    }
                                    prop:value=move || edit_measurement_height_unit.get()
                                >
                                    {["cm", "in"]
                                        .iter()
                                        .map(|label| {
                                            let display = (*label).to_string();
                                            view! { <option value=display.clone()>{display}</option> }
                                        })
                                        .collect_view()}
                                </select>
                            </label>
                        </div>
                        <label>"Body measurement unit"</label>
                        <select
                            on:change=move |ev| edit_measurement_unit.set(event_target_value(&ev))
                            prop:value=move || edit_measurement_unit.get()
                        >
                            {["cm", "in"]
                                .iter()
                                .map(|label| {
                                    let display = (*label).to_string();
                                    view! { <option value=display.clone()>{display}</option> }
                                })
                                .collect_view()}
                        </select>
                        <div class="measurement-grid">
                            <input
                                type="text"
                                step="any"
                                placeholder="Underbust"
                                on:input=move |ev| edit_measurement_underbust.set(event_target_value(&ev))
                                prop:value=move || edit_measurement_underbust.get()
                            />
                            <input
                                type="text"
                                step="any"
                                placeholder="Bust"
                                on:input=move |ev| edit_measurement_bust.set(event_target_value(&ev))
                                prop:value=move || edit_measurement_bust.get()
                            />
                            <input
                                type="text"
                                step="any"
                                placeholder="Bideltoid (shoulder)"
                                on:input=move |ev| edit_measurement_bideltoid.set(event_target_value(&ev))
                                prop:value=move || edit_measurement_bideltoid.get()
                            />
                            <input
                                type="text"
                                step="any"
                                placeholder="Waist"
                                on:input=move |ev| edit_measurement_waist.set(event_target_value(&ev))
                                prop:value=move || edit_measurement_waist.get()
                            />
                            <input
                                type="text"
                                step="any"
                                placeholder="Hip"
                                on:input=move |ev| edit_measurement_hip.set(event_target_value(&ev))
                                prop:value=move || edit_measurement_hip.get()
                            />
                        </div>
                        <label>
                            "Bra size"
                            <input
                                type="text"
                                on:input=move |ev| edit_measurement_bra_size.set(event_target_value(&ev))
                                prop:value=move || edit_measurement_bra_size.get()
                            />
                        </label>
                        <div class="modal-actions">
                            <button type="button" on:click={
                                let store = store_measure_modal.clone();
                                let confirm_delete = confirm_delete;
                                let confirm_title = confirm_title;
                                let confirm_action = confirm_action;
                                let edit_measurement_id = edit_measurement_id;
                                let edit_measurement_date = edit_measurement_date;
                                move |_: leptos::ev::MouseEvent| {
                                    let target_id = edit_measurement_id.get();
                                    let target_date = edit_measurement_date.get();
                                    if target_id.is_none() && target_date.is_none() {
                                        return;
                                    }
                                    confirm_title.set("Delete measurement?".to_string());
                                    confirm_delete.set(Some(
                                        target_id
                                            .clone()
                                            .unwrap_or_else(|| {
                                                format!("measurement-{}", target_date.unwrap_or_default())
                                            }),
                                    ));
                                    let store = store.clone();
                                    confirm_action.set(Some(Rc::new(move || {
                                        store.data.update(|d| {
                                            d.measurements.retain(|entry| {
                                                !measurement_matches_target(
                                                    entry,
                                                    target_id.as_deref(),
                                                    target_date,
                                                )
                                            });
                                        });
                                        store.mark_dirty();
                                        store.save();
                                        edit_measurement_id.set(None);
                                        edit_measurement_date.set(None);
                                    })));
                                }
                            }>
                                "Delete"
                            </button>
                            <button type="button" on:click={
                                let store = store_measure_modal.clone();
                                let edit_measurement_id = edit_measurement_id;
                                move |_: leptos::ev::MouseEvent| {
                                    let target_id = edit_measurement_id.get();
                                    let target_date = edit_measurement_date.get();
                                    if target_id.is_none() && target_date.is_none() {
                                        return;
                                    }
                                    let new_date = parse_datetime_local(&edit_measurement_date_text.get());
                                    let weight = parse_optional_num(&edit_measurement_weight.get());
                                    let height = parse_optional_num(&edit_measurement_height.get());
                                    let underbust = parse_optional_num(&edit_measurement_underbust.get());
                                    let bust = parse_optional_num(&edit_measurement_bust.get());
                                    let bideltoid = parse_optional_num(&edit_measurement_bideltoid.get());
                                    let waist = parse_optional_num(&edit_measurement_waist.get());
                                    let hip = parse_optional_num(&edit_measurement_hip.get());
                                    let unit = edit_measurement_unit.get();
                                    let weight_unit = edit_measurement_weight_unit.get();
                                    let height_unit = edit_measurement_height_unit.get();
                                    let bra_size = edit_measurement_bra_size.get();
                                    store.data.update(|d| {
                                        for entry in &mut d.measurements {
                                            if measurement_matches_target(
                                                entry,
                                                target_id.as_deref(),
                                                target_date,
                                            ) {
                                                entry.date = new_date;
                                                entry.weight = weight;
                                                entry.weightUnit = parse_weight_unit(&weight_unit);
                                                entry.height = height;
                                                entry.heightUnit = parse_length_unit(&height_unit);
                                                entry.underbust = underbust;
                                                entry.bust = bust;
                                                entry.bideltoid = bideltoid;
                                                entry.waist = waist;
                                                entry.hip = hip;
                                                entry.bodyMeasurementUnit = parse_length_unit(&unit);
                                                entry.braSize = if bra_size.trim().is_empty() {
                                                    None
                                                } else {
                                                    Some(bra_size.clone())
                                                };
                                            }
                                        }
                                    });
                                    store.mark_dirty();
                                    store.save();
                                    edit_measurement_id.set(None);
                                    edit_measurement_date.set(None);
                                }
                            }>
                                "Save"
                            </button>
                            <button type="button" on:click={
                                let edit_measurement_id = edit_measurement_id;
                                let edit_measurement_date = edit_measurement_date;
                                move |_: leptos::ev::MouseEvent| {
                                    edit_measurement_id.set(None);
                                    edit_measurement_date.set(None);
                                }
                            }>
                                "Cancel"
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
        }
        .into_view(),
    )
}
