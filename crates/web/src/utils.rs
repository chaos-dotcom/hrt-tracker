use chrono::{Local, TimeZone};
use hrt_shared::types::{HormoneUnits, HrtData, LengthUnit};
use js_sys::Date;

pub fn parse_date_or_now(value: &str) -> i64 {
    if value.trim().is_empty() {
        return Date::now() as i64;
    }
    let parts: Vec<i64> = value
        .split('-')
        .filter_map(|v| v.parse::<i64>().ok())
        .collect();
    if parts.len() != 3 {
        return Date::now() as i64;
    }
    let (year, month, day) = (parts[0], parts[1], parts[2]);
    if year <= 0 || month == 0 || day == 0 {
        return Date::now() as i64;
    }
    let date = Date::new_with_year_month_day(year as u32, (month - 1) as i32, day as i32);
    date.get_time() as i64
}

pub fn parse_hormone_unit(value: &str) -> Option<HormoneUnits> {
    match value {
        "pg/mL" => Some(HormoneUnits::E2PgMl),
        "pmol/L" => Some(HormoneUnits::E2PmolL),
        "ng/dL" => Some(HormoneUnits::TNgDl),
        "nmol/L" => Some(HormoneUnits::TNmolL),
        "mg" => Some(HormoneUnits::Mg),
        "ng/mL" => Some(HormoneUnits::NgMl),
        "mIU/mL" => Some(HormoneUnits::MIuMl),
        "mIU/L" => Some(HormoneUnits::MIuL),
        "U/L" => Some(HormoneUnits::UL),
        _ => None,
    }
}

pub fn parse_length_unit(value: &str) -> Option<LengthUnit> {
    match value {
        "cm" => Some(LengthUnit::CM),
        "in" => Some(LengthUnit::IN),
        _ => None,
    }
}

pub fn parse_decimal(value: &str) -> Option<f64> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let normalized = trimmed.replace(',', ".");
    normalized.parse::<f64>().ok().filter(|v| v.is_finite())
}

pub fn parse_decimal_or_nan(value: &str) -> f64 {
    parse_decimal(value).unwrap_or(f64::NAN)
}

pub fn hormone_unit_label(unit: &HormoneUnits) -> &'static str {
    match unit {
        HormoneUnits::E2PmolL => "pmol/L",
        HormoneUnits::E2PgMl => "pg/mL",
        HormoneUnits::TNgDl => "ng/dL",
        HormoneUnits::TNmolL => "nmol/L",
        HormoneUnits::Mg => "mg",
        HormoneUnits::NgMl => "ng/mL",
        HormoneUnits::MIuMl => "mIU/mL",
        HormoneUnits::MIuL => "mIU/L",
        HormoneUnits::UL => "U/L",
    }
}

pub fn convert_estradiol_to_display(
    value: f64,
    unit: &HormoneUnits,
    display_unit: &HormoneUnits,
) -> f64 {
    if display_unit == &HormoneUnits::E2PmolL {
        if unit == &HormoneUnits::E2PmolL {
            value
        } else {
            value * 3.6713
        }
    } else if unit == &HormoneUnits::E2PmolL {
        value / 3.6713
    } else {
        value
    }
}

pub fn estradiol_conversion_factor(display_unit: &HormoneUnits) -> f64 {
    if display_unit == &HormoneUnits::E2PmolL {
        3.6713
    } else {
        1.0
    }
}

pub fn convert_testosterone_to_ng_dl(value: f64, unit: &HormoneUnits) -> f64 {
    if unit == &HormoneUnits::TNmolL {
        value * 28.818
    } else {
        value
    }
}

pub fn convert_fsh_to_miu_ml(value: f64, unit: &HormoneUnits) -> f64 {
    match unit {
        HormoneUnits::MIuL => value / 1000.0,
        HormoneUnits::UL => value,
        _ => value,
    }
}

pub fn convert_lh_to_miu_ml(value: f64, unit: &HormoneUnits) -> f64 {
    match unit {
        HormoneUnits::MIuL => value / 1000.0,
        HormoneUnits::UL => value,
        _ => value,
    }
}

pub fn convert_progesterone_to_ng_ml(value: f64, unit: &HormoneUnits) -> f64 {
    if unit == &HormoneUnits::TNmolL {
        value * 0.31
    } else {
        value
    }
}

