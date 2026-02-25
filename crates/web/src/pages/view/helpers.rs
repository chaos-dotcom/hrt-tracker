use js_sys::Date;
use wasm_bindgen::JsValue;

use crate::utils::{hormone_unit_label, parse_decimal};
use hrt_shared::types::{
    DosageHistoryEntry, DosagePhoto, HormoneUnits, InjectionSites, LengthUnit, Measurement,
    ProgesteroneRoutes, SyringeKinds, WeightUnit,
};

use super::types::{PhotoView, INJECTION_SITE_OPTIONS};

pub(super) fn to_local_input_value(ms: i64) -> String {
    let date = Date::new(&JsValue::from_f64(ms as f64));
    let year = date.get_full_year();
    let month = date.get_month() + 1;
    let day = date.get_date();
    let hour = date.get_hours();
    let minute = date.get_minutes();
    format!(
        "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}",
        year = year,
        month = month,
        day = day,
        hour = hour,
        minute = minute
    )
}

pub(super) fn parse_datetime_local(value: &str) -> i64 {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Date::now() as i64;
    }
    let parsed = Date::parse(trimmed);
    if parsed.is_nan() {
        Date::now() as i64
    } else {
        parsed as i64
    }
}

pub(super) fn parse_optional_num(value: &str) -> Option<f64> {
    parse_decimal(value)
}

pub(super) fn parse_date_only(value: &str) -> i64 {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Date::now() as i64;
    }
    let parsed = Date::parse(trimmed);
    if parsed.is_nan() {
        Date::now() as i64
    } else {
        parsed as i64
    }
}

pub(super) fn weight_unit_label(unit: &WeightUnit) -> &'static str {
    match unit {
        WeightUnit::KG => "kg",
        WeightUnit::LBS => "lbs",
    }
}

pub(super) fn length_unit_label(unit: &LengthUnit) -> &'static str {
    match unit {
        LengthUnit::CM => "cm",
        LengthUnit::IN => "in",
    }
}

pub(super) fn parse_weight_unit(value: &str) -> Option<WeightUnit> {
    match value {
        "kg" => Some(WeightUnit::KG),
        "lbs" => Some(WeightUnit::LBS),
        _ => None,
    }
}

pub(super) fn measurement_key(entry: &Measurement) -> String {
    entry
        .id
        .clone()
        .unwrap_or_else(|| format!("measurement-legacy-{}", entry.date))
}

pub(super) fn measurement_matches_target(
    entry: &Measurement,
    id: Option<&str>,
    date: Option<i64>,
) -> bool {
    if let Some(id) = id {
        entry.id.as_deref() == Some(id)
    } else if let Some(date) = date {
        entry.id.is_none() && entry.date == date
    } else {
        false
    }
}

pub(super) fn injection_site_label(site: &InjectionSites) -> &'static str {
    match site {
        InjectionSites::StomachRight => "Stomach right",
        InjectionSites::StomachLeft => "Stomach left",
        InjectionSites::TopThighRight => "Top thigh right",
        InjectionSites::TopThighLeft => "Top thigh left",
        InjectionSites::InnerThighRight => "Inner thigh right",
        InjectionSites::InnerThighLeft => "Inner thigh left",
        InjectionSites::OuterThighRight => "Outer thigh right",
        InjectionSites::OuterThighLeft => "Outer thigh left",
        InjectionSites::ThighRight => "Thigh right",
        InjectionSites::ThighLeft => "Thigh left",
        InjectionSites::ButtockRight => "Buttock right",
        InjectionSites::ButtockLeft => "Buttock left",
    }
}

pub(super) fn injection_site_from_label(value: &str) -> Option<InjectionSites> {
    INJECTION_SITE_OPTIONS
        .iter()
        .find(|site| injection_site_label(site) == value)
        .cloned()
}

pub(super) fn syringe_kind_label(kind: &SyringeKinds) -> &'static str {
    match kind {
        SyringeKinds::RegularSyringe => "Regular syringe",
        SyringeKinds::LowWasteSyringe => "Low waste syringe",
        SyringeKinds::LowWasteNeedle => "Low waste needle",
        SyringeKinds::InsulinSyringe => "Insulin syringe",
        SyringeKinds::InsulinPen => "Insulin pen",
    }
}

pub(super) fn hormone_unit_labels() -> Vec<String> {
    [
        HormoneUnits::E2PgMl,
        HormoneUnits::E2PmolL,
        HormoneUnits::TNgDl,
        HormoneUnits::TNmolL,
        HormoneUnits::Mg,
        HormoneUnits::NgMl,
        HormoneUnits::MIuMl,
        HormoneUnits::MIuL,
        HormoneUnits::UL,
    ]
    .iter()
    .map(|unit| hormone_unit_label(unit).to_string())
    .collect()
}

pub(super) fn progesterone_route_label(route: &ProgesteroneRoutes) -> &'static str {
    match route {
        ProgesteroneRoutes::Oral => "Oral",
        ProgesteroneRoutes::Boofed => "Boofed",
    }
}

pub(super) fn dosage_photo_view(photo: &DosagePhoto) -> PhotoView {
    match photo {
        DosagePhoto::Legacy(file) => PhotoView {
            file: file.clone(),
            note: String::new(),
        },
        DosagePhoto::Entry { file, note } => PhotoView {
            file: file.clone(),
            note: note.clone().unwrap_or_default(),
        },
    }
}

pub(super) fn update_photo_note(photo: &mut DosagePhoto, note: String) {
    match photo {
        DosagePhoto::Legacy(file) => {
            *photo = DosagePhoto::Entry {
                file: file.clone(),
                note: if note.trim().is_empty() {
                    None
                } else {
                    Some(note)
                },
            };
        }
        DosagePhoto::Entry {
            note: entry_note, ..
        } => {
            *entry_note = if note.trim().is_empty() {
                None
            } else {
                Some(note)
            };
        }
    }
}

pub(super) fn bloodtest_pdf_url(filename: &str) -> String {
    format!("/api/bloodtest-pdf/{}", urlencoding::encode(filename))
}

pub(super) fn dosage_entry_date(entry: &DosageHistoryEntry) -> i64 {
    match entry {
        DosageHistoryEntry::InjectableEstradiol { date, .. }
        | DosageHistoryEntry::OralEstradiol { date, .. }
        | DosageHistoryEntry::Antiandrogen { date, .. }
        | DosageHistoryEntry::Progesterone { date, .. } => *date,
    }
}

pub(super) fn dosage_entry_matches_key(entry: &DosageHistoryEntry, key: &str) -> bool {
    match entry {
        DosageHistoryEntry::InjectableEstradiol { date, id, .. }
        | DosageHistoryEntry::OralEstradiol { date, id, .. }
        | DosageHistoryEntry::Antiandrogen { date, id, .. }
        | DosageHistoryEntry::Progesterone { date, id, .. } => id
            .as_ref()
            .map(|value| value == key)
            .unwrap_or_else(|| date.to_string() == key),
    }
}
