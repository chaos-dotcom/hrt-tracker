use chrono::{Local, TimeZone};
use leptos::window;
use plotters::coord::ranged1d::{AsRangedCoord, ValueFormatter};
use plotters::coord::Shift;
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

/// Which series a plotted point belongs to.
///
/// The order of the blood series is the order series appear in small multiples
/// and in any series list built from a chart, so it is part of the chart's
/// appearance. `Dose` is not a blood series: it only ever marks a logged dose,
/// which is why it comes last and is not in `ChartSeries::ALL`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChartSeries {
    Estradiol,
    Testosterone,
    Progesterone,
    Fsh,
    Lh,
    Prolactin,
    Shbg,
    FreeAndrogenIndex,
    FudgeFactor,
    Dose,
}

impl ChartSeries {
    /// The blood series, in the order panels and legends list them.
    pub const ALL: [ChartSeries; 9] = [
        ChartSeries::Estradiol,
        ChartSeries::Testosterone,
        ChartSeries::Progesterone,
        ChartSeries::Fsh,
        ChartSeries::Lh,
        ChartSeries::Prolactin,
        ChartSeries::Shbg,
        ChartSeries::FreeAndrogenIndex,
        ChartSeries::FudgeFactor,
    ];

    /// Short name, used for a small-multiple panel heading.
    pub fn label(self) -> &'static str {
        match self {
            ChartSeries::Estradiol => "Estradiol",
            ChartSeries::Testosterone => "Testosterone",
            ChartSeries::Progesterone => "Progesterone",
            ChartSeries::Fsh => "FSH",
            ChartSeries::Lh => "LH",
            ChartSeries::Prolactin => "Prolactin",
            ChartSeries::Shbg => "SHBG",
            ChartSeries::FreeAndrogenIndex => "FAI",
            ChartSeries::FudgeFactor => "Efficacy (FF)",
            ChartSeries::Dose => "Doses",
        }
    }

    /// The series colour. These are the colours the chart has always used.
    pub fn color(self) -> RGBColor {
        match self {
            ChartSeries::Estradiol => RGBColor(70, 130, 180),
            ChartSeries::Testosterone => RGBColor(220, 20, 60),
            ChartSeries::Progesterone => RGBColor(148, 0, 211),
            ChartSeries::Fsh => RGBColor(34, 139, 34),
            ChartSeries::Lh => RGBColor(0, 139, 139),
            ChartSeries::Prolactin => RGBColor(139, 69, 19),
            ChartSeries::Shbg => RGBColor(255, 20, 147),
            ChartSeries::FreeAndrogenIndex => RGBColor(0, 0, 0),
            ChartSeries::FudgeFactor => RGBColor(255, 99, 132),
            // A dose point always carries its own colour; this is a fallback.
            ChartSeries::Dose => RGBColor(0, 114, 178),
        }
    }

    /// Heading for this series' own panel, including its display unit when it
    /// has a fixed one. Prolactin and SHBG are plotted in whatever unit was
    /// recorded, so they only say that.
    pub fn panel_title(self, e2_unit: E2DisplayUnit) -> String {
        let unit = match self {
            ChartSeries::Estradiol => Some(e2_unit.label()),
            ChartSeries::Testosterone => Some("nmol/L"),
            ChartSeries::Progesterone => Some("nmol/L"),
            ChartSeries::Fsh | ChartSeries::Lh => Some("mIU/mL"),
            ChartSeries::Prolactin | ChartSeries::Shbg => Some("recorded unit"),
            ChartSeries::FudgeFactor => Some("%"),
            ChartSeries::Dose => None,
            ChartSeries::FreeAndrogenIndex => None,
        };
        match unit {
            Some(unit) => format!("{} ({})", self.label(), unit),
            None => self.label().to_string(),
        }
    }
}

/// The unit the estradiol series is drawn in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum E2DisplayUnit {
    #[default]
    PmolL,
    PgMl,
}

impl E2DisplayUnit {
    pub fn label(self) -> &'static str {
        match self {
            E2DisplayUnit::PmolL => "pmol/L",
            E2DisplayUnit::PgMl => "pg/mL",
        }
    }

    pub fn hormone_units(self) -> HormoneUnits {
        match self {
            E2DisplayUnit::PmolL => HormoneUnits::E2PmolL,
            E2DisplayUnit::PgMl => HormoneUnits::E2PgMl,
        }
    }

    /// The estradiol display unit a `HormoneUnits` value corresponds to; any
    /// other unit is treated as pmol/L, which is what the settings default to.
    pub fn from_hormone_units(unit: &HormoneUnits) -> Self {
        match unit {
            HormoneUnits::E2PgMl => E2DisplayUnit::PgMl,
            _ => E2DisplayUnit::PmolL,
        }
    }
}

/// Chart appearance switches. `ChartOptions::default()` draws exactly what the
/// chart drew before these switches existed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChartOptions {
    /// Unit for the estradiol series only; every other series keeps its own.
    pub e2_unit: E2DisplayUnit,
    /// Logarithmic y axis, when the plotted values allow one.
    pub log_scale: bool,
    /// A quiet tick per logged dose, behind the series.
    pub show_dose_ticks: bool,
    /// Shade the min..max of the plotted values. Descriptive, not a target.
    pub show_value_band: bool,
    /// One panel per series instead of one shared plot.
    pub separate_panels: bool,
}