pub fn fmt_decimal(value: f64, max_decimals: usize) -> String {
    if !value.is_finite() {
        return "-".to_string();
    }
    let formatted = format!("{value:.precision$}", precision = max_decimals);
    formatted
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

pub fn fmt_blood_value(value: f64) -> String {
    fmt_decimal(value, 4)
}

fn vial_concentration_mg_ml(data: &HrtData, vial_id: &str) -> Option<f64> {
    data.vials
        .iter()
        .find(|vial| vial.id == vial_id)
        .and_then(|vial| vial.concentrationMgPerMl)
        .filter(|conc| *conc > 0.0)
}

pub fn injectable_concentration_mg_ml(
    data: &HrtData,
    vial_id: Option<&String>,
    schedule_vial_id: Option<&String>,
) -> Option<f64> {
    vial_id
        .and_then(|id| vial_concentration_mg_ml(data, id))
        .or_else(|| schedule_vial_id.and_then(|id| vial_concentration_mg_ml(data, id)))
}

pub fn injectable_iu_from_dose(
    data: &HrtData,
    dose: f64,
    unit: &HormoneUnits,
    vial_id: Option<&String>,
    schedule_vial_id: Option<&String>,
) -> Option<f64> {
    if *unit != HormoneUnits::Mg || !dose.is_finite() || dose <= 0.0 {
        return None;
    }
    let conc = injectable_concentration_mg_ml(data, vial_id, schedule_vial_id);
    let conc = conc?;
    let ml = dose / conc;
    let iu = ml * 100.0;
    if iu.is_finite() && iu > 0.0 {
        Some(iu)
    } else {
        None
    }
}

pub fn injectable_dose_from_iu(
    data: &HrtData,
    iu: f64,
    vial_id: Option<&String>,
    schedule_vial_id: Option<&String>,
) -> Option<f64> {
    if !iu.is_finite() || iu <= 0.0 {
        return None;
    }
    let conc = injectable_concentration_mg_ml(data, vial_id, schedule_vial_id)?;
    let ml = iu / 100.0;
    let dose = ml * conc;
    if dose.is_finite() && dose > 0.0 {
        Some(dose)
    } else {
        None
    }
}

pub fn format_injectable_dose(
    data: &HrtData,
    dose: f64,
    unit: &HormoneUnits,
    vial_id: Option<&String>,
    schedule_vial_id: Option<&String>,
    use_iu: bool,
) -> String {
    if !dose.is_finite() {
        return "-".to_string();
    }
    if !use_iu || *unit != HormoneUnits::Mg {
        return format!("{} {}", fmt_decimal(dose, 3), hormone_unit_label(unit));
    }
    if let Some(iu) = injectable_iu_from_dose(data, dose, unit, vial_id, schedule_vial_id) {
        return format!("{} IU ({} mg)", iu.round() as i64, fmt_decimal(dose, 3));
    }
    format!("{} {}", fmt_decimal(dose, 3), hormone_unit_label(unit))
}

pub fn compute_fudge_factor(
    measured_pg_ml: Option<f64>,
    predicted_pg_ml: Option<f64>,
) -> Option<f64> {
    let Some(measured) = measured_pg_ml else {
        return None;
    };
    if !measured.is_finite() {
        return None;
    }
    let predicted = predicted_pg_ml.filter(|value| value.is_finite() && *value > 0.0);
    let fudge = if let Some(predicted) = predicted {
        let ratio = measured / predicted;
        if ratio.is_finite() {
            ratio
        } else {
            1.0
        }
    } else {
        1.0
    };
    Some((fudge * 1000.0).round() / 1000.0)
}

pub fn fmt_date_label(date_ms: i64, axis_mode: &str, first_dose: Option<i64>) -> String {
    const DAY_MS: i64 = 24 * 60 * 60 * 1000;
    if axis_mode == "days" {
        if let Some(first) = first_dose {
            let days = (date_ms - first) as f64 / DAY_MS as f64;
            return format!("Day {:.1}", days);
        }
    }
    Local
        .timestamp_millis_opt(date_ms)
        .single()
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| date_ms.to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        injectable_dose_from_iu, injectable_iu_from_dose, parse_decimal, parse_decimal_or_nan,
    };
    use hrt_shared::types::{HormoneUnits, HrtData, Vial};

    #[test]
    fn parse_decimal_accepts_dot_and_comma_inputs() {
        assert_eq!(parse_decimal("0.2"), Some(0.2));
        assert_eq!(parse_decimal("0,2"), Some(0.2));
        assert_eq!(parse_decimal(" 1.25 "), Some(1.25));
        assert_eq!(parse_decimal(" 1,25 "), Some(1.25));
    }

    #[test]
    fn parse_decimal_rejects_empty_and_invalid_values() {
        assert_eq!(parse_decimal(""), None);
        assert_eq!(parse_decimal("   "), None);
        assert_eq!(parse_decimal("."), None);
        assert_eq!(parse_decimal(","), None);
        assert_eq!(parse_decimal("abc"), None);
    }

    #[test]
    fn parse_decimal_or_nan_returns_nan_for_invalid_values() {
        assert!(parse_decimal_or_nan("not-a-number").is_nan());
        assert_eq!(parse_decimal_or_nan("0.2"), 0.2);
        assert_eq!(parse_decimal_or_nan("0,2"), 0.2);
    }

    #[test]
    fn injectable_mg_and_iu_conversion_roundtrips_with_known_vial() {
        let mut data = HrtData::default();
        data.vials.push(Vial {
            id: "vial-1".to_string(),
            esterKind: None,
            suspensionOil: None,
            otherIngredients: None,
            batchNumber: None,
            source: None,
            concentrationMgPerMl: Some(40.0),
            isSpent: None,
            spentAt: None,
            useBy: None,
            createdAt: 0,
            subVials: Vec::new(),
        });
        let vial_id = "vial-1".to_string();
        let vial = Some(&vial_id);

        let iu = injectable_iu_from_dose(&data, 4.0, &HormoneUnits::Mg, vial, None)
            .expect("expected IU conversion");
        assert_eq!(iu, 10.0);

        let mg = injectable_dose_from_iu(&data, iu, vial, None).expect("expected mg conversion");
        assert!((mg - 4.0).abs() < 1e-6);
    }

    #[test]
    fn injectable_iu_conversion_requires_known_concentration() {
        let data = HrtData::default();
        let missing_vial_id = "missing".to_string();
        let vial = Some(&missing_vial_id);
        assert!(injectable_iu_from_dose(&data, 4.0, &HormoneUnits::Mg, vial, None,).is_none());
        assert!(injectable_dose_from_iu(&data, 10.0, vial, None).is_none());
    }
}
