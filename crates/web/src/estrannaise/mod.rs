use hrt_shared::estrannaise::e2_multidose_3c;
use hrt_shared::types::{
    BloodTest, EstrannaiseModel, HormoneUnits, HrtData, InjectableEstradiols, Settings,
};

use crate::charts::estrannaise::{EstrannaisePoint, EstrannaiseSeries};
use crate::utils::{
    convert_estradiol_to_display, estradiol_conversion_factor, fmt_date_label, hormone_unit_label,
};

fn map_estrannaise_model(kind: &InjectableEstradiols) -> Option<EstrannaiseModel> {
    match kind {
        InjectableEstradiols::Benzoate => Some(EstrannaiseModel::EbIm),
        InjectableEstradiols::Valerate => Some(EstrannaiseModel::EvIm),
        InjectableEstradiols::Enanthate => Some(EstrannaiseModel::EEnIm),
        InjectableEstradiols::Cypionate => Some(EstrannaiseModel::EcIm),
        InjectableEstradiols::Undecylate => Some(EstrannaiseModel::EUnIm),
        InjectableEstradiols::PolyestradiolPhosphate => None,
    }
}

fn extract_fudge_series(tests: &[BloodTest]) -> Vec<(i64, f64)> {
    let mut series: Vec<(i64, f64)> = tests
        .iter()
        .filter_map(|t| t.fudgeFactor.map(|value| (t.date, value)))
        .collect();
    series.sort_by_key(|(date, _)| *date);
    if series.is_empty() {
        series.push((js_sys::Date::now() as i64, 1.0));
    }
    series
}

fn blend_fudge(series: &[(i64, f64)], target: i64) -> f64 {
    if series.is_empty() {
        return 1.0;
    }
    if target <= series[0].0 {
        return series[0].1;
    }
    let last = series[series.len() - 1];
    if target >= last.0 {
        return last.1;
    }
    for window in series.windows(2) {
        let (prev_date, prev_val) = window[0];
        let (next_date, next_val) = window[1];
        if target <= next_date {
            let span = (next_date - prev_date) as f64;
            if span <= 0.0 {
                return prev_val;
            }
            let ratio = (target - prev_date) as f64 / span;
            return prev_val + (next_val - prev_val) * ratio;
        }
    }
    last.1
}

fn step_fudge(series: &[(i64, f64)], target: i64) -> f64 {
    if series.is_empty() {
        return 1.0;
    }
    if target <= series[0].0 {
        return series[0].1;
    }
    for window in series.windows(2) {
        let (_prev_date, prev_val) = window[0];
        let (next_date, _) = window[1];
        if target < next_date {
            return prev_val;
        }
    }
    series[series.len() - 1].1
}