impl Default for ChartOptions {
    fn default() -> Self {
        Self {
            e2_unit: E2DisplayUnit::PmolL,
            log_scale: false,
            show_dose_ticks: false,
            show_value_band: false,
            separate_panels: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ViewChartPoint {
    pub x: f64,
    pub y: f64,
    pub label: String,
    pub color: RGBColor,
    /// Which series the point belongs to.
    pub series: ChartSeries,
    /// Unit `y` is in, for display (empty for a unitless index).
    pub unit: String,
    /// Wall-clock time of the measurement, which `x` only carries indirectly
    /// when the axis is in days.
    pub timestamp_ms: i64,
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
    /// The unit `points` holds the estradiol series in.
    pub e2_unit: E2DisplayUnit,
}

/// One drawing panel: the single combined plot, or one series of a
/// small-multiples stack.
#[derive(Clone, Debug, PartialEq)]
pub struct ChartPanel {
    /// `None` for the combined plot.
    pub series: Option<ChartSeries>,
    /// Panel heading, also used as the y axis description.
    pub title: String,
    pub y_min: f64,
    pub y_max: f64,
    /// The panel's points, already in the unit they are drawn in.
    pub points: Vec<ViewChartPoint>,
}

#[allow(clippy::too_many_arguments)]
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
                    color: ChartSeries::Estradiol.color(),
                    series: ChartSeries::Estradiol,
                    unit: unit_label.to_string(),
                    timestamp_ms: test.date,
                });
                all_values.push(plot_val);
                has_data = true;
            }
        }
        if show_t {
            if let Some(value) = test.testLevel {
                // Testosterone plots in the unit chosen in Settings (nmol/L by
                // default); a result recorded in the other unit converts once, here.
                let raw_unit = test.testUnit.clone().unwrap_or(HormoneUnits::TNmolL);
                let display_ng_dl = settings
                    .displayTestosteroneUnit
                    .clone()
                    .unwrap_or(HormoneUnits::TNmolL)
                    == HormoneUnits::TNgDl;
                let plot_val = match (raw_unit == HormoneUnits::TNmolL, display_ng_dl) {
                    (true, false) => value,
                    (true, true) => value * 28.818,
                    (false, true) => value,
                    (false, false) => value / 28.818,
                };
                let unit_label = if display_ng_dl { "ng/dL" } else { "nmol/L" };
                let tooltip = if raw_unit == HormoneUnits::TNmolL {
                    format!(
                        "Testosterone: {} {} ({})",
                        fmt_blood_value(plot_val),
                        unit_label,
                        date_short
                    )
                } else {
                    format!(
                        "Testosterone: {} {} -> {} {} ({})",
                        fmt_blood_value(value),
                        hormone_unit_label(&raw_unit),
                        fmt_blood_value(plot_val),
                        unit_label,
                        date_short
                    )
                };
                points.push(ViewChartPoint {
                    x,
                    y: plot_val,
                    label: tooltip,
                    color: ChartSeries::Testosterone.color(),
                    series: ChartSeries::Testosterone,
                    unit: unit_label.to_string(),
                    timestamp_ms: test.date,
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
                // Progesterone likewise plots in the chosen unit, nmol/L by default.
                let display_ng_ml = settings
                    .displayProgesteroneUnit
                    .clone()
                    .unwrap_or(HormoneUnits::TNmolL)
                    == HormoneUnits::NgMl;
                let plot_val = match (raw_unit == HormoneUnits::TNmolL, display_ng_ml) {
                    (true, false) => value,
                    (true, true) => value * 0.31,
                    (false, true) => value,
                    (false, false) => value / 0.31,
                };
                let unit_label = if display_ng_ml { "ng/mL" } else { "nmol/L" };
                let tooltip = if raw_unit == HormoneUnits::TNmolL {
                    format!(
                        "Progesterone: {} {} ({})",
                        fmt_blood_value(plot_val),
                        unit_label,
                        date_short
                    )
                } else {
                    format!(
                        "Progesterone: {} {} -> {} {} ({})",
                        fmt_blood_value(value),
                        hormone_unit_label(&raw_unit),
                        fmt_blood_value(plot_val),
                        unit_label,
                        date_short
                    )
                };
                points.push(ViewChartPoint {
                    x,
                    y: plot_val,
                    label: tooltip,
                    color: ChartSeries::Progesterone.color(),
                    series: ChartSeries::Progesterone,
                    unit: unit_label.to_string(),
                    timestamp_ms: test.date,
                });
                all_values.push(plot_val);
                has_data = true;
            }
        }
        if show_fsh {
            if let Some(value) = test.fshLevel {
                let raw_unit = test.fshUnit.clone().unwrap_or(HormoneUnits::UL);
                let display_miu_l = settings
                    .displayFshUnit
                    .clone()
                    .unwrap_or(HormoneUnits::MIuMl)
                    == HormoneUnits::MIuL;
                let per_ml = convert_fsh_to_miu_ml(value, &raw_unit);
                let plot_val = if display_miu_l { per_ml * 1000.0 } else { per_ml };
                let unit_label = if display_miu_l { "mIU/L" } else { "mIU/mL" };
                let tooltip = format!(
                    "FSH: {} {} ({})",
                    fmt_blood_value(plot_val),
                    unit_label,
                    date_short
                );
                points.push(ViewChartPoint {
                    x,
                    y: plot_val,
                    label: tooltip,
                    color: ChartSeries::Fsh.color(),
                    series: ChartSeries::Fsh,
                    unit: unit_label.to_string(),
                    timestamp_ms: test.date,
                });
                all_values.push(plot_val);
                has_data = true;
            }
        }
        if show_lh {
            if let Some(value) = test.lhLevel {
                let raw_unit = test.lhUnit.clone().unwrap_or(HormoneUnits::UL);
                let display_miu_l = settings
                    .displayLhUnit
                    .clone()
                    .unwrap_or(HormoneUnits::MIuMl)
                    == HormoneUnits::MIuL;
                let per_ml = convert_lh_to_miu_ml(value, &raw_unit);
                let plot_val = if display_miu_l { per_ml * 1000.0 } else { per_ml };
                let unit_label = if display_miu_l { "mIU/L" } else { "mIU/mL" };
                let tooltip = format!(
                    "LH: {} {} ({})",
                    fmt_blood_value(plot_val),
                    unit_label,
                    date_short
                );
                points.push(ViewChartPoint {
                    x,
                    y: plot_val,
                    label: tooltip,
                    color: ChartSeries::Lh.color(),
                    series: ChartSeries::Lh,
                    unit: unit_label.to_string(),
                    timestamp_ms: test.date,
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
                    color: ChartSeries::Prolactin.color(),
                    series: ChartSeries::Prolactin,
                    unit: unit_label.to_string(),
                    timestamp_ms: test.date,
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
                    color: ChartSeries::Shbg.color(),
                    series: ChartSeries::Shbg,
                    unit: unit_label.to_string(),
                    timestamp_ms: test.date,
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
                    color: ChartSeries::FreeAndrogenIndex.color(),
                    series: ChartSeries::FreeAndrogenIndex,
                    unit: String::new(),
                    timestamp_ms: test.date,
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
                    color: ChartSeries::FudgeFactor.color(),
                    series: ChartSeries::FudgeFactor,
                    unit: "%".to_string(),
                    timestamp_ms: test.date,
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
                series: ChartSeries::Dose,
                unit: String::new(),
                timestamp_ms: date,
            });

            all_values.push(value);
            has_data = true;
        }
    }

    let (y_min, y_max) = padded_y_range(&all_values);

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
        e2_unit: E2DisplayUnit::from_hormone_units(&display_unit),
    }
}

