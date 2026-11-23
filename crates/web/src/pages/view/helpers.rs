use js_sys::Date;
use wasm_bindgen::JsValue;

use crate::utils::{fmt_decimal, hormone_unit_label, parse_decimal};
use hrt_shared::types::{
    Antiandrogens, BloodTest, DosageHistoryEntry, DosagePhoto, HormoneUnits, InjectableEstradiols,
    InjectionSites, LengthUnit, Measurement, MetabolicMarker, MetabolicPanel, OralEstradiols,
    ProgesteroneRoutes, Progesterones, SyringeKinds, WeightUnit,
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

/// Delete the one history entry whose id is `entry_id`.
///
/// Returns `true` when something was removed. Entries with no id (older data)
/// are never touched. Undo used to call `retain(|entry| matches(...))`, which
/// kept the new entry and deleted every other dose — the predicate is easy to
/// invert, so it lives here with a test.
pub(super) fn remove_history_entry(history: &mut Vec<DosageHistoryEntry>, entry_id: &str) -> bool {
    let before = history.len();
    history.retain(|entry| !dosage_entry_matches_key(entry, entry_id));
    history.len() != before
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

// ---- human labels ---------------------------------------------------------
// The summaries below used to print the Rust `Debug` form of the domain enums
// ("Micronized (Boofed), 100.00 Mg"). Everything user-facing goes through these
// label helpers instead.

pub(super) fn injectable_estradiol_label(kind: &InjectableEstradiols) -> &'static str {
    match kind {
        InjectableEstradiols::Benzoate => "Estradiol benzoate",
        InjectableEstradiols::Cypionate => "Estradiol cypionate",
        InjectableEstradiols::Enanthate => "Estradiol enanthate",
        InjectableEstradiols::Undecylate => "Estradiol undecylate",
        InjectableEstradiols::Valerate => "Estradiol valerate",
        InjectableEstradiols::PolyestradiolPhosphate => "Polyestradiol phosphate",
    }
}

pub(super) fn oral_estradiol_label(kind: &OralEstradiols) -> &'static str {
    match kind {
        OralEstradiols::Hemihydrate => "Estradiol hemihydrate",
        OralEstradiols::Valerate => "Estradiol valerate",
        OralEstradiols::Premarin => "Premarin",
    }
}

pub(super) fn antiandrogen_label(kind: &Antiandrogens) -> &'static str {
    match kind {
        Antiandrogens::CPA => "Cyproterone acetate",
        Antiandrogens::Spiro => "Spironolactone",
        Antiandrogens::Bica => "Bicalutamide",
        Antiandrogens::Finasteride => "Finasteride",
    }
}

pub(super) fn progesterone_label(kind: &Progesterones) -> &'static str {
    match kind {
        Progesterones::Micronized => "Micronized progesterone",
    }
}

/// The route as it reads in a summary: "oral" or "rectal" (the stored route is
/// still `ProgesteroneRoutes::Boofed`).
pub(super) fn progesterone_route_word(route: &ProgesteroneRoutes) -> &'static str {
    match route {
        ProgesteroneRoutes::Oral => "oral",
        ProgesteroneRoutes::Boofed => "rectal",
    }
}

pub(super) fn dose_mg(dose: f64) -> String {
    format!("{} mg", fmt_decimal(dose, 2))
}

/// "daily" for a one-day interval, otherwise "every N days".
pub(super) fn frequency_phrase(days: f64) -> String {
    if (days - 1.0).abs() < f64::EPSILON {
        "daily".to_string()
    } else {
        format!("every {} days", fmt_decimal(days, 2))
    }
}

/// How a dose made of pills reads back: "2 × 100 mg", or "100 mg" for one pill.
pub(super) fn pill_detail(dose: f64, quantity: f64) -> String {
    if quantity > 1.0 {
        format!("{} × {}", fmt_decimal(quantity, 2), dose_mg(dose))
    } else {
        dose_mg(dose)
    }
}

pub(super) fn regimen_summary_injection(
    kind: &InjectableEstradiols,
    dose_label: &str,
    frequency: f64,
) -> String {
    format!(
        "{} · {} · {}",
        injectable_estradiol_label(kind),
        dose_label,
        frequency_phrase(frequency)
    )
}

pub(super) fn regimen_summary_oral_estradiol(dose: f64, frequency: f64) -> String {
    format!(
        "Oral estradiol · {} · {}",
        dose_mg(dose),
        frequency_phrase(frequency)
    )
}

pub(super) fn regimen_summary_antiandrogen(
    kind: &Antiandrogens,
    dose: f64,
    frequency: f64,
) -> String {
    format!(
        "{} · {} · {}",
        antiandrogen_label(kind),
        dose_mg(dose),
        frequency_phrase(frequency)
    )
}

