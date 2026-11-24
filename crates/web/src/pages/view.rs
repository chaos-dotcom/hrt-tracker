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
    antiandrogen_label, bloodtest_pdf_url, dosage_entry_date, dosage_entry_matches_key,
    dosage_photo_view, dose_mg, hormone_unit_labels, injectable_estradiol_label,
    injection_site_from_label, injection_site_label, length_unit_label, measurement_key,
    measurement_matches_target, metabolic_rows, oral_estradiol_label, parse_date_only,
    parse_datetime_local,
    parse_optional_num, parse_weight_unit, pill_detail, progesterone_label,
    progesterone_route_label, progesterone_route_word, regimen_summary_antiandrogen,
    remove_history_entry,
    regimen_summary_injection, regimen_summary_oral_estradiol, regimen_summary_progesterone,
    shows_liver_markers, syringe_kind_label, to_local_input_value, update_photo_note,
    weight_unit_label,
};
use self::scheduling::{generate_estrannaise_url, get_next_scheduled_candidate};
use self::types::{
    RegimenKey, UploadResponse, DAY_MS, INJECTION_SITE_OPTIONS, SYRINGE_KIND_OPTIONS,
};

use crate::charts::view::{
    compute_view_chart_state, displayed_value, draw_view_chart, find_nearest_point, layout_panels,
    plan_y_axis, plotted_y_range, series_present, ChartOptions, ChartSeries, E2DisplayUnit,
    ViewChartPoint, ViewChartState,
};
use crate::charts::{
    chart_padding, clamp_zoom, compute_chart_bounds, data_to_canvas_x, DragState, ViewZoom,
};
use plotters::style::RGBColor;
use crate::layout::page_layout;
use crate::store::use_store;
use crate::utils::{
    compute_fudge_factor, fmt_blood_value, fmt_date_label, fmt_decimal, format_injectable_dose,
    hormone_unit_label, injectable_dose_from_iu, parse_hormone_unit, parse_length_unit,
};
use hrt_shared::logic::{uses_oral_estradiol, predict_e2_pg_ml, snap_to_next_injection_boundary};
use hrt_shared::types::{
    DiaryEntry, DosageHistoryEntry, DosagePhoto, HormoneUnits, HrtData, ProgesteroneRoutes,
};

// ---------------------------------------------------------------------------
// Chart interaction
//
// The chart itself is drawn by `charts::view`. Everything here is the arithmetic
// that drives it from the page: the pixel geometry of the plot, snapping the
// crosshair to an x, the pinned readout, brush-to-zoom and the table rows. None
// of it touches js_sys, so it is all covered by the tests at the bottom of this
// file.
// ---------------------------------------------------------------------------

/// How far a drag has to travel before it counts as a brush rather than a click.
const BRUSH_MIN_DRAG_PX: f64 = 6.0;

/// Width below which the selection rectangle is not drawn at all.
const BRUSH_MIN_WIDTH_PX: f64 = 2.0;

/// A drag across the plot, in canvas pixels. Panning keeps the older
/// `DragState` (it carries the window the drag started from); brushing only
/// needs the two ends.
#[derive(Clone, Copy, Debug, PartialEq)]
struct BrushState {
    start_x: f64,
    current_x: f64,
}

impl BrushState {
    fn span(self) -> f64 {
        (self.current_x - self.start_x).abs()
    }
}

/// The unit estradiol is drawn in: the chip the user picked, or the unit their
/// settings already put the series in when they have not picked one.
///
/// `compute_view_chart_state` normalises estradiol into the settings unit and
/// stores it as `state.e2_unit`, so seeding the options from it is what keeps a
/// pg/mL user on pg/mL.
fn effective_e2_unit(settings_unit: E2DisplayUnit, chosen: Option<E2DisplayUnit>) -> E2DisplayUnit {
    chosen.unwrap_or(settings_unit)
}

#[allow(clippy::too_many_arguments)]
fn chart_options_from(
    state: &ViewChartState,
    chosen_e2_unit: Option<E2DisplayUnit>,
    log_scale: bool,
    dose_marks: bool,
    value_band: bool,
    small_multiples: bool,
) -> ChartOptions {
    ChartOptions {
        e2_unit: effective_e2_unit(state.e2_unit, chosen_e2_unit),
        log_scale,
        show_dose_ticks: dose_marks,
        show_value_band: value_band,
        separate_panels: small_multiples,
    }
}

/// An `aria-pressed` value for a toggle chip.
fn pressed(value: bool) -> &'static str {
    if value {
        "true"
    } else {
        "false"
    }
}

/// The pixel geometry of the plot area inside the canvas, in CSS pixels. It
/// mirrors what `draw_view_chart` lays out for a single panel: the same padding,
/// the same zoom window.
#[derive(Clone, Copy, Debug, PartialEq)]
struct ChartGeometry {
    left: f64,
    top: f64,
    width: f64,
    height: f64,
    x_min: f64,
    x_max: f64,
}

impl ChartGeometry {
    fn x_span(self) -> f64 {
        (self.x_max - self.x_min).abs().max(1.0)
    }

    fn x_to_px(self, x: f64) -> f64 {
        data_to_canvas_x(x, self.x_min, self.x_span(), self.width, self.left)
    }

    fn px_to_x(self, px: f64) -> f64 {
        self.x_min + ((px - self.left) / self.width) * self.x_span()
    }

    fn right(self) -> f64 {
        self.left + self.width
    }
}

fn chart_geometry(
    state: &ViewChartState,
    zoom: ViewZoom,
    canvas_width: f64,
    canvas_height: f64,
) -> ChartGeometry {
    let padding = chart_padding();
    let x_min = zoom.x_min.unwrap_or(state.domain_min);
    let x_max = zoom.x_max.unwrap_or(state.domain_max);
    let (width, height, _, _) = compute_chart_bounds(
        canvas_width,
        canvas_height,
        padding,
        x_min,
        x_max,
        state.y_min,
        state.y_max,
    );
    ChartGeometry {
        left: padding.0,
        top: padding.1,
        width,
        height,
        x_min,
        x_max,
    }
}

/// The x column nearest the cursor.
///
/// `find_nearest_point` does the y-aware snap while the pointer is on a point;
/// past its 48px hit radius the crosshair still follows the nearest column,
/// which is what a readout needs. Recovering the data x from the returned pixel
/// keeps this using exactly the hit test the chart already had.
fn snapped_x(
    state: &ViewChartState,
    options: &ChartOptions,
    geometry: ChartGeometry,
    cursor_x: f64,
    cursor_y: f64,
) -> Option<f64> {
    let padding = chart_padding();
    let (y_min, y_max) = plotted_y_range(state, options);
    let y_span = (y_max - y_min).abs().max(1.0);
    let mut best: Option<(f64, f64)> = None;
    for points in [&state.points, &state.dosage_points] {
        if let Some((tooltip, distance)) = find_nearest_point(
            points,
            geometry.x_min,
            geometry.x_span(),
            y_min,
            y_span,
            geometry.width,
            geometry.height,
            padding,
            cursor_x,
            cursor_y,
        ) {
            let x = geometry.px_to_x(tooltip.x);
            // A point outside a zoomed window is behind the axis, not on it.
            if x < geometry.x_min || x > geometry.x_max {
                continue;
            }
            if best.map(|(best_distance, _)| distance < best_distance).unwrap_or(true) {
                best = Some((distance, x));
            }
        }
    }
    match best {
        Some((_, x)) => Some(x),
        None => nearest_column_x(state, geometry, cursor_x),
    }
}

/// Fallback snap: the plotted x closest to the cursor horizontally, whatever
/// the y. Ties keep the first point in chart order.
fn nearest_column_x(
    state: &ViewChartState,
    geometry: ChartGeometry,
    cursor_x: f64,
) -> Option<f64> {
    let cursor_x = cursor_x.clamp(geometry.left, geometry.right());
    let mut best: Option<(f64, f64)> = None;
    for point in state.points.iter().chain(state.dosage_points.iter()) {
        if !point.x.is_finite() || point.x < geometry.x_min || point.x > geometry.x_max {
            continue;
        }
        let distance = (geometry.x_to_px(point.x) - cursor_x).abs();
        if best.map(|(best_distance, _)| distance < best_distance).unwrap_or(true) {
            best = Some((distance, point.x));
        }
    }
    best.map(|(_, x)| x)
}

fn css_color(color: RGBColor) -> String {
    format!("#{:02x}{:02x}{:02x}", color.0, color.1, color.2)
}

fn chart_series_css_color(series: ChartSeries) -> String {
    css_color(series.color())
}

/// A chart value at readout precision: two decimals, trailing zeros trimmed, so
/// 340.0 reads as `340` and 90.44 as `90.44`.
fn format_chart_value(value: f64) -> String {
    if !value.is_finite() {
        return "-".to_string();
    }
    fmt_decimal(value, 2)
}

/// The unit a record was made in, when the plotted value was converted out of
/// it. The chart's own labels read
/// `"Estradiol: 340 pmol/L -> 90.4 pg/mL (Nov 23)"`, so the part before the
/// arrow is the recorded unit. A label with no conversion has no note.
fn recorded_note(label: &str) -> Option<String> {
    let (before, _) = label.split_once(" -> ")?;
    let (_, recorded) = before.split_once(": ")?;
    let recorded = recorded.trim();
    if recorded.is_empty() {
        None
    } else {
        Some(format!("recorded {recorded}"))
    }
}

