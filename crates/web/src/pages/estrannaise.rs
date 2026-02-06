use gloo_events::EventListener;
use leptos::window;
use leptos::*;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::HtmlCanvasElement;

use crate::charts::estrannaise::{draw_estrannaise_chart, find_nearest_estrannaise_point};
use crate::charts::{
    chart_padding, clamp_zoom, compute_chart_bounds, ChartTooltip, DragState, ViewZoom,
};
use crate::estrannaise::compute_estrannaise_series;
use crate::layout::page_layout;
use crate::store::use_store;
use crate::utils::{compute_fudge_factor, fmt_blood_value, fmt_date_label, hormone_unit_label};
use hrt_shared::logic::predict_e2_pg_ml;
use hrt_shared::types::{BloodTest, HormoneUnits};

#[component]
pub fn EstrannaisePage() -> impl IntoView {
    let store = use_store();
    let x_axis_mode = create_rw_signal("date".to_string());
    let forecast_enabled = create_rw_signal(true);
    let forecast_weeks = create_rw_signal(8_i64);
    let forecast_dose_override = create_rw_signal(String::new());
    let forecast_frequency_override = create_rw_signal(String::new());
    let selected_blood_test = create_rw_signal(String::new());
    let estrannaise_zoom = create_rw_signal(ViewZoom::default());
    let estrannaise_tooltip = create_rw_signal(None::<ChartTooltip>);

    let blood_test_options = create_memo({
        let settings = store.settings;
        move |_| {
            let data_value = store.data.get();
            let display_unit = settings
                .get()
                .displayEstradiolUnit
                .unwrap_or(HormoneUnits::E2PmolL);
            let fallback_label = hormone_unit_label(&display_unit);
            let mut tests: Vec<&BloodTest> = data_value
                .bloodTests
                .iter()
                .filter(|test| test.estradiolLevel.is_some())
                .collect();
            tests.sort_by_key(|test| test.date);
            tests.reverse();
            tests
                .into_iter()
                .map(|test| {
                    let date_label = fmt_date_label(test.date, "date", None);
                    let value_label = test
                        .estradiolLevel
                        .map(fmt_blood_value)
                        .unwrap_or_else(|| "-".to_string());
                    let unit_label = test
                        .estradiolUnit
                        .as_ref()
                        .map(hormone_unit_label)
                        .unwrap_or(fallback_label);
                    (
                        test.date.to_string(),
                        format!("{date_label} · E2 {value_label} {unit_label}"),
                    )
                })
                .collect::<Vec<_>>()
        }
    });

    let selected_fudge_factor = create_memo({
        let store = store.clone();
        move |_| {
            let selected = selected_blood_test.get();
            if selected.trim().is_empty() {
                return None;
            }
            let date = selected.trim().parse::<i64>().ok()?;
            let data_value = store.data.get();
            let test = data_value
                .bloodTests
                .iter()
                .find(|test| test.date == date)?;
            if let Some(fudge) = test.fudgeFactor {
                return Some(fudge);
            }
            let measured = measured_e2_pg_ml(test);
            let predicted = predict_e2_pg_ml(&data_value, test.date);
            compute_fudge_factor(measured, predicted)
        }
    });

    let estrannaise_series = create_memo({
        let settings = store.settings;
        move |_| {
            let data_value = store.data.get();
            let settings_value = settings.get();
            let dose_override = forecast_dose_override.get().trim().parse::<f64>().ok();
            let freq_override = forecast_frequency_override.get().trim().parse::<f64>().ok();
            compute_estrannaise_series(
                &data_value,
                &settings_value,
                &x_axis_mode.get(),
                forecast_enabled.get(),
                forecast_weeks.get(),
                dose_override,
                freq_override,
                selected_fudge_factor.get(),
            )
        }
    });

    const ESTRANNAISE_CANVAS_ID: &str = "estrannaise-chart-canvas";
    let estrannaise_drag = Rc::new(RefCell::new(None::<DragState>));

    let on_mouse_move = {
        let estrannaise_series = estrannaise_series.clone();
        let estrannaise_zoom = estrannaise_zoom;
        let estrannaise_tooltip = estrannaise_tooltip;
        let estrannaise_drag = estrannaise_drag.clone();
        move |ev: leptos::ev::MouseEvent| {
            let Some(canvas) = window()
                .document()
                .and_then(|doc| doc.get_element_by_id(ESTRANNAISE_CANVAS_ID))
                .and_then(|el| el.dyn_into::<HtmlCanvasElement>().ok())
            else {
                return;
            };
            let rect = canvas.get_bounding_client_rect();
            let cursor_x = ev.client_x() as f64 - rect.left();
            let cursor_y = ev.client_y() as f64 - rect.top();
            let series = estrannaise_series.get();
            let zoom = estrannaise_zoom.get();
            let x_min = zoom.x_min.unwrap_or(series.domain_min);
            let x_max = zoom.x_max.unwrap_or(series.domain_max);
            let padding = chart_padding();
            let (width, height, domain_span, y_span) = compute_chart_bounds(
                rect.width(),
                rect.height(),
                padding,
                x_min,
                x_max,
                series.y_min,
                series.y_max,
            );
            let mut best: Option<(ChartTooltip, f64)> = None;
            for set in [&series.blended, &series.stepped, &series.blood] {
                if let Some(candidate) = find_nearest_estrannaise_point(
                    set,
                    x_min,
                    domain_span,
                    series.y_min,
                    y_span,
                    width,
                    height,
                    padding,
                    cursor_x,
                    cursor_y,
                ) {
                    if best
                        .as_ref()
                        .map(|(_, dist)| *dist)
                        .unwrap_or(f64::INFINITY)
                        > candidate.1
                    {
                        best = Some(candidate);
                    }
                }
            }
            if let Some(drag) = estrannaise_drag.borrow().as_ref() {
                estrannaise_tooltip.set(None);
                let delta_px = cursor_x - drag.start_x;
                let span = x_max - x_min;
                let delta_domain = -(delta_px / width) * span;
                let next_min = drag.start_min + delta_domain;
                let next_max = drag.start_max + delta_domain;
                estrannaise_zoom.set(clamp_zoom(
                    series.domain_min,
                    series.domain_max,
                    next_min,
                    next_max,
                ));
            } else {
                estrannaise_tooltip.set(best.map(|(tip, _)| tip));
            }
        }
    };

    let on_mouse_leave = {
        let estrannaise_drag = estrannaise_drag.clone();
        let estrannaise_tooltip = estrannaise_tooltip;
        move |_| {
            estrannaise_drag.replace(None);
            estrannaise_tooltip.set(None);
        }
    };

    let on_mouse_down = {
        let estrannaise_drag = estrannaise_drag.clone();
        let estrannaise_zoom = estrannaise_zoom;
        let estrannaise_series = estrannaise_series.clone();
        move |ev: leptos::ev::MouseEvent| {
            let Some(canvas) = window()
                .document()
                .and_then(|doc| doc.get_element_by_id(ESTRANNAISE_CANVAS_ID))
                .and_then(|el| el.dyn_into::<HtmlCanvasElement>().ok())
            else {
                return;
            };
            let rect = canvas.get_bounding_client_rect();
            let cursor_x = ev.client_x() as f64 - rect.left();
            let series = estrannaise_series.get();
            let zoom = estrannaise_zoom.get();
            let x_min = zoom.x_min.unwrap_or(series.domain_min);
            let x_max = zoom.x_max.unwrap_or(series.domain_max);
            estrannaise_drag.replace(Some(DragState {
                start_x: cursor_x,
                start_min: x_min,
                start_max: x_max,
            }));
        }
    };

    let on_mouse_up = {
        let estrannaise_drag = estrannaise_drag.clone();
        move |_| {
            estrannaise_drag.replace(None);
        }
    };

    let on_wheel = {
        let estrannaise_zoom = estrannaise_zoom;
        let estrannaise_series = estrannaise_series.clone();
        move |ev: leptos::ev::WheelEvent| {
            ev.prevent_default();
            let Some(canvas) = window()
                .document()
                .and_then(|doc| doc.get_element_by_id(ESTRANNAISE_CANVAS_ID))
                .and_then(|el| el.dyn_into::<HtmlCanvasElement>().ok())
            else {
                return;
            };
            let rect = canvas.get_bounding_client_rect();
            let cursor_x = ev.client_x() as f64 - rect.left();
            let series = estrannaise_series.get();
            let zoom = estrannaise_zoom.get();
            let x_min = zoom.x_min.unwrap_or(series.domain_min);
            let x_max = zoom.x_max.unwrap_or(series.domain_max);
            let padding = chart_padding();
            let (width, _, domain_span, _) = compute_chart_bounds(
                rect.width(),
                rect.height(),
                padding,
                x_min,
                x_max,
                series.y_min,
                series.y_max,
            );
            let cursor_ratio = ((cursor_x - padding.0) / width).clamp(0.0, 1.0);
            let zoom_factor = if ev.delta_y() < 0.0 { 0.85 } else { 1.15 };
            let new_span = (domain_span * zoom_factor).max(1.0);
            let center = x_min + domain_span * cursor_ratio;
            let new_min = center - new_span * cursor_ratio;
            let new_max = new_min + new_span;
            estrannaise_zoom.set(clamp_zoom(
                series.domain_min,
                series.domain_max,
                new_min,
                new_max,
            ));
        }
    };

    create_effect({
        let estrannaise_series = estrannaise_series.clone();
        let estrannaise_zoom = estrannaise_zoom;
        move |_| {
            let series = estrannaise_series.get();
            if series.blended.is_empty() && series.stepped.is_empty() && series.blood.is_empty() {
                return;
            }
            draw_estrannaise_chart(ESTRANNAISE_CANVAS_ID, &series, estrannaise_zoom.get());
        }
    });

    let resize_listener: Rc<RefCell<Option<EventListener>>> = Rc::new(RefCell::new(None));
    create_effect({
        let estrannaise_series = estrannaise_series.clone();
        let estrannaise_zoom = estrannaise_zoom;
        let resize_listener = resize_listener.clone();
        move |_| {
            estrannaise_series.get();
            let window = window();
            let listener = EventListener::new(&window, "resize", move |_| {
                let series = estrannaise_series.get();
                if series.blended.is_empty() && series.stepped.is_empty() && series.blood.is_empty()
                {
                    return;
                }
                draw_estrannaise_chart(ESTRANNAISE_CANVAS_ID, &series, estrannaise_zoom.get());
            });
            resize_listener.replace(Some(listener));
        }
    });

    let reset_zoom = {
        let estrannaise_zoom = estrannaise_zoom;
        move |_| estrannaise_zoom.set(ViewZoom::default())
    };

    let tooltip_value = move || estrannaise_tooltip.get();

    page_layout(
        "Estrannaise",
        view! {
            <div class="view-layout">
                <div class="view-header">
                    <div>
                        <h2>"Estrannaise"</h2>
                        <p class="muted">
                            "Estrannaise-style modeling with blended vs. step fudge factors, plus forecasted schedule windows."
                        </p>
                    </div>
                </div>

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
                            prop:disabled=move || estrannaise_series.get().first_dose.is_none()
                        >
                            "Days since first dose"
                        </button>
                    </div>
                    <div class="chart-toolbar-group">
                        <label class="muted">"Forecast"</label>
                        <input
                            type="checkbox"
                            on:change=move |ev| forecast_enabled.set(event_target_checked(&ev))
                            prop:checked=move || forecast_enabled.get()
                        />
                    </div>
                    <div class="chart-toolbar-group">
                        <label class="muted">"Weeks"</label>
                        <select on:change=move |ev| forecast_weeks.set(event_target_value(&ev).parse::<i64>().unwrap_or(8))>
                            <option value="4" selected=move || forecast_weeks.get() == 4>"4"</option>
                            <option value="6" selected=move || forecast_weeks.get() == 6>"6"</option>
                            <option value="8" selected=move || forecast_weeks.get() == 8>"8"</option>
                        </select>
                    </div>
                    <div class="chart-toolbar-group">
                        <label class="muted">"Dose"</label>
                        <input
                            type="number"
                            step="0.1"
                            min="0"
                            class="chart-input"
                            placeholder="auto"
                            on:input=move |ev| forecast_dose_override.set(event_target_value(&ev))
                            prop:value=move || forecast_dose_override.get()
                        />
                    </div>
                    <div class="chart-toolbar-group">
                        <label class="muted">"Every (days)"</label>
                        <input
                            type="number"
                            step="any"
                            min="1"
                            class="chart-input"
                            placeholder="auto"
                            on:input=move |ev| forecast_frequency_override.set(event_target_value(&ev))
                            prop:value=move || forecast_frequency_override.get()
                        />
                    </div>
                    <div class="chart-toolbar-group">
                        <label class="muted">"Pink line"</label>
                        <select
                            on:change=move |ev| selected_blood_test.set(event_target_value(&ev))
                            prop:value=move || selected_blood_test.get()
                        >
                            <option value="">"Auto (use all tests)"</option>
                            <For
                                each=move || blood_test_options.get()
                                key=|entry| entry.0.clone()
                                children=move |entry| {
                                    let value = entry.0.clone();
                                    let label = entry.1.clone();
                                    view! { <option value=value.clone()>{label}</option> }
                                }
                            />
                        </select>
                    </div>
                    <div class="chart-toolbar-group">
                        <button on:click=reset_zoom disabled=move || estrannaise_zoom.get().x_min.is_none()>
                            "Reset zoom"
                        </button>
                    </div>
                </div>

                <div class="chart-card chart-interactive">
                    <Show
                        when=move || !estrannaise_series.get().blended.is_empty()
                            || !estrannaise_series.get().stepped.is_empty()
                            || !estrannaise_series.get().blood.is_empty()
                        fallback=move || view! {
                            <div class="empty-state">
                                <p>"No Estrannaise data available."</p>
                                <p class="muted">"Add injectable estradiol history and blood tests to see model lines."</p>
                            </div>
                        }
                    >
                        <canvas
                            id=ESTRANNAISE_CANVAS_ID
                            width="900"
                            height="420"
                            on:mousemove=on_mouse_move.clone()
                            on:mouseleave=on_mouse_leave.clone()
                            on:mousedown=on_mouse_down.clone()
                            on:mouseup=on_mouse_up.clone()
                            on:wheel=on_wheel.clone()
                        ></canvas>
                        <Show when=move || tooltip_value().is_some()>
                            <div
                                class="chart-tooltip"
                                style=move || {
                                    tooltip_value()
                                        .map(|tip| format!("left: {:.0}px; top: {:.0}px;", tip.x + 12.0, tip.y + 12.0))
                                        .unwrap_or_default()
                                }
                            >
                                {move || tooltip_value().map(|tip| tip.text).unwrap_or_default()}
                            </div>
                        </Show>
                    </Show>
                    <div class="chart-note muted">
                        <p>"* Blue line blends fudge factor between blood tests."</p>
                        <p>"* Pink dashed line steps to each test's fudge factor."</p>
                        <p>"* Orange points show measured E2 in display units."</p>
                        <p>"* Shaded region is forecasted schedule window."</p>
                        <Show when=move || selected_fudge_factor.get().is_some()>
                            <p>{move || {
                                let value = selected_fudge_factor.get().unwrap_or(1.0);
                                format!("* Selected test fudge factor: {:.3}", value)
                            }}</p>
                        </Show>
                    </div>
                </div>
            </div>
        }
        .into_view(),
    )
}
fn measured_e2_pg_ml(test: &BloodTest) -> Option<f64> {
    let value = test.estradiolLevel?;
    let unit = test.estradiolUnit.clone().unwrap_or(HormoneUnits::E2PgMl);
    let measured = if unit == HormoneUnits::E2PmolL {
        value / 3.671
    } else {
        value
    };
    if measured.is_finite() {
        Some(measured)
    } else {
        None
    }
}