pub(super) fn regimen_summary_progesterone(
    route: &ProgesteroneRoutes,
    dose: f64,
    frequency: f64,
) -> String {
    format!(
        "Progesterone · {} · {} · {}",
        dose_mg(dose),
        progesterone_route_word(route),
        frequency_phrase(frequency)
    )
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

/// One row of the liver and lipid card.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct MetabolicRow {
    pub marker: MetabolicMarker,
    pub date: i64,
    pub value: f64,
    pub previous: Option<f64>,
    /// `Some(true)` above the typical range, `Some(false)` below it.
    pub flag: Option<bool>,
}

/// Newest liver and lipid values, with the previous test beside each marker.
///
/// The liver half is dropped unless the person takes estradiol as a tablet, or
/// already has liver results recorded: a tablet is swallowed and reaches the
/// liver before the blood, while an injection does not, and recorded results
/// are never hidden.
pub(super) fn metabolic_rows(tests: &[BloodTest], show_liver: bool) -> Vec<MetabolicRow> {
    let mut panels: Vec<(i64, MetabolicPanel)> = tests
        .iter()
        .filter_map(|test| test.metabolic.clone().map(|panel| (test.date, panel)))
        .filter(|(_, panel)| !panel.is_empty())
        .collect();
    panels.sort_by_key(|(date, _)| *date);
    let Some((date, newest)) = panels.last().cloned() else {
        return Vec::new();
    };
    let previous = panels.iter().rev().nth(1).map(|(_, panel)| panel.clone());
    let mut rows = Vec::new();
    for marker in MetabolicMarker::ALL {
        if marker.is_liver() && !show_liver {
            continue;
        }
        let Some(value) = marker.get(&newest) else {
            continue;
        };
        rows.push(MetabolicRow {
            marker,
            date,
            value,
            previous: previous.as_ref().and_then(|panel| marker.get(panel)),
            flag: marker.above_range(value),
        });
    }
    rows
}

