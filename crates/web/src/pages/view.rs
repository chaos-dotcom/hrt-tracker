use chrono::{Local, TimeZone};
use gloo_events::EventListener;
use leptos::*;
use leptos::window;
use leptos_router::A;
use std::cell::RefCell;
use std::rc::Rc;
use web_sys::HtmlCanvasElement;

use crate::charts::{
    chart_padding, clamp_zoom, compute_chart_bounds, ChartTooltip, DragState, ViewZoom,
};
use crate::charts::view::{compute_view_chart_state, draw_view_chart, find_nearest_point};
use crate::layout::page_layout;
use crate::store::use_store;
use crate::utils::{parse_date_or_now, parse_hormone_unit, parse_length_unit};
use hrt_shared::types::DosageHistoryEntry;

#[component]
pub fn ViewPage() -> impl IntoView {
    let store = use_store();
    let data = store.data;
    let rows = move || data.get().dosageHistory.clone();
    let blood_rows = move || data.get().bloodTests.clone();
    let measurement_rows = move || data.get().measurements.clone();
    let editing_key = create_rw_signal(None::<String>);
    let editing_date = create_rw_signal(String::new());
    let editing_dose = create_rw_signal(String::new());
    let editing_note = create_rw_signal(String::new());
    let confirm_delete = create_rw_signal(None::<String>);
    let confirm_title = create_rw_signal(String::new());
    let confirm_action = create_rw_signal(None::<Rc<dyn Fn()>>);

    let edit_blood_date = create_rw_signal(None::<i64>);
    let edit_blood_date_text = create_rw_signal(String::new());
    let edit_blood_e2 = create_rw_signal(String::new());
    let edit_blood_e2_unit = create_rw_signal(String::new());
    let edit_blood_t = create_rw_signal(String::new());
    let edit_blood_t_unit = create_rw_signal(String::new());
    let edit_blood_notes = create_rw_signal(String::new());

    let edit_measurement_date = create_rw_signal(None::<i64>);
    let edit_measurement_date_text = create_rw_signal(String::new());
    let edit_measurement_weight = create_rw_signal(String::new());
    let edit_measurement_waist = create_rw_signal(String::new());
    let edit_measurement_hip = create_rw_signal(String::new());
    let edit_measurement_unit = create_rw_signal(String::new());

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

    let entry_matches = |entry: &DosageHistoryEntry, key: &str| match entry {
        DosageHistoryEntry::InjectableEstradiol { date, id, .. }
        | DosageHistoryEntry::OralEstradiol { date, id, .. }
        | DosageHistoryEntry::Antiandrogen { date, id, .. }
        | DosageHistoryEntry::Progesterone { date, id, .. } => id
            .as_ref()
            .map(|v| v == key)
            .unwrap_or_else(|| date.to_string() == key),
    };

    let on_save_edit = Rc::new({
        let store_edit = store.clone();
        move || {
            let key = match editing_key.get() {
                Some(value) => value,
                None => return,
            };
            let dose_value = match editing_dose.get().trim().parse::<f64>().ok() {
                Some(value) => value,
                None => return,
            };
            let date_value = parse_date_or_now(&editing_date.get());
            let note_text = editing_note.get();
            let note_value = if note_text.trim().is_empty() {
                None
            } else {
                Some(note_text.clone())
            };

            store_edit.data.update(|d| {
                for entry in &mut d.dosageHistory {
                    if entry_matches(entry, &key) {
                        match entry {
                            DosageHistoryEntry::InjectableEstradiol {
                                date, dose, note, ..
                            }
                            | DosageHistoryEntry::OralEstradiol {
                                date, dose, note, ..
                            }
                            | DosageHistoryEntry::Antiandrogen {
                                date, dose, note, ..
                            }
                            | DosageHistoryEntry::Progesterone {
                                date, dose, note, ..
                            } => {
                                *date = date_value;
                                *dose = dose_value;
                                *note = note_value.clone();
                            }
                        }
                    }
                }
            });
            store_edit.mark_dirty();
            editing_key.set(None);
        }
    });

    let on_cancel_edit = move |_| editing_key.set(None);

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
                    <h3>"Current Regimen"</h3>
                    <div class="view-summary">
                        <Show when=move || view_chart_state.get().first_dose.is_some()>
                            <p>
                                <strong>"Days since first dose: "</strong>
                                {move || view_chart_state.get().first_dose.map(|first| {
                                    let now = js_sys::Date::now() as i64;
                                    let diff = (now - first) / (24 * 60 * 60 * 1000);
                                    diff.max(0)
                                }).unwrap_or(0)}
                            </p>
                        </Show>
                        <Show when=move || store.data.get().injectableEstradiol.is_some()>
                            <p>
                                <strong>"Injectable Estradiol: "</strong>
                                {move || store
                                    .data
                                    .get()
                                    .injectableEstradiol
                                    .as_ref()
                                    .map(|cfg| format!("{:?}, {:.2} {:?} every {:.1} days", cfg.kind, cfg.dose, cfg.unit, cfg.frequency))
                                    .unwrap_or_default()}
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

            <section>
                <h2>"Dosage History"</h2>
                <Show
                    when=move || !rows().is_empty()
                    fallback=move || view! { <div class="empty-state">"No dosage history yet."</div> }
                >
                    <table class="table">
                        <thead>
                            <tr>
                                <th>"Date"</th>
                                <th>"Medication"</th>
                                <th>"Dose"</th>
                                <th>"Unit"</th>
                                <th>"Actions"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <For
                                each=rows
                                key=|entry| match entry {
                                    DosageHistoryEntry::InjectableEstradiol { date, id, .. } => {
                                        id.clone().unwrap_or_else(|| date.to_string())
                                    }
                                    DosageHistoryEntry::OralEstradiol { date, id, .. } => {
                                        id.clone().unwrap_or_else(|| date.to_string())
                                    }
                                    DosageHistoryEntry::Antiandrogen { date, id, .. } => {
                                        id.clone().unwrap_or_else(|| date.to_string())
                                    }
                                    DosageHistoryEntry::Progesterone { date, id, .. } => {
                                        id.clone().unwrap_or_else(|| date.to_string())
                                    }
                                }
                                children=move |entry| {
                                    let (date, name, dose, unit, id_opt, note) = match entry {
                                        DosageHistoryEntry::InjectableEstradiol {
                                            date,
                                            id,
                                            kind,
                                            dose,
                                            unit,
                                            note,
                                            ..
                                        } => (date, format!("{:?}", kind), dose, format!("{:?}", unit), id, note),
                                        DosageHistoryEntry::OralEstradiol {
                                            date,
                                            id,
                                            kind,
                                            dose,
                                            unit,
                                            note,
                                            ..
                                        } => (date, format!("{:?}", kind), dose, format!("{:?}", unit), id, note),
                                        DosageHistoryEntry::Antiandrogen {
                                            date,
                                            id,
                                            kind,
                                            dose,
                                            unit,
                                            note,
                                            ..
                                        } => (date, format!("{:?}", kind), dose, format!("{:?}", unit), id, note),
                                        DosageHistoryEntry::Progesterone {
                                            date,
                                            id,
                                            kind,
                                            dose,
                                            unit,
                                            note,
                                            ..
                                        } => (date, format!("{:?}", kind), dose, format!("{:?}", unit), id, note),
                                    };
                                    let date_text = Local
                                        .timestamp_millis_opt(date)
                                        .single()
                                        .map(|d| d.format("%Y-%m-%d").to_string())
                                        .unwrap_or_else(|| date.to_string());
                                    let entry_key = id_opt.clone().unwrap_or_else(|| date.to_string());
                                    let note_value = note.clone().unwrap_or_default();
                                    let on_delete = {
                                        let store = use_store();
                                        let entry_key = entry_key.clone();
                                        let confirm_delete = confirm_delete;
                                        let confirm_title = confirm_title;
                                        let confirm_action = confirm_action;
                                        move |_: leptos::ev::MouseEvent| {
                                            confirm_title.set("Delete dosage entry?".to_string());
                                            confirm_delete.set(Some(entry_key.clone()));
                                            let store = store.clone();
                                            let entry_key = entry_key.clone();
                                            confirm_action.set(Some(Rc::new(move || {
                                                store.data.update(|d| {
                                                    d.dosageHistory.retain(|item| !entry_matches(item, &entry_key));
                                                });
                                                store.mark_dirty();
                                            })));
                                        }
                                    };
                                    let on_edit = {
                                        let entry_key = entry_key.clone();
                                        let editing_key = editing_key;
                                        let editing_date = editing_date;
                                        let editing_dose = editing_dose;
                                        let editing_note = editing_note;
                                        let note_value = note_value.clone();
                                        let date_text_edit = date_text.clone();
                                        move |_: leptos::ev::MouseEvent| {
                                            editing_key.set(Some(entry_key.clone()));
                                            editing_date.set(date_text_edit.clone());
                                            editing_dose.set(format!("{:.2}", dose));
                                            editing_note.set(note_value.clone());
                                        }
                                    };
                                    view! {
                                        <tr>
                                            <td>{date_text}</td>
                                            <td>{name}</td>
                                            <td>{format!("{:.2}", dose)}</td>
                                            <td>{unit}</td>
                                            <td>
                                                <button type="button" on:click=on_edit>"Edit"</button>
                                                <button type="button" on:click=on_delete>"Delete"</button>
                                            </td>
                                        </tr>
                                        <tr>
                                            <td colspan="5" class="muted">{note_value}</td>
                                        </tr>
                                    }
                                }
                            />
                        </tbody>
                    </table>
                </Show>
            </section>

            <section>
                <h2>"Blood Tests"</h2>
                <Show
                    when=move || !blood_rows().is_empty()
                    fallback=move || view! { <div class="empty-state">"No blood tests yet."</div> }
                >
                    <table class="table">
                        <thead>
                            <tr>
                                <th>"Date"</th>
                                <th>"Estradiol"</th>
                                <th>"Testosterone"</th>
                                <th>"Notes"</th>
                                <th>"Actions"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <For
                                each=blood_rows
                                key=|entry| entry.date
                                children=move |entry| {
                                    let date_text = Local
                                        .timestamp_millis_opt(entry.date)
                                        .single()
                                        .map(|d| d.format("%Y-%m-%d").to_string())
                                        .unwrap_or_else(|| entry.date.to_string());
                                    let e2 = entry
                                        .estradiolLevel
                                        .map(|v| format!("{:.2}", v))
                                        .unwrap_or_else(|| "-".to_string());
                                    let e2_unit = entry
                                        .estradiolUnit
                                        .as_ref()
                                        .map(|u| format!("{:?}", u))
                                        .unwrap_or_else(|| "".to_string());
                                    let t = entry
                                        .testLevel
                                        .map(|v| format!("{:.2}", v))
                                        .unwrap_or_else(|| "-".to_string());
                                    let t_unit = entry
                                        .testUnit
                                        .as_ref()
                                        .map(|u| format!("{:?}", u))
                                        .unwrap_or_else(|| "".to_string());
                                    let on_delete = {
                                        let store = use_store();
                                        let date = entry.date;
                                        let confirm_delete = confirm_delete;
                                        let confirm_title = confirm_title;
                                        let confirm_action = confirm_action;
                                        move |_: leptos::ev::MouseEvent| {
                                            confirm_title.set("Delete blood test?".to_string());
                                            confirm_delete.set(Some(date.to_string()));
                                            let store = store.clone();
                                            confirm_action.set(Some(Rc::new(move || {
                                                store
                                                    .data
                                                    .update(|d| d.bloodTests.retain(|b| b.date != date));
                                                store.mark_dirty();
                                            })));
                                        }
                                    };
                                    let on_edit = {
                                        let date = entry.date;
                                        let edit_blood_date = edit_blood_date;
                                        let edit_blood_date_text = edit_blood_date_text;
                                        let edit_blood_e2 = edit_blood_e2;
                                        let edit_blood_e2_unit = edit_blood_e2_unit;
                                        let edit_blood_t = edit_blood_t;
                                        let edit_blood_t_unit = edit_blood_t_unit;
                                        let edit_blood_notes = edit_blood_notes;
                                        let e2_unit_value = entry
                                            .estradiolUnit
                                            .as_ref()
                                            .map(|u| format!("{:?}", u))
                                            .unwrap_or_else(|| "".to_string());
                                        let t_unit_value = entry
                                            .testUnit
                                            .as_ref()
                                            .map(|u| format!("{:?}", u))
                                            .unwrap_or_else(|| "".to_string());
                                        let e2_value = entry
                                            .estradiolLevel
                                            .map(|v| format!("{:.2}", v))
                                            .unwrap_or_default();
                                        let t_value = entry
                                            .testLevel
                                            .map(|v| format!("{:.2}", v))
                                            .unwrap_or_default();
                                        let notes_value = entry.notes.clone().unwrap_or_default();
                                        let date_text_edit = date_text.clone();
                                        move |_: leptos::ev::MouseEvent| {
                                            edit_blood_date.set(Some(date));
                                            edit_blood_date_text.set(date_text_edit.clone());
                                            edit_blood_e2.set(e2_value.clone());
                                            edit_blood_e2_unit.set(e2_unit_value.clone());
                                            edit_blood_t.set(t_value.clone());
                                            edit_blood_t_unit.set(t_unit_value.clone());
                                            edit_blood_notes.set(notes_value.clone());
                                        }
                                    };
                                    view! {
                                        <tr>
                                            <td>{date_text}</td>
                                            <td>{format!("{} {}", e2, e2_unit)}</td>
                                            <td>{format!("{} {}", t, t_unit)}</td>
                                            <td>{entry.notes.unwrap_or_default()}</td>
                                            <td>
                                                <button type="button" on:click=on_edit>"Edit"</button>
                                                <button type="button" on:click=on_delete>"Delete"</button>
                                            </td>
                                        </tr>
                                    }
                                }
                            />
                        </tbody>
                    </table>
                </Show>
            </section>

            <section>
                <h2>"Measurements"</h2>
                <Show
                    when=move || !measurement_rows().is_empty()
                    fallback=move || view! { <div class="empty-state">"No measurements yet."</div> }
                >
                    <table class="table">
                        <thead>
                            <tr>
                                <th>"Date"</th>
                                <th>"Weight"</th>
                                <th>"Waist"</th>
                                <th>"Hip"</th>
                                <th>"Actions"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <For
                                each=measurement_rows
                                key=|entry| entry.date
                                children=move |entry| {
                                    let date_text = Local
                                        .timestamp_millis_opt(entry.date)
                                        .single()
                                        .map(|d| d.format("%Y-%m-%d").to_string())
                                        .unwrap_or_else(|| entry.date.to_string());
                                    let weight = entry
                                        .weight
                                        .map(|v| format!("{:.2}", v))
                                        .unwrap_or_else(|| "-".to_string());
                                    let weight_unit = entry
                                        .weightUnit
                                        .map(|u| format!("{:?}", u))
                                        .unwrap_or_else(|| "".to_string());
                                    let waist = entry
                                        .waist
                                        .map(|v| format!("{:.1}", v))
                                        .unwrap_or_else(|| "-".to_string());
                                    let hip = entry
                                        .hip
                                        .map(|v| format!("{:.1}", v))
                                        .unwrap_or_else(|| "-".to_string());
                                    let unit = entry
                                        .bodyMeasurementUnit
                                        .as_ref()
                                        .map(|u| format!("{:?}", u))
                                        .unwrap_or_else(|| "".to_string());
                                    let on_delete = {
                                        let store = use_store();
                                        let date = entry.date;
                                        let confirm_delete = confirm_delete;
                                        let confirm_title = confirm_title;
                                        let confirm_action = confirm_action;
                                        move |_: leptos::ev::MouseEvent| {
                                            confirm_title.set("Delete measurement?".to_string());
                                            confirm_delete.set(Some(date.to_string()));
                                            let store = store.clone();
                                            confirm_action.set(Some(Rc::new(move || {
                                                store
                                                    .data
                                                    .update(|d| d.measurements.retain(|m| m.date != date));
                                                store.mark_dirty();
                                            })));
                                        }
                                    };
                                    let on_edit = {
                                        let date = entry.date;
                                        let edit_measurement_date = edit_measurement_date;
                                        let edit_measurement_date_text = edit_measurement_date_text;
                                        let edit_measurement_weight = edit_measurement_weight;
                                        let edit_measurement_waist = edit_measurement_waist;
                                        let edit_measurement_hip = edit_measurement_hip;
                                        let edit_measurement_unit = edit_measurement_unit;
                                        let weight_value = entry
                                            .weight
                                            .map(|v| format!("{:.2}", v))
                                            .unwrap_or_default();
                                        let waist_value = entry
                                            .waist
                                            .map(|v| format!("{:.1}", v))
                                            .unwrap_or_default();
                                        let hip_value = entry
                                            .hip
                                            .map(|v| format!("{:.1}", v))
                                            .unwrap_or_default();
                                        let unit_value = entry
                                            .bodyMeasurementUnit
                                            .as_ref()
                                            .map(|u| format!("{:?}", u))
                                            .unwrap_or_default();
                                        let date_text_edit = date_text.clone();
                                        move |_| {
                                            edit_measurement_date.set(Some(date));
                                            edit_measurement_date_text.set(date_text_edit.clone());
                                            edit_measurement_weight.set(weight_value.clone());
                                            edit_measurement_waist.set(waist_value.clone());
                                            edit_measurement_hip.set(hip_value.clone());
                                            edit_measurement_unit.set(unit_value.clone());
                                        }
                                    };
                                    view! {
                                        <tr>
                                            <td>{date_text}</td>
                                            <td>{format!("{} {}", weight, weight_unit)}</td>
                                            <td>{format!("{} {}", waist, unit)}</td>
                                            <td>{format!("{} {}", hip, unit)}</td>
                                            <td>
                                                <button type="button" on:click=on_edit>"Edit"</button>
                                                <button type="button" on:click=on_delete>"Delete"</button>
                                            </td>
                                        </tr>
                                    }
                                }
                            />
                        </tbody>
                    </table>
                </Show>
            </section>

            <Show when=move || editing_key.get().is_some()>
                <div class="modal-backdrop" on:click=move |_| editing_key.set(None)>
                    <div class="modal" on:click=move |ev| ev.stop_propagation()>
                        <h3>"Edit Dosage Entry"</h3>
                        <label>"Date"</label>
                        <input
                            type="date"
                            on:input=move |ev| editing_date.set(event_target_value(&ev))
                            prop:value=move || editing_date.get()
                        />
                        <label>"Dose"</label>
                        <input
                            type="number"
                            step="0.01"
                            on:input=move |ev| editing_dose.set(event_target_value(&ev))
                            prop:value=move || editing_dose.get()
                        />
                        <label>"Notes"</label>
                        <textarea
                            rows="3"
                            on:input=move |ev| editing_note.set(event_target_value(&ev))
                            prop:value=move || editing_note.get()
                        ></textarea>
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
                <div class="modal-backdrop" on:click=move |_| confirm_delete.set(None)>
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
                        <label>"Date"</label>
                        <input
                            type="date"
                            on:input=move |ev| edit_blood_date_text.set(event_target_value(&ev))
                            prop:value=move || edit_blood_date_text.get()
                        />
                        <label>"Estradiol"</label>
                        <div class="inline">
                            <input
                                type="number"
                                step="0.01"
                                on:input=move |ev| edit_blood_e2.set(event_target_value(&ev))
                                prop:value=move || edit_blood_e2.get()
                            />
                            <select
                                on:change=move |ev| edit_blood_e2_unit.set(event_target_value(&ev))
                                prop:value=move || edit_blood_e2_unit.get()
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
                                on:input=move |ev| edit_blood_t.set(event_target_value(&ev))
                                prop:value=move || edit_blood_t.get()
                            />
                            <select
                                on:change=move |ev| edit_blood_t_unit.set(event_target_value(&ev))
                                prop:value=move || edit_blood_t_unit.get()
                            >
                                <option value="ng/dL">"ng/dL"</option>
                                <option value="nmol/L">"nmol/L"</option>
                            </select>
                        </div>
                        <label>"Notes"</label>
                        <textarea
                            rows="3"
                            on:input=move |ev| edit_blood_notes.set(event_target_value(&ev))
                            prop:value=move || edit_blood_notes.get()
                        ></textarea>
                        <div class="modal-actions">
                            <button type="button" on:click={
                                let store = store_blood_modal.clone();
                                move |_: leptos::ev::MouseEvent| {
                                    let date = match edit_blood_date.get() {
                                        Some(value) => value,
                                        None => return,
                                    };
                                    let new_date = parse_date_or_now(&edit_blood_date_text.get());
                                    let e2_value = edit_blood_e2.get().trim().parse::<f64>().ok();
                                    let t_value = edit_blood_t.get().trim().parse::<f64>().ok();
                                    let e2_unit = edit_blood_e2_unit.get();
                                    let t_unit = edit_blood_t_unit.get();
                                    let notes = edit_blood_notes.get();
                                    store.data.update(|d| {
                                        for entry in &mut d.bloodTests {
                                            if entry.date == date {
                                                entry.date = new_date;
                                                entry.estradiolLevel = e2_value;
                                                entry.testLevel = t_value;
                                                entry.estradiolUnit = parse_hormone_unit(&e2_unit);
                                                entry.testUnit = parse_hormone_unit(&t_unit);
                                                entry.notes = if notes.trim().is_empty() {
                                                    None
                                                } else {
                                                    Some(notes.clone())
                                                };
                                            }
                                        }
                                    });
                                    store.mark_dirty();
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

            <Show when=move || edit_measurement_date.get().is_some()>
                <div class="modal-backdrop" on:click=move |_| edit_measurement_date.set(None)>
                    <div class="modal" on:click=move |ev| ev.stop_propagation()>
                        <h3>"Edit Measurement"</h3>
                        <label>"Date"</label>
                        <input
                            type="date"
                            on:input=move |ev| edit_measurement_date_text.set(event_target_value(&ev))
                            prop:value=move || edit_measurement_date_text.get()
                        />
                        <label>"Weight"</label>
                        <input
                            type="number"
                            step="0.01"
                            on:input=move |ev| edit_measurement_weight.set(event_target_value(&ev))
                            prop:value=move || edit_measurement_weight.get()
                        />
                        <label>"Waist"</label>
                        <input
                            type="number"
                            step="0.1"
                            on:input=move |ev| edit_measurement_waist.set(event_target_value(&ev))
                            prop:value=move || edit_measurement_waist.get()
                        />
                        <label>"Hip"</label>
                        <input
                            type="number"
                            step="0.1"
                            on:input=move |ev| edit_measurement_hip.set(event_target_value(&ev))
                            prop:value=move || edit_measurement_hip.get()
                        />
                        <label>"Unit"</label>
                        <input
                            type="text"
                            on:input=move |ev| edit_measurement_unit.set(event_target_value(&ev))
                            prop:value=move || edit_measurement_unit.get()
                        />
                        <div class="modal-actions">
                            <button type="button" on:click={
                                let store = store_measure_modal.clone();
                                move |_: leptos::ev::MouseEvent| {
                                    let date = match edit_measurement_date.get() {
                                        Some(value) => value,
                                        None => return,
                                    };
                                    let new_date = parse_date_or_now(&edit_measurement_date_text.get());
                                    let weight = edit_measurement_weight.get().trim().parse::<f64>().ok();
                                    let waist = edit_measurement_waist.get().trim().parse::<f64>().ok();
                                    let hip = edit_measurement_hip.get().trim().parse::<f64>().ok();
                                    let unit = edit_measurement_unit.get();
                                    store.data.update(|d| {
                                        for entry in &mut d.measurements {
                                            if entry.date == date {
                                                entry.date = new_date;
                                                entry.weight = weight;
                                                entry.waist = waist;
                                                entry.hip = hip;
                                                entry.bodyMeasurementUnit = parse_length_unit(&unit);
                                            }
                                        }
                                    });
                                    store.mark_dirty();
                                    edit_measurement_date.set(None);
                                }
                            }>
                                "Save"
                            </button>
                            <button type="button" on:click={
                                let edit_measurement_date = edit_measurement_date;
                                move |_: leptos::ev::MouseEvent| edit_measurement_date.set(None)
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