/// The unit a point is drawn in: the estradiol chip's unit, or the series' own
/// recorded unit. The readout and the table both go through this, so the two can
/// never disagree after a unit switch.
fn plotted_unit(point: &ViewChartPoint, options: &ChartOptions) -> String {
    if point.series == ChartSeries::Estradiol {
        options.e2_unit.label().to_string()
    } else {
        point.unit.clone()
    }
}

/// One line of the pinned readout: a series and the value it is drawn at.
#[derive(Clone, Debug, PartialEq)]
struct ReadoutRow {
    series: ChartSeries,
    value: String,
    unit: String,
    /// What the record said, when it was recorded in another unit.
    recorded: Option<String>,
    color: String,
}

/// One row per visible series that has a point at `x`, in the chart's own
/// series order, with the value in the unit it is drawn in.
///
/// Doses are not levels and are drawn at a scaled height, so a dose column
/// reports the dose's own text instead of a number.
fn readout_rows(state: &ViewChartState, options: &ChartOptions, x: f64) -> Vec<ReadoutRow> {
    let mut rows = Vec::new();
    for series in series_present(&state.points) {
        let Some(point) = state
            .points
            .iter()
            .find(|point| point.series == series && (point.x - x).abs() < 1e-6)
        else {
            continue;
        };
        rows.push(ReadoutRow {
            series,
            value: format_chart_value(displayed_value(point, state, options)),
            unit: plotted_unit(point, options),
            recorded: recorded_note(&point.label),
            color: chart_series_css_color(series),
        });
    }
    if let Some(dose) = state
        .dosage_points
        .iter()
        .find(|point| (point.x - x).abs() < 1e-6)
    {
        rows.push(ReadoutRow {
            series: ChartSeries::Dose,
            value: dose_text(dose),
            unit: String::new(),
            recorded: None,
            color: css_color(dose.color),
        });
    }
    rows
}

/// A dose marker's label without its trailing date, since the readout names the
/// date once for the whole column.
fn dose_text(dose: &ViewChartPoint) -> String {
    match dose.label.rsplit_once(" (") {
        Some((text, _)) => text.to_string(),
        None => dose.label.clone(),
    }
}

/// The date column heading for a readout.
fn readout_date_text(state: &ViewChartState, x: f64) -> String {
    let Some(point) = state
        .points
        .iter()
        .chain(state.dosage_points.iter())
        .find(|point| (point.x - x).abs() < 1e-6)
    else {
        return String::new();
    };
    let axis_mode = if state.use_days { "days" } else { "date" };
    fmt_date_label(point.timestamp_ms, axis_mode, state.first_dose)
}

/// How far up an axis a value sits, 0.0 at the floor and 1.0 at the top.
/// Logarithmic axes space it geometrically, which is where the chart draws it.
fn y_fraction(value: f64, axis_min: f64, axis_max: f64, logarithmically: bool) -> f64 {
    if !value.is_finite() || !axis_min.is_finite() || !axis_max.is_finite() || axis_max <= axis_min {
        return 0.5;
    }
    let fraction = if logarithmically {
        if axis_min <= 0.0 || value <= 0.0 {
            0.0
        } else {
            (value / axis_min).ln() / (axis_max / axis_min).ln()
        }
    } else {
        (value - axis_min) / (axis_max - axis_min)
    };
    fraction.clamp(0.0, 1.0)
}

#[derive(Clone, Debug, PartialEq)]
struct CrosshairDot {
    x: f64,
    y: f64,
    color: String,
}

/// Where the crosshair dots go: one per plotted point at `x`, inside the panel
/// that point belongs to. Small multiples split the canvas into equal panels,
/// each with its own axis, so the dot has to use that panel's range.
fn crosshair_dots(
    state: &ViewChartState,
    options: &ChartOptions,
    geometry: ChartGeometry,
    x: f64,
) -> Vec<CrosshairDot> {
    let padding = chart_padding();
    let panels = layout_panels(state, options);
    let count = panels.len().max(1);
    let canvas_height = geometry.height + padding.1 + padding.3;
    let panel_height = canvas_height / count as f64;
    let mut dots = Vec::new();
    for (index, panel) in panels.iter().enumerate() {
        let panel_top = index as f64 * panel_height;
        let is_bottom = index + 1 == count;
        let plot_top = panel_top + padding.1;
        let plot_bottom = panel_top + panel_height
            - if is_bottom { padding.3 } else { padding.1 };
        let plot_height = (plot_bottom - plot_top).max(1.0);
        let values: Vec<f64> = panel.points.iter().map(|point| point.y).collect();
        let plan = plan_y_axis(options.log_scale, panel.y_min, panel.y_max, &values);
        let (axis_min, axis_max) = plan.bounds();
        for point in panel
            .points
            .iter()
            .filter(|point| (point.x - x).abs() < 1e-6)
        {
            let fraction = y_fraction(point.y, axis_min, axis_max, plan.is_log());
            dots.push(CrosshairDot {
                x: geometry.x_to_px(point.x),
                y: plot_bottom - fraction * plot_height,
                color: css_color(point.color),
            });
        }
    }
    dots
}

/// The crosshair line, its dots, and the brush rectangle, all in canvas pixels.
#[derive(Clone, Debug, PartialEq)]
struct ChartOverlay {
    /// `(left, top, height)` of the vertical line.
    crosshair: Option<(f64, f64, f64)>,
    dots: Vec<CrosshairDot>,
    /// `(left, width)` of the selection rectangle.
    brush: Option<(f64, f64)>,
}

fn chart_overlay(
    state: &ViewChartState,
    options: &ChartOptions,
    zoom: ViewZoom,
    canvas_width: f64,
    canvas_height: f64,
    hover_x: Option<f64>,
    brush: Option<BrushState>,
) -> ChartOverlay {
    let padding = chart_padding();
    let geometry = chart_geometry(state, zoom, canvas_width, canvas_height);
    let plot_height = (canvas_height - padding.1 - padding.3).max(1.0);
    // A crosshair only means something inside the window being drawn.
    let hover_x = hover_x.filter(|x| *x >= geometry.x_min && *x <= geometry.x_max);
    ChartOverlay {
        crosshair: hover_x.map(|x| (geometry.x_to_px(x), padding.1, plot_height)),
        dots: hover_x
            .map(|x| crosshair_dots(state, options, geometry, x))
            .unwrap_or_default(),
        brush: brush.and_then(|brush| brush_rect(geometry, brush)),
    }
}

/// The selection rectangle a brush covers, clamped to the plot area, or `None`
/// while the drag is too short to be anything but a click.
fn brush_rect(geometry: ChartGeometry, brush: BrushState) -> Option<(f64, f64)> {
    let clamp = |px: f64| px.clamp(geometry.left, geometry.right());
    let left = clamp(brush.start_x.min(brush.current_x));
    let right = clamp(brush.start_x.max(brush.current_x));
    if right - left < BRUSH_MIN_WIDTH_PX {
        None
    } else {
        Some((left, right - left))
    }
}

/// The x window a drag covers, in data units, always ordered low to high.
fn brush_window(geometry: ChartGeometry, from_px: f64, to_px: f64) -> (f64, f64) {
    let clamp = |px: f64| px.clamp(geometry.left, geometry.right());
    let (a, b) = (clamp(from_px), clamp(to_px));
    if a <= b {
        (geometry.px_to_x(a), geometry.px_to_x(b))
    } else {
        (geometry.px_to_x(b), geometry.px_to_x(a))
    }
}

/// Zoom to the brushed window, through `clamp_zoom` so a drag across the whole
/// domain falls back to the full domain instead of pinning a tiny window.
fn brush_zoom(
    state: &ViewChartState,
    geometry: ChartGeometry,
    from_px: f64,
    to_px: f64,
) -> ViewZoom {
    let (min, max) = brush_window(geometry, from_px, to_px);
    clamp_zoom(state.domain_min, state.domain_max, min, max)
}

/// The zoom a finished drag asks for, or `None` when the pointer barely moved
/// and the gesture was a click rather than a brush.
fn brush_zoom_for_drag(
    state: &ViewChartState,
    geometry: ChartGeometry,
    brush: BrushState,
) -> Option<ViewZoom> {
    if brush.span() < BRUSH_MIN_DRAG_PX {
        return None;
    }
    Some(brush_zoom(state, geometry, brush.start_x, brush.current_x))
}

/// The pinned readout for one x: the date, every series at that x, and where
/// the crosshair dots land.
#[derive(Clone, Debug, PartialEq)]
struct ChartReadout {
    x_px: f64,
    date: String,
    rows: Vec<ReadoutRow>,
    dots: Vec<CrosshairDot>,
}

fn chart_readout(
    state: &ViewChartState,
    options: &ChartOptions,
    zoom: ViewZoom,
    canvas_width: f64,
    canvas_height: f64,
    x: f64,
) -> Option<ChartReadout> {
    let geometry = chart_geometry(state, zoom, canvas_width, canvas_height);
    let rows = readout_rows(state, options, x);
    if rows.is_empty() {
        return None;
    }
    Some(ChartReadout {
        x_px: geometry.x_to_px(x),
        date: readout_date_text(state, x),
        rows,
        dots: crosshair_dots(state, options, geometry, x),
    })
}

/// One row of the table view: a plotted blood value in the unit it is drawn in.
#[derive(Clone, Debug, PartialEq)]
struct ChartTableRow {
    date: String,
    series: ChartSeries,
    value: String,
    unit: String,
    color: String,
    timestamp_ms: i64,
}