/// Whether the liver half of the panel should be on screen at all.
pub(super) fn shows_liver_markers(tests: &[BloodTest], on_tablets: bool) -> bool {
    on_tablets || tests.iter().any(|test| {
        test.metabolic
            .as_ref()
            .is_some_and(|panel| panel.has_liver_values())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_with_panel(date: i64, markers: &[(MetabolicMarker, f64)]) -> BloodTest {
        let mut panel = MetabolicPanel::default();
        for (marker, value) in markers {
            marker.set(&mut panel, Some(*value));
        }
        BloodTest {
            date,
            estradiolLevel: None,
            testLevel: None,
            estradiolUnit: None,
            testUnit: None,
            progesteroneLevel: None,
            progesteroneUnit: None,
            fshLevel: None,
            fshUnit: None,
            lhLevel: None,
            lhUnit: None,
            prolactinLevel: None,
            prolactinUnit: None,
            shbgLevel: None,
            shbgUnit: None,
            freeAndrogenIndex: None,
            estrannaiseNumber: None,
            fudgeFactor: None,
            notes: None,
            estrogenType: None,
            pdfFiles: None,
            metabolic: Some(panel),
        }
    }

    #[test]
    fn liver_rows_are_hidden_for_injections_unless_already_recorded() {
        let tests = vec![test_with_panel(
            1_000,
            &[
                (MetabolicMarker::Alt, 23.0),
                (MetabolicMarker::TotalCholesterol, 4.4),
            ],
        )];
        // On injections with nothing recorded: lipids only.
        let rows = metabolic_rows(&tests, false);
        assert!(rows.iter().all(|row| !row.marker.is_liver()));
        assert!(rows.iter().any(|row| row.marker == MetabolicMarker::TotalCholesterol));

        // On tablets the liver rows appear.
        let rows = metabolic_rows(&tests, true);
        assert!(rows.iter().any(|row| row.marker == MetabolicMarker::Alt));

        // Injections but the result is already recorded: still shown.
        let rows = metabolic_rows(&tests, shows_liver_markers(&tests, false));
        assert!(rows.iter().any(|row| row.marker == MetabolicMarker::Alt));
        assert!(shows_liver_markers(&tests, false));
        assert!(!shows_liver_markers(&[], false));
    }

    #[test]
    fn the_newest_panel_wins_and_shows_the_previous_result() {
        let tests = vec![
            test_with_panel(1_000, &[(MetabolicMarker::Alt, 20.0)]),
            test_with_panel(2_000, &[(MetabolicMarker::Alt, 60.0)]),
        ];
        let rows = metabolic_rows(&tests, true);
        let alt = rows
            .iter()
            .find(|row| row.marker == MetabolicMarker::Alt)
            .expect("ALT row");
        assert_eq!(alt.date, 2_000);
        assert_eq!(alt.value, 60.0);
        assert_eq!(alt.previous, Some(20.0));
        assert_eq!(alt.flag, Some(true), "60 U/L is above the typical range");
    }

    #[test]
    fn a_marker_missing_from_the_newer_test_falls_back_to_nothing() {
        let tests = vec![
            test_with_panel(1_000, &[(MetabolicMarker::Alt, 20.0), (MetabolicMarker::Ggt, 18.0)]),
            test_with_panel(2_000, &[(MetabolicMarker::Alt, 22.0)]),
        ];
        let rows = metabolic_rows(&tests, true);
        assert!(rows.iter().all(|row| row.marker != MetabolicMarker::Ggt));
        let alt = rows
            .iter()
            .find(|row| row.marker == MetabolicMarker::Alt)
            .expect("ALT row");
        assert_eq!(alt.previous, Some(20.0));
        assert_eq!(alt.flag, None);
    }

    #[test]
    fn tests_without_a_panel_are_skipped() {
        let mut empty = test_with_panel(1_000, &[]);
        empty.metabolic = None;
        assert!(metabolic_rows(&[empty], true).is_empty());
    }


    #[test]
    fn regimen_summaries_read_as_labels_not_debug() {
        assert_eq!(
            regimen_summary_progesterone(&ProgesteroneRoutes::Oral, 100.0, 1.0),
            "Progesterone · 100 mg · oral · daily"
        );
        assert_eq!(
            regimen_summary_antiandrogen(&Antiandrogens::CPA, 12.5, 2.0),
            "Cyproterone acetate · 12.5 mg · every 2 days"
        );
        assert_eq!(
            regimen_summary_oral_estradiol(2.0, 1.0),
            "Oral estradiol · 2 mg · daily"
        );
        assert_eq!(
            regimen_summary_injection(&InjectableEstradiols::Valerate, "5 mg", 7.0),
            "Estradiol valerate · 5 mg · every 7 days"
        );
    }

    #[test]
    fn summaries_never_leak_debug_forms() {
        let summaries = [
            regimen_summary_progesterone(&ProgesteroneRoutes::Boofed, 200.0, 1.0),
            regimen_summary_antiandrogen(&Antiandrogens::CPA, 12.5, 2.0),
            regimen_summary_oral_estradiol(2.0, 1.0),
            regimen_summary_injection(&InjectableEstradiols::Valerate, "5 mg", 7.0),
            pill_detail(100.0, 2.0),
            pill_detail(100.0, 1.0),
        ];
        for summary in summaries {
            // `HormoneUnits::Mg` used to reach the screen as "Mg" via `{:?}`.
            assert!(!summary.contains("Mg"), "{summary} leaks a Debug unit");
            assert!(!summary.contains("DosageHistoryEntry"), "{summary}");
            assert!(
                !summary.contains("Micronized"),
                "{summary} leaks the enum variant"
            );
            assert!(!summary.contains("None"), "{summary}");
        }
    }

    #[test]
    fn history_pill_details_read_as_multiplication() {
        assert_eq!(pill_detail(100.0, 2.0), "2 × 100 mg");
        assert_eq!(pill_detail(100.0, 1.0), "100 mg");
        assert_eq!(pill_detail(2.0, 3.0), "3 × 2 mg");
    }

    #[test]
    fn frequency_phrase_says_daily_for_one_day() {
        assert_eq!(frequency_phrase(1.0), "daily");
        assert_eq!(frequency_phrase(2.0), "every 2 days");
        assert_eq!(frequency_phrase(7.0), "every 7 days");
    }

    #[test]
    fn progesterone_route_word_keeps_the_stored_variant() {
        assert_eq!(
            progesterone_route_word(&ProgesteroneRoutes::Boofed),
            "rectal"
        );
        assert_eq!(progesterone_route_word(&ProgesteroneRoutes::Oral), "oral");
    }
    #[test]
    fn undo_removes_only_the_entry_it_logged() {
        // Regression: the dashboard toast's Undo called
        // retain(|entry| dosage_entry_matches_key(entry, id)), which kept the new
        // entry and deleted every other dose in the history.
        let entry = |id: Option<&str>, dose: f64| DosageHistoryEntry::Progesterone {
            date: 0,
            id: id.map(str::to_string),
            kind: Progesterones::Micronized,
            route: ProgesteroneRoutes::Oral,
            dose,
            unit: HormoneUnits::Mg,
            pillQuantity: Some(1.0),
            note: None,
        };
        let mut history = vec![
            entry(Some("keep-1"), 100.0),
            entry(None, 50.0),
            entry(Some("undo-me"), 200.0),
        ];

        assert!(remove_history_entry(&mut history, "undo-me"));

        assert_eq!(history.len(), 2, "only the logged entry may be removed");
        assert!(!remove_history_entry(&mut history, "undo-me"), "already gone");
        assert!(!remove_history_entry(&mut history, "never-existed"));
        assert_eq!(history.len(), 2);
        assert!(history.iter().any(|e| {
            matches!(e, DosageHistoryEntry::Progesterone { id: Some(id), .. } if id == "keep-1")
        }));
        assert!(
            history.iter().any(|e| {
                matches!(e, DosageHistoryEntry::Progesterone { id: None, .. })
            }),
            "entries without an id must survive"
        );
    }
}