/// The y range the chart draws for a set of plotted values: an 8% pad, floored
/// at zero, with a single value widened to a unit either side of itself.
///
/// `compute_view_chart_state` uses this for `y_min`/`y_max` and the drawing
/// code uses it again once units or panels change, so the two cannot drift.
pub fn padded_y_range(values: &[f64]) -> (f64, f64) {
    let mut values = values.to_vec();
    if values.is_empty() {
        values.push(0.0);
        values.push(1.0);
    }
    let mut y_min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let mut y_max = values
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);
    if (y_min - y_max).abs() < f64::EPSILON {
        y_min = (y_min - 1.0).max(0.0);
        y_max += 1.0;
    } else {
        let pad = (y_max - y_min) * 0.08;
        y_min = (y_min - pad).max(0.0);
        y_max += pad;
    }
    (y_min, y_max)
}

/// A value plotted in a unit other than the one its series is stored in.
///
/// Estradiol is the only series with a display unit to choose; everything else
/// is already normalised by `compute_view_chart_state` and is returned as is.
pub fn displayed_value(
    point: &ViewChartPoint,
    state: &ViewChartState,
    options: &ChartOptions,
) -> f64 {
    if point.series == ChartSeries::Estradiol {
        e2_value_in_unit(point.y, state.e2_unit, options.e2_unit)
    } else {
        point.y
    }
}

/// Convert an estradiol value from one display unit to the other, using the
/// same helper the rest of the app converts with.
pub fn e2_value_in_unit(value: f64, from: E2DisplayUnit, to: E2DisplayUnit) -> f64 {
    convert_estradiol_to_display(value, &from.hormone_units(), &to.hormone_units())
}

/// The series represented in a set of points, in `ChartSeries::ALL` order.
pub fn series_present(points: &[ViewChartPoint]) -> Vec<ChartSeries> {
    ChartSeries::ALL
        .iter()
        .copied()
        .filter(|series| points.iter().any(|point| point.series == *series))
        .collect()
}

/// The y axis description for the combined plot.
///
/// It only names a unit when the axis is not in the unit the settings already
/// chose, so the default chart still reads "Levels".
pub fn combined_y_label(
    state_label: &str,
    state_unit: E2DisplayUnit,
    options_unit: E2DisplayUnit,
    has_e2: bool,
) -> String {
    if has_e2 && state_unit != options_unit {
        format!("{} (E2 {})", state_label, options_unit.label())
    } else {
        state_label.to_string()
    }
}

/// What the y axis of a single-multiples-less chart draws, after the estradiol
/// unit switch has been applied. Hit testing wants the same numbers.
pub fn plotted_y_range(state: &ViewChartState, options: &ChartOptions) -> (f64, f64) {
    let mut values: Vec<f64> = state
        .points
        .iter()
        .map(|point| displayed_value(point, state, options))
        .collect();
    values.extend(state.dosage_points.iter().map(|point| point.y));
    padded_y_range(&values)
}

/// Split the chart into the panels it should draw.
///
/// One panel always, unless small multiples are on and there is more than one
/// series to separate. Each panel carries its own points, in the unit they are
/// drawn in, and its own y range.
pub fn layout_panels(state: &ViewChartState, options: &ChartOptions) -> Vec<ChartPanel> {
    let plotted: Vec<ViewChartPoint> = state
        .points
        .iter()
        .map(|point| {
            let y = displayed_value(point, state, options);
            let unit = if point.series == ChartSeries::Estradiol {
                options.e2_unit.label().to_string()
            } else {
                point.unit.clone()
            };
            ViewChartPoint {
                y,
                unit,
                ..point.clone()
            }
        })
        .collect();

    let present = series_present(&plotted);
    // Series with different units cannot share one axis honestly: a progesterone
    // result in nmol/L against estradiol in pmol/L reads as a flat line on the
    // floor. Split those automatically; the chip only forces it for equal units.
    let units: std::collections::BTreeSet<&str> =
        plotted.iter().map(|point| point.unit.as_str()).collect();
    let mixed_units = units.len() > 1;
    if (options.separate_panels || mixed_units) && present.len() > 1 {
        return present
            .into_iter()
            .map(|series| {
                let points: Vec<ViewChartPoint> = plotted
                    .iter()
                    .filter(|point| point.series == series)
                    .cloned()
                    .collect();
                let values: Vec<f64> = points.iter().map(|point| point.y).collect();
                let (y_min, y_max) = padded_y_range(&values);
                ChartPanel {
                    series: Some(series),
                    // Name the unit the points really carry, so a panel cannot claim
                    // ng/dL while its axis reads nmol/L.
                    title: points
                        .iter()
                        .find(|point| point.series == series)
                        .map(|point| {
                            if point.unit.is_empty() {
                                series.label().to_string()
                            } else {
                                format!("{} ({})", series.label(), point.unit)
                            }
                        })
                        .unwrap_or_else(|| series.panel_title(options.e2_unit)),
                    y_min,
                    y_max,
                    points,
                }
            })
            .collect();
    }

    let (y_min, y_max) = plotted_y_range(state, options);
    vec![ChartPanel {
        series: None,
        title: combined_y_label(
            &state.y_label,
            state.e2_unit,
            options.e2_unit,
            present.contains(&ChartSeries::Estradiol),
        ),
        y_min,
        y_max,
        points: plotted,
    }]
}

