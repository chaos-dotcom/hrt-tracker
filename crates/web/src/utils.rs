use chrono::{Local, TimeZone};
use hrt_shared::types::{HormoneUnits, LengthUnit};
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