/// The plotted blood values, newest first, in the unit each series is drawn in,
/// so the table and the canvas always agree, including after a unit switch.
fn chart_table_rows(state: &ViewChartState, options: &ChartOptions) -> Vec<ChartTableRow> {
    let axis_mode = if state.use_days { "days" } else { "date" };
    let mut rows: Vec<(usize, ChartTableRow)> = state
        .points
        .iter()
        .map(|point| {
            let order = ChartSeries::ALL
                .iter()
                .position(|series| *series == point.series)
                .unwrap_or(usize::MAX);
            (
                order,
                ChartTableRow {
                    date: fmt_date_label(point.timestamp_ms, axis_mode, state.first_dose),
                    series: point.series,
                    value: format_chart_value(displayed_value(point, state, options)),
                    unit: plotted_unit(point, options),
                    color: chart_series_css_color(point.series),
                    timestamp_ms: point.timestamp_ms,
                },
            )
        })
        .collect();
    rows.sort_by(|(a_order, a), (b_order, b)| {
        b.timestamp_ms
            .cmp(&a.timestamp_ms)
            .then(a_order.cmp(b_order))
    });
    rows.into_iter().map(|(_, row)| row).collect()
}

#[component]
pub fn ViewPage() -> impl IntoView {
    let store = use_store();
    let data = store.data;
    let rows = move || {
        let mut entries = data.get().dosageHistory.clone();
        entries.sort_by_key(|e| std::cmp::Reverse(dosage_entry_date(e)));
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
    let show_fudge_factor = create_rw_signal(true);
    let view_zoom = create_rw_signal(ViewZoom::default());
    // Chart chips. The estradiol unit starts from whatever the settings put the
    // series in (see `effective_e2_unit`); there is no settings field behind the
    // other four, so they live here and reset on reload.
    let chart_e2_unit = create_rw_signal(None::<E2DisplayUnit>);
    let chart_log_scale = create_rw_signal(false);
    let chart_dose_marks = create_rw_signal(false);
    let chart_value_band = create_rw_signal(false);
    let chart_small_multiples = create_rw_signal(false);
    let chart_table_view = create_rw_signal(false);
    // Hover and drag state for the crosshair, the pinned readout and the brush.
    let view_hover_x = create_rw_signal(None::<f64>);
    // Where the pointer is, for the floating tooltip that follows it.
    let view_hover_pos = create_rw_signal(None::<(f64, f64)>);
    let view_brush = create_rw_signal(None::<BrushState>);
    let chart_canvas_size = create_rw_signal((900.0_f64, 420.0_f64));

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
                show_fudge_factor.get(),
            )
        }
    });

    let chart_options = create_memo(move |_| {
        let state = view_chart_state.get();
        chart_options_from(
            &state,
            chart_e2_unit.get(),
            chart_log_scale.get(),
            chart_dose_marks.get(),
            chart_value_band.get(),
            chart_small_multiples.get(),
        )
    });

    // Recomputed from the hovered x, so a unit switch rewrites the readout and
    // the table with it instead of leaving stale numbers behind.
    let chart_readout_view = create_memo(move |_| {
        let x = view_hover_x.get()?;
        let state = view_chart_state.get();
        let options = chart_options.get();
        let (width, height) = chart_canvas_size.get();
        chart_readout(&state, &options, view_zoom.get(), width, height, x)
    });

    let chart_table_view_rows = create_memo(move |_| {
        let state = view_chart_state.get();
        chart_table_rows(&state, &chart_options.get())
    });

    let first_dose_date =
        create_memo(move |_| data.get().dosageHistory.iter().map(dosage_entry_date).min());

    let days_since_first_dose = create_memo(move |_| {
        first_dose_date.get().map(|first| {
            let now = Date::now() as i64;
            (now - first).abs() / DAY_MS
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

    // A tablet is swallowed, so it reaches the liver before the blood; an
    // injection does not. That is the only reason to watch the liver panel.
    let on_tablets = create_memo(move |_| uses_oral_estradiol(&data.get()));

    let show_liver_panel = create_memo(move |_| {
        shows_liver_markers(&data.get().bloodTests, on_tablets.get())
    });

    let panel_rows = create_memo(move |_| {
        metabolic_rows(&data.get().bloodTests, show_liver_panel.get())
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

    // The dashboard button used to write silently; the shared toast (with Undo)
    // now reports exactly what it wrote.
    let dose_toast = create_rw_signal(None::<(String, String)>);
    let dose_toast_timeout: Rc<RefCell<Option<gloo_timers::callback::Timeout>>> =
        Rc::new(RefCell::new(None));

    let undo_dose_toast: StoredValue<Rc<dyn Fn()>> = StoredValue::new({
        let store = store.clone();
        Rc::new(move || {
            let Some((_, entry_id)) = dose_toast.get() else {
                return;
            };
            store.data.update(|data| {
                remove_history_entry(&mut data.dosageHistory, &entry_id);
            });
            store.mark_dirty();
            store.save();
            dose_toast.set(None);
        })
    });

    let record_next_dose_now = {
        let store = store.clone();
        let dose_toast_timeout = dose_toast_timeout.clone();
        move |_| {
            let candidate = next_scheduled_candidate.get();
            let Some(candidate) = candidate else {
                return;
            };
            let now = Date::now() as i64;
            let entry_id = format!("dose-{now}-{}", (Math::random() * 1_000_000.0) as i64);
            store.data.update(|data| match candidate.med_type {
                RegimenKey::InjectableEstradiol => {
                    if let Some(cfg) = data.injectableEstradiol.as_mut() {
                        let record = DosageHistoryEntry::InjectableEstradiol {
                            date: now,
                            id: Some(entry_id.clone()),
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
                            id: Some(entry_id.clone()),
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
                            id: Some(entry_id.clone()),
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
                            id: Some(entry_id.clone()),
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

            dose_toast.set(Some((format!("Logged {}", candidate.label), entry_id)));
            if let Some(existing) = dose_toast_timeout.borrow_mut().take() {
                drop(existing);
            }
            *dose_toast_timeout.borrow_mut() =
                Some(gloo_timers::callback::Timeout::new(6000, move || {
                    dose_toast.set(None);
                }));
        }
    };

    const VIEW_CANVAS_ID: &str = "view-chart-canvas";
    let view_pan_drag = Rc::new(RefCell::new(None::<DragState>));

    /// The canvas' own size in CSS pixels.
    fn canvas_metrics(canvas_id: &str) -> Option<(f64, f64)> {
        let canvas = window()
            .document()?
            .get_element_by_id(canvas_id)?
            .dyn_into::<HtmlCanvasElement>()
            .ok()?;
        let rect = canvas.get_bounding_client_rect();
        Some((rect.width(), rect.height()))
    }

    /// `(canvas width, canvas height, cursor x, cursor y)`, cursor in canvas
    /// pixels.
    fn cursor_in_canvas(canvas_id: &str, ev: &leptos::ev::MouseEvent) -> Option<(f64, f64, f64, f64)> {
        let canvas = window()
            .document()?
            .get_element_by_id(canvas_id)?
            .dyn_into::<HtmlCanvasElement>()
            .ok()?;
        let rect = canvas.get_bounding_client_rect();
        Some((
            rect.width(),
            rect.height(),
            ev.client_x() as f64 - rect.left(),
            ev.client_y() as f64 - rect.top(),
        ))
    }

    /// The same, for a wheel event (its own position accessors).
    fn cursor_in_canvas_wheel(canvas_id: &str, ev: &leptos::ev::WheelEvent) -> Option<(f64, f64, f64, f64)> {
        let canvas = window()
            .document()?
            .get_element_by_id(canvas_id)?
            .dyn_into::<HtmlCanvasElement>()
            .ok()?;
        let rect = canvas.get_bounding_client_rect();
        Some((
            rect.width(),
            rect.height(),
            ev.client_x() as f64 - rect.left(),
            ev.client_y() as f64 - rect.top(),
        ))
    }

    let on_view_mouse_move = StoredValue::new({
        let view_pan_drag = view_pan_drag.clone();
        move |ev: leptos::ev::MouseEvent| {
            let Some((width, height, cursor_x, cursor_y)) =
                cursor_in_canvas(VIEW_CANVAS_ID, &ev)
            else {
                return;
            };
            chart_canvas_size.set((width, height));
            let state = view_chart_state.get();
            let zoom = view_zoom.get();
            if let Some(drag) = view_pan_drag.borrow().as_ref() {
                // Shift-drag pans: the window the drag started from moves with
                // the pointer, measured in plot pixels.
                view_hover_x.set(None);
                view_hover_pos.set(None);
                let geometry = chart_geometry(&state, zoom, width, height);
                let delta_px = cursor_x - drag.start_x;
                let span = drag.start_max - drag.start_min;
                let delta_domain = -(delta_px / geometry.width) * span;
                view_zoom.set(clamp_zoom(
                    state.domain_min,
                    state.domain_max,
                    drag.start_min + delta_domain,
                    drag.start_max + delta_domain,
                ));
                return;
            }
            if let Some(brush) = view_brush.get() {
                view_brush.set(Some(BrushState {
                    current_x: cursor_x,
                    ..brush
                }));
                return;
            }
            let geometry = chart_geometry(&state, zoom, width, height);
            let options = chart_options.get();
            view_hover_x.set(snapped_x(&state, &options, geometry, cursor_x, cursor_y));
            view_hover_pos.set(Some((cursor_x, cursor_y)));
        }
    });

    let on_view_mouse_leave = StoredValue::new({
        let view_pan_drag = view_pan_drag.clone();
        move |_| {
            view_pan_drag.replace(None);
            view_brush.set(None);
            view_hover_x.set(None);
            view_hover_pos.set(None);
        }
    });

    let on_view_mouse_down = StoredValue::new({
        let view_pan_drag = view_pan_drag.clone();
        move |ev: leptos::ev::MouseEvent| {
            let Some((width, height, cursor_x, _)) = cursor_in_canvas(VIEW_CANVAS_ID, &ev) else {
                return;
            };
            chart_canvas_size.set((width, height));
            if ev.shift_key() {
                let state = view_chart_state.get();
                let zoom = view_zoom.get();
                view_brush.set(None);
                view_pan_drag.replace(Some(DragState {
                    start_x: cursor_x,
                    start_min: zoom.x_min.unwrap_or(state.domain_min),
                    start_max: zoom.x_max.unwrap_or(state.domain_max),
                }));
            } else {
                view_pan_drag.replace(None);
                view_brush.set(Some(BrushState {
                    start_x: cursor_x,
                    current_x: cursor_x,
                }));
            }
        }
    });

    let on_view_mouse_up = StoredValue::new({
        let view_pan_drag = view_pan_drag.clone();
        move |ev: leptos::ev::MouseEvent| {
            view_pan_drag.replace(None);
            let Some(brush) = view_brush.get() else {
                return;
            };
            view_brush.set(None);
            let Some((width, height, _, _)) = cursor_in_canvas(VIEW_CANVAS_ID, &ev) else {
                return;
            };
            let state = view_chart_state.get();
            let geometry = chart_geometry(&state, view_zoom.get(), width, height);
            if let Some(zoom) = brush_zoom_for_drag(&state, geometry, brush) {
                view_zoom.set(zoom);
            }
        }
    });

    let on_view_double_click = StoredValue::new({
        let view_pan_drag = view_pan_drag.clone();
        move |_| {
            view_pan_drag.replace(None);
            view_brush.set(None);
            view_zoom.set(ViewZoom::default());
        }
    });

    let on_view_wheel = StoredValue::new({
        move |ev: leptos::ev::WheelEvent| {
            ev.prevent_default();
            let Some((width, height, cursor_x, _)) = cursor_in_canvas_wheel(VIEW_CANVAS_ID, &ev)
            else {
                return;
            };
            chart_canvas_size.set((width, height));
            let state = view_chart_state.get();
            let geometry = chart_geometry(&state, view_zoom.get(), width, height);
            let cursor_ratio = ((cursor_x - geometry.left) / geometry.width).clamp(0.0, 1.0);
            let zoom_factor = if ev.delta_y() < 0.0 { 0.85 } else { 1.15 };
            let span = geometry.x_span();
            let new_span = (span * zoom_factor).max(1.0);
            let center = geometry.x_min + span * cursor_ratio;
            let new_min = center - new_span * cursor_ratio;
            view_zoom.set(clamp_zoom(
                state.domain_min,
                state.domain_max,
                new_min,
                new_min + new_span,
            ));
        }
    });

    create_effect({
        let canvas_size = chart_canvas_size;
        move |_| {
            let state = view_chart_state.get();
            let options = chart_options.get();
            let zoom = view_zoom.get();
            // Reading the table toggle keeps this effect re-running when the canvas
            // comes back into the page.
            if chart_table_view.get() || !state.has_data {
                return;
            }
            if let Some(size) = canvas_metrics(VIEW_CANVAS_ID) {
                canvas_size.set(size);
            }
            draw_view_chart(VIEW_CANVAS_ID, &state, zoom, &options);
            // This effect can run before the rows above the canvas have settled
            // their layout, which leaves the canvas backing store sized from a
            // stale rectangle (a blurred chart until something forces a resize).
            // Draw once more on the next frame, when the box is final.
            let state = view_chart_state;
            let options = chart_options;
            let zoom = view_zoom;
            let canvas_size = canvas_size;
            let _ = window().request_animation_frame(
                wasm_bindgen::closure::Closure::once_into_js(move || {
                    let state = state.get();
                    if chart_table_view.get_untracked() || !state.has_data {
                        return;
                    }
                    if let Some(size) = canvas_metrics(VIEW_CANVAS_ID) {
                        canvas_size.set(size);
                    }
                    draw_view_chart(VIEW_CANVAS_ID, &state, zoom.get_untracked(), &options.get_untracked());
                })
                .unchecked_ref(),
            );
        }
    });

    let view_resize_listener: Rc<RefCell<Option<EventListener>>> = Rc::new(RefCell::new(None));
    create_effect({
        let view_resize_listener = view_resize_listener.clone();
        move |_| {
            view_chart_state.get();
            chart_options.get();
            let window = window();
            let listener = EventListener::new(&window, "resize", move |_| {
                let state = view_chart_state.get();
                if chart_table_view.get() || !state.has_data {
                    return;
                }
                if let Some(size) = canvas_metrics(VIEW_CANVAS_ID) {
                    chart_canvas_size.set(size);
                }
                draw_view_chart(VIEW_CANVAS_ID, &state, view_zoom.get(), &chart_options.get());
            });
            view_resize_listener.replace(Some(listener));
        }
    });

    let reset_view_zoom = move |_| {
        view_brush.set(None);
        view_zoom.set(ViewZoom::default());
    };

    let x_axis_days_disabled = move || view_chart_state.get().first_dose.is_none();
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
                if let DosageHistoryEntry::InjectableEstradiol {
                    photos: Some(items),
                    ..
                } = entry
                {
                    output.extend(items.iter().map(dosage_photo_view));
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
                let iu_conversion_data = HrtData {
                    vials: d.vials.clone(),
                    ..Default::default()
                };
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
                <div class="toast" role="status" hidden=move || dose_toast.get().is_none()>
                    <span>{move || dose_toast.get().map(|(message, _)| message).unwrap_or_default()}</span>
                    <button
                        class="toast-undo"
                        type="button"
                        on:click=move |_| undo_dose_toast.with_value(|undo| undo())
                    >
                        "Undo"
                    </button>
                </div>
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
                            <A href="/create/dosage/schedule">"Edit Schedule"</A>
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
                                            regimen_summary_injection(&cfg.kind, &dose_label, cfg.frequency)
                                        })
                                        .unwrap_or_default()
                                }}
                            </p>
                        </Show>
                        <Show when=move || store.data.get().oralEstradiol.is_some()>
                            <p>
                                {move || store
                                    .data
                                    .get()
                                    .oralEstradiol
                                    .as_ref()
                                    .map(|cfg| regimen_summary_oral_estradiol(cfg.dose, cfg.frequency))
                                    .unwrap_or_default()}
                            </p>
                        </Show>
                        <Show when=move || store.data.get().antiandrogen.is_some()>
                            <p>
                                {move || store
                                    .data
                                    .get()
                                    .antiandrogen
                                    .as_ref()
                                    .map(|cfg| regimen_summary_antiandrogen(&cfg.kind, cfg.dose, cfg.frequency))
                                    .unwrap_or_default()}
                            </p>
                        </Show>
                        <Show when=move || store.data.get().progesterone.is_some()>
                            <p>
                                {move || store
                                    .data
                                    .get()
                                    .progesterone
                                    .as_ref()
                                    .map(|cfg| regimen_summary_progesterone(&cfg.route, cfg.dose, cfg.frequency))
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
                                prop:disabled=x_axis_days_disabled
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

                    <div class="chart-toolbar view-chart-options">
                        <span class="muted">"Estradiol unit:"</span>
                        <div class="chip-row" role="group" aria-label="Estradiol display unit">
                            <button
                                type="button"
                                class="chip"
                                class:active=move || chart_options.get().e2_unit == E2DisplayUnit::PmolL
                                aria-pressed=move || pressed(chart_options.get().e2_unit == E2DisplayUnit::PmolL)
                                on:click=move |_| chart_e2_unit.set(Some(E2DisplayUnit::PmolL))
                            >
                                "pmol/L"
                            </button>
                            <button
                                type="button"
                                class="chip"
                                class:active=move || chart_options.get().e2_unit == E2DisplayUnit::PgMl
                                aria-pressed=move || pressed(chart_options.get().e2_unit == E2DisplayUnit::PgMl)
                                on:click=move |_| chart_e2_unit.set(Some(E2DisplayUnit::PgMl))
                            >
                                "pg/mL"
                            </button>
                        </div>
                        <div class="chip-row" role="group" aria-label="Chart display options">
                            <button
                                type="button"
                                class="chip"
                                class:active=move || chart_log_scale.get()
                                aria-pressed=move || pressed(chart_log_scale.get())
                                on:click=move |_| chart_log_scale.set(!chart_log_scale.get())
                            >
                                "Log scale"
                            </button>
                            <button
                                type="button"
                                class="chip"
                                class:active=move || chart_dose_marks.get()
                                aria-pressed=move || pressed(chart_dose_marks.get())
                                on:click=move |_| chart_dose_marks.set(!chart_dose_marks.get())
                            >
                                "Dose marks"
                            </button>
                            <button
                                type="button"
                                class="chip"
                                class:active=move || chart_value_band.get()
                                aria-pressed=move || pressed(chart_value_band.get())
                                on:click=move |_| chart_value_band.set(!chart_value_band.get())
                            >
                                "Recorded range"
                            </button>
                            <button
                                type="button"
                                class="chip"
                                class:active=move || chart_small_multiples.get()
                                aria-pressed=move || pressed(chart_small_multiples.get())
                                on:click=move |_| chart_small_multiples.set(!chart_small_multiples.get())
                            >
                                "Small multiples"
                            </button>
                            <button
                                type="button"
                                class="chip"
                                class:active=move || chart_table_view.get()
                                aria-pressed=move || pressed(chart_table_view.get())
                                on:click=move |_| chart_table_view.set(!chart_table_view.get())
                            >
                                "Table"
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
                        <button
                            class:active=move || show_fudge_factor.get()
                            on:click=move |_| show_fudge_factor.set(!show_fudge_factor.get())
                        >
                            "FF %"
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
                        <Show
                            when=move || !chart_table_view.get()
                            fallback=move || view! {
                                <div class="chart-table-scroll">
                                    <table class="table chart-table">
                                        <caption class="muted">
                                            {move || {
                                                let rows = chart_table_view_rows.get();
                                                format!(
                                                    "Blood test values as plotted, newest first ({} rows).",
                                                    rows.len()
                                                )
                                            }}
                                        </caption>
                                        <thead>
                                            <tr>
                                                <th scope="col">"Date"</th>
                                                <th scope="col">"Series"</th>
                                                <th scope="col">"Value"</th>
                                                <th scope="col">"Unit"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {move || {
                                                chart_table_view_rows
                                                    .get()
                                                    .into_iter()
                                                    .map(|row| {
                                                        view! {
                                                            <tr>
                                                                <td>{row.date}</td>
                                                                <td>
                                                                    <span
                                                                        class="chart-table-swatch"
                                                                        style=format!("background: {};", row.color)
                                                                    ></span>
                                                                    {row.series.label()}
                                                                </td>
                                                                <td class="chart-table-value">{row.value}</td>
                                                                <td>{row.unit}</td>
                                                            </tr>
                                                        }
                                                    })
                                                    .collect_view()
                                            }}
                                        </tbody>
                                    </table>
                                    <p class="chart-table-note muted">
                                        "Dose markers are drawn at a scaled height for visibility, so they are not tabulated here; the dosage page lists every dose."
                                    </p>
                                </div>
                            }
                        >
                            <div class="chart-canvas-wrap">
                                <canvas
                                    id=VIEW_CANVAS_ID
                                    width="900"
                                    height="420"
                                    on:mousemove=move |ev| on_view_mouse_move.with_value(|handler| handler(ev))
                                    on:mouseleave=move |ev| on_view_mouse_leave.with_value(|handler| handler(ev))
                                    on:mousedown=move |ev| on_view_mouse_down.with_value(|handler| handler(ev))
                                    on:mouseup=move |ev| on_view_mouse_up.with_value(|handler| handler(ev))
                                    on:dblclick=move |ev| on_view_double_click.with_value(|handler| handler(ev))
                                    on:wheel=move |ev| on_view_wheel.with_value(|handler| handler(ev))
                                ></canvas>
                                <Show when=move || {
                                    view_hover_pos.get().is_some() && chart_readout_view.get().is_some()
                                }>
                                    <div
                                        class="chart-tooltip"
                                        style=move || {
                                            view_hover_pos
                                                .get()
                                                .map(|(x, y)| format!("left: {:.0}px; top: {:.0}px;", x + 16.0, y + 14.0))
                                                .unwrap_or_default()
                                        }
                                    >
                                        {move || match chart_readout_view.get() {
                                            Some(readout) => {
                                                let date = readout.date.clone();
                                                let rows = readout
                                                    .rows
                                                    .into_iter()
                                                    .map(|row| {
                                                        view! {
                                                            <span class="chart-tooltip-row">
                                                                <span
                                                                    class="chart-tooltip-swatch"
                                                                    style=format!("background: {};", row.color)
                                                                ></span>
                                                                <span class="chart-tooltip-series">
                                                                    {row.series.label()}
                                                                </span>
                                                                <strong>{row.value}</strong>
                                                                <span class="chart-tooltip-unit">{row.unit}</span>
                                                            </span>
                                                        }
                                                    })
                                                    .collect_view();
                                                view! {
                                                    <span class="chart-tooltip-date">{date}</span>
                                                    {rows}
                                                }
                                                    .into_view()
                                            }
                                            None => ().into_view(),
                                        }}
                                    </div>
                                </Show>
                                <div class="chart-overlay" aria-hidden="true">
                                    {move || {
                                        let overlay = chart_overlay(
                                            &view_chart_state.get(),
                                            &chart_options.get(),
                                            view_zoom.get(),
                                            chart_canvas_size.get().0,
                                            chart_canvas_size.get().1,
                                            view_hover_x.get(),
                                            view_brush.get(),
                                        );
                                        let padding = chart_padding();
                                        view! {
                                            {overlay
                                                .crosshair
                                                .map(|(left, top, height)| {
                                                    view! {
                                                        <div
                                                            class="chart-crosshair"
                                                            style=format!(
                                                                "left: {left:.1}px; top: {top:.1}px; height: {height:.1}px;",
                                                            )
                                                        ></div>
                                                    }
                                                })}
                                            {overlay
                                                .dots
                                                .into_iter()
                                                .map(|dot| {
                                                    view! {
                                                        <div
                                                            class="chart-crosshair-dot"
                                                            style=format!(
                                                                "left: {:.1}px; top: {:.1}px; background: {};",
                                                                dot.x,
                                                                dot.y,
                                                                dot.color,
                                                            )
                                                        ></div>
                                                    }
                                                })
                                                .collect_view()}
                                            {overlay
                                                .brush
                                                .map(|(left, width)| {
                                                    view! {
                                                        <div
                                                            class="chart-brush"
                                                            style=format!(
                                                                "left: {left:.1}px; width: {width:.1}px; top: {}px; bottom: {}px;",
                                                                padding.1, padding.3,
                                                            )
                                                        ></div>
                                                    }
                                                })}
                                        }
                                    }}
                                </div>
                            </div>
                            <div class="chart-readout">
                                {move || match chart_readout_view.get() {
                                    Some(readout) => {
                                        let date = readout.date.clone();
                                        let rows = readout
                                            .rows
                                            .into_iter()
                                            .map(|row| {
                                                view! {
                                                    <span class="chart-readout-row">
                                                        <span
                                                            class="chart-readout-swatch"
                                                            style=format!("background: {};", row.color)
                                                        ></span>
                                                        <span class="chart-readout-series">
                                                            {row.series.label()}
                                                        </span>
                                                        <span class="chart-readout-value">{row.value}</span>
                                                        <span class="chart-readout-unit">{row.unit}</span>
                                                        {row
                                                            .recorded
                                                            .map(|note| {
                                                                view! {
                                                                    <span class="chart-readout-note">{note}</span>
                                                                }
                                                            })}
                                                    </span>
                                                }
                                            })
                                            .collect_view();
                                        view! {
                                            <span class="chart-readout-date">{date}</span>
                                            {rows}
                                        }
                                            .into_view()
                                    }
                                    None => {
                                        view! {
                                            <span class="muted">
                                                "Hover the chart to read a value. Drag to zoom, shift-drag to pan, double-click to reset."
                                            </span>
                                        }
                                            .into_view()
                                    }
                                }}
                            </div>
                        </Show>
                    </Show>

                    <div class="chart-note muted">
                        <p>"* Dosage values are scaled for visibility on the chart."</p>
                        <p>"* Hover for a readout, drag to zoom, shift-drag to pan, double-click to reset."</p>
                        <Show when=move || chart_small_multiples.get()>
                            <p>"* Small multiples draw one panel per series; the level chips above choose which panels exist."</p>
                        </Show>
                        <Show when=move || !store.data.get().bloodTests.is_empty()>
                            <p>"* Hormone measurements are normalized to display units for charting; hover shows recorded units."</p>
                        </Show>
                        <Show when=move || show_fudge_factor.get()>
                            <p>"* FF is shown as efficacy percent (fudge factor x 100)."</p>
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
                                    let lookup_id = note_id.clone();
                                    let current = create_memo(move |_| {
                                        sorted_notes.get().into_iter().find(|n| n.id == lookup_id).unwrap_or_else(|| DiaryEntry {
                                            id: String::new(),
                                            date: 0,
                                            title: None,
                                            content: String::new(),
                                        })
                                    });
                                    let is_editing = move || editing_note_id.get().as_ref() == Some(&note_id_for_edit);
                                    let note_id_value = StoredValue::new(note_id.clone());
                                    let save_edit_note = save_edit_note;
                                    view! {
                                        <li class="note-item">
                                            <Show when=is_editing fallback=move || view! {
                                                <div class="note-display">
                                                    <div>
                                                        <div class="note-date">{move || fmt_date_label(current.get().date, &x_axis_mode.get(), first_dose_date.get())}</div>
                                                        <Show when=move || current.get().title.as_ref().is_some_and(|t| !t.is_empty())>
                                                            <div class="note-title">{move || current.get().title.clone().unwrap_or_default()}</div>
                                                        </Show>
                                                        <div class="note-content">{move || current.get().content.clone()}</div>
                                                    </div>
                                                    <div class="note-actions">
                                                        <button
                                                            type="button"
                                                            class="action-button"
                                                            on:click=move |_| {
                                                                start_edit_note(current.get());
                                                            }
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

                <Show when=move || !panel_rows.get().is_empty()>
                    <div class="card">
                        <h3>"Liver & lipids"</h3>
                        <p class="muted">
                            {move || {
                                let head = "Newest result of each marker, with the previous test beside it. Ranges are typical adult ones; your report carries its own.";
                                if !show_liver_panel.get() {
                                    format!("{head} The liver markers are hidden because estradiol is not taken as a tablet.")
                                } else if on_tablets.get() {
                                    format!("{head} A tablet is swallowed, so it reaches the liver before the blood, which is why the liver markers are here.")
                                } else {
                                    format!("{head} These liver results were recorded earlier; your estradiol is an injection now.")
                                }
                            }}
                        </p>
                        <ul class="history-list">
                            <For
                                each=move || panel_rows.get()
                                key=|row| row.marker.label()
                                children=move |row| {
                                    let marker = row.marker;
                                    let value = row.value;
                                    let previous = row.previous;
                                    let flag = row.flag;
                                    let date = row.date;
                                    let (low, high) = marker.typical_range();
                                    let range = match (low, high) {
                                        (Some(low), Some(high)) => format!("typical {low} to {high}"),
                                        (Some(low), None) => format!("typical above {low}"),
                                        (None, Some(high)) => format!("typical below {high}"),
                                        (None, None) => String::new(),
                                    };
                                    let flag_text = match flag {
                                        Some(true) => "above range",
                                        Some(false) => "below range",
                                        None => "",
                                    };
                                    let measured = format!("{} {}", fmt_blood_value(value), marker.unit());
                                    let previous_text = previous
                                        .map(|value| format!("previous {}", fmt_blood_value(value)))
                                        .unwrap_or_default();
                                    let when = move || fmt_date_label(date, &x_axis_mode.get(), first_dose_date.get());
                                    view! {
                                        <li class="history-item">
                                            <div>
                                                <div>
                                                    <strong>{marker.label()}</strong>
                                                    " "
                                                    <span class="muted">{measured}</span>
                                                    " "
                                                    <span class="muted">{range}</span>
                                                </div>
                                                <div class="muted">
                                                    {when}
                                                    " · "
                                                    {previous_text}
                                                    <Show when=move || !flag_text.is_empty()>
                                                        " · "
                                                        <span class="muted">{flag_text}</span>
                                                    </Show>
                                                </div>
                                            </div>
                                        </li>
                                    }
                                }
                            />
                        </ul>
                    </div>
                </Show>

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
                                                        editing_route.set(progesterone_route_label(route).to_string());
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
                                                let summary = format!(
                                                    "Injection · {} · {dose_label}",
                                                    injectable_estradiol_label(kind)
                                                );
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
                                            DosageHistoryEntry::OralEstradiol { kind, dose, unit: _, pillQuantity, note, .. } => {
                                                let qty = pillQuantity.unwrap_or(1.0);
                                                let summary = format!("Oral estradiol · {}", oral_estradiol_label(kind));
                                                let details = vec![pill_detail(*dose, qty)];
                                                let mut meta = Vec::new();
                                                if let Some(note) = note {
                                                    meta.push(format!("Note: {note}"));
                                                }
                                                (summary, details, meta)
                                            }
                                            DosageHistoryEntry::Progesterone { kind, route, dose, unit: _, pillQuantity, note, .. } => {
                                                let qty = pillQuantity.unwrap_or(1.0);
                                                let summary = format!(
                                                    "Progesterone · {} · {}",
                                                    progesterone_route_word(route),
                                                    progesterone_label(kind)
                                                );
                                                let details = vec![pill_detail(*dose, qty)];
                                                let mut meta = Vec::new();
                                                if let Some(note) = note {
                                                    meta.push(format!("Note: {note}"));
                                                }
                                                (summary, details, meta)
                                            }
                                            DosageHistoryEntry::Antiandrogen { kind, dose, unit: _, note, .. } => {
                                                let summary = antiandrogen_label(kind).to_string();
                                                let details = vec![dose_mg(*dose)];
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
                                let confirm_action = confirm_action;
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

#[cfg(test)]
mod chart_interaction_tests {
    use super::*;
    use crate::charts::view::e2_value_in_unit;
    use chrono::{Local, TimeZone};

    fn ts(year: i32, month: u32, day: u32) -> i64 {
        Local
            .with_ymd_and_hms(year, month, day, 12, 0, 0)
            .unwrap()
            .timestamp_millis()
    }

    fn chart_point(
        x: f64,
        y: f64,
        series: ChartSeries,
        unit: &str,
        label: &str,
        timestamp_ms: i64,
    ) -> ViewChartPoint {
        ViewChartPoint {
            x,
            y,
            label: label.to_string(),
            color: series.color(),
            series,
            unit: unit.to_string(),
            timestamp_ms,
        }
    }

    fn test_state(points: Vec<ViewChartPoint>, e2_unit: E2DisplayUnit) -> ViewChartState {
        let mut domain_min = points.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
        let mut domain_max = points
            .iter()
            .map(|p| p.x)
            .fold(f64::NEG_INFINITY, f64::max);
        if (domain_max - domain_min).abs() < f64::EPSILON {
            domain_min -= 1.0;
            domain_max += 1.0;
        }
        ViewChartState {
            domain_min,
            domain_max,
            y_min: 0.0,
            y_max: 100.0,
            x_label: "Date".to_string(),
            y_label: "Levels".to_string(),
            points,
            dosage_points: Vec::new(),
            first_dose: None,
            use_days: false,
            has_data: true,
            e2_unit,
        }
    }

    fn e2_state(e2_unit: E2DisplayUnit) -> ViewChartState {
        test_state(
            vec![
                chart_point(
                    0.0,
                    340.0,
                    ChartSeries::Estradiol,
                    "pmol/L",
                    "Estradiol: 340 pmol/L (Nov 01)",
                    ts(2026, 11, 1),
                ),
                chart_point(
                    10.0,
                    410.0,
                    ChartSeries::Estradiol,
                    "pmol/L",
                    "Estradiol: 410 pmol/L (Nov 11)",
                    ts(2026, 11, 11),
                ),
            ],
            e2_unit,
        )
    }

    /// A plot area with round numbers, so pixel arithmetic is checkable by eye:
    /// x 0..100 across 800px starting at 100px.
    fn fixed_geometry() -> ChartGeometry {
        ChartGeometry {
            left: 100.0,
            top: 10.0,
            width: 800.0,
            height: 400.0,
            x_min: 0.0,
            x_max: 100.0,
        }
    }

    #[test]
    fn chart_options_follow_the_settings_unit_until_a_chip_is_chosen() {
        // The regression: a pg/mL user must stay on pg/mL with no chip pressed.
        let pg_ml_state = e2_state(E2DisplayUnit::PgMl);
        let options = chart_options_from(&pg_ml_state, None, false, false, false, false);
        assert_eq!(options.e2_unit, E2DisplayUnit::PgMl);

        // Choosing a unit wins over the settings.
        let options = chart_options_from(&pg_ml_state, Some(E2DisplayUnit::PmolL), false, false, false, false);
        assert_eq!(options.e2_unit, E2DisplayUnit::PmolL);

        // And the default chart is untouched: pmol/L settings with nothing
        // pressed is exactly `ChartOptions::default()`.
        let pmol_state = e2_state(E2DisplayUnit::PmolL);
        let options = chart_options_from(&pmol_state, None, false, false, false, false);
        assert_eq!(options, ChartOptions::default());
    }

    #[test]
    fn a_pg_ml_user_reads_pg_ml_in_the_readout() {
        let state = test_state(
            vec![chart_point(
                0.0,
                92.6,
                ChartSeries::Estradiol,
                "pg/mL",
                "Estradiol: 340 pmol/L -> 92.6 pg/mL (Nov 01)",
                ts(2026, 11, 1),
            )],
            E2DisplayUnit::PgMl,
        );
        let options = chart_options_from(&state, None, false, false, false, false);
        let rows = readout_rows(&state, &options, 0.0);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].unit, "pg/mL");
        assert_eq!(rows[0].value, "92.6");
        assert_eq!(rows[0].recorded.as_deref(), Some("recorded 340 pmol/L"));
    }

    #[test]
    fn the_unit_chip_converts_the_readout_and_the_table_together() {
        let state = e2_state(E2DisplayUnit::PmolL);
        let pmol_options = chart_options_from(&state, None, false, false, false, false);
        let pg_ml_options = chart_options_from(&state, Some(E2DisplayUnit::PgMl), false, false, false, false);

        let pmol_rows = readout_rows(&state, &pmol_options, 0.0);
        let pg_ml_rows = readout_rows(&state, &pg_ml_options, 0.0);
        let expected = e2_value_in_unit(340.0, E2DisplayUnit::PmolL, E2DisplayUnit::PgMl);
        assert_eq!(pmol_rows[0].unit, "pmol/L");
        assert_eq!(pmol_rows[0].value, "340");
        assert_eq!(pg_ml_rows[0].unit, "pg/mL");
        assert_eq!(pg_ml_rows[0].value, format_chart_value(expected));

        // The table reports the same numbers as the readout, in both units. It
        // is newest first, so the old point (x = 0) is its last row.
        let pmol_table = chart_table_rows(&state, &pmol_options);
        let pg_ml_table = chart_table_rows(&state, &pg_ml_options);
        let pmol_oldest = pmol_table.last().expect("table rows");
        let pg_ml_oldest = pg_ml_table.last().expect("table rows");
        assert_eq!(pmol_oldest.value, pmol_rows[0].value);
        assert_eq!(pmol_oldest.unit, "pmol/L");
        assert_eq!(pg_ml_oldest.value, pg_ml_rows[0].value);
        assert_eq!(pg_ml_oldest.unit, "pg/mL");
    }

    #[test]
    fn readout_lists_every_visible_series_at_the_hovered_x() {
        let mut state = test_state(
            vec![
                chart_point(0.0, 340.0, ChartSeries::Estradiol, "pmol/L", "", ts(2026, 11, 1)),
                chart_point(0.0, 45.0, ChartSeries::Testosterone, "ng/dL", "", ts(2026, 11, 1)),
                chart_point(10.0, 410.0, ChartSeries::Estradiol, "pmol/L", "", ts(2026, 11, 11)),
            ],
            E2DisplayUnit::PmolL,
        );
        let options = ChartOptions::default();
        let rows = readout_rows(&state, &options, 0.0);
        assert_eq!(
            rows.iter().map(|row| row.series).collect::<Vec<_>>(),
            vec![ChartSeries::Estradiol, ChartSeries::Testosterone]
        );
        assert_eq!(rows[1].unit, "ng/dL");
        assert_eq!(rows[1].value, "45");

        // A column with one series lists one series.
        assert_eq!(readout_rows(&state, &options, 10.0).len(), 1);

        // A series that is not plotted at all never appears.
        state.points.retain(|point| point.series == ChartSeries::Estradiol);
        assert_eq!(readout_rows(&state, &options, 0.0).len(), 1);
    }

    #[test]
    fn a_dose_column_reports_the_dose_not_a_scaled_height() {
        let mut state = test_state(
            vec![chart_point(5.0, 300.0, ChartSeries::Estradiol, "pmol/L", "", ts(2026, 11, 6))],
            E2DisplayUnit::PmolL,
        );
        state.dosage_points = vec![chart_point(
            5.0,
            50.0,
            ChartSeries::Dose,
            "",
            "Injection: 5.00 mg (2026-11-06)",
            ts(2026, 11, 6),
        )];
        let rows = readout_rows(&state, &ChartOptions::default(), 5.0);
        let dose = rows
            .iter()
            .find(|row| row.series == ChartSeries::Dose)
            .expect("dose row");
        assert_eq!(dose.value, "Injection: 5.00 mg");
        assert_eq!(dose.unit, "");
    }

    #[test]
    fn recorded_note_keeps_only_a_converted_record() {
        assert_eq!(
            recorded_note("Estradiol: 340 pmol/L -> 92.6 pg/mL (Nov 01)").as_deref(),
            Some("recorded 340 pmol/L")
        );
        assert_eq!(recorded_note("Estradiol: 340 pmol/L (Nov 01)"), None);
        assert_eq!(recorded_note(""), None);
    }

    #[test]
    fn chart_geometry_uses_the_padding_the_chart_draws_with() {
        let state = e2_state(E2DisplayUnit::PmolL);
        let geometry = chart_geometry(&state, ViewZoom::default(), 900.0, 420.0);
        let padding = chart_padding();
        assert_eq!(geometry.left, padding.0);
        assert_eq!(geometry.top, padding.1);
        assert!((geometry.width - (900.0 - padding.0 - padding.2)).abs() < 1e-9);
        assert!((geometry.height - (420.0 - padding.1 - padding.3)).abs() < 1e-9);
        assert_eq!(geometry.x_min, state.domain_min);
        assert_eq!(geometry.x_max, state.domain_max);
    }

    #[test]
    fn geometry_follows_a_zoomed_window() {
        let state = e2_state(E2DisplayUnit::PmolL);
        let zoom = ViewZoom {
            x_min: Some(2.0),
            x_max: Some(8.0),
        };
        let geometry = chart_geometry(&state, zoom, 900.0, 420.0);
        assert_eq!(geometry.x_min, 2.0);
        assert_eq!(geometry.x_max, 8.0);
        assert!((geometry.x_to_px(2.0) - geometry.left).abs() < 1e-9);
        assert!((geometry.x_to_px(8.0) - geometry.right()).abs() < 1e-9);
    }

    #[test]
    fn nearest_column_snaps_to_the_closest_point() {
        let state = test_state(
            vec![
                chart_point(0.0, 10.0, ChartSeries::Estradiol, "pmol/L", "", 1),
                chart_point(10.0, 20.0, ChartSeries::Estradiol, "pmol/L", "", 2),
                chart_point(20.0, 30.0, ChartSeries::Estradiol, "pmol/L", "", 3),
            ],
            E2DisplayUnit::PmolL,
        );
        let geometry = fixed_geometry();
        // x 10 sits at px 180; the cursor just to the right of it.
        assert_eq!(nearest_column_x(&state, geometry, 190.0), Some(10.0));
        assert_eq!(nearest_column_x(&state, geometry, 395.0), Some(20.0));
        // Past the end of the domain the cursor still snaps to the last column,
        // clamped to the plot edge.
        assert_eq!(nearest_column_x(&state, geometry, 5000.0), Some(20.0));
    }

    #[test]
    fn between_two_points_the_nearer_column_wins() {
        let state = test_state(
            vec![
                chart_point(0.0, 10.0, ChartSeries::Estradiol, "pmol/L", "", 1),
                chart_point(10.0, 20.0, ChartSeries::Estradiol, "pmol/L", "", 2),
            ],
            E2DisplayUnit::PmolL,
        );
        let geometry = fixed_geometry();
        let mid_left = (geometry.x_to_px(0.0) + geometry.x_to_px(10.0)) / 2.0 - 1.0;
        let mid_right = mid_left + 2.0;
        assert_eq!(nearest_column_x(&state, geometry, mid_left), Some(0.0));
        assert_eq!(nearest_column_x(&state, geometry, mid_right), Some(10.0));
    }

    #[test]
    fn a_cursor_away_from_every_point_still_snaps_to_a_column() {
        // Two points close together in x but far up the plot; the cursor is at
        // the foot of the plot, well outside find_nearest_point's 48px radius.
        let state = test_state(
            vec![
                chart_point(0.0, 95.0, ChartSeries::Estradiol, "pmol/L", "", 1),
                chart_point(10.0, 95.0, ChartSeries::Estradiol, "pmol/L", "", 2),
            ],
            E2DisplayUnit::PmolL,
        );
        let options = ChartOptions::default();
        let geometry = chart_geometry(&state, ViewZoom::default(), 900.0, 420.0);
        let cursor_x = geometry.x_to_px(9.0);
        let snapped = snapped_x(&state, &options, geometry, cursor_x, geometry.top + geometry.height - 4.0);
        assert_eq!(snapped, Some(10.0));
    }

    #[test]
    fn a_cursor_on_a_point_snaps_to_that_points_x() {
        let state = test_state(
            vec![
                chart_point(0.0, 20.0, ChartSeries::Estradiol, "pmol/L", "", 1),
                chart_point(10.0, 80.0, ChartSeries::Estradiol, "pmol/L", "", 2),
            ],
            E2DisplayUnit::PmolL,
        );
        let options = ChartOptions::default();
        let geometry = chart_geometry(&state, ViewZoom::default(), 900.0, 420.0);
        let (y_min, y_max) = plotted_y_range(&state, &options);
        let cursor_x = geometry.x_to_px(10.0);
        let cursor_y = geometry.top + geometry.height - y_fraction(80.0, y_min, y_max, false) * geometry.height;
        assert_eq!(
            snapped_x(&state, &options, geometry, cursor_x, cursor_y),
            Some(10.0)
        );
    }

    #[test]
    fn no_points_snap_to_nothing() {
        let state = test_state(vec![], E2DisplayUnit::PmolL);
        assert_eq!(nearest_column_x(&state, fixed_geometry(), 400.0), None);
        assert_eq!(
            snapped_x(&state, &ChartOptions::default(), fixed_geometry(), 400.0, 200.0),
            None
        );
    }

    #[test]
    fn a_brush_maps_pixels_to_the_domain() {
        let geometry = fixed_geometry();
        // 300px -> x 25, 500px -> x 50 on this plot.
        assert_eq!(brush_window(geometry, 300.0, 500.0), (25.0, 50.0));
        // Dragging right to left selects the same window.
        assert_eq!(brush_window(geometry, 500.0, 300.0), (25.0, 50.0));
        // A drag that leaves the plot clamps to its edges.
        assert_eq!(brush_window(geometry, -50.0, 2000.0), (0.0, 100.0));
    }

    #[test]
    fn a_brush_zooms_through_clamp_zoom() {
        let state = e2_state(E2DisplayUnit::PmolL);
        // The state's own domain is 0..10 days.
        let geometry = ChartGeometry {
            left: 100.0,
            top: 10.0,
            width: 800.0,
            height: 400.0,
            x_min: state.domain_min,
            x_max: state.domain_max,
        };
        let zoom = brush_zoom(&state, geometry, geometry.left, geometry.left + 400.0);
        assert_eq!(zoom.x_min, Some(0.0));
        assert_eq!(zoom.x_max, Some(5.0));

        // A drag across the whole plot resets rather than pinning the domain.
        let zoom = brush_zoom(&state, geometry, geometry.left, geometry.right());
        assert_eq!(zoom, ViewZoom::default());
    }

    #[test]
    fn the_selection_rectangle_only_shows_once_the_drag_moves() {
        let geometry = fixed_geometry();
        let click = BrushState {
            start_x: 300.0,
            current_x: 300.5,
        };
        assert_eq!(brush_rect(geometry, click), None);

        let drag = BrushState {
            start_x: 300.0,
            current_x: 340.0,
        };
        assert_eq!(brush_rect(geometry, drag), Some((300.0, 40.0)));

        // Dragging backwards gives the same rectangle, and it stops at the edge.
        let drag = BrushState {
            start_x: 880.0,
            current_x: 940.0,
        };
        assert_eq!(brush_rect(geometry, drag), Some((880.0, 20.0)));
    }

    #[test]
    fn a_short_drag_is_a_click_not_a_zoom() {
        let state = e2_state(E2DisplayUnit::PmolL);
        let geometry = fixed_geometry();
        // A click, and a drag shorter than the threshold, ask for no zoom at all.
        for span in [0.0, 1.0, BRUSH_MIN_DRAG_PX - 0.5] {
            let click = BrushState {
                start_x: 300.0,
                current_x: 300.0 + span,
            };
            assert_eq!(brush_zoom_for_drag(&state, geometry, click), None);
        }
        // Past the threshold the drag is a brush and zooms.
        let drag = BrushState {
            start_x: 300.0,
            current_x: 340.0,
        };
        assert_eq!(
            brush_zoom_for_drag(&state, geometry, drag),
            Some(brush_zoom(&state, geometry, 300.0, 340.0))
        );
    }

    #[test]
    fn y_fractions_are_linear_or_geometric_with_the_axis() {
        assert_eq!(y_fraction(50.0, 0.0, 100.0, false), 0.5);
        assert_eq!(y_fraction(0.0, 0.0, 100.0, false), 0.0);
        assert_eq!(y_fraction(100.0, 0.0, 100.0, false), 1.0);
        // One decade up a two-decade axis.
        assert_eq!(y_fraction(10.0, 1.0, 100.0, true), 0.5);
        assert_eq!(y_fraction(1.0, 1.0, 100.0, true), 0.0);
        // Values a log axis cannot place sit on the floor rather than a NaN.
        assert_eq!(y_fraction(0.0, 1.0, 100.0, true), 0.0);
        assert_eq!(y_fraction(f64::NAN, 0.0, 100.0, false), 0.5);
        assert_eq!(y_fraction(5.0, 10.0, 10.0, false), 0.5);
    }

    #[test]
    fn crosshair_dots_land_on_the_plotted_points() {
        let state = test_state(
            vec![
                chart_point(0.0, 20.0, ChartSeries::Estradiol, "pmol/L", "", 1),
                chart_point(10.0, 80.0, ChartSeries::Estradiol, "pmol/L", "", 2),
            ],
            E2DisplayUnit::PmolL,
        );
        let options = ChartOptions::default();
        let geometry = chart_geometry(&state, ViewZoom::default(), 900.0, 420.0);
        let dots = crosshair_dots(&state, &options, geometry, 0.0);
        assert_eq!(dots.len(), 1);
        assert!((dots[0].x - geometry.x_to_px(0.0)).abs() < 1e-9);
        assert!((dots[0].y - geometry.top).abs() < geometry.height);
        assert_eq!(dots[0].color, chart_series_css_color(ChartSeries::Estradiol));
        // Outside the domain nothing is marked.
        assert!(crosshair_dots(&state, &options, geometry, 99.0).is_empty());
    }

    #[test]
    fn small_multiples_put_each_dot_in_its_own_panel() {
        let state = test_state(
            vec![
                chart_point(0.0, 300.0, ChartSeries::Estradiol, "pmol/L", "", 1),
                chart_point(0.0, 45.0, ChartSeries::Testosterone, "ng/dL", "", 1),
            ],
            E2DisplayUnit::PmolL,
        );
        let options = chart_options_from(&state, None, false, false, false, true);
        let geometry = chart_geometry(&state, ViewZoom::default(), 900.0, 420.0);
        let dots = crosshair_dots(&state, &options, geometry, 0.0);
        assert_eq!(dots.len(), 2);
        let canvas_height = geometry.height + chart_padding().1 + chart_padding().3;
        // Estradiol's panel is the top half, testosterone's the bottom half.
        assert!(dots[0].y < canvas_height / 2.0);
        assert!(dots[1].y > canvas_height / 2.0);
    }

    #[test]
    fn a_crosshair_outside_the_zoomed_window_is_not_drawn() {
        let state = e2_state(E2DisplayUnit::PmolL);
        let options = ChartOptions::default();
        let zoom = ViewZoom {
            x_min: Some(2.0),
            x_max: Some(8.0),
        };
        let overlay = chart_overlay(&state, &options, zoom, 900.0, 420.0, Some(9.5), None);
        assert_eq!(overlay.crosshair, None);
        assert!(overlay.dots.is_empty());

        let inside = chart_overlay(&state, &options, zoom, 900.0, 420.0, Some(4.0), None);
        assert!(inside.crosshair.is_some());
    }

    #[test]
    fn snapping_ignores_points_behind_a_zoomed_axis() {
        let state = test_state(
            vec![
                chart_point(0.0, 20.0, ChartSeries::Estradiol, "pmol/L", "", 1),
                chart_point(10.0, 80.0, ChartSeries::Estradiol, "pmol/L", "", 2),
            ],
            E2DisplayUnit::PmolL,
        );
        let options = ChartOptions::default();
        // Zoomed to the second half: the point at x = 0 is off screen.
        let zoom = ViewZoom {
            x_min: Some(5.0),
            x_max: Some(10.0),
        };
        let geometry = chart_geometry(&state, zoom, 900.0, 420.0);
        let (y_min, y_max) = plotted_y_range(&state, &options);
        let cursor_x = geometry.x_to_px(10.0) - 30.0;
        let cursor_y = geometry.top + geometry.height
            - y_fraction(80.0, y_min, y_max, false) * geometry.height;
        assert_eq!(
            snapped_x(&state, &options, geometry, cursor_x, cursor_y),
            Some(10.0)
        );
    }

    #[test]
    fn overlay_bundles_the_crosshair_the_dots_and_the_brush() {
        let state = e2_state(E2DisplayUnit::PmolL);
        let options = ChartOptions::default();
        let padding = chart_padding();
        let empty = chart_overlay(&state, &options, ViewZoom::default(), 900.0, 420.0, None, None);
        assert_eq!(empty.crosshair, None);
        assert!(empty.dots.is_empty());
        assert_eq!(empty.brush, None);

        let hovered = chart_overlay(
            &state,
            &options,
            ViewZoom::default(),
            900.0,
            420.0,
            Some(0.0),
            Some(BrushState {
                start_x: 200.0,
                current_x: 300.0,
            }),
        );
        let (left, top, height) = hovered.crosshair.expect("crosshair");
        assert_eq!(top, padding.1);
        assert!((height - (420.0 - padding.1 - padding.3)).abs() < 1e-9);
        assert!((left - chart_geometry(&state, ViewZoom::default(), 900.0, 420.0).left).abs() < 1e-9);
        assert_eq!(hovered.dots.len(), 1);
        assert_eq!(hovered.brush, Some((200.0, 100.0)));
    }

    #[test]
    fn table_rows_are_newest_first_with_their_series_and_unit() {
        let mut state = test_state(
            vec![
                chart_point(0.0, 340.0, ChartSeries::Estradiol, "pmol/L", "", ts(2026, 11, 1)),
                chart_point(10.0, 45.0, ChartSeries::Testosterone, "ng/dL", "", ts(2026, 11, 11)),
            ],
            E2DisplayUnit::PmolL,
        );
        state.use_days = false;
        let rows = chart_table_rows(&state, &ChartOptions::default());
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].series, ChartSeries::Testosterone);
        assert_eq!(rows[0].date, fmt_date_label(ts(2026, 11, 11), "date", None));
        assert_eq!(rows[0].unit, "ng/dL");
        assert_eq!(rows[1].series, ChartSeries::Estradiol);
        assert_eq!(rows[1].unit, "pmol/L");
    }

    #[test]
    fn table_rows_keep_chart_order_within_one_date() {
        let state = test_state(
            vec![
                chart_point(0.0, 45.0, ChartSeries::Testosterone, "ng/dL", "", ts(2026, 11, 1)),
                chart_point(0.0, 340.0, ChartSeries::Estradiol, "pmol/L", "", ts(2026, 11, 1)),
            ],
            E2DisplayUnit::PmolL,
        );
        let rows = chart_table_rows(&state, &ChartOptions::default());
        assert_eq!(rows[0].series, ChartSeries::Estradiol);
        assert_eq!(rows[1].series, ChartSeries::Testosterone);
    }

    #[test]
    fn chart_values_read_at_two_decimals_without_trailing_zeros() {
        assert_eq!(format_chart_value(340.0), "340");
        assert_eq!(format_chart_value(92.44), "92.44");
        assert_eq!(format_chart_value(0.1 + 0.2), "0.3");
        assert_eq!(format_chart_value(f64::NAN), "-");
        assert_eq!(format_chart_value(f64::INFINITY), "-");
    }

    #[test]
    fn pressed_reports_the_toggle_state_for_aria() {
        assert_eq!(pressed(true), "true");
        assert_eq!(pressed(false), "false");
    }
}