/// Smallest value a logarithmic axis may start at, so a padded range that
/// reaches zero still has a positive floor to map from.
pub const LOG_AXIS_FLOOR: f64 = 0.01;

/// How much of the y span a dose tick covers, measured up from the axis floor.
pub const DOSE_TICK_HEIGHT: f64 = 0.05;

/// The y axis a panel is drawn against, after the log switch and its guard.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum YAxisPlan {
    Linear { min: f64, max: f64 },
    Log { min: f64, max: f64 },
}

impl YAxisPlan {
    pub fn bounds(self) -> (f64, f64) {
        match self {
            YAxisPlan::Linear { min, max } | YAxisPlan::Log { min, max } => (min, max),
        }
    }

    pub fn is_log(self) -> bool {
        matches!(self, YAxisPlan::Log { .. })
    }
}

/// Decide the y axis for a panel.
///
/// A logarithmic axis cannot place a value at or below zero, and a zero would
/// either vanish or draw as a NaN artifact. So the whole panel stays linear
/// unless every plotted value is positive; when they are, a padded range that
/// reaches zero is clamped up to `LOG_AXIS_FLOOR` rather than falling back.
pub fn plan_y_axis(log_scale: bool, y_min: f64, y_max: f64, values: &[f64]) -> YAxisPlan {
    if !log_scale {
        return YAxisPlan::Linear {
            min: y_min,
            max: y_max,
        };
    }
    let loggable = y_min.is_finite()
        && y_max.is_finite()
        && y_max > y_min
        && values.iter().all(|value| value.is_finite() && *value > 0.0);
    if !loggable {
        return YAxisPlan::Linear {
            min: y_min,
            max: y_max,
        };
    }
    let min = if y_min > LOG_AXIS_FLOOR {
        y_min
    } else {
        LOG_AXIS_FLOOR
    };
    let max = if y_max > min { y_max } else { min * 10.0 };
    YAxisPlan::Log { min, max }
}

/// The value sitting `fraction` of the way up an axis, 0.0 being the floor.
/// On a logarithmic axis that spacing is geometric, which is what makes a tick
/// or a band edge land where the eye expects it.
pub fn axis_value_at_fraction(
    axis_min: f64,
    axis_max: f64,
    logarithmically: bool,
    fraction: f64,
) -> f64 {
    if !axis_min.is_finite() || !axis_max.is_finite() || axis_max <= axis_min {
        return axis_min;
    }
    let fraction = fraction.clamp(0.0, 1.0);
    if logarithmically && axis_min > 0.0 {
        axis_min * (axis_max / axis_min).powf(fraction)
    } else {
        axis_min + (axis_max - axis_min) * fraction
    }
}

/// One vertical segment per logged dose: `(x, bottom, top)` in data coordinates.
///
/// The ticks are a rug at the foot of the panel, so they never compete with the
/// series for the same pixels.
pub fn dose_tick_segments(
    doses: &[ViewChartPoint],
    axis_min: f64,
    axis_max: f64,
    logarithmically: bool,
) -> Vec<(f64, f64, f64)> {
    let top = axis_value_at_fraction(axis_min, axis_max, logarithmically, DOSE_TICK_HEIGHT);
    doses
        .iter()
        .filter(|dose| dose.x.is_finite())
        .map(|dose| (dose.x, axis_min, top))
        .collect()
}

/// The range the plotted values actually span, for the descriptive band, or
/// `None` when there is nothing to shade.
pub fn value_band(points: &[ViewChartPoint]) -> Option<(f64, f64)> {
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    for point in points {
        if !point.y.is_finite() {
            continue;
        }
        min = min.min(point.y);
        max = max.max(point.y);
    }
    if min.is_finite() && max.is_finite() && max > min {
        Some((min, max))
    } else {
        None
    }
}

#[allow(clippy::too_many_arguments)]
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

pub fn draw_view_chart(
    canvas_id: &str,
    state: &ViewChartState,
    zoom: ViewZoom,
    options: &ChartOptions,
) {
    let Some(canvas) = window()
        .document()
        .and_then(|doc| doc.get_element_by_id(canvas_id))
        .and_then(|el| el.dyn_into::<HtmlCanvasElement>().ok())
    else {
        return;
    };
    // A split chart needs roughly 170 px per panel for its plot plus a label row;
    // at the CSS height of a single chart the panels and their axes overlap.
    let panels = layout_panels(state, options);
    let wanted_height = if panels.len() > 1 {
        (panels.len() as f64 * 170.0).max(280.0)
    } else {
        0.0
    };
    if wanted_height > 0.0 {
        let current = canvas.get_bounding_client_rect().height();
        if (current - wanted_height).abs() > 1.0 {
            let style = format!("height: {:.0}px;", wanted_height);
            if canvas.get_attribute("style").as_deref() != Some(style.as_str()) {
                canvas.set_attribute("style", &style).ok();
            }
        }
    } else if canvas.get_attribute("style").is_some() {
        canvas.remove_attribute("style").ok();
    }
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
    if panels.is_empty() {
        backend.present().ok();
        return;
    }

    if panels.len() == 1 {
        let panel = &panels[0];
        let plotted: Vec<f64> = panel.points.iter().map(|point| point.y).collect();
        let plan = plan_y_axis(options.log_scale, panel.y_min, panel.y_max, &plotted);
        match plan {
            YAxisPlan::Linear { min, max } => draw_one_panel(
                &backend, panel, state, options, plan, x_min, x_max, min..max, true, true,
            ),
            YAxisPlan::Log { min, max } => draw_one_panel(
                &backend,
                panel,
                state,
                options,
                plan,
                x_min,
                x_max,
                (min..max).log_scale(),
                true,
                true,
            ),
        }
    } else {
        let areas = backend.split_evenly((panels.len(), 1));
        for (index, (area, panel)) in areas.iter().zip(panels.iter()).enumerate() {
            let is_bottom = index + 1 == panels.len();
            if index > 0 {
                let panel_w = area.dim_in_pixel().0 as i32;
                area.draw(&PathElement::new(
                    vec![(0, 1), (panel_w, 1)],
                    ShapeStyle::from(&RGBColor(58, 52, 80)).stroke_width(1),
                ))
                .ok();
            }
            let plotted: Vec<f64> = panel.points.iter().map(|point| point.y).collect();
            let plan = plan_y_axis(options.log_scale, panel.y_min, panel.y_max, &plotted);
            match plan {
                YAxisPlan::Linear { min, max } => draw_one_panel(
                    area, panel, state, options, plan, x_min, x_max, min..max, is_bottom, false,
                ),
                YAxisPlan::Log { min, max } => draw_one_panel(
                    area,
                    panel,
                    state,
                    options,
                    plan,
                    x_min,
                    x_max,
                    (min..max).log_scale(),
                    is_bottom,
                    false,
                ),
            }
        }
    }

    backend.present().ok();
}

