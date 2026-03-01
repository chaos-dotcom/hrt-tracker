use chrono::{Local, TimeZone};
use leptos::window;
use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use wasm_bindgen::JsCast;
use web_sys::HtmlCanvasElement;

use crate::charts::{
    data_to_canvas_x, data_to_canvas_y, ChartTooltip, ViewZoom, CHART_MARGIN, CHART_X_LABEL,
    CHART_Y_LABEL,
};
use crate::utils::{
    convert_estradiol_to_display, convert_fsh_to_miu_ml, convert_lh_to_miu_ml,
    convert_progesterone_to_ng_ml, convert_testosterone_to_ng_dl, fmt_blood_value, fmt_date_label,
    hormone_unit_label,
};
use hrt_shared::types::{BloodTest, DosageHistoryEntry, HormoneUnits, HrtData, Settings};

fn inferred_fudge_factor(test: &BloodTest) -> Option<f64> {
    if let Some(value) = test.fudgeFactor {
        if value.is_finite() && value > 0.0 {
            return Some(value);
        }
    }

    let measured = test.estradiolLevel?;
    if !measured.is_finite() {
        return None;
    }
    let measured_pg_ml = if test.estradiolUnit == Some(HormoneUnits::E2PmolL) {
        measured / 3.671
    } else {
        measured
    };
    if !measured_pg_ml.is_finite() {
        return None;
    }

    let predicted = test
        .estrannaiseNumber
        .filter(|value| value.is_finite() && *value > 0.0)?;
    let ratio = measured_pg_ml / predicted;
    if ratio.is_finite() && ratio > 0.0 {
        Some((ratio * 1000.0).round() / 1000.0)
    } else {
        None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ViewChartPoint {
    pub x: f64,
    pub y: f64,
    pub label: String,
    pub color: RGBColor,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ViewChartState {
    pub domain_min: f64,
    pub domain_max: f64,
    pub y_min: f64,
    pub y_max: f64,
    pub x_label: String,
    pub y_label: String,
    pub points: Vec<ViewChartPoint>,
    pub dosage_points: Vec<ViewChartPoint>,
    pub first_dose: Option<i64>,
    pub use_days: bool,
    pub has_data: bool,
}

pub fn compute_view_chart_state(
    data: &HrtData,
    settings: &Settings,
    axis_mode: &str,
    time_range_days: i64,
    show_medications: bool,
    show_e2: bool,
    show_t: bool,
    show_prog: bool,
    show_fsh: bool,
    show_lh: bool,
    show_prolactin: bool,
    show_shbg: bool,
    show_fai: bool,
    show_fudge_factor: bool,
) -> ViewChartState {
    let now = js_sys::Date::now() as i64;
    let start_time = now - time_range_days * 24 * 60 * 60 * 1000;
    let display_unit = settings
        .displayEstradiolUnit
        .clone()
        .unwrap_or(HormoneUnits::E2PmolL);
    let first_dose = data
        .dosageHistory
        .iter()
        .map(|d| match d {
            DosageHistoryEntry::InjectableEstradiol { date, .. }
            | DosageHistoryEntry::OralEstradiol { date, .. }
            | DosageHistoryEntry::Antiandrogen { date, .. }
            | DosageHistoryEntry::Progesterone { date, .. } => *date,
        })
        .min();

    let use_days = axis_mode == "days" && first_dose.is_some();
    let x_label = if use_days {
        "Days since first dose".to_string()
    } else {
        "Date".to_string()
    };

    let mut points = Vec::new();
    let mut all_values = Vec::new();
    let mut has_data = false;

    for test in data.bloodTests.iter().filter(|t| t.date >= start_time) {
        let x = if use_days {
            (test.date - first_dose.unwrap_or(test.date)) as f64 / (24.0 * 60.0 * 60.0 * 1000.0)
        } else {
            test.date as f64
        };
        let date_label = fmt_date_label(test.date, axis_mode, first_dose);
        let date_short = if use_days {
            date_label.clone()
        } else {
            Local
                .timestamp_millis_opt(test.date)
                .single()
                .map(|d| d.format("%b %d").to_string())
                .unwrap_or_else(|| date_label.clone())
        };
        if show_e2 {
            if let Some(value) = test.estradiolLevel {
                let raw_unit = test.estradiolUnit.clone().unwrap_or(display_unit.clone());
                let plot_val = convert_estradiol_to_display(value, &raw_unit, &display_unit);
                let unit_label = hormone_unit_label(&display_unit);
                let tooltip = if raw_unit != display_unit {
                    format!(
                        "Estradiol: {} {} -> {} {} ({})",
                        fmt_blood_value(value),
                        hormone_unit_label(&raw_unit),
                        fmt_blood_value(plot_val),
                        unit_label,
                        date_short
                    )
                } else {
                    format!(
                        "Estradiol: {} {} ({})",
                        fmt_blood_value(plot_val),
                        unit_label,
                        date_short
                    )
                };
                points.push(ViewChartPoint {
                    x,
                    y: plot_val,
                    label: tooltip,
                    color: RGBColor(70, 130, 180),
                });
                all_values.push(plot_val);
                has_data = true;
            }
        }
        if show_t {
            if let Some(value) = test.testLevel {
                let raw_unit = test.testUnit.clone().unwrap_or(HormoneUnits::TNmolL);
                let plot_val = convert_testosterone_to_ng_dl(value, &raw_unit);
                let unit_label = "ng/dL";
                let tooltip = if raw_unit != HormoneUnits::TNgDl {
                    format!(
                        "Testosterone: {} {} -> {} {} ({})",
                        fmt_blood_value(value),
                        hormone_unit_label(&raw_unit),
                        fmt_blood_value(plot_val),
                        unit_label,
                        date_short
                    )
                } else {
                    format!(
                        "Testosterone: {} {} ({})",
                        fmt_blood_value(plot_val),
                        unit_label,
                        date_short
                    )
                };
                points.push(ViewChartPoint {
                    x,
                    y: plot_val,
                    label: tooltip,
                    color: RGBColor(220, 20, 60),
                });
                all_values.push(plot_val);
                has_data = true;
            }
        }
        if show_prog {
            if let Some(value) = test.progesteroneLevel {
                let raw_unit = test
                    .progesteroneUnit
                    .clone()
                    .unwrap_or(HormoneUnits::TNmolL);
                let plot_val = convert_progesterone_to_ng_ml(value, &raw_unit);
                let tooltip = format!(
                    "Progesterone: {} ng/mL ({})",
                    fmt_blood_value(plot_val),
                    date_short
                );
                points.push(ViewChartPoint {
                    x,
                    y: plot_val,
                    label: tooltip,
                    color: RGBColor(148, 0, 211),
                });
                all_values.push(plot_val);
                has_data = true;
            }
        }
        if show_fsh {
            if let Some(value) = test.fshLevel {
                let raw_unit = test.fshUnit.clone().unwrap_or(HormoneUnits::UL);
                let plot_val = convert_fsh_to_miu_ml(value, &raw_unit);
                let tooltip = format!("FSH: {} mIU/mL ({})", fmt_blood_value(plot_val), date_short);
                points.push(ViewChartPoint {
                    x,
                    y: plot_val,
                    label: tooltip,
                    color: RGBColor(34, 139, 34),
                });
                all_values.push(plot_val);
                has_data = true;
            }
        }
        if show_lh {
            if let Some(value) = test.lhLevel {
                let raw_unit = test.lhUnit.clone().unwrap_or(HormoneUnits::UL);
                let plot_val = convert_lh_to_miu_ml(value, &raw_unit);
                let tooltip = format!("LH: {} mIU/mL ({})", fmt_blood_value(plot_val), date_short);
                points.push(ViewChartPoint {
                    x,
                    y: plot_val,
                    label: tooltip,
                    color: RGBColor(0, 139, 139),
                });
                all_values.push(plot_val);
                has_data = true;
            }
        }
        if show_prolactin {
            if let Some(value) = test.prolactinLevel {
                let raw_unit = test.prolactinUnit.clone().unwrap_or(HormoneUnits::MIuL);
                let unit_label = hormone_unit_label(&raw_unit);
                let tooltip = format!(
                    "Prolactin: {} {} ({})",
                    fmt_blood_value(value),
                    unit_label,
                    date_short
                );
                points.push(ViewChartPoint {
                    x,
                    y: value,
                    label: tooltip,
                    color: RGBColor(139, 69, 19),
                });
                all_values.push(value);
                has_data = true;
            }
        }
        if show_shbg {
            if let Some(value) = test.shbgLevel {
                let raw_unit = test.shbgUnit.clone().unwrap_or(HormoneUnits::TNmolL);
                let unit_label = hormone_unit_label(&raw_unit);
                let tooltip = format!(
                    "SHBG: {} {} ({})",
                    fmt_blood_value(value),
                    unit_label,
                    date_short
                );
                points.push(ViewChartPoint {
                    x,
                    y: value,
                    label: tooltip,
                    color: RGBColor(255, 20, 147),
                });
                all_values.push(value);
                has_data = true;
            }
        }
        if show_fai {
            if let Some(value) = test.freeAndrogenIndex {
                let tooltip = format!("FAI: {} ({})", fmt_blood_value(value), date_short);
                points.push(ViewChartPoint {
                    x,
                    y: value,
                    label: tooltip,
                    color: RGBColor(0, 0, 0),
                });
                all_values.push(value);
                has_data = true;
            }
        }
        if show_fudge_factor {
            if let Some(fudge_factor) = inferred_fudge_factor(test) {
                let efficacy_percent = fudge_factor * 100.0;
                let tooltip = format!(
                    "Efficacy (FF): {}% ({}x) ({})",
                    fmt_blood_value(efficacy_percent),
                    fmt_blood_value(fudge_factor),
                    date_short
                );
                points.push(ViewChartPoint {
                    x,
                    y: efficacy_percent,
                    label: tooltip,
                    color: RGBColor(255, 99, 132),
                });
                all_values.push(efficacy_percent);
                has_data = true;
            }
        }
    }

    let mut dosage_points = Vec::new();
    if show_medications {
        for dose in data.dosageHistory.iter() {
            let date = match dose {
                DosageHistoryEntry::InjectableEstradiol { date, .. }
                | DosageHistoryEntry::OralEstradiol { date, .. }
                | DosageHistoryEntry::Antiandrogen { date, .. }
                | DosageHistoryEntry::Progesterone { date, .. } => *date,
            };
            if date < start_time {
                continue;
            }
            let (label, value, color) = match dose {
                DosageHistoryEntry::InjectableEstradiol {
                    kind, dose, unit, ..
                } => (
                    format!(
                        "Injection: {:?}, {:.2} {}",
                        kind,
                        dose,
                        hormone_unit_label(unit)
                    ),
                    (*dose * 20.0).min(300.0),
                    RGBColor(0, 114, 178),
                ),
                DosageHistoryEntry::OralEstradiol {
                    kind, dose, unit, ..
                } => (
                    format!(
                        "Oral E: {:?}, {:.2} {}",
                        kind,
                        dose,
                        hormone_unit_label(unit)
                    ),
                    (*dose * 10.0).min(200.0),
                    RGBColor(46, 139, 87),
                ),
                DosageHistoryEntry::Antiandrogen {
                    kind, dose, unit, ..
                } => (
                    format!("AA: {:?}, {:.2} {}", kind, dose, hormone_unit_label(unit)),
                    (*dose * 10.0).min(200.0),
                    RGBColor(255, 140, 0),
                ),
                DosageHistoryEntry::Progesterone {
                    kind, dose, unit, ..
                } => (
                    format!(
                        "Progesterone: {:?}, {:.2} {}",
                        kind,
                        dose,
                        hormone_unit_label(unit)
                    ),
                    (*dose).min(400.0),
                    RGBColor(255, 215, 0),
                ),
            };
            let x = if use_days {
                (date - first_dose.unwrap_or(date)) as f64 / (24.0 * 60.0 * 60.0 * 1000.0)
            } else {
                date as f64
            };
            let date_label = fmt_date_label(date, axis_mode, first_dose);
            dosage_points.push(ViewChartPoint {
                x,
                y: value,
                label: format!("{} ({})", label, date_label),
                color,
            });

            all_values.push(value);
            has_data = true;
        }
    }

    if all_values.is_empty() {
        all_values.push(0.0);
        all_values.push(1.0);
    }
    let mut y_min = all_values.iter().cloned().fold(f64::INFINITY, f64::min);
    let mut y_max = all_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    if (y_min - y_max).abs() < f64::EPSILON {
        y_min = (y_min - 1.0).max(0.0);
        y_max += 1.0;
    } else {
        let pad = (y_max - y_min) * 0.08;
        y_min = (y_min - pad).max(0.0);
        y_max += pad;
    }

    let mut x_values: Vec<f64> = points.iter().map(|p| p.x).collect();
    x_values.extend(dosage_points.iter().map(|p| p.x));

    let (domain_min, domain_max) = if x_values.is_empty() {
        if use_days {
            (0.0, 30.0)
        } else {
            (start_time as f64, now as f64)
        }
    } else {
        let mut min_x = x_values.iter().cloned().fold(f64::INFINITY, f64::min);
        let mut max_x = x_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        if (min_x - max_x).abs() < f64::EPSILON {
            min_x -= 1.0;
            max_x += 1.0;
        }
        (min_x, max_x)
    };

    ViewChartState {
        domain_min,
        domain_max,
        y_min,
        y_max,
        x_label,
        y_label: "Levels".to_string(),
        points,
        dosage_points,
        first_dose,
        use_days,
        has_data,
    }
}

pub fn find_nearest_point(
    points: &[ViewChartPoint],
    domain_min: f64,
    domain_span: f64,
    y_min: f64,
    y_span: f64,
    width: f64,
    height: f64,
    padding: (f64, f64, f64, f64),
    cursor_x: f64,
    cursor_y: f64,
) -> Option<(ChartTooltip, f64)> {
    let (left, top, _, _) = padding;
    let mut best: Option<(f64, &ViewChartPoint, f64, f64)> = None;
    for point in points {
        let px = data_to_canvas_x(point.x, domain_min, domain_span, width, left);
        let py = data_to_canvas_y(point.y, y_min, y_span, height, top);
        let dx = px - cursor_x;
        let dy = py - cursor_y;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist < 48.0 {
            match best {
                Some((best_dist, _, _, _)) if dist >= best_dist => {}
                _ => best = Some((dist, point, px, py)),
            }
        }
    }
    best.map(|(dist, point, px, py)| {
        (
            ChartTooltip {
                text: point.label.clone(),
                x: px,
                y: py,
            },
            dist,
        )
    })
}

pub fn draw_view_chart(canvas_id: &str, state: &ViewChartState, zoom: ViewZoom) {
    let Some(canvas) = window()
        .document()
        .and_then(|doc| doc.get_element_by_id(canvas_id))
        .and_then(|el| el.dyn_into::<HtmlCanvasElement>().ok())
    else {
        return;
    };
    let rect = canvas.get_bounding_client_rect();
    let width = rect.width().max(320.0);
    let height = rect.height().max(280.0);
    let dpr = window().device_pixel_ratio();
    canvas.set_width((width * dpr) as u32);
    canvas.set_height((height * dpr) as u32);

    let backend = CanvasBackend::with_canvas_object(canvas)
        .expect("canvas backend")
        .into_drawing_area();
    backend.fill(&RGBColor(15, 17, 26)).ok();

    let x_min = zoom.x_min.unwrap_or(state.domain_min);
    let x_max = zoom.x_max.unwrap_or(state.domain_max);
    let mut chart = match ChartBuilder::on(&backend)
        .margin(CHART_MARGIN as i32)
        .x_label_area_size(CHART_X_LABEL as i32)
        .y_label_area_size(CHART_Y_LABEL as i32)
        .build_cartesian_2d(x_min..x_max, state.y_min..state.y_max)
    {
        Ok(chart) => chart,
        Err(_) => return,
    };

    let label_style = ("Quicksand", 24)
        .into_font()
        .color(&RGBColor(200, 188, 214));
    let axis_desc_style = ("Quicksand", 24)
        .into_font()
        .color(&RGBColor(200, 188, 214));
    let bold_grid = ShapeStyle::from(&RGBColor(64, 58, 86)).stroke_width(1);
    let light_grid = ShapeStyle::from(&RGBColor(42, 38, 60)).stroke_width(1);
    let use_days = state.use_days;
    let x_label_formatter = move |value: &f64| {
        if use_days {
            format!("Day {}", value.round() as i64)
        } else {
            Local
                .timestamp_millis_opt(*value as i64)
                .single()
                .map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| format!("{:.0}", value))
        }
    };

    chart
        .configure_mesh()
        .x_labels(6)
        .y_labels(6)
        .x_label_formatter(&x_label_formatter)
        .label_style(label_style)
        .axis_desc_style(axis_desc_style)
        .axis_style(&RGBColor(96, 86, 120))
        .bold_line_style(bold_grid)
        .light_line_style(light_grid)
        .x_desc(state.x_label.clone())
        .y_desc(state.y_label.clone())
        .draw()
        .ok();

    let mut line_map: std::collections::HashMap<RGBColor, Vec<(f64, f64)>> =
        std::collections::HashMap::new();
    for point in &state.points {
        line_map
            .entry(point.color)
            .or_default()
            .push((point.x, point.y));
    }
    for (color, mut series) in line_map {
        series.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        chart.draw_series(LineSeries::new(series, &color)).ok();
    }

    for point in &state.points {
        chart
            .draw_series(std::iter::once(Circle::new(
                (point.x, point.y),
                10,
                point.color.filled(),
            )))
            .ok();
    }

    for point in &state.dosage_points {
        chart
            .draw_series(std::iter::once(TriangleMarker::new(
                (point.x, point.y),
                16,
                point.color.filled(),
            )))
            .ok();
    }

    backend.present().ok();
}