pub fn compute_estrannaise_series(
    data: &HrtData,
    settings: &Settings,
    axis_mode: &str,
    forecast_enabled: bool,
    forecast_weeks: i64,
    forecast_dose_override: Option<f64>,
    forecast_freq_override: Option<f64>,
    stepped_fudge_override: Option<f64>,
) -> EstrannaiseSeries {
    let display_unit = settings
        .displayEstradiolUnit
        .clone()
        .unwrap_or(HormoneUnits::E2PmolL);
    let conversion = estradiol_conversion_factor(&display_unit);
    let stepped_fudge_override =
        stepped_fudge_override.filter(|value| value.is_finite() && *value > 0.0);
    let dose_history: Vec<_> = data
        .dosageHistory
        .iter()
        .filter_map(|d| match d {
            hrt_shared::types::DosageHistoryEntry::InjectableEstradiol {
                date,
                kind,
                dose,
                unit: _,
                ..
            } => Some((*date, kind.clone(), *dose)),
            _ => None,
        })
        .collect();
    let mut dose_history = dose_history;
    dose_history.sort_by_key(|(date, _, _)| *date);

    let earliest_date = if !dose_history.is_empty() {
        Some(dose_history.first().map(|(date, _, _)| *date).unwrap())
    } else {
        data.bloodTests.iter().map(|t| t.date).min()
    };
    let Some(first_dose) = earliest_date else {
        return EstrannaiseSeries::default();
    };

    let last_dose = dose_history
        .last()
        .map(|(date, _, _)| *date)
        .unwrap_or(first_dose);
    let start_date = first_dose;
    let base_end = (last_dose + 30 * 24 * 60 * 60 * 1000).max(js_sys::Date::now() as i64);
    let forecast_weeks = forecast_weeks.clamp(4, 8);
    let forecast_end =
        base_end.max(js_sys::Date::now() as i64 + forecast_weeks * 7 * 24 * 60 * 60 * 1000);

    let schedule = data.injectableEstradiol.clone();
    let forecast_start = js_sys::Date::now() as i64;
    let forecast_start_date = schedule
        .as_ref()
        .and_then(|s| s.nextDoseDate)
        .unwrap_or(forecast_start)
        .max(forecast_start);
    let forecast_dose = forecast_dose_override.or_else(|| schedule.as_ref().map(|s| s.dose));
    let forecast_freq = forecast_freq_override.or_else(|| schedule.as_ref().map(|s| s.frequency));
    let forecast_type = schedule
        .as_ref()
        .map(|s| s.kind.clone())
        .or_else(|| dose_history.last().map(|(_, kind, _)| kind.clone()));

    let mut forecast_doses = Vec::new();
    if forecast_enabled {
        if let (Some(dose), Some(freq), Some(kind)) = (forecast_dose, forecast_freq, forecast_type)
        {
            let mut t = forecast_start_date;
            while t <= forecast_end {
                forecast_doses.push((t, kind.clone(), dose));
                t += (freq * 24.0 * 60.0 * 60.0 * 1000.0) as i64;
            }
        }
    }

    let mut all_doses = dose_history.clone();
    all_doses.extend(forecast_doses.clone());
    all_doses.sort_by_key(|(date, _, _)| *date);

    let series = extract_fudge_series(&data.bloodTests);
    let base_step_entry = if series.len() > 1 {
        series[series.len() - 2]
    } else {
        series[series.len() - 1]
    };
    let step_ms = 6 * 60 * 60 * 1000;
    let mut blended = Vec::new();
    let mut stepped = Vec::new();
    let mut y_values = Vec::new();

    let mut time_map = Vec::new();
    let mut dose_map = Vec::new();
    let mut model_map = Vec::new();
    for (date, kind, dose) in &all_doses {
        if let Some(model) = map_estrannaise_model(kind) {
            time_map.push((*date - start_date) as f64 / (24.0 * 60.0 * 60.0 * 1000.0));
            dose_map.push(*dose);
            model_map.push(model);
        }
    }

    if !model_map.is_empty() {
        let mut t = start_date;
        while t <= forecast_end {
            let day_value = (t - start_date) as f64 / (24.0 * 60.0 * 60.0 * 1000.0);
            let blended_fudge = blend_fudge(&series, t);
            let step_fudge = if let Some(override_value) = stepped_fudge_override {
                override_value
            } else if t >= base_step_entry.0 {
                base_step_entry.1
            } else {
                step_fudge(&series, t)
            };
            let blended_val = e2_multidose_3c(
                day_value,
                &dose_map,
                &time_map,
                &model_map,
                blended_fudge * conversion,
                false,
            );
            let stepped_val = e2_multidose_3c(
                day_value,
                &dose_map,
                &time_map,
                &model_map,
                step_fudge * conversion,
                false,
            );
            let x = if axis_mode == "days" {
                day_value
            } else {
                t as f64
            };
            let label = fmt_date_label(t, axis_mode, Some(start_date));
            blended.push(EstrannaisePoint {
                x,
                y: blended_val,
                label: format!("Blended: {:.1} ({})", blended_val, label),
            });
            stepped.push(EstrannaisePoint {
                x,
                y: stepped_val,
                label: format!("Step: {:.1} ({})", stepped_val, label),
            });
            y_values.push(blended_val);
            y_values.push(stepped_val);
            t += step_ms;
        }
    }

    let blood: Vec<EstrannaisePoint> = data
        .bloodTests
        .iter()
        .filter_map(|test| {
            test.estradiolLevel
                .map(|value| (test.date, value, test.estradiolUnit.clone()))
        })
        .map(|(date, value, unit)| {
            let raw_unit = unit.unwrap_or(display_unit.clone());
            let plot_val = convert_estradiol_to_display(value, &raw_unit, &display_unit);
            let x = if axis_mode == "days" {
                (date - first_dose) as f64 / (24.0 * 60.0 * 60.0 * 1000.0)
            } else {
                date as f64
            };
            let label = fmt_date_label(date, axis_mode, Some(first_dose));
            y_values.push(plot_val);
            EstrannaisePoint {
                x,
                y: plot_val,
                label: format!("Blood test: {:.1} ({})", plot_val, label),
            }
        })
        .collect();

    let mut y_min = y_values.iter().cloned().fold(f64::INFINITY, f64::min);
    let mut y_max = y_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    if (y_min - y_max).abs() < f64::EPSILON {
        y_min = y_min - 1.0;
        y_max += 1.0;
    } else {
        let pad = (y_max - y_min) * 0.08;
        y_min -= pad;
        y_max += pad;
    }

    let domain_min = if axis_mode == "days" {
        0.0
    } else {
        start_date as f64
    };
    let domain_max = if axis_mode == "days" {
        ((forecast_end - start_date) as f64 / (24.0 * 60.0 * 60.0 * 1000.0)).max(30.0)
    } else {
        forecast_end as f64
    };

    let forecast = if forecast_enabled {
        let start_x = if axis_mode == "days" {
            (forecast_start - start_date) as f64 / (24.0 * 60.0 * 60.0 * 1000.0)
        } else {
            forecast_start as f64
        };
        let end_x = if axis_mode == "days" {
            (forecast_end - start_date) as f64 / (24.0 * 60.0 * 60.0 * 1000.0)
        } else {
            forecast_end as f64
        };
        Some((start_x, end_x))
    } else {
        None
    };

    EstrannaiseSeries {
        blended,
        stepped,
        blood,
        forecast,
        step_split_x: if forecast_enabled && stepped_fudge_override.is_none() {
            let split_date = base_step_entry.0;
            Some(if axis_mode == "days" {
                (split_date - start_date) as f64 / (24.0 * 60.0 * 60.0 * 1000.0)
            } else {
                split_date as f64
            })
        } else {
            None
        },
        domain_min,
        domain_max,
        y_min,
        y_max,
        x_label: if axis_mode == "days" {
            "Days since first dose".to_string()
        } else {
            "Date".to_string()
        },
        y_label: format!("E2 ({})", hormone_unit_label(&display_unit)),
        first_dose: Some(first_dose),
        use_days: axis_mode == "days",
    }
}
