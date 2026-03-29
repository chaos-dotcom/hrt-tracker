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
    let normalized = trimmed
        .replace(['\u{00a0}', '\u{202f}', '\u{2009}', ' ', '_', '\''], "");
    if normalized.is_empty() {
        return None;
    }

    let parse = |raw: String| raw.parse::<f64>().ok().filter(|v| v.is_finite());

    let comma_count = normalized.matches(',').count();
    let dot_count = normalized.matches('.').count();

    if comma_count > 0 && dot_count > 0 {
        let last_comma = normalized.rfind(',').unwrap_or(0);
        let last_dot = normalized.rfind('.').unwrap_or(0);
        if last_comma > last_dot {
            // 1.234,56 -> 1234.56
            return parse(normalized.replace('.', "").replace(',', "."));
        }
        // 1,234.56 -> 1234.56
        return parse(normalized.replace(',', ""));
    }

    if comma_count > 0 {
        if comma_count > 1 {
            // 1,234,567 -> 1234567
            return parse(normalized.replace(',', ""));
        }
        let split_idx = normalized.find(',').unwrap_or(0);
        let int_part = &normalized[..split_idx];
        let frac_part = &normalized[split_idx + 1..];
        let int_digits = int_part.trim_start_matches(['+', '-']);
        let grouped_thousands = frac_part.len() == 3
            && frac_part.chars().all(|ch| ch.is_ascii_digit())
            && !int_digits.is_empty()
            && int_digits != "0"
            && !int_digits.starts_with('0')
            && int_part
                .chars()
                .all(|ch| ch.is_ascii_digit() || ch == '+' || ch == '-');
        if grouped_thousands {
            // 1,000 -> 1000
            return parse(normalized.replace(',', ""));
        }
        // 1,25 -> 1.25
        return parse(normalized.replace(',', "."));
    }

    if dot_count > 1 {
        // 1.234.567 -> 1234567
        return parse(normalized.replace('.', ""));
    }

    parse(normalized)
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
    let measured = measured_pg_ml?;
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
        assert_eq!(parse_decimal("1.000"), Some(1.0));
        assert_eq!(parse_decimal("0.125"), Some(0.125));
        assert_eq!(parse_decimal("0,125"), Some(0.125));
    }

    #[test]
    fn parse_decimal_accepts_grouped_thousands_inputs() {
        assert_eq!(parse_decimal("1,000"), Some(1000.0));
        assert_eq!(parse_decimal("12,345"), Some(12345.0));
        assert_eq!(parse_decimal("1,234.5"), Some(1234.5));
        assert_eq!(parse_decimal("1.234,5"), Some(1234.5));
        assert_eq!(parse_decimal("1.234.567"), Some(1234567.0));
        assert_eq!(parse_decimal("1 234,5"), Some(1234.5));
        assert_eq!(parse_decimal("1'234,5"), Some(1234.5));
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

    #[test]
    fn parse_hormone_unit_all_variants() {
        use super::parse_hormone_unit;
        assert_eq!(parse_hormone_unit("pg/mL"), Some(HormoneUnits::E2PgMl));
        assert_eq!(parse_hormone_unit("pmol/L"), Some(HormoneUnits::E2PmolL));
        assert_eq!(parse_hormone_unit("ng/dL"), Some(HormoneUnits::TNgDl));
        assert_eq!(parse_hormone_unit("nmol/L"), Some(HormoneUnits::TNmolL));
        assert_eq!(parse_hormone_unit("mg"), Some(HormoneUnits::Mg));
        assert_eq!(parse_hormone_unit("ng/mL"), Some(HormoneUnits::NgMl));
        assert_eq!(parse_hormone_unit("mIU/mL"), Some(HormoneUnits::MIuMl));
        assert_eq!(parse_hormone_unit("mIU/L"), Some(HormoneUnits::MIuL));
        assert_eq!(parse_hormone_unit("U/L"), Some(HormoneUnits::UL));
        assert_eq!(parse_hormone_unit("invalid"), None);
    }

    #[test]
    fn parse_length_unit_variants() {
        use super::parse_length_unit;
        use hrt_shared::types::LengthUnit;
        assert_eq!(parse_length_unit("cm"), Some(LengthUnit::CM));
        assert_eq!(parse_length_unit("in"), Some(LengthUnit::IN));
        assert_eq!(parse_length_unit("m"), None);
    }

    #[test]
    fn hormone_unit_label_roundtrips() {
        use super::{hormone_unit_label, parse_hormone_unit};
        let units = ["pg/mL", "pmol/L", "ng/dL", "nmol/L", "mg", "ng/mL", "mIU/mL", "mIU/L", "U/L"];
        for label in units {
            let parsed = parse_hormone_unit(label).unwrap();
            assert_eq!(hormone_unit_label(&parsed), label);
        }
    }

    #[test]
    fn fmt_decimal_basic() {
        use super::fmt_decimal;
        assert_eq!(fmt_decimal(1.0, 3), "1");
        assert_eq!(fmt_decimal(1.5, 3), "1.5");
        assert_eq!(fmt_decimal(1.500, 3), "1.5");
        assert_eq!(fmt_decimal(0.125, 3), "0.125");
        assert_eq!(fmt_decimal(1234.0, 2), "1234");
    }

    #[test]
    fn fmt_decimal_non_finite() {
        use super::fmt_decimal;
        assert_eq!(fmt_decimal(f64::NAN, 3), "-");
        assert_eq!(fmt_decimal(f64::INFINITY, 3), "-");
        assert_eq!(fmt_decimal(f64::NEG_INFINITY, 3), "-");
    }

    #[test]
    fn fmt_blood_value_uses_4_decimals() {
        use super::fmt_blood_value;
        assert_eq!(fmt_blood_value(1.23456), "1.2346");
        assert_eq!(fmt_blood_value(100.0), "100");
        assert_eq!(fmt_blood_value(f64::NAN), "-");
    }

    #[test]
    fn convert_estradiol_to_display_pg_to_pmol() {
        use super::convert_estradiol_to_display;
        let result = convert_estradiol_to_display(100.0, &HormoneUnits::E2PgMl, &HormoneUnits::E2PmolL);
        assert!((result - 367.13).abs() < 0.1, "got {result}");
    }

    #[test]
    fn convert_estradiol_to_display_pmol_to_pg() {
        use super::convert_estradiol_to_display;
        let result = convert_estradiol_to_display(367.13, &HormoneUnits::E2PmolL, &HormoneUnits::E2PgMl);
        assert!((result - 100.0).abs() < 0.1, "got {result}");
    }

    #[test]
    fn convert_estradiol_to_display_same_unit() {
        use super::convert_estradiol_to_display;
        assert_eq!(convert_estradiol_to_display(42.0, &HormoneUnits::E2PgMl, &HormoneUnits::E2PgMl), 42.0);
        assert_eq!(convert_estradiol_to_display(42.0, &HormoneUnits::E2PmolL, &HormoneUnits::E2PmolL), 42.0);
    }

    #[test]
    fn estradiol_conversion_factor_values() {
        use super::estradiol_conversion_factor;
        assert_eq!(estradiol_conversion_factor(&HormoneUnits::E2PmolL), 3.6713);
        assert_eq!(estradiol_conversion_factor(&HormoneUnits::E2PgMl), 1.0);
    }

    #[test]
    fn convert_testosterone_to_ng_dl_from_nmol() {
        use super::convert_testosterone_to_ng_dl;
        let result = convert_testosterone_to_ng_dl(1.0, &HormoneUnits::TNmolL);
        assert!((result - 28.818).abs() < 0.001);
    }

    #[test]
    fn convert_testosterone_to_ng_dl_passthrough() {
        use super::convert_testosterone_to_ng_dl;
        assert_eq!(convert_testosterone_to_ng_dl(50.0, &HormoneUnits::TNgDl), 50.0);
    }

    #[test]
    fn convert_fsh_to_miu_ml_from_miu_l() {
        use super::convert_fsh_to_miu_ml;
        assert_eq!(convert_fsh_to_miu_ml(5000.0, &HormoneUnits::MIuL), 5.0);
    }

    #[test]
    fn convert_fsh_to_miu_ml_from_u_l() {
        use super::convert_fsh_to_miu_ml;
        assert_eq!(convert_fsh_to_miu_ml(5.0, &HormoneUnits::UL), 5.0);
    }

    #[test]
    fn convert_lh_to_miu_ml_from_miu_l() {
        use super::convert_lh_to_miu_ml;
        assert_eq!(convert_lh_to_miu_ml(3000.0, &HormoneUnits::MIuL), 3.0);
    }

    #[test]
    fn convert_progesterone_to_ng_ml_from_nmol() {
        use super::convert_progesterone_to_ng_ml;
        let result = convert_progesterone_to_ng_ml(10.0, &HormoneUnits::TNmolL);
        assert!((result - 3.1).abs() < 0.001);
    }

    #[test]
    fn compute_fudge_factor_basic() {
        use super::compute_fudge_factor;
        // measured 200, predicted 100 -> fudge = 2.0
        assert_eq!(compute_fudge_factor(Some(200.0), Some(100.0)), Some(2.0));
    }

    #[test]
    fn compute_fudge_factor_no_measured() {
        use super::compute_fudge_factor;
        assert_eq!(compute_fudge_factor(None, Some(100.0)), None);
    }

    #[test]
    fn compute_fudge_factor_no_predicted() {
        use super::compute_fudge_factor;
        // No prediction -> default to 1.0
        assert_eq!(compute_fudge_factor(Some(200.0), None), Some(1.0));
    }

    #[test]
    fn compute_fudge_factor_zero_predicted() {
        use super::compute_fudge_factor;
        assert_eq!(compute_fudge_factor(Some(200.0), Some(0.0)), Some(1.0));
    }

    #[test]
    fn compute_fudge_factor_nan_measured() {
        use super::compute_fudge_factor;
        assert_eq!(compute_fudge_factor(Some(f64::NAN), Some(100.0)), None);
    }

    #[test]
    fn compute_fudge_factor_rounding() {
        use super::compute_fudge_factor;
        // 100 / 300 = 0.33333... -> rounded to 0.333
        assert_eq!(compute_fudge_factor(Some(100.0), Some(300.0)), Some(0.333));
    }

    #[test]
    fn injectable_iu_rejects_non_mg_unit() {
        let data = HrtData::default();
        let result = injectable_iu_from_dose(&data, 4.0, &HormoneUnits::E2PgMl, None, None);
        assert!(result.is_none());
    }

    #[test]
    fn injectable_iu_rejects_negative_dose() {
        let data = HrtData::default();
        let result = injectable_iu_from_dose(&data, -1.0, &HormoneUnits::Mg, None, None);
        assert!(result.is_none());
    }

    #[test]
    fn injectable_dose_from_iu_rejects_negative() {
        let data = HrtData::default();
        assert!(injectable_dose_from_iu(&data, -10.0, None, None).is_none());
        assert!(injectable_dose_from_iu(&data, 0.0, None, None).is_none());
    }

    #[test]
    fn format_injectable_dose_without_iu() {
        use super::format_injectable_dose;
        let data = HrtData::default();
        let result = format_injectable_dose(&data, 4.0, &HormoneUnits::Mg, None, None, false);
        assert_eq!(result, "4 mg");
    }

    #[test]
    fn format_injectable_dose_non_finite() {
        use super::format_injectable_dose;
        let data = HrtData::default();
        assert_eq!(format_injectable_dose(&data, f64::NAN, &HormoneUnits::Mg, None, None, false), "-");
    }

    #[test]
    fn format_injectable_dose_with_iu_and_vial() {
        use super::format_injectable_dose;
        use hrt_shared::types::Vial;
        let mut data = HrtData::default();
        data.vials.push(Vial {
            id: "v1".to_string(),
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
        let vid = "v1".to_string();
        let result = format_injectable_dose(&data, 4.0, &HormoneUnits::Mg, Some(&vid), None, true);
        assert_eq!(result, "10 IU (4 mg)");
    }

    #[test]
    fn parse_decimal_negative_values() {
        assert_eq!(parse_decimal("-1.5"), Some(-1.5));
        assert_eq!(parse_decimal("-0.25"), Some(-0.25));
        assert_eq!(parse_decimal("+3.25"), Some(3.25));
    }

    #[test]
    fn parse_decimal_unicode_spaces() {
        // Non-breaking space, narrow no-break space, thin space
        assert_eq!(parse_decimal("1\u{00a0}234,5"), Some(1234.5));
        assert_eq!(parse_decimal("1\u{202f}234,5"), Some(1234.5));
        assert_eq!(parse_decimal("1\u{2009}234,5"), Some(1234.5));
    }

    #[test]
    fn injectable_concentration_uses_fallback() {
        use super::injectable_concentration_mg_ml;
        use hrt_shared::types::Vial;
        let mut data = HrtData::default();
        data.vials.push(Vial {
            id: "schedule-vial".to_string(),
            esterKind: None,
            suspensionOil: None,
            otherIngredients: None,
            batchNumber: None,
            source: None,
            concentrationMgPerMl: Some(20.0),
            isSpent: None,
            spentAt: None,
            useBy: None,
            createdAt: 0,
            subVials: Vec::new(),
        });
        let missing = "nonexistent".to_string();
        let schedule_id = "schedule-vial".to_string();
        // Primary vial not found, falls back to schedule vial
        let result = injectable_concentration_mg_ml(&data, Some(&missing), Some(&schedule_id));
        assert_eq!(result, Some(20.0));
    }
}