#[allow(clippy::too_many_arguments)]
fn draw_one_panel<YR>(
    area: &DrawingArea<CanvasBackend, Shift>,
    panel: &ChartPanel,
    state: &ViewChartState,
    options: &ChartOptions,
    plan: YAxisPlan,
    x_min: f64,
    x_max: f64,
    y_range: YR,
    is_bottom: bool,
    is_only_panel: bool,
) where
    YR: AsRangedCoord<Value = f64>,
    YR::CoordDescType: ValueFormatter<f64>,
{
    // A split panel is short: it gets fewer ticks, a wider gutter for them and a
    // smaller tick font, so the axis description does not sit on top of the numbers.
    let y_label_area = if is_only_panel {
        CHART_Y_LABEL as i32
    } else {
        CHART_Y_LABEL as i32 + 16
    };
    // dim_in_pixel is in device pixels; a phone at dpr 2 or 3 would otherwise look wide.
    let canvas_width_css = (area.dim_in_pixel().0 as f64 / window().device_pixel_ratio()) as i32;
    // A narrow plot needs room on the right for its final date label.
    let margin_right = if canvas_width_css < 560 { 34 } else { CHART_MARGIN as i32 };
    let margin_bottom = if is_only_panel {
        CHART_MARGIN as i32
    } else {
        CHART_MARGIN as i32 + 14
    };
    let mut chart = match ChartBuilder::on(area)
        .margin(CHART_MARGIN as i32)
        .margin_bottom(margin_bottom)
        .margin_right(margin_right)
        .x_label_area_size(if is_bottom { CHART_X_LABEL as i32 } else { 0 })
        .y_label_area_size(y_label_area)
        .build_cartesian_2d(x_min..x_max, y_range)
    {
        Ok(chart) => chart,
        Err(_) => return,
    };

    let tick_font = if is_only_panel { 24 } else { 16 };
    let label_style = ("Quicksand", tick_font)
        .into_font()
        .color(&RGBColor(200, 188, 214));
    let axis_desc_style = ("Quicksand", 24)
        .into_font()
        .color(&RGBColor(200, 188, 214));
    let bold_grid = ShapeStyle::from(&RGBColor(64, 58, 86)).stroke_width(1);
    // Several short panels stacked would turn a dense minor grid into noise.
    let light_grid = if is_only_panel {
        ShapeStyle::from(&RGBColor(42, 38, 60)).stroke_width(1)
    } else {
        ShapeStyle::from(&RGBColor(30, 28, 44)).stroke_width(1)
    };
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

    let mut mesh = chart.configure_mesh();
    let x_tick_count = if canvas_width_css < 560 { 3 } else { 6 };
    mesh.x_labels(if is_bottom { x_tick_count } else { 0 })
        .y_labels(if is_only_panel { 6 } else { 3 })
        .x_label_formatter(&x_label_formatter)
        .label_style(label_style)
        .axis_desc_style(axis_desc_style)
        .axis_style(RGBColor(96, 86, 120))
        .bold_line_style(bold_grid)
        .light_line_style(light_grid);
    // Only the bottom panel names the shared x axis; the others keep the space
    // for their plots.
    if is_bottom {
        mesh.x_desc(state.x_label.clone());
    }
    // One rotated description per panel would stack three labels in the same
    // gutter on a split chart, so only a lone panel gets one.
    if is_only_panel {
        mesh.y_desc(panel.title.clone());
    }
    mesh.draw().ok();

    if !is_only_panel {
        let name_style = ("Quicksand", 15)
            .into_font()
            .color(&RGBColor(200, 188, 214));
        // Clear of the tick labels in the gutter, at the top of the plot.
        area.draw(&Text::new(
            panel.title.clone(),
            (CHART_MARGIN as i32 + y_label_area + 8, 6),
            name_style,
        ))
        .ok();
    }

    let (axis_min, axis_max) = plan.bounds();

    // The descriptive band sits furthest back, so it can only ever be a wash
    // behind the data. It is the plain min..max of what is plotted.
    if options.show_value_band {
        if let Some((band_min, band_max)) = value_band(&panel.points) {
            let band_min = band_min.max(axis_min);
            let band_max = band_max.min(axis_max);
            if band_max > band_min {
                chart
                    .draw_series(std::iter::once(Rectangle::new(
                        [(x_min, band_min), (x_max, band_max)],
                        ShapeStyle::from(&RGBAColor(200, 188, 214, 0.06)).filled(),
                    )))
                    .ok();
                chart
                    .draw_series(std::iter::once(Text::new(
                        "recorded range",
                        (x_min, band_max),
                        ("Quicksand", 18)
                            .into_font()
                            .color(&RGBColor(150, 140, 172)),
                    )))
                    .ok();
            }
        }
    }

    // Dose marks: one quiet tick per dose, still behind the series.
    if options.show_dose_ticks {
        let segments = dose_tick_segments(
            &state.dosage_points,
            axis_min,
            axis_max,
            plan.is_log(),
        );
        let style = ShapeStyle::from(&RGBAColor(200, 188, 214, 0.22)).stroke_width(1);
        chart
            .draw_series(
                segments
                    .into_iter()
                    .map(|(x, bottom, top)| PathElement::new(vec![(x, bottom), (x, top)], style)),
            )
            .ok();
    }

    for series in ChartSeries::ALL {
        let mut line: Vec<(f64, f64)> = panel
            .points
            .iter()
            .filter(|point| point.series == series)
            .map(|point| (point.x, point.y))
            .collect();
        if line.is_empty() {
            continue;
        }
        line.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        let color = series.color();
        chart.draw_series(LineSeries::new(line, &color)).ok();
    }

    for point in &panel.points {
        chart
            .draw_series(std::iter::once(Circle::new(
                (point.x, point.y),
                10,
                point.color.filled(),
            )))
            .ok();
    }

    // The dose markers are scaled for visibility, not measured levels, so they
    // only belong on the combined plot with its linear axis.
    if panel.series.is_none() && !plan.is_log() {
        for point in &state.dosage_points {
            chart
                .draw_series(std::iter::once(TriangleMarker::new(
                    (point.x, point.y),
                    16,
                    point.color.filled(),
                )))
                .ok();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(x: f64, y: f64, series: ChartSeries) -> ViewChartPoint {
        ViewChartPoint {
            x,
            y,
            label: String::new(),
            color: series.color(),
            series,
            unit: String::new(),
            timestamp_ms: x as i64,
        }
    }

    fn state(
        points: Vec<ViewChartPoint>,
        dosage_points: Vec<ViewChartPoint>,
        e2_unit: E2DisplayUnit,
    ) -> ViewChartState {
        ViewChartState {
            domain_min: 0.0,
            domain_max: 100.0,
            y_min: 0.0,
            y_max: 1.0,
            x_label: "Date".to_string(),
            y_label: "Levels".to_string(),
            points,
            dosage_points,
            first_dose: None,
            use_days: false,
            has_data: true,
            e2_unit,
        }
    }

    fn two_series_state() -> ViewChartState {
        let mut estradiol = point(10.0, 367.13, ChartSeries::Estradiol);
        estradiol.unit = "pmol/L".to_string();
        let mut testosterone = point(20.0, 1.91, ChartSeries::Testosterone);
        testosterone.unit = "nmol/L".to_string();
        let mut estradiol_late = point(30.0, 734.26, ChartSeries::Estradiol);
        estradiol_late.unit = "pmol/L".to_string();
        state(
            vec![estradiol, estradiol_late, testosterone],
            Vec::new(),
            E2DisplayUnit::PmolL,
        )
    }

    #[test]
    fn e2_unit_conversion_matches_the_display_helpers() {
        let pg = e2_value_in_unit(367.13, E2DisplayUnit::PmolL, E2DisplayUnit::PgMl);
        assert!((pg - 100.0).abs() < 0.1, "got {pg}");
        let pmol = e2_value_in_unit(100.0, E2DisplayUnit::PgMl, E2DisplayUnit::PmolL);
        assert!((pmol - 367.13).abs() < 0.1, "got {pmol}");
        assert_eq!(
            e2_value_in_unit(42.0, E2DisplayUnit::PmolL, E2DisplayUnit::PmolL),
            42.0
        );
        assert_eq!(
            e2_value_in_unit(42.0, E2DisplayUnit::PgMl, E2DisplayUnit::PgMl),
            42.0
        );
    }

    #[test]
    fn e2_unit_switch_converts_estradiol_and_nothing_else() {
        let state = two_series_state();
        let options = ChartOptions {
            e2_unit: E2DisplayUnit::PgMl,
            ..ChartOptions::default()
        };
        // Estradiol in pg/mL and testosterone in nmol/L cannot share an axis, so
        // these two series get a panel each.
        let panels = layout_panels(&state, &options);
        assert_eq!(panels.len(), 2);
        let estradiol_panel = panels
            .iter()
            .find(|panel| panel.series == Some(ChartSeries::Estradiol))
            .expect("estradiol panel");
        assert_eq!(estradiol_panel.points.len(), 2);
        assert!(
            (estradiol_panel.points[0].y - 100.0).abs() < 0.1,
            "got {}",
            estradiol_panel.points[0].y
        );
        assert!(
            (estradiol_panel.points[1].y - 200.0).abs() < 0.1,
            "got {}",
            estradiol_panel.points[1].y
        );
        assert_eq!(estradiol_panel.points[0].unit, "pg/mL");
        let testosterone_panel = panels
            .iter()
            .find(|panel| panel.series == Some(ChartSeries::Testosterone))
            .expect("testosterone panel");
        let testosterone = &testosterone_panel.points[0];
        assert_eq!(testosterone.y, 1.91);
        assert_eq!(testosterone.unit, "nmol/L");
    }

    #[test]
    fn log_axis_plan_keeps_a_positive_range_as_given() {
        let plan = plan_y_axis(true, 20.0, 800.0, &[30.0, 500.0]);
        assert_eq!(plan, YAxisPlan::Log { min: 20.0, max: 800.0 });
        assert!(plan.is_log());
        assert_eq!(plan.bounds(), (20.0, 800.0));
    }

    #[test]
    fn log_axis_plan_clamps_a_padded_zero_floor() {
        // padded_y_range floors at zero even for positive data, and zero has no
        // log position, so the floor is lifted instead of falling back.
        let plan = plan_y_axis(true, 0.0, 800.0, &[100.0, 300.0]);
        assert_eq!(
            plan,
            YAxisPlan::Log {
                min: LOG_AXIS_FLOOR,
                max: 800.0
            }
        );
    }

    #[test]
    fn log_axis_plan_falls_back_to_linear_for_non_positive_values() {
        let zero = plan_y_axis(true, 0.0, 800.0, &[0.0, 300.0]);
        assert_eq!(zero, YAxisPlan::Linear { min: 0.0, max: 800.0 });
        assert!(!zero.is_log());
        let negative = plan_y_axis(true, -5.0, 10.0, &[-5.0, 10.0]);
        assert_eq!(negative, YAxisPlan::Linear { min: -5.0, max: 10.0 });
        let empty = plan_y_axis(true, 0.0, 1.0, &[]);
        assert_eq!(empty, YAxisPlan::Log { min: LOG_AXIS_FLOOR, max: 1.0 });
    }

    #[test]
    fn log_axis_plan_is_linear_when_the_switch_is_off() {
        assert_eq!(
            plan_y_axis(false, 0.0, 800.0, &[100.0]),
            YAxisPlan::Linear { min: 0.0, max: 800.0 }
        );
    }

    #[test]
    fn axis_fraction_spaces_log_axes_geometrically() {
        // Half way up 1..100 on a log axis is 10, not 50.5.
        let log_mid = axis_value_at_fraction(1.0, 100.0, true, 0.5);
        assert!((log_mid - 10.0).abs() < 1e-9, "got {log_mid}");
        let linear_mid = axis_value_at_fraction(1.0, 100.0, false, 0.5);
        assert!((linear_mid - 50.5).abs() < 1e-9, "got {linear_mid}");
        assert_eq!(axis_value_at_fraction(2.0, 8.0, true, 0.0), 2.0);
        assert!((axis_value_at_fraction(2.0, 8.0, true, 1.0) - 8.0).abs() < 1e-9);
        // A degenerate or non-finite span cannot be divided, so it stays put.
        assert_eq!(axis_value_at_fraction(5.0, 5.0, true, 0.5), 5.0);
        assert_eq!(axis_value_at_fraction(5.0, 1.0, false, 0.5), 5.0);
    }

    #[test]
    fn settings_pick_the_unit_a_hormone_is_shown_in() {
        // nmol/L converted to ng/dL and back, the two directions the setting allows.
        let mg_dl: f64 = 1.94 * 28.818;
        assert!((mg_dl - 55.91).abs() < 0.05);
        let nmol: f64 = 55.0 / 28.818;
        assert!((nmol - 1.909).abs() < 0.01);
        // Progesterone: nmol/L to ng/mL.
        let ng_ml: f64 = 3.2 * 0.31;
        assert!((ng_ml - 0.992).abs() < 0.001);
        let back: f64 = 1.0 / 0.31;
        assert!((back - 3.226).abs() < 0.01);
    }

    #[test]
    fn default_options_match_todays_rendering() {
        let options = ChartOptions::default();
        assert_eq!(options.e2_unit, E2DisplayUnit::PmolL);
        assert!(!options.log_scale);
        assert!(!options.show_dose_ticks);
        assert!(!options.show_value_band);
        assert!(!options.separate_panels);

        // One series, or several that share a unit, keep the single combined plot.
        let mut only_estradiol = two_series_state();
        only_estradiol
            .points
            .retain(|point| point.series == ChartSeries::Estradiol);
        let panels = layout_panels(&only_estradiol, &options);
        assert_eq!(panels.len(), 1);
        assert_eq!(panels[0].series, None);
        assert_eq!(panels[0].title, "Levels");
        assert_eq!(panels[0].points.len(), 2);
        // Default options leave every value and unit exactly as computed.
        assert_eq!(panels[0].points[0].y, 367.13);
        assert_eq!(panels[0].points[0].unit, "pmol/L");

        // Series in different units split, because one axis cannot carry pmol/L and
        // nmol/L honestly.
        let state = two_series_state();
        let mixed = layout_panels(&state, &options);
        assert_eq!(mixed.len(), 2);
        // And the axis range is the one compute_view_chart_state already stored.
        let mut values: Vec<f64> = state.points.iter().map(|point| point.y).collect();
        values.extend(state.dosage_points.iter().map(|point| point.y));
        assert_eq!(plotted_y_range(&state, &options), padded_y_range(&values));
        assert_eq!(
            plan_y_axis(options.log_scale, panels[0].y_min, panels[0].y_max, &[1.0]),
            YAxisPlan::Linear {
                min: panels[0].y_min,
                max: panels[0].y_max
            }
        );
    }

    #[test]
    fn small_multiples_give_one_panel_per_series() {
        let state = two_series_state();
        let options = ChartOptions {
            separate_panels: true,
            ..ChartOptions::default()
        };
        let panels = layout_panels(&state, &options);
        assert_eq!(panels.len(), 2);
        assert_eq!(panels[0].series, Some(ChartSeries::Estradiol));
        assert_eq!(panels[1].series, Some(ChartSeries::Testosterone));
        assert_eq!(panels[0].title, "Estradiol (pmol/L)");
        assert_eq!(panels[1].title, "Testosterone (nmol/L)");
        assert_eq!(panels[0].points.len(), 2);
        assert_eq!(panels[1].points.len(), 1);
        assert!(panels[0]
            .points
            .iter()
            .all(|point| point.series == ChartSeries::Estradiol));
        // Each panel has its own range: estradiol runs to 734 pmol/L, testo stays near 55.
        assert!(panels[0].y_max > panels[1].y_max);
        assert!(panels[0].y_min > panels[1].y_min);
        // Every point in every panel still carries its timestamp, value, series and unit.
        for panel in &panels {
            for point in &panel.points {
                assert!(point.timestamp_ms == point.x as i64);
                assert!(!point.unit.is_empty());
            }
        }
    }

    #[test]
    fn dose_markers_never_count_as_a_series_or_a_panel() {
        let mut dose = point(10.0, 100.0, ChartSeries::Dose);
        dose.unit = String::new();
        let mut base = two_series_state();
        base.dosage_points = vec![dose];
        let options = ChartOptions {
            separate_panels: true,
            show_dose_ticks: true,
            ..ChartOptions::default()
        };
        let panels = layout_panels(&base, &options);
        // Two blood series, two panels; the dose adds none.
        assert_eq!(panels.len(), 2);
        assert!(series_present(&base.dosage_points).is_empty());
        // The combined axis still covers the scaled dose values, as it always has.
        assert!(plotted_y_range(&base, &ChartOptions::default()).1 >= 100.0);
    }

    #[test]
    fn small_multiples_with_one_series_stays_a_single_panel() {
        let mut estradiol = point(10.0, 367.13, ChartSeries::Estradiol);
        estradiol.unit = "pmol/L".to_string();
        let state = state(vec![estradiol], Vec::new(), E2DisplayUnit::PmolL);
        let options = ChartOptions {
            separate_panels: true,
            ..ChartOptions::default()
        };
        let panels = layout_panels(&state, &options);
        assert_eq!(panels.len(), 1);
        assert_eq!(panels[0].series, None);
    }

    #[test]
    fn dose_ticks_are_one_per_dose_at_the_foot_of_the_panel() {
        let doses = vec![
            point(10.0, 100.0, ChartSeries::Estradiol),
            point(20.0, 100.0, ChartSeries::Estradiol),
        ];
        let segments = dose_tick_segments(&doses, 0.0, 800.0, false);
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].0, 10.0);
        assert_eq!(segments[1].0, 20.0);
        let expected_top = 800.0 * DOSE_TICK_HEIGHT;
        for (_, bottom, top) in &segments {
            assert_eq!(*bottom, 0.0);
            assert!((*top - expected_top).abs() < 1e-9, "got {top}");
            assert!(*top < 800.0);
        }
    }

    #[test]
    fn dose_ticks_are_geometric_on_a_log_axis() {
        let doses = vec![point(10.0, 100.0, ChartSeries::Estradiol)];
        let segments = dose_tick_segments(&doses, 1.0, 10000.0, true);
        assert_eq!(segments.len(), 1);
        let (_, bottom, top) = segments[0];
        assert_eq!(bottom, 1.0);
        // 5% of four decades up from 1 is 10^0.2.
        assert!((top - 10000f64.powf(DOSE_TICK_HEIGHT)).abs() < 1e-9, "got {top}");
    }

    #[test]
    fn dose_ticks_skip_doses_with_no_x_position() {
        let doses = vec![
            point(f64::NAN, 100.0, ChartSeries::Estradiol),
            point(20.0, 100.0, ChartSeries::Estradiol),
        ];
        let segments = dose_tick_segments(&doses, 0.0, 800.0, false);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].0, 20.0);
    }

    #[test]
    fn value_band_spans_the_recorded_min_and_max() {
        let points = vec![
            point(1.0, 120.0, ChartSeries::Estradiol),
            point(2.0, 480.0, ChartSeries::Estradiol),
            point(3.0, 240.0, ChartSeries::Estradiol),
        ];
        assert_eq!(value_band(&points), Some((120.0, 480.0)));
        assert_eq!(value_band(&[]), None);
        let flat = vec![
            point(1.0, 200.0, ChartSeries::Estradiol),
            point(2.0, 200.0, ChartSeries::Estradiol),
        ];
        assert_eq!(value_band(&flat), None);
        let unknown = vec![point(1.0, f64::NAN, ChartSeries::Estradiol)];
        assert_eq!(value_band(&unknown), None);
    }

    #[test]
    fn combined_label_names_the_unit_only_when_it_differs() {
        assert_eq!(
            combined_y_label("Levels", E2DisplayUnit::PmolL, E2DisplayUnit::PmolL, true),
            "Levels"
        );
        assert_eq!(
            combined_y_label("Levels", E2DisplayUnit::PmolL, E2DisplayUnit::PgMl, true),
            "Levels (E2 pg/mL)"
        );
        // No estradiol on the chart means the unit is irrelevant.
        assert_eq!(
            combined_y_label("Levels", E2DisplayUnit::PmolL, E2DisplayUnit::PgMl, false),
            "Levels"
        );
    }

    #[test]
    fn padded_range_pads_eight_percent_and_floors_at_zero() {
        let (min, max) = padded_y_range(&[100.0, 300.0]);
        assert!((min - 84.0).abs() < 1e-9, "got {min}");
        assert!((max - 316.0).abs() < 1e-9, "got {max}");
        // A single value is widened a unit either side, without going negative.
        assert_eq!(padded_y_range(&[50.0]), (49.0, 51.0));
        assert_eq!(padded_y_range(&[0.0]), (0.0, 1.0));
        // An empty set falls back to 0..1 and is then padded like any other.
        let (min, max) = padded_y_range(&[]);
        assert_eq!(min, 0.0);
        assert!((max - 1.08).abs() < 1e-9, "got {max}");
    }

    #[test]
    fn series_palette_is_unchanged() {
        assert_eq!(ChartSeries::Estradiol.color(), RGBColor(70, 130, 180));
        assert_eq!(ChartSeries::Testosterone.color(), RGBColor(220, 20, 60));
        assert_eq!(ChartSeries::Progesterone.color(), RGBColor(148, 0, 211));
        assert_eq!(ChartSeries::Fsh.color(), RGBColor(34, 139, 34));
        assert_eq!(ChartSeries::Lh.color(), RGBColor(0, 139, 139));
        assert_eq!(ChartSeries::Prolactin.color(), RGBColor(139, 69, 19));
        assert_eq!(ChartSeries::Shbg.color(), RGBColor(255, 20, 147));
        assert_eq!(ChartSeries::FreeAndrogenIndex.color(), RGBColor(0, 0, 0));
        assert_eq!(ChartSeries::FudgeFactor.color(), RGBColor(255, 99, 132));
    }

    #[test]
    fn panel_titles_carry_each_series_unit() {
        assert_eq!(
            ChartSeries::Estradiol.panel_title(E2DisplayUnit::PgMl),
            "Estradiol (pg/mL)"
        );
        assert_eq!(
            ChartSeries::Testosterone.panel_title(E2DisplayUnit::PmolL),
            "Testosterone (nmol/L)"
        );
        assert_eq!(
            ChartSeries::FreeAndrogenIndex.panel_title(E2DisplayUnit::PmolL),
            "FAI"
        );
        assert_eq!(
            ChartSeries::FudgeFactor.panel_title(E2DisplayUnit::PmolL),
            "Efficacy (FF) (%)"
        );
    }

    #[test]
    fn series_present_lists_selected_series_in_chart_order() {
        let state = two_series_state();
        assert_eq!(
            series_present(&state.points),
            vec![ChartSeries::Estradiol, ChartSeries::Testosterone]
        );
        assert!(series_present(&[]).is_empty());
    }
}
