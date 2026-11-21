use gloo_timers::callback::Timeout;
use js_sys::{Date, Math};
use leptos::window;
use leptos::*;
use leptos_router::{use_navigate, A};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsValue;

use crate::layout::page_layout;
use crate::store::use_store;
use crate::utils::{
    injectable_dose_from_iu, injectable_iu_from_dose, parse_date_or_now, parse_decimal,
    parse_decimal_or_nan,
};
use hrt_shared::logic::{backfill_scheduled_doses, day_start_ms, scheduled_dose_dates};
use hrt_shared::types::{
    AntiandrogenSchedule, Antiandrogens, DosageHistoryEntry, HormoneUnits, InjectableEstradiols,
    InjectableSchedule, InjectionSites, OralEstradiols, OralSchedule, ProgesteroneRoutes,
    ProgesteroneSchedule, Progesterones, SyringeKinds,
};

const INJECTABLE_OPTIONS: [InjectableEstradiols; 6] = [
    InjectableEstradiols::Benzoate,
    InjectableEstradiols::Cypionate,
    InjectableEstradiols::Enanthate,
    InjectableEstradiols::Undecylate,
    InjectableEstradiols::Valerate,
    InjectableEstradiols::PolyestradiolPhosphate,
];

const ORAL_OPTIONS: [OralEstradiols; 3] = [
    OralEstradiols::Hemihydrate,
    OralEstradiols::Valerate,
    OralEstradiols::Premarin,
];

const ANTIANDROGEN_OPTIONS: [Antiandrogens; 4] = [
    Antiandrogens::CPA,
    Antiandrogens::Spiro,
    Antiandrogens::Bica,
    Antiandrogens::Finasteride,
];

const PROGESTERONE_OPTIONS: [Progesterones; 1] = [Progesterones::Micronized];

const INJECTION_SITES: [InjectionSites; 12] = [
    InjectionSites::StomachRight,
    InjectionSites::StomachLeft,
    InjectionSites::ThighRight,
    InjectionSites::ThighLeft,
    InjectionSites::TopThighRight,
    InjectionSites::TopThighLeft,
    InjectionSites::InnerThighRight,
    InjectionSites::InnerThighLeft,
    InjectionSites::OuterThighRight,
    InjectionSites::OuterThighLeft,
    InjectionSites::ButtockRight,
    InjectionSites::ButtockLeft,
];

const SYRINGE_KINDS: [SyringeKinds; 5] = [
    SyringeKinds::RegularSyringe,
    SyringeKinds::LowWasteSyringe,
    SyringeKinds::LowWasteNeedle,
    SyringeKinds::InsulinSyringe,
    SyringeKinds::InsulinPen,
];

fn injectable_label(kind: &InjectableEstradiols) -> &'static str {
    match kind {
        InjectableEstradiols::Benzoate => "Estradiol Benzoate",
        InjectableEstradiols::Cypionate => "Estradiol Cypionate",
        InjectableEstradiols::Enanthate => "Estradiol Enanthate",
        InjectableEstradiols::Undecylate => "Estradiol Undecylate",
        InjectableEstradiols::Valerate => "Estradiol Valerate",
        InjectableEstradiols::PolyestradiolPhosphate => "Polyestradiol Phosphate",
    }
}

fn oral_label(kind: &OralEstradiols) -> &'static str {
    match kind {
        OralEstradiols::Hemihydrate => "Estradiol Hemihydrate",
        OralEstradiols::Valerate => "Estradiol Valerate",
        OralEstradiols::Premarin => "Premarin",
    }
}

fn antiandrogen_label(kind: &Antiandrogens) -> &'static str {
    match kind {
        Antiandrogens::CPA => "Cyproterone Acetate",
        Antiandrogens::Spiro => "Spironolactone",
        Antiandrogens::Bica => "Bicalutamide",
        Antiandrogens::Finasteride => "Finasteride",
    }
}

fn progesterone_label(kind: &Progesterones) -> &'static str {
    match kind {
        Progesterones::Micronized => "Micronized Progesterone",
    }
}

fn injection_site_label(site: &InjectionSites) -> &'static str {
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

fn syringe_kind_label(kind: &SyringeKinds) -> &'static str {
    match kind {
        SyringeKinds::RegularSyringe => "Regular syringe",
        SyringeKinds::LowWasteSyringe => "Low waste syringe",
        SyringeKinds::LowWasteNeedle => "Low waste needle",
        SyringeKinds::InsulinSyringe => "Insulin syringe",
        SyringeKinds::InsulinPen => "Insulin pen",
    }
}

fn injectable_from_label(value: &str) -> InjectableEstradiols {
    INJECTABLE_OPTIONS
        .iter()
        .find(|kind| injectable_label(kind) == value)
        .cloned()
        .unwrap_or(InjectableEstradiols::Benzoate)
}

fn oral_from_label(value: &str) -> OralEstradiols {
    ORAL_OPTIONS
        .iter()
        .find(|kind| oral_label(kind) == value)
        .cloned()
        .unwrap_or(OralEstradiols::Valerate)
}

fn antiandrogen_from_label(value: &str) -> Antiandrogens {
    ANTIANDROGEN_OPTIONS
        .iter()
        .find(|kind| antiandrogen_label(kind) == value)
        .cloned()
        .unwrap_or(Antiandrogens::Spiro)
}

fn progesterone_from_label(value: &str) -> Progesterones {
    PROGESTERONE_OPTIONS
        .iter()
        .find(|kind| progesterone_label(kind) == value)
        .cloned()
        .unwrap_or(Progesterones::Micronized)
}

fn injection_site_from_label(value: &str) -> Option<InjectionSites> {
    INJECTION_SITES
        .iter()
        .find(|site| injection_site_label(site) == value)
        .cloned()
}

fn to_local_input_value(ms: i64) -> String {
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

fn to_local_date_value(ms: i64) -> String {
    to_local_input_value(ms).chars().take(10).collect()
}

/// Local date `offset_days` from today at 10:00, formatted for a date/datetime input.
fn day_at_ten(offset_days: f64) -> String {
    let date = Date::new(&JsValue::from_f64(Date::now() + offset_days * 86_400_000.0));
    format!(
        "{:04}-{:02}-{:02}T10:00",
        date.get_full_year(),
        date.get_month() + 1,
        date.get_date()
    )
}

/// Parse a `<input type="date">` value into a local timestamp at 10:00, the
/// time the app uses for scheduled doses. Returns `None` for an empty value.
fn parse_regimen_start(value: &str) -> Option<i64> {
    parse_regimen_start_at(value, "10:00", "10:00")
}

/// Parse a `<input type="date">` value plus an `HH:MM` time into a local
/// timestamp. Returns `None` for an empty date; a missing or malformed time
/// falls back to `fallback_time` (the app's 10:00 default).
fn parse_regimen_start_at(value: &str, time: &str, fallback_time: &str) -> Option<i64> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(parse_date_or_now(trimmed) + clock_offset_ms(time, fallback_time))
}

/// Milliseconds to add to a local midnight for `time`, falling back to
/// `fallback_time` when `time` is empty or malformed.
fn clock_offset_ms(time: &str, fallback_time: &str) -> i64 {
    let (hours, minutes) = parse_clock(time)
        .or_else(|| parse_clock(fallback_time))
        .unwrap_or((10, 0));
    (hours * 60 + minutes) * 60_000
}

/// Which medication row is expanded. One at a time, so the page stays short.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum MedRow {
    Estrogen,
    Antiandrogen,
    Progesterone,
}

/// "logged 22:14 · 100 mg" / "last 19 Nov · 2 × 100 mg" / "not logged yet".
fn med_row_meta(entries: &[DosageHistoryEntry], medication: Medication, today_start: i64) -> String {
    let Some(entry) = latest_entry(entries, medication) else {
        return "not logged yet".to_string();
    };
    let date = dosage_entry_date(entry);
    let dose = entry_dose_text(entry);
    if date >= today_start {
        format!("logged {} · {}", clock_label(date), dose)
    } else {
        let stamp = to_local_input_value(date);
        let (day, _) = stamp.split_once('T').unwrap_or((stamp.as_str(), ""));
        format!("last {} · {}", day, dose)
    }
}

/// `HH:MM` (24h) to hours and minutes.
fn parse_clock(value: &str) -> Option<(i64, i64)> {
    let (hours, minutes) = value.trim().split_once(':')?;
    let hours: i64 = hours.trim().parse().ok()?;
    let minutes: i64 = minutes.trim().parse().ok()?;
    if !(0..24).contains(&hours) || !(0..60).contains(&minutes) {
        return None;
    }
    Some((hours, minutes))
}

/// The first scheduled dose at or after `now`, given a regimen start and a
/// frequency in days. `None` when the frequency cannot produce a step.
fn next_dose_at_or_after(start: i64, frequency_days: f64, now: i64) -> Option<i64> {
    if !frequency_days.is_finite() || frequency_days <= 0.0 {
        return None;
    }
    let step = (frequency_days * 86_400_000.0) as i64;
    if step <= 0 {
        return None;
    }
    if start >= now {
        return Some(start);
    }
    let elapsed = now - start;
    let steps = (elapsed + step - 1) / step;
    Some(start + steps * step)
}

/// Interval chips for a regular regimen: label plus the number of days it means.
/// "every day" / "every other day" / "every N days".
fn interval_phrase(days: f64) -> String {
    if !days.is_finite() || days <= 0.0 {
        return "no interval set".to_string();
    }
    if (days - 1.0).abs() < f64::EPSILON {
        "every day".to_string()
    } else if (days - 2.0).abs() < f64::EPSILON {
        "every other day".to_string()
    } else {
        format!("every {} days", fmt(days, 2))
    }
}

const REGIMEN_INTERVALS: [(&str, &str); 3] = [("1", "Every day"), ("2", "Every other day"), ("3", "Every 3 days")];



fn parse_datetime_local(value: &str) -> i64 {
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

fn parse_optional_datetime(value: &str) -> Option<i64> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let parsed = Date::parse(trimmed);
    if parsed.is_nan() {
        None
    } else {
        Some(parsed as i64)
    }
}

fn parse_num(value: &str) -> f64 {
    parse_decimal_or_nan(value)
}

fn parse_optional_num(value: &str) -> Option<f64> {
    parse_decimal(value)
}

fn fmt(value: f64, decimals: usize) -> String {
    if !value.is_finite() {
        return "—".to_string();
    }
    let formatted = format!("{value:.decimals$}");
    formatted
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

fn fmt_iu_from_ml(ml: f64) -> String {
    if !ml.is_finite() {
        return "—".to_string();
    }
    format!("{}", (ml * 100.0).round() as i64)
}

/// The static suffix shown inside a dose input. Dosing on this page is in
/// milligrams; the injectable IU view is a display setting, not a unit choice.
fn dose_unit_suffix(display_in_iu: bool) -> &'static str {
    if display_in_iu {
        "IU"
    } else {
        "mg"
    }
}

/// Split a progesterone dose given as a *total* in mg into the per-pill amount
/// stored on the history entry and the number of pills taken: a 200 mg dose is
/// two 100 mg pills, anything else is that many mg in a single pill.
fn progesterone_dose_amounts(total_mg: f64) -> (f64, f64) {
    if total_mg == 200.0 {
        (100.0, 2.0)
    } else {
        (total_mg, 1.0)
    }
}

/// Label of a progesterone route chip; `Rectal` is the UI word for the route
/// stored as `ProgesteroneRoutes::Boofed`.
fn progesterone_route_chip_label(route: &ProgesteroneRoutes) -> &'static str {
    match route {
        ProgesteroneRoutes::Oral => "Oral",
        ProgesteroneRoutes::Boofed => "Rectal",
    }
}

fn progesterone_route_from_chip(label: &str) -> ProgesteroneRoutes {
    match label {
        "Rectal" => ProgesteroneRoutes::Boofed,
        _ => ProgesteroneRoutes::Oral,
    }
}

/// The word a medication is logged under in a commit button, e.g. "cyproterone".
fn antiandrogen_action_word(kind: &Antiandrogens) -> &'static str {
    match kind {
        Antiandrogens::CPA => "cyproterone",
        Antiandrogens::Spiro => "spironolactone",
        Antiandrogens::Bica => "bicalutamide",
        Antiandrogens::Finasteride => "finasteride",
    }
}

/// Text of the progesterone card's commit button, e.g. "Log progesterone · 100 mg · oral".
fn progesterone_commit_label(total_mg: f64, route: &ProgesteroneRoutes) -> String {
    format!(
        "Log progesterone · {} mg · {}",
        fmt(total_mg, 2),
        progesterone_route_chip_label(route).to_lowercase()
    )
}

/// Which medication a history entry belongs to, for the per-card draft, the
/// "already logged today" guard and the undo toast.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Medication {
    Estrogen,
    Antiandrogen,
    Progesterone,
}

fn entry_medication(entry: &DosageHistoryEntry) -> Medication {
    match entry {
        DosageHistoryEntry::InjectableEstradiol { .. }
        | DosageHistoryEntry::OralEstradiol { .. } => Medication::Estrogen,
        DosageHistoryEntry::Antiandrogen { .. } => Medication::Antiandrogen,
        DosageHistoryEntry::Progesterone { .. } => Medication::Progesterone,
    }
}

fn dosage_entry_date(entry: &DosageHistoryEntry) -> i64 {
    match entry {
        DosageHistoryEntry::InjectableEstradiol { date, .. }
        | DosageHistoryEntry::OralEstradiol { date, .. }
        | DosageHistoryEntry::Antiandrogen { date, .. }
        | DosageHistoryEntry::Progesterone { date, .. } => *date,
    }
}

fn dosage_entry_id(entry: &DosageHistoryEntry) -> Option<&str> {
    match entry {
        DosageHistoryEntry::InjectableEstradiol { id, .. }
        | DosageHistoryEntry::OralEstradiol { id, .. }
        | DosageHistoryEntry::Antiandrogen { id, .. }
        | DosageHistoryEntry::Progesterone { id, .. } => id.as_deref(),
    }
}

/// Most recent history entry of one medication, the first source a card's
/// draft is pre-filled from.
fn latest_entry(
    entries: &[DosageHistoryEntry],
    medication: Medication,
) -> Option<&DosageHistoryEntry> {
    entries
        .iter()
        .filter(|entry| entry_medication(entry) == medication)
        .max_by_key(|entry| dosage_entry_date(entry))
}

/// How an entry's amount should read back, e.g. "100 mg" or "2 × 100 mg".
fn entry_dose_text(entry: &DosageHistoryEntry) -> String {
    match entry {
        DosageHistoryEntry::InjectableEstradiol { dose, .. }
        | DosageHistoryEntry::Antiandrogen { dose, .. } => format!("{} mg", fmt(*dose, 2)),
        DosageHistoryEntry::OralEstradiol {
            dose, pillQuantity, ..
        }
        | DosageHistoryEntry::Progesterone {
            dose, pillQuantity, ..
        } => {
            let quantity = pillQuantity.unwrap_or(1.0);
            if quantity > 1.0 {
                format!("{} × {} mg", fmt(quantity, 2), fmt(*dose, 2))
            } else {
                format!("{} mg", fmt(*dose, 2))
            }
        }
    }
}

/// "Already logged today at 22:14 · 100 mg" for the newest entry of one
/// medication on the local day starting at `day_start`.
fn logged_today_note(
    entries: &[DosageHistoryEntry],
    day_start: i64,
    medication: Medication,
) -> Option<String> {
    entries
        .iter()
        .filter(|entry| entry_medication(entry) == medication)
        .filter(|entry| day_start_ms(dosage_entry_date(entry)) == day_start)
        .max_by_key(|entry| dosage_entry_date(entry))
        .map(|entry| {
            format!(
                "Already logged today at {} · {}",
                clock_label(dosage_entry_date(entry)),
                entry_dose_text(entry)
            )
        })
}

/// `HH:MM` in local time, for "Logged 22:14" and the already-logged note.
fn clock_label(ms: i64) -> String {
    let date = Date::new(&JsValue::from_f64(ms as f64));
    format!("{:02}:{:02}", date.get_hours(), date.get_minutes())
}

/// Unique id for a history entry pushed from this page, so the toast's Undo can
/// remove exactly that entry.
fn new_dose_id(date_ms: i64) -> String {
    format!("dose-{date_ms}-{}", (Math::random() * 1_000_000.0) as i64)
}

/// State behind the shared `.toast`: what was written, and what Undo removes.
#[derive(Clone)]
struct ToastState {
    message: String,
    entry_id: String,
    medication: Medication,
}

fn dosage_editor_page(schedule_only: bool) -> impl IntoView {
    let store = use_store();
    let navigate = use_navigate();

    if !schedule_only {
        create_effect({
            let navigate = navigate.clone();
            move |_| {
                let search = window().location().search().unwrap_or_default();
                if search.contains("mode=schedule") {
                    navigate("/create/dosage/schedule", Default::default());
                }
            }
        });
    }

    let mode = create_rw_signal(if schedule_only {
        "schedule".to_string()
    } else {
        "record".to_string()
    });
    let estrogen_method = create_rw_signal("injection".to_string());
    let settings = store.settings;

    let injectable_type =
        create_rw_signal(injectable_label(&InjectableEstradiols::Benzoate).to_string());
    let oral_type = create_rw_signal(oral_label(&OralEstradiols::Valerate).to_string());
    let estrogen_dose = create_rw_signal(String::new());
    let injection_frequency = create_rw_signal("7".to_string());
    let oral_frequency = create_rw_signal("1".to_string());
    let estrogen_next_date = create_rw_signal(String::new());

    let inj_conv_dose_mg = create_rw_signal("4".to_string());
    let inj_conv_conc_mg_ml = create_rw_signal("40".to_string());
    let inj_conv_dose_as_mg = create_memo({
        let settings = settings;
        move |_| {
            let dose = parse_num(&inj_conv_dose_mg.get());
            let conc = parse_num(&inj_conv_conc_mg_ml.get());
            if settings.get().displayInjectableInIU.unwrap_or(false) {
                if dose.is_finite() && conc.is_finite() && conc > 0.0 {
                    (dose / 100.0) * conc
                } else {
                    f64::NAN
                }
            } else {
                dose
            }
        }
    });
    let inj_conv_vol_ml = create_memo(move |_| {
        let dose_mg = inj_conv_dose_as_mg.get();
        let conc = parse_num(&inj_conv_conc_mg_ml.get());
        if dose_mg.is_finite() && conc.is_finite() && conc > 0.0 {
            dose_mg / conc
        } else {
            f64::NAN
        }
    });

    let inj_conv_vol2_ml = create_rw_signal("0.1".to_string());
    let inj_conv_conc2_mg_ml = create_rw_signal("40".to_string());
    let inj_conv_dose2_mg = create_memo(move |_| {
        let vol = parse_num(&inj_conv_vol2_ml.get());
        let conc = parse_num(&inj_conv_conc2_mg_ml.get());
        if vol.is_finite() && conc.is_finite() && conc > 0.0 {
            vol * conc
        } else {
            f64::NAN
        }
    });

    let aa_type = create_rw_signal(antiandrogen_label(&ANTIANDROGEN_OPTIONS[0]).to_string());
    let aa_dose = create_rw_signal(String::new());
    let aa_frequency = create_rw_signal("1".to_string());
    let aa_next_date = create_rw_signal(String::new());

    // Progesterone is edited as a *total* dose in mg. `prog_total` is the single
    // draft value behind the chips and the free-text input; the per-pill amount
    // and pill count written to history are derived from it.
    let prog_type = create_rw_signal(progesterone_label(&Progesterones::Micronized).to_string());
    let prog_total = create_rw_signal(String::new());
    let prog_route =
        create_rw_signal(progesterone_route_chip_label(&ProgesteroneRoutes::Oral).to_string());
    let prog_frequency = create_rw_signal("1".to_string());
    let prog_next_date = create_rw_signal(String::new());
    // Time of day a progesterone dose is scheduled for; evening by default.
    let prog_time = create_rw_signal("22:00".to_string());

    let bonus_dose = create_rw_signal(false);

    // Whether each medication has an active recurring schedule (schedule mode only).
    let schedule_estrogen = create_rw_signal(true);
    let schedule_aa = create_rw_signal(false);
    let schedule_prog = create_rw_signal(false);
    // Regimen start dates (used to backfill past doses).
    let estrogen_start = create_rw_signal(String::new());
    let aa_start = create_rw_signal(String::new());
    let prog_start = create_rw_signal(String::new());
    let backfill_estrogen = create_rw_signal(false);
    let backfill_aa = create_rw_signal(false);
    let backfill_prog = create_rw_signal(false);
    let form_error = create_rw_signal(None::<String>);

    let estrogen_note = create_rw_signal(String::new());
    let aa_note = create_rw_signal(String::new());
    let prog_note = create_rw_signal(String::new());
    let estrogen_pill_qty = create_rw_signal("1".to_string());
    // Each card's note is collapsed behind a "+ note" reveal.
    let estrogen_note_open = create_rw_signal(false);
    let aa_note_open = create_rw_signal(false);
    let prog_note_open = create_rw_signal(false);

    // Per-card feedback and per-field validation. Field errors clear as soon as
    // the user edits the field they belong to.
    let estrogen_logged = create_rw_signal(None::<String>);
    let aa_logged = create_rw_signal(None::<String>);
    let prog_logged = create_rw_signal(None::<String>);
    let estrogen_dose_error = create_rw_signal(None::<String>);
    let aa_dose_error = create_rw_signal(None::<String>);
    let prog_dose_error = create_rw_signal(None::<String>);
    let estrogen_frequency_error = create_rw_signal(None::<String>);
    let aa_frequency_error = create_rw_signal(None::<String>);
    let prog_frequency_error = create_rw_signal(None::<String>);
    let toast = create_rw_signal(None::<ToastState>);
    let toast_timeout: Rc<RefCell<Option<Timeout>>> = Rc::new(RefCell::new(None));

    let injection_site = create_rw_signal(String::new());
    let syringe_kind = create_rw_signal(String::new());
    let needle_length = create_rw_signal(String::new());
    let needle_gauge = create_rw_signal(String::new());

    let selected_vial_id = create_rw_signal(String::new());
    let selected_sub_vial_id = create_rw_signal(String::new());
    let estrogen_dose_in_iu = create_rw_signal(false);

    let injectable_dose_field_in_iu = create_memo({
        let store = store.clone();
        move |_| {
            if mode.get() != "record" {
                return false;
            }
            if estrogen_method.get() != "injection" {
                return false;
            }
            if !settings.get().displayInjectableInIU.unwrap_or(false) {
                return false;
            }
            let data_value = store.data.get();
            let selected_vial = selected_vial_id.get();
            let selected_vial_id = if selected_vial.trim().is_empty() {
                None
            } else {
                Some(&selected_vial)
            };
            let schedule_vial_id = data_value
                .injectableEstradiol
                .as_ref()
                .and_then(|cfg| cfg.vialId.as_ref());
            injectable_dose_from_iu(&data_value, 1.0, selected_vial_id, schedule_vial_id).is_some()
        }
    });

    create_effect({
        let store = store.clone();
        move |_| {
            let target_iu_mode = injectable_dose_field_in_iu.get();
            let current_iu_mode = estrogen_dose_in_iu.get();
            if target_iu_mode == current_iu_mode {
                return;
            }

            // Read the field value without subscribing, so typing in the dose
            // field doesn't re-trigger a conversion on every keystroke.
            if target_iu_mode {
                if let Some(current_value) = parse_optional_num(&estrogen_dose.get_untracked()) {
                    let data_value = store.data.get();
                    let selected_vial = selected_vial_id.get();
                    let selected_vial_id = if selected_vial.trim().is_empty() {
                        None
                    } else {
                        Some(&selected_vial)
                    };
                    let schedule_vial_id = data_value
                        .injectableEstradiol
                        .as_ref()
                        .and_then(|cfg| cfg.vialId.as_ref());
                    // Converting into IU is the only meaningful direction (a
                    // concentration is now available). If it fails we leave the
                    // value untouched.
                    if let Some(iu) = injectable_iu_from_dose(
                        &data_value,
                        current_value,
                        &HormoneUnits::Mg,
                        selected_vial_id,
                        schedule_vial_id,
                    ) {
                        estrogen_dose.set(fmt(iu, 3));
                    }
                }
            }
            // Leaving IU mode (vial concentration removed, or the method changed
            // away from injection) keeps the typed value and just relabels it
            // rather than wiping or silently reinterpreting it.
            estrogen_dose_in_iu.set(target_iu_mode);
        }
    });

    let estrogen_dose_as_mg = create_memo({
        let store = store.clone();
        move |_| {
            if estrogen_method.get() != "injection" {
                return None;
            }
            let dose_value = parse_num(&estrogen_dose.get());
            if !dose_value.is_finite() || dose_value <= 0.0 {
                return None;
            }
            if !estrogen_dose_in_iu.get() {
                return Some(dose_value);
            }
            let data_value = store.data.get();
            let selected_vial = selected_vial_id.get();
            let selected_vial_id = if selected_vial.trim().is_empty() {
                None
            } else {
                Some(&selected_vial)
            };
            let schedule_vial_id = data_value
                .injectableEstradiol
                .as_ref()
                .and_then(|cfg| cfg.vialId.as_ref());
            injectable_dose_from_iu(&data_value, dose_value, selected_vial_id, schedule_vial_id)
        }
    });

    let record_date_time = create_rw_signal(String::new());

    let schedule_feedback = create_rw_signal(false);
    let feedback_timeout: Rc<RefCell<Option<Timeout>>> = Rc::new(RefCell::new(None));

    // Each medication's draft is pre-filled from its most recent history entry,
    // else from its schedule. A card with nothing to pre-fill stays collapsed
    // behind its "+ Add" button; clicking that seeds the same draft, falling
    // back to the defaults below.
    let seed_estrogen: StoredValue<Rc<dyn Fn(bool)>> = StoredValue::new({
        let store = store.clone();
        Rc::new(move |with_defaults: bool| {
            let data = store.data.get();
            if let Some(entry) = latest_entry(&data.dosageHistory, Medication::Estrogen) {
                match entry {
                    DosageHistoryEntry::InjectableEstradiol {
                        kind,
                        dose,
                        unit,
                        vialId,
                        subVialId,
                        syringeKind,
                        needleLength,
                        needleGauge,
                        ..
                    } => {
                        estrogen_method.set("injection".to_string());
                        injectable_type.set(injectable_label(kind).to_string());
                        let dose_in_iu = mode.get() == "record"
                            && settings.get().displayInjectableInIU.unwrap_or(false)
                            && *unit == HormoneUnits::Mg
                            && injectable_iu_from_dose(
                                &data,
                                *dose,
                                unit,
                                vialId.as_ref(),
                                vialId.as_ref(),
                            )
                            .is_some();
                        let dose_label = if dose_in_iu {
                            injectable_iu_from_dose(
                                &data,
                                *dose,
                                unit,
                                vialId.as_ref(),
                                vialId.as_ref(),
                            )
                            .map(|iu| fmt(iu, 3))
                            .unwrap_or_else(|| fmt(*dose, 3))
                        } else {
                            fmt(*dose, 3)
                        };
                        estrogen_dose.set(dose_label);
                        estrogen_dose_in_iu.set(dose_in_iu);
                        selected_vial_id.set(vialId.clone().unwrap_or_default());
                        selected_sub_vial_id.set(subVialId.clone().unwrap_or_default());
                        syringe_kind.set(syringeKind.clone().unwrap_or_default());
                        needle_length.set(needleLength.clone().unwrap_or_default());
                        needle_gauge.set(needleGauge.clone().unwrap_or_default());
                    }
                    DosageHistoryEntry::OralEstradiol {
                        kind,
                        dose,
                        pillQuantity,
                        ..
                    } => {
                        estrogen_method.set("oral".to_string());
                        oral_type.set(oral_label(kind).to_string());
                        estrogen_dose.set(fmt(*dose, 3));
                        estrogen_dose_in_iu.set(false);
                        estrogen_pill_qty.set(fmt(pillQuantity.unwrap_or(1.0), 2));
                        selected_vial_id.set(String::new());
                        selected_sub_vial_id.set(String::new());
                        syringe_kind.set(String::new());
                        needle_length.set(String::new());
                        needle_gauge.set(String::new());
                    }
                    _ => {}
                }
                return;
            }
            if let Some(inj) = data.injectableEstradiol.as_ref() {
                estrogen_method.set("injection".to_string());
                injectable_type.set(injectable_label(&inj.kind).to_string());
                let dose_in_iu = mode.get() == "record"
                    && settings.get().displayInjectableInIU.unwrap_or(false)
                    && inj.unit == HormoneUnits::Mg
                    && injectable_iu_from_dose(
                        &data,
                        inj.dose,
                        &inj.unit,
                        inj.vialId.as_ref(),
                        inj.vialId.as_ref(),
                    )
                    .is_some();
                let dose_label = if dose_in_iu {
                    injectable_iu_from_dose(
                        &data,
                        inj.dose,
                        &inj.unit,
                        inj.vialId.as_ref(),
                        inj.vialId.as_ref(),
                    )
                    .map(|iu| fmt(iu, 3))
                    .unwrap_or_else(|| fmt(inj.dose, 3))
                } else {
                    fmt(inj.dose, 3)
                };
                estrogen_dose.set(dose_label);
                estrogen_dose_in_iu.set(dose_in_iu);
                selected_vial_id.set(inj.vialId.clone().unwrap_or_default());
                selected_sub_vial_id.set(inj.subVialId.clone().unwrap_or_default());
                syringe_kind.set(inj.syringeKind.clone().unwrap_or_default());
                needle_length.set(inj.needleLength.clone().unwrap_or_default());
                needle_gauge.set(inj.needleGauge.clone().unwrap_or_default());
                return;
            }
            if let Some(oral) = data.oralEstradiol.as_ref() {
                estrogen_method.set("oral".to_string());
                oral_type.set(oral_label(&oral.kind).to_string());
                estrogen_dose.set(fmt(oral.dose, 3));
                estrogen_dose_in_iu.set(false);
                estrogen_pill_qty.set("1".to_string());
                selected_vial_id.set(String::new());
                selected_sub_vial_id.set(String::new());
                syringe_kind.set(String::new());
                needle_length.set(String::new());
                needle_gauge.set(String::new());
                return;
            }
            if with_defaults {
                estrogen_method.set("oral".to_string());
                oral_type.set(oral_label(&OralEstradiols::Valerate).to_string());
                estrogen_dose.set("2".to_string());
                estrogen_dose_in_iu.set(false);
                estrogen_pill_qty.set("1".to_string());
            }
        })
    });

    let seed_aa: StoredValue<Rc<dyn Fn(bool)>> = StoredValue::new({
        let store = store.clone();
        Rc::new(move |with_defaults: bool| {
            let data = store.data.get();
            if let Some(entry) = latest_entry(&data.dosageHistory, Medication::Antiandrogen) {
                if let DosageHistoryEntry::Antiandrogen { kind, dose, .. } = entry {
                    aa_type.set(antiandrogen_label(kind).to_string());
                    aa_dose.set(fmt(*dose, 3));
                }
                return;
            }
            if let Some(aa) = data.antiandrogen.as_ref() {
                aa_type.set(antiandrogen_label(&aa.kind).to_string());
                aa_dose.set(fmt(aa.dose, 3));
            }
            // With no history and no schedule the card stays collapsed; its
            // "+ Add" button then opens it with the default antiandrogen and a
            // 12.5 mg dose, the usual cyproterone dose.
            else if with_defaults {
                aa_type.set(antiandrogen_label(&Antiandrogens::CPA).to_string());
                aa_dose.set("12.5".to_string());
            }
        })
    });

    let seed_prog: StoredValue<Rc<dyn Fn(bool)>> = StoredValue::new({
        let store = store.clone();
        Rc::new(move |with_defaults: bool| {
            let data = store.data.get();
            if let Some(entry) = latest_entry(&data.dosageHistory, Medication::Progesterone) {
                if let DosageHistoryEntry::Progesterone {
                    kind,
                    route,
                    dose,
                    pillQuantity,
                    ..
                } = entry
                {
                    let quantity = pillQuantity.unwrap_or(1.0);
                    prog_type.set(progesterone_label(kind).to_string());
                    prog_route.set(progesterone_route_chip_label(route).to_string());
                    prog_total.set(fmt(*dose * quantity, 2));
                }
                return;
            }
            if let Some(prog) = data.progesterone.as_ref() {
                prog_type.set(progesterone_label(&prog.kind).to_string());
                prog_route.set(progesterone_route_chip_label(&prog.route).to_string());
                prog_total.set(fmt(prog.dose, 2));
                return;
            }
            if with_defaults {
                prog_type.set(progesterone_label(&Progesterones::Micronized).to_string());
                prog_route
                    .set(progesterone_route_chip_label(&ProgesteroneRoutes::Oral).to_string());
                prog_total.set("100".to_string());
            }
        })
    });

    let initialized = create_rw_signal(false);
    create_effect({
        let store = store.clone();
        let seed_estrogen = seed_estrogen.clone();
        let seed_aa = seed_aa.clone();
        let seed_prog = seed_prog.clone();
        move |_| {
            if initialized.get() {
                return;
            }
            if store.is_loading.get() {
                return;
            }
            let data = store.data.get();
            // Schedule-only fields always come from the saved schedule.
            if let Some(inj) = data.injectableEstradiol.as_ref() {
                injection_frequency.set(fmt(inj.frequency, 2));
                estrogen_next_date.set(
                    inj.nextDoseDate
                        .map(to_local_input_value)
                        .unwrap_or_default(),
                );
            } else if let Some(oral) = data.oralEstradiol.as_ref() {
                oral_frequency.set(fmt(oral.frequency, 2));
                estrogen_next_date.set(
                    oral.nextDoseDate
                        .map(to_local_input_value)
                        .unwrap_or_default(),
                );
            }
            if let Some(aa) = data.antiandrogen.as_ref() {
                aa_frequency.set(fmt(aa.frequency, 2));
                aa_next_date.set(
                    aa.nextDoseDate
                        .map(to_local_input_value)
                        .unwrap_or_default(),
                );
            }
            if let Some(prog) = data.progesterone.as_ref() {
                if prog.dose > 0.0 {
                    prog_total.set(fmt(prog.dose, 2));
                }
                prog_frequency.set(fmt(prog.frequency, 2));
                prog_next_date.set(
                    prog.nextDoseDate
                        .map(to_local_input_value)
                        .unwrap_or_default(),
                );
                // Round-trip the time of day the regimen is scheduled at.
                if let Some(start) = prog.startDate {
                    prog_start.set(to_local_date_value(start));
                    let stamp = to_local_input_value(start);
                    if let Some((_, time)) = stamp.split_once('T') {
                        prog_time.set(time.chars().take(5).collect());
                    }
                } else if let Some(next) = prog.nextDoseDate {
                    let stamp = to_local_input_value(next);
                    if let Some((_, time)) = stamp.split_once('T') {
                        prog_time.set(time.chars().take(5).collect());
                    }
                }
            }
            seed_estrogen.with_value(|seed| seed(false));
            seed_aa.with_value(|seed| seed(false));
            seed_prog.with_value(|seed| seed(false));
            // Setting a regimen up for the first time should offer to fill in the
            // doses you have already taken; editing an existing one should not.
            backfill_prog.set(data.progesterone.is_none());
            schedule_estrogen
                .set(data.injectableEstradiol.is_some() || data.oralEstradiol.is_some());
            schedule_aa.set(data.antiandrogen.is_some());
            schedule_prog.set(data.progesterone.is_some());
            estrogen_start.set(
                data.injectableEstradiol
                    .as_ref()
                    .and_then(|cfg| cfg.startDate)
                    .or_else(|| data.oralEstradiol.as_ref().and_then(|cfg| cfg.startDate))
                    .map(to_local_date_value)
                    .unwrap_or_default(),
            );
            aa_start.set(
                data.antiandrogen
                    .as_ref()
                    .and_then(|cfg| cfg.startDate)
                    .map(to_local_date_value)
                    .unwrap_or_default(),
            );
            prog_start.set(
                data.progesterone
                    .as_ref()
                    .and_then(|cfg| cfg.startDate)
                    .map(to_local_date_value)
                    .unwrap_or_default(),
            );
            initialized.set(true);
        }
    });

    create_effect({
        let record_date_time = record_date_time;
        move |_| {
            if mode.get() == "record" && record_date_time.get().is_empty() {
                record_date_time.set(to_local_input_value(Date::now() as i64));
            }
        }
    });

    create_effect({
        let store = store.clone();
        let selected_sub_vial_id = selected_sub_vial_id;
        move |_| {
            let selected_vial = selected_vial_id.get();
            let sub_id = selected_sub_vial_id.get();
            if selected_vial.is_empty() {
                selected_sub_vial_id.set(String::new());
                return;
            }
            let has_sub = store
                .data
                .get()
                .vials
                .iter()
                .find(|v| v.id == selected_vial)
                .map(|v| v.subVials.iter().any(|s| s.id == sub_id))
                .unwrap_or(false);
            if !has_sub {
                selected_sub_vial_id.set(String::new());
            }
        }
    });

    let estrogen_pill_total = create_memo(move |_| {
        let dose = parse_num(&estrogen_dose.get());
        let qty = parse_num(&estrogen_pill_qty.get());
        if dose.is_finite() && qty.is_finite() {
            dose * qty
        } else {
            0.0
        }
    });

    let estrogen_backfill_dates = create_memo({
        let store = store.clone();
        move |_| {
            let Some(start) = parse_regimen_start(&estrogen_start.get()) else {
                return Vec::new();
            };
            let injection = estrogen_method.get() == "injection";
            let frequency = if injection {
                parse_num(&injection_frequency.get())
            } else {
                parse_num(&oral_frequency.get())
            };
            if !frequency.is_finite() || frequency <= 0.0 {
                return Vec::new();
            }
            let data_value = store.data.get();
            let existing: Vec<_> = data_value
                .dosageHistory
                .iter()
                .filter_map(|entry| match entry {
                    DosageHistoryEntry::InjectableEstradiol { date, .. } if injection => {
                        Some(*date)
                    }
                    DosageHistoryEntry::OralEstradiol { date, .. } if !injection => Some(*date),
                    _ => None,
                })
                .collect();
            scheduled_dose_dates(start, frequency, &existing)
        }
    });

    let aa_backfill_dates = create_memo({
        let store = store.clone();
        move |_| {
            let Some(start) = parse_regimen_start(&aa_start.get()) else {
                return Vec::new();
            };
            let frequency = parse_num(&aa_frequency.get());
            if !frequency.is_finite() || frequency <= 0.0 {
                return Vec::new();
            }
            let data_value = store.data.get();
            let existing: Vec<_> = data_value
                .dosageHistory
                .iter()
                .filter_map(|entry| match entry {
                    DosageHistoryEntry::Antiandrogen { date, .. } => Some(*date),
                    _ => None,
                })
                .collect();
            scheduled_dose_dates(start, frequency, &existing)
        }
    });

    // What the regimen the user is describing actually amounts to.
    let prog_regimen_summary = create_memo(move |_| {
        let dose = parse_num(&prog_total.get());
        let route = prog_route.get().to_lowercase();
        let time = prog_time.get();
        let frequency = parse_num(&prog_frequency.get());
        let dose_text = if dose.is_finite() && dose > 0.0 {
            format!("{} mg", fmt(dose, 2))
        } else {
            "no dose yet".to_string()
        };
        let start = parse_regimen_start_at(&prog_start.get(), &time, "22:00");
        let next = start
            .and_then(|start| next_dose_at_or_after(start, frequency, Date::now() as i64))
            .map(|ms| to_local_input_value(ms).replace('T', " "));
        match next {
            Some(next) => format!(
                "{} · {} · {} · {} — next dose {}",
                interval_phrase(frequency),
                dose_text,
                route,
                time,
                next
            ),
            None => format!(
                "{} · {} · {} · {} — set a start date to see the next dose",
                interval_phrase(frequency),
                dose_text,
                route,
                time
            ),
        }
    });

    let prog_backfill_dates = create_memo({
        let store = store.clone();
        move |_| {
            let Some(start) = parse_regimen_start_at(&prog_start.get(), &prog_time.get(), "22:00") else {
                return Vec::new();
            };
            let frequency = parse_num(&prog_frequency.get());
            if !frequency.is_finite() || frequency <= 0.0 {
                return Vec::new();
            }
            let data_value = store.data.get();
            let existing: Vec<_> = data_value
                .dosageHistory
                .iter()
                .filter_map(|entry| match entry {
                    DosageHistoryEntry::Progesterone { date, .. } => Some(*date),
                    _ => None,
                })
                .collect();
            let now = Date::now() as i64;
            scheduled_dose_dates(start, frequency, &existing)
                .into_iter()
                .filter(|date| *date <= now)
                .collect()
        }
    });

    // ---- record mode: one commit per card ----------------------------------
    // A card is open when its draft holds something; a card with no draft and no
    // schedule collapses to its "+ Add" button. Card contents are the intent.
    let estrogen_card_open = create_memo(move |_| !estrogen_dose.get().trim().is_empty());
    let aa_card_open = create_memo(move |_| !aa_dose.get().trim().is_empty());
    let prog_card_open = create_memo(move |_| !prog_total.get().trim().is_empty());

    // Schedule mode could not start a progesterone regimen without first finding a
    // switch in the card header, so the card begins as one button that opens a
    // regimen already set up the way most people take it.
    let start_prog_regimen: StoredValue<Rc<dyn Fn()>> = StoredValue::new({
        Rc::new(move || {
            schedule_prog.set(true);
            if prog_total.get().trim().is_empty() {
                let data = store.data.get();
                if let Some(entry) = latest_entry(&data.dosageHistory, Medication::Progesterone) {
                    if let DosageHistoryEntry::Progesterone {
                        dose,
                        pillQuantity,
                        route,
                        ..
                    } = entry
                    {
                        prog_total.set(fmt(*dose * pillQuantity.unwrap_or(1.0), 2));
                        prog_route.set(progesterone_route_chip_label(route).to_string());
                    }
                } else if let Some(prog) = data.progesterone.as_ref() {
                    prog_total.set(fmt(prog.dose, 2));
                    prog_route.set(progesterone_route_chip_label(&prog.route).to_string());
                } else {
                    prog_total.set("100".to_string());
                    prog_route.set(
                        progesterone_route_chip_label(&ProgesteroneRoutes::Oral).to_string(),
                    );
                }
            }
            if prog_frequency.get().trim().is_empty() {
                prog_frequency.set("1".to_string());
            }
            if prog_start.get().trim().is_empty() {
                prog_start.set(to_local_date_value(Date::now() as i64));
            }
            if prog_time.get().trim().is_empty() {
                prog_time.set("22:00".to_string());
            }
            backfill_prog.set(true);
        })
    });

    // "100 mg · oral · every day · 22:00" for the card header.
    let prog_regimen_short = create_memo(move |_| {
        let dose = parse_num(&prog_total.get());
        let dose_text = if dose.is_finite() && dose > 0.0 {
            format!("{} mg", fmt(dose, 2))
        } else {
            "no dose".to_string()
        };
        format!(
            "{} · {} · {} · {}",
            dose_text,
            prog_route.get().to_lowercase(),
            interval_phrase(parse_num(&prog_frequency.get())),
            prog_time.get()
        )
    });

    // The dose in mg this card would write, or `None` when it has nothing valid.
    let estrogen_record_dose = create_memo(move |_| {
        if estrogen_method.get() == "injection" {
            estrogen_dose_as_mg
                .get()
                .filter(|dose| dose.is_finite() && *dose > 0.0)
        } else {
            parse_optional_num(&estrogen_dose.get()).filter(|dose| dose.is_finite() && *dose > 0.0)
        }
    });
    let aa_record_dose =
        create_memo(move |_| parse_optional_num(&aa_dose.get()).filter(|dose| *dose > 0.0));
    let prog_record_total =
        create_memo(move |_| parse_optional_num(&prog_total.get()).filter(|dose| *dose > 0.0));

    // How the estrogen dose is shown on the button and summary.
    let estrogen_dose_label = move || -> String {
        let Some(dose) = estrogen_record_dose.get() else {
            return String::new();
        };
        if estrogen_method.get() == "injection" {
            if estrogen_dose_in_iu.get() {
                format!("{} IU", fmt(dose, 3))
            } else {
                format!("{} mg", fmt(dose, 3))
            }
        } else {
            let quantity = parse_optional_num(&estrogen_pill_qty.get()).unwrap_or(1.0);
            if quantity > 1.0 {
                format!("{} × {} mg", fmt(quantity, 2), fmt(dose, 2))
            } else {
                format!("{} mg", fmt(dose, 2))
            }
        }
    };

    // The local day "already logged today" is measured against.
    let today_start = day_start_ms(Date::now() as i64);
    let estrogen_logged_today = create_memo({
        let store = store.clone();
        move |_| {
            logged_today_note(
                &store.data.get().dosageHistory,
                today_start,
                Medication::Estrogen,
            )
        }
    });
    let aa_logged_today = create_memo({
        let store = store.clone();
        move |_| {
            logged_today_note(
                &store.data.get().dosageHistory,
                today_start,
                Medication::Antiandrogen,
            )
        }
    });
    let prog_logged_today = create_memo({
        let store = store.clone();
        move |_| {
            logged_today_note(
                &store.data.get().dosageHistory,
                today_start,
                Medication::Progesterone,
            )
        }
    });

    let show_toast: StoredValue<Rc<dyn Fn(String, String, Medication)>> = StoredValue::new({
        let toast_timeout = toast_timeout.clone();
        Rc::new(
            move |message: String, entry_id: String, medication: Medication| {
                toast.set(Some(ToastState {
                    message,
                    entry_id,
                    medication,
                }));
                if let Some(existing) = toast_timeout.borrow_mut().take() {
                    drop(existing);
                }
                *toast_timeout.borrow_mut() = Some(Timeout::new(6000, move || {
                    toast.set(None);
                }));
            },
        )
    });

    let undo_toast: StoredValue<Rc<dyn Fn()>> = StoredValue::new({
        let store = store.clone();
        Rc::new(move || {
            let Some(state) = toast.get() else {
                return;
            };
            store.data.update(|data| {
                data.dosageHistory
                    .retain(|entry| dosage_entry_id(entry) != Some(state.entry_id.as_str()));
            });
            match state.medication {
                Medication::Estrogen => estrogen_logged.set(None),
                Medication::Antiandrogen => aa_logged.set(None),
                Medication::Progesterone => prog_logged.set(None),
            }
            store.mark_dirty();
            store.save();
            toast.set(None);
        })
    });

    // Each commit writes exactly its own card's values: no navigation, a toast
    // with Undo, and the card keeps a "Logged 22:14 · …" line.
    let commit_estrogen: StoredValue<Rc<dyn Fn()>> = StoredValue::new({
        let store = store.clone();
        Rc::new(move || {
            let Some(dose) = estrogen_record_dose.get() else {
                estrogen_dose_error.set(Some("Choose a dose first".to_string()));
                return;
            };
            estrogen_dose_error.set(None);
            let dose_label = estrogen_dose_label();
            let record_ms = parse_datetime_local(&record_date_time.get());
            let entry_id = new_dose_id(record_ms);
            let note = if estrogen_note.get().trim().is_empty() {
                None
            } else {
                Some(estrogen_note.get())
            };
            let injection = estrogen_method.get() == "injection";
            store.data.update(|data| {
                if injection {
                    data.dosageHistory
                        .push(DosageHistoryEntry::InjectableEstradiol {
                            date: record_ms,
                            id: Some(entry_id.clone()),
                            kind: injectable_from_label(&injectable_type.get()),
                            dose,
                            unit: HormoneUnits::Mg,
                            note: note.clone(),
                            bonusDose: if bonus_dose.get() { Some(true) } else { None },
                            injectionSite: injection_site_from_label(&injection_site.get()),
                            vialId: if selected_vial_id.get().is_empty() {
                                None
                            } else {
                                Some(selected_vial_id.get())
                            },
                            subVialId: if selected_sub_vial_id.get().is_empty() {
                                None
                            } else {
                                Some(selected_sub_vial_id.get())
                            },
                            syringeKind: if syringe_kind.get().is_empty() {
                                None
                            } else {
                                Some(syringe_kind.get())
                            },
                            needleLength: if needle_length.get().trim().is_empty() {
                                None
                            } else {
                                Some(needle_length.get())
                            },
                            needleGauge: if needle_gauge.get().trim().is_empty() {
                                None
                            } else {
                                Some(needle_gauge.get())
                            },
                            photos: None,
                        });
                } else {
                    let pill_qty =
                        parse_optional_num(&estrogen_pill_qty.get()).filter(|value| *value > 0.0);
                    data.dosageHistory.push(DosageHistoryEntry::OralEstradiol {
                        date: record_ms,
                        id: Some(entry_id.clone()),
                        kind: oral_from_label(&oral_type.get()),
                        dose,
                        unit: HormoneUnits::Mg,
                        pillQuantity: pill_qty,
                        note: note.clone(),
                    });
                }
            });
            store.mark_dirty();
            store.save();
            let logged = format!("Logged {} · {}", clock_label(record_ms), dose_label);
            estrogen_logged.set(Some(logged));
            show_toast.with_value(|toast| {
                toast(
                    format!("Estrogen {dose_label} logged"),
                    entry_id,
                    Medication::Estrogen,
                )
            });
        })
    });

    let commit_aa: StoredValue<Rc<dyn Fn()>> = StoredValue::new({
        let store = store.clone();
        Rc::new(move || {
            let Some(dose) = aa_record_dose.get() else {
                aa_dose_error.set(Some("Choose a dose first".to_string()));
                return;
            };
            aa_dose_error.set(None);
            let kind = antiandrogen_from_label(&aa_type.get());
            let record_ms = parse_datetime_local(&record_date_time.get());
            let entry_id = new_dose_id(record_ms);
            let note = if aa_note.get().trim().is_empty() {
                None
            } else {
                Some(aa_note.get())
            };
            store.data.update(|data| {
                data.dosageHistory.push(DosageHistoryEntry::Antiandrogen {
                    date: record_ms,
                    id: Some(entry_id.clone()),
                    kind: kind.clone(),
                    dose,
                    unit: HormoneUnits::Mg,
                    note: note.clone(),
                });
            });
            store.mark_dirty();
            store.save();
            let logged = format!("Logged {} · {} mg", clock_label(record_ms), fmt(dose, 2));
            aa_logged.set(Some(logged));
            show_toast.with_value(|toast| {
                toast(
                    format!(
                        "Logged {} · {} mg",
                        antiandrogen_action_word(&kind),
                        fmt(dose, 2)
                    ),
                    entry_id,
                    Medication::Antiandrogen,
                )
            });
        })
    });

    let commit_prog: StoredValue<Rc<dyn Fn()>> = StoredValue::new({
        let store = store.clone();
        Rc::new(move || {
            let Some(total) = prog_record_total.get() else {
                prog_dose_error.set(Some("Choose a dose first".to_string()));
                return;
            };
            prog_dose_error.set(None);
            let (dose, pill_quantity) = progesterone_dose_amounts(total);
            let route = progesterone_route_from_chip(&prog_route.get());
            let record_ms = parse_datetime_local(&record_date_time.get());
            let entry_id = new_dose_id(record_ms);
            let note = if prog_note.get().trim().is_empty() {
                None
            } else {
                Some(prog_note.get())
            };
            store.data.update(|data| {
                data.dosageHistory.push(DosageHistoryEntry::Progesterone {
                    date: record_ms,
                    id: Some(entry_id.clone()),
                    kind: progesterone_from_label(&prog_type.get()),
                    route: route.clone(),
                    dose,
                    unit: HormoneUnits::Mg,
                    pillQuantity: Some(pill_quantity),
                    note: note.clone(),
                });
            });
            store.mark_dirty();
            store.save();
            let logged = format!("Logged {} · {} mg", clock_label(record_ms), fmt(total, 2));
            prog_logged.set(Some(logged));
            show_toast.with_value(|toast| {
                toast(
                    format!(
                        "Logged progesterone · {} mg · {}",
                        fmt(total, 2),
                        progesterone_route_chip_label(&route).to_lowercase()
                    ),
                    entry_id,
                    Medication::Progesterone,
                )
            });
        })
    });

    // Exactly one medication row is expanded at a time; the page reads as a list
    // of three rows instead of three cards stacked to the floor.
    let open_row = create_rw_signal(Some(MedRow::Progesterone));
    let toggle_row = move |row: MedRow| {
        open_row.update(|current| {
            *current = if *current == Some(row) { None } else { Some(row) };
        });
    };

    let row_today_start = day_start_ms(Date::now() as i64);
    let estrogen_row_meta = create_memo({
        let store = store.clone();
        move |_| {
            med_row_meta(
                &store.data.get().dosageHistory,
                Medication::Estrogen,
                row_today_start,
            )
        }
    });
    let aa_row_meta = create_memo({
        let store = store.clone();
        move |_| {
            med_row_meta(
                &store.data.get().dosageHistory,
                Medication::Antiandrogen,
                row_today_start,
            )
        }
    });
    let prog_row_meta = create_memo({
        let store = store.clone();
        move |_| {
            med_row_meta(
                &store.data.get().dosageHistory,
                Medication::Progesterone,
                row_today_start,
            )
        }
    });

    // What the footer button would write, and the button itself.
    let open_row_label = create_memo(move |_| match open_row.get() {
        Some(MedRow::Estrogen) => {
            if estrogen_record_dose.get().is_some() {
                format!("Log estrogen · {}", estrogen_dose_label())
            } else {
                "Choose an estrogen dose".to_string()
            }
        }
        Some(MedRow::Antiandrogen) => match aa_record_dose.get() {
            Some(dose) => format!("Log cyproterone · {} mg", fmt(dose, 2)),
            None => "Choose an antiandrogen dose".to_string(),
        },
        Some(MedRow::Progesterone) => match prog_record_total.get() {
            Some(total) => progesterone_commit_label(
                total,
                &progesterone_route_from_chip(&prog_route.get()),
            ),
            None => "Choose a progesterone dose".to_string(),
        },
        None => "Pick a medication above".to_string(),
    });

    let open_row_ready = create_memo(move |_| match open_row.get() {
        Some(MedRow::Estrogen) => estrogen_record_dose.get().is_some(),
        Some(MedRow::Antiandrogen) => aa_record_dose.get().is_some(),
        Some(MedRow::Progesterone) => prog_record_total.get().is_some(),
        None => false,
    });

    let commit_open_row: StoredValue<Rc<dyn Fn()>> = StoredValue::new(Rc::new(move || {
        match open_row.get() {
            Some(MedRow::Estrogen) => commit_estrogen.with_value(|commit| commit()),
            Some(MedRow::Antiandrogen) => commit_aa.with_value(|commit| commit()),
            Some(MedRow::Progesterone) => commit_prog.with_value(|commit| commit()),
            None => {}
        }
    }));


    let on_submit = {
        let store = store.clone();
        let feedback_timeout = feedback_timeout.clone();
        move |ev: leptos::ev::SubmitEvent| {
            ev.prevent_default();
            form_error.set(None);
            if mode.get() != "schedule" {
                // Record mode has no page-level submit: each card commits itself.
                return;
            }
            let estrogen_method_value = estrogen_method.get();
            let estrogen_input_dose = parse_num(&estrogen_dose.get());
            let estrogen_dose_value =
                if estrogen_method_value == "injection" && estrogen_dose_in_iu.get() {
                    let data_value = store.data.get();
                    let selected_vial = selected_vial_id.get();
                    let selected_vial_id = if selected_vial.trim().is_empty() {
                        None
                    } else {
                        Some(&selected_vial)
                    };
                    let schedule_vial_id = data_value
                        .injectableEstradiol
                        .as_ref()
                        .and_then(|cfg| cfg.vialId.as_ref());
                    injectable_dose_from_iu(
                        &data_value,
                        estrogen_input_dose,
                        selected_vial_id,
                        schedule_vial_id,
                    )
                    .unwrap_or(estrogen_input_dose)
                } else {
                    estrogen_input_dose
                };

            let mut errors: Vec<String> = Vec::new();
            estrogen_dose_error.set(None);
            estrogen_frequency_error.set(None);
            aa_dose_error.set(None);
            aa_frequency_error.set(None);
            prog_dose_error.set(None);
            prog_frequency_error.set(None);
            if schedule_estrogen.get() {
                if !estrogen_dose_value.is_finite() || estrogen_dose_value <= 0.0 {
                    estrogen_dose_error.set(Some("Enter a dose above 0 mg.".to_string()));
                    errors.push("Enter a valid estrogen dose.".to_string());
                }
                let frequency = if estrogen_method_value == "injection" {
                    parse_num(&injection_frequency.get())
                } else {
                    parse_num(&oral_frequency.get())
                };
                if !frequency.is_finite() || frequency <= 0.0 {
                    estrogen_frequency_error
                        .set(Some("Enter a number of days above 0.".to_string()));
                    errors.push("Enter a valid estrogen frequency.".to_string());
                }
            }
            if schedule_aa.get() {
                let dose = parse_num(&aa_dose.get());
                if !dose.is_finite() || dose <= 0.0 {
                    aa_dose_error.set(Some("Enter a dose above 0 mg.".to_string()));
                    errors.push("Enter a valid antiandrogen dose.".to_string());
                }
                let frequency = parse_num(&aa_frequency.get());
                if !frequency.is_finite() || frequency <= 0.0 {
                    aa_frequency_error.set(Some("Enter a number of days above 0.".to_string()));
                    errors.push("Enter a valid antiandrogen frequency.".to_string());
                }
            }
            if schedule_prog.get() {
                let dose = parse_num(&prog_total.get());
                if !dose.is_finite() || dose <= 0.0 {
                    prog_dose_error.set(Some("Enter a dose above 0 mg.".to_string()));
                    errors.push("Enter a valid progesterone dose.".to_string());
                }
                let frequency = parse_num(&prog_frequency.get());
                if !frequency.is_finite() || frequency <= 0.0 {
                    prog_frequency_error.set(Some("Enter a number of days above 0.".to_string()));
                    errors.push("Enter a valid progesterone frequency.".to_string());
                }
            }

            if !errors.is_empty() {
                form_error.set(Some(errors.join(" ")));
                return;
            }

            store.data.update(|data| {
                if !schedule_estrogen.get() {
                    data.injectableEstradiol = None;
                    data.oralEstradiol = None;
                } else if estrogen_method_value == "injection" {
                    let schedule = InjectableSchedule {
                        kind: injectable_from_label(&injectable_type.get()),
                        dose: estrogen_dose_value,
                        unit: HormoneUnits::Mg,
                        frequency: parse_num(&injection_frequency.get()).max(1.0),
                        vialId: if selected_vial_id.get().is_empty() {
                            None
                        } else {
                            Some(selected_vial_id.get())
                        },
                        subVialId: if selected_sub_vial_id.get().is_empty() {
                            None
                        } else {
                            Some(selected_sub_vial_id.get())
                        },
                        syringeKind: if syringe_kind.get().is_empty() {
                            None
                        } else {
                            Some(syringe_kind.get())
                        },
                        needleLength: if needle_length.get().trim().is_empty() {
                            None
                        } else {
                            Some(needle_length.get())
                        },
                        needleGauge: if needle_gauge.get().trim().is_empty() {
                            None
                        } else {
                            Some(needle_gauge.get())
                        },
                        nextDoseDate: parse_optional_datetime(&estrogen_next_date.get()),
                        startDate: parse_regimen_start(&estrogen_start.get()),
                    };
                    data.injectableEstradiol = Some(schedule);
                    data.oralEstradiol = None;
                } else {
                    let schedule = OralSchedule {
                        kind: oral_from_label(&oral_type.get()),
                        dose: estrogen_dose_value,
                        unit: HormoneUnits::Mg,
                        frequency: parse_num(&oral_frequency.get()).max(1.0),
                        nextDoseDate: parse_optional_datetime(&estrogen_next_date.get()),
                        startDate: parse_regimen_start(&estrogen_start.get()),
                    };
                    data.oralEstradiol = Some(schedule);
                    data.injectableEstradiol = None;
                }

                if !schedule_aa.get() || aa_type.get().trim().is_empty() {
                    data.antiandrogen = None;
                } else {
                    let schedule = AntiandrogenSchedule {
                        kind: antiandrogen_from_label(&aa_type.get()),
                        dose: parse_num(&aa_dose.get()),
                        unit: HormoneUnits::Mg,
                        frequency: parse_num(&aa_frequency.get()).max(1.0),
                        nextDoseDate: parse_optional_datetime(&aa_next_date.get()),
                        startDate: parse_regimen_start(&aa_start.get()),
                    };
                    data.antiandrogen = Some(schedule);
                }

                if !schedule_prog.get() || prog_type.get().trim().is_empty() {
                    data.progesterone = None;
                } else {
                    // One start, one frequency: the next dose follows from them
                    // unless the schedule already carries an explicit one.
                    let frequency = parse_num(&prog_frequency.get());
                    let start_ms = parse_regimen_start_at(&prog_start.get(), &prog_time.get(), "22:00");
                    let manual_next = parse_optional_datetime(&prog_next_date.get());
                    let schedule = ProgesteroneSchedule {
                        kind: progesterone_from_label(&prog_type.get()),
                        route: progesterone_route_from_chip(&prog_route.get()),
                        dose: parse_num(&prog_total.get()),
                        unit: HormoneUnits::Mg,
                        frequency: frequency.max(1.0),
                        nextDoseDate: manual_next
                            .or_else(|| {
                                start_ms.and_then(|start| {
                                    next_dose_at_or_after(start, frequency.max(1.0), Date::now() as i64)
                                })
                            }),
                        startDate: start_ms,
                    };
                    data.progesterone = Some(schedule);
                }

                // Backfill past doses for any regimen that started in the past.
                if schedule_estrogen.get() && backfill_estrogen.get() {
                    if let Some(start) = parse_regimen_start(&estrogen_start.get()) {
                        if let Some(cfg) = data.injectableEstradiol.clone() {
                            let existing: Vec<_> = data
                                .dosageHistory
                                .iter()
                                .filter_map(|entry| match entry {
                                    DosageHistoryEntry::InjectableEstradiol { date, .. } => {
                                        Some(*date)
                                    }
                                    _ => None,
                                })
                                .collect();
                            for date in scheduled_dose_dates(start, cfg.frequency, &existing) {
                                data.dosageHistory
                                    .push(DosageHistoryEntry::InjectableEstradiol {
                                        date,
                                        id: None,
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
                                    });
                            }
                        }
                        if let Some(cfg) = data.oralEstradiol.clone() {
                            let existing: Vec<_> = data
                                .dosageHistory
                                .iter()
                                .filter_map(|entry| match entry {
                                    DosageHistoryEntry::OralEstradiol { date, .. } => Some(*date),
                                    _ => None,
                                })
                                .collect();
                            for date in scheduled_dose_dates(start, cfg.frequency, &existing) {
                                data.dosageHistory.push(DosageHistoryEntry::OralEstradiol {
                                    date,
                                    id: None,
                                    kind: cfg.kind.clone(),
                                    dose: cfg.dose,
                                    unit: cfg.unit.clone(),
                                    pillQuantity: Some(1.0),
                                    note: None,
                                });
                            }
                        }
                    }
                }

                if schedule_aa.get() && backfill_aa.get() {
                    if let (Some(start), Some(cfg)) = (
                        parse_regimen_start(&aa_start.get()),
                        data.antiandrogen.clone(),
                    ) {
                        let existing: Vec<_> = data
                            .dosageHistory
                            .iter()
                            .filter_map(|entry| match entry {
                                DosageHistoryEntry::Antiandrogen { date, .. } => Some(*date),
                                _ => None,
                            })
                            .collect();
                        for date in scheduled_dose_dates(start, cfg.frequency, &existing) {
                            data.dosageHistory.push(DosageHistoryEntry::Antiandrogen {
                                date,
                                id: None,
                                kind: cfg.kind.clone(),
                                dose: cfg.dose,
                                unit: cfg.unit.clone(),
                                note: None,
                            });
                        }
                    }
                }

                if schedule_prog.get() && backfill_prog.get() {
                    if let (Some(start), Some(cfg)) = (
                        parse_regimen_start_at(&prog_start.get(), &prog_time.get(), "22:00"),
                        data.progesterone.clone(),
                    ) {
                        let existing: Vec<_> = data
                            .dosageHistory
                            .iter()
                            .filter_map(|entry| match entry {
                                DosageHistoryEntry::Progesterone { date, .. } => Some(*date),
                                _ => None,
                            })
                            .collect();
                        let now = Date::now() as i64;
                        for date in scheduled_dose_dates(start, cfg.frequency, &existing)
                            .into_iter()
                            .filter(|date| *date <= now)
                        {
                            data.dosageHistory.push(DosageHistoryEntry::Progesterone {
                                date,
                                id: None,
                                kind: cfg.kind.clone(),
                                route: cfg.route.clone(),
                                dose: cfg.dose,
                                unit: cfg.unit.clone(),
                                pillQuantity: Some(1.0),
                                note: None,
                            });
                        }
                    }
                }

                backfill_scheduled_doses(data);
            });

            store.mark_dirty();
            store.save();

            schedule_feedback.set(true);
            if let Some(existing) = feedback_timeout.borrow_mut().take() {
                drop(existing);
            }
            let schedule_feedback = schedule_feedback;
            *feedback_timeout.borrow_mut() = Some(Timeout::new(3000, move || {
                schedule_feedback.set(false);
            }));
        }
    };

    page_layout(
        if schedule_only {
            "Edit schedule"
        } else {
            "Record dosage"
        },
        view! {
            <div class="view-layout">
                <div class="view-header">
                    <div>
                        <p class="muted">
                            {if schedule_only {
                                "Configure recurring schedules for each medication."
                            } else {
                                "Record one-off doses for your regimen."
                            }}
                        </p>
                    </div>
                    <div class="header-actions">
                        <div class="segmented">
                            <A
                                class="segmented-item"
                                active_class="active"
                                exact=true
                                href="/create/dosage"
                            >
                                "Record dose"
                            </A>
                            <A
                                class="segmented-item"
                                active_class="active"
                                exact=true
                                href="/create/dosage/schedule"
                            >
                                "Schedule"
                            </A>
                        </div>
                        <A href="/view">"View dosage history"</A>
                    </div>
                </div>

                <form class="form-wide" on:submit=on_submit>
                    <Show when=move || {
                        estrogen_method.get() == "injection"
                            && if schedule_only {
                                schedule_estrogen.get()
                            } else {
                                open_row.get() == Some(MedRow::Estrogen)
                            }
                    }>
                        <section class="card">
                            <h3>"Injection helper"</h3>
                            <div class="calc-grid">
                                <div class="calc-block">
                                    <h4>"Dose and Concentration to Volume"</h4>
                                    <div class="inline-equal">
                                        <label>
                                            "Dose"
                                            <span class="unit-field">
                                                <input
                                                    type="text"
                                                    step="any"
                                                    inputmode="decimal"
                                                    on:input=move |ev| inj_conv_dose_mg.set(event_target_value(&ev))
                                                    prop:value=move || inj_conv_dose_mg.get()
                                                />
                                                <span class="unit">
                                                    {move || dose_unit_suffix(
                                                        settings.get().displayInjectableInIU.unwrap_or(false),
                                                    )}
                                                </span>
                                            </span>
                                        </label>
                                        <label>
                                            "Concentration"
                                            <span class="unit-field">
                                                <input
                                                    type="text"
                                                    step="any"
                                                    inputmode="decimal"
                                                    on:input=move |ev| inj_conv_conc_mg_ml.set(event_target_value(&ev))
                                                    prop:value=move || inj_conv_conc_mg_ml.get()
                                                />
                                                <span class="unit">"mg/mL"</span>
                                            </span>
                                        </label>
                                    </div>
                                    <p class="muted">
                                        {move || {
                                            if settings.get().displayInjectableInIU.unwrap_or(false) {
                                                "Volume = IU ÷ 100 = "
                                            } else {
                                                "Volume = Dose ÷ Concentration = "
                                            }
                                        }}
                                        <strong>{move || fmt(inj_conv_vol_ml.get(), 3)}</strong>
                                        " mL"
                                        <Show
                                            when=move || {
                                                inj_conv_vol_ml.get().is_finite()
                                                    && !settings.get().displayInjectableInIU.unwrap_or(false)
                                            }
                                        >
                                            <span>
                                                " (≈ "
                                                <strong>{move || fmt_iu_from_ml(inj_conv_vol_ml.get())}</strong>
                                                " IU)"
                                            </span>
                                        </Show>
                                        <Show
                                            when=move || {
                                                inj_conv_vol_ml.get().is_finite()
                                                    && settings.get().displayInjectableInIU.unwrap_or(false)
                                            }
                                        >
                                            <span>
                                                " ("
                                                <strong>{move || fmt(inj_conv_dose_as_mg.get(), 3)}</strong>
                                                " mg)"
                                            </span>
                                        </Show>
                                    </p>
                                </div>
                                <div class="calc-block">
                                    <h4>"Volume and Concentration to Dose"</h4>
                                    <div class="inline-equal">
                                        <label>
                                            "Volume"
                                            <span class="unit-field">
                                                <input
                                                    type="text"
                                                    step="any"
                                                    inputmode="decimal"
                                                    on:input=move |ev| inj_conv_vol2_ml.set(event_target_value(&ev))
                                                    prop:value=move || inj_conv_vol2_ml.get()
                                                />
                                                <span class="unit">"mL"</span>
                                            </span>
                                        </label>
                                        <label>
                                            "Concentration"
                                            <span class="unit-field">
                                                <input
                                                    type="text"
                                                    step="any"
                                                    inputmode="decimal"
                                                    on:input=move |ev| inj_conv_conc2_mg_ml.set(event_target_value(&ev))
                                                    prop:value=move || inj_conv_conc2_mg_ml.get()
                                                />
                                                <span class="unit">"mg/mL"</span>
                                            </span>
                                        </label>
                                    </div>
                                    <p class="muted">
                                        "Dose = Volume × Concentration = "
                                        <strong>{move || fmt(inj_conv_dose2_mg.get(), 3)}</strong>
                                        " mg"
                                    </p>
                                </div>
                            </div>
                        </section>
                    </Show>

                    <Show when=move || !schedule_only>
                        <section class="card">
                            <h3>"Dose timing"</h3>
                            <div class="chip-row">
                                <button
                                    type="button"
                                    class="chip"
                                    on:click=move |_| {
                                        record_date_time.set(to_local_input_value(Date::now() as i64))
                                    }
                                >
                                    "Now"
                                </button>
                                <button
                                    type="button"
                                    class="chip"
                                    on:click=move |_| record_date_time.set(day_at_ten(0.0))
                                >
                                    "Today 10:00"
                                </button>
                                <button
                                    type="button"
                                    class="chip"
                                    on:click=move |_| record_date_time.set(day_at_ten(-1.0))
                                >
                                    "Yesterday"
                                </button>
                                <button
                                    type="button"
                                    class="chip"
                                    on:click=move |_| record_date_time.set(day_at_ten(-2.0))
                                >
                                    "2 days ago"
                                </button>
                            </div>
                            <label>
                                "Date / time"
                                <input
                                    type="datetime-local"
                                    on:input=move |ev| record_date_time.set(event_target_value(&ev))
                                    prop:value=move || record_date_time.get()
                                    required
                                />
                            </label>
                        </section>
                    </Show>

                    <section class="med-row" class:open=move || open_row.get() == Some(MedRow::Estrogen)>
                        <div class="med-row-head">
                            <button
                                type="button"
                                class="med-row-toggle"
                                aria-expanded=move || if open_row.get() == Some(MedRow::Estrogen) { "true" } else { "false" }
                                aria-controls="row-estrogen"
                                on:click=move |_| toggle_row(MedRow::Estrogen)
                            >
                                <span class="med-row-name">"Estrogen"</span>
                                <span class="med-row-meta">{move || estrogen_row_meta.get()}</span>
                                <span class="med-row-chev" aria-hidden="true">"⌄"</span>
                            </button>
                            <Show when=move || schedule_only && schedule_estrogen.get()>
                                <button
                                    type="button"
                                    class="chip"
                                    on:click=move |_| schedule_estrogen.set(false)
                                >
                                    "Remove regimen"
                                </button>
                            </Show>
                        </div>
                        <Show when=move || schedule_only && !schedule_estrogen.get()>
                            <p class="muted">"No estrogen regimen yet."</p>
                            <button
                                type="button"
                                class="chip"
                                on:click=move |_| schedule_estrogen.set(true)
                            >
                                "+ Add estrogen regimen"
                            </button>
                        </Show>
                        <div class="med-row-body" id="row-estrogen" hidden=move || open_row.get() != Some(MedRow::Estrogen)>
                        <Show when=move || !schedule_only || schedule_estrogen.get()>
                        <Show when=move || !schedule_only || schedule_aa.get()>
                        <div class="option-group">
                            <label class="toggle toggle-wide">
                                <input
                                    type="radio"
                                    name="estrogen-method"
                                    value="injection"
                                    on:change=move |_| estrogen_method.set("injection".to_string())
                                    prop:checked=move || estrogen_method.get() == "injection"
                                />
                                <span class="toggle-track" aria-hidden="true"></span>
                                <span class="toggle-label">"Injection"</span>
                            </label>
                            <label class="toggle toggle-wide">
                                <input
                                    type="radio"
                                    name="estrogen-method"
                                    value="oral"
                                    on:change=move |_| estrogen_method.set("oral".to_string())
                                    prop:checked=move || estrogen_method.get() == "oral"
                                />
                                <span class="toggle-track" aria-hidden="true"></span>
                                <span class="toggle-label">"Oral"</span>
                            </label>
                        </div>

                        <Show when=move || !schedule_only>
                            <button
                                type="button"
                                class="chip"
                                aria-expanded=move || if estrogen_note_open.get() { "true" } else { "false" }
                                on:click=move |_| estrogen_note_open.set(!estrogen_note_open.get())
                            >
                                "+ note"
                            </button>
                            <Show when=move || estrogen_note_open.get()>
                                <label>
                                    "Note (optional)"
                                    <textarea
                                        rows="2"
                                        on:input=move |ev| estrogen_note.set(event_target_value(&ev))
                                        prop:value=move || estrogen_note.get()
                                    ></textarea>
                                </label>
                            </Show>
                        </Show>

                        <Show when=move || !schedule_only && estrogen_method.get() == "injection">
                            <label class="toggle toggle-wide">
                                <input
                                    type="checkbox"
                                    on:change=move |ev| bonus_dose.set(event_target_checked(&ev))
                                    prop:checked=move || bonus_dose.get()
                                />
                                <span class="toggle-track" aria-hidden="true"></span>
                                <span class="toggle-label">"Bonus dose (doesn't move schedule)"</span>
                            </label>
                        </Show>

                        <Show when=move || !schedule_only && estrogen_method.get() == "injection">
                            <label>
                                "Injection site (optional)"
                                <select on:change=move |ev| injection_site.set(event_target_value(&ev))>
                                    <option value="" selected=move || injection_site.get().is_empty()>
                                        "Select injection site"
                                    </option>
                                    <For
                                        each=move || INJECTION_SITES.to_vec()
                                        key=|site| injection_site_label(site).to_string()
                                        children=move |site| {
                                            let label = injection_site_label(&site).to_string();
                                            let value = label.clone();
                                            view! {
                                                <option value=value.clone() selected=move || injection_site.get() == value>
                                                    {label}
                                                </option>
                                            }
                                        }
                                    />
                                </select>
                            </label>
                        </Show>

                        <Show when=move || estrogen_method.get() == "injection">
                            <label>"Vial (optional)"</label>
                            <div class="inline-equal">
                                    <select on:change=move |ev| selected_vial_id.set(event_target_value(&ev))>
                                        <option value="" selected=move || selected_vial_id.get().is_empty()>
                                            "None"
                                        </option>
                                        <For
                                            each=move || {
                                                store
                                                    .data
                                                    .get()
                                                    .vials
                                                    .into_iter()
                                                    .filter(|v| !v.isSpent.unwrap_or(false) || v.id == selected_vial_id.get())
                                                    .collect::<Vec<_>>()
                                            }
                                            key=|vial| vial.id.clone()
                                            children=move |vial| {
                                                let label = format!(
                                                    "{} · {}{}",
                                                    vial.esterKind.clone().unwrap_or_else(|| "Unknown ester".to_string()),
                                                    vial.batchNumber.clone().unwrap_or_else(|| "batch ?".to_string()),
                                                    vial.source
                                                        .as_ref()
                                                        .map(|source| format!(" · {}", source))
                                                        .unwrap_or_default()
                                                );
                                                let value = vial.id.clone();
                                                view! {
                                                    <option value=value.clone() selected=move || selected_vial_id.get() == value>
                                                        {label}
                                                    </option>
                                                }
                                            }
                                        />
                                    </select>
                                <A class="pill-button" href="/vials/create">"New..."</A>
                            </div>

                            <Show when=move || !selected_vial_id.get().is_empty()>
                                <label>
                                    "Sub-vial / cartridge (optional)"
                                    <select on:change=move |ev| selected_sub_vial_id.set(event_target_value(&ev))>
                                        <option value="" selected=move || selected_sub_vial_id.get().is_empty()>
                                            "None"
                                        </option>
                                        <For
                                            each=move || {
                                                store
                                                    .data
                                                    .get()
                                                    .vials
                                                    .iter()
                                                    .find(|v| v.id == selected_vial_id.get())
                                                    .map(|v| v.subVials.clone())
                                                    .unwrap_or_default()
                                            }
                                            key=|sub| sub.id.clone()
                                            children=move |sub| {
                                                let label = format!("#{}", sub.personalNumber);
                                                let value = sub.id.clone();
                                                view! {
                                                    <option value=value.clone() selected=move || selected_sub_vial_id.get() == value>
                                                        {label}
                                                    </option>
                                                }
                                            }
                                        />
                                    </select>
                                </label>
                            </Show>

                            <label>
                                "Syringe kind (optional)"
                                <select on:change=move |ev| syringe_kind.set(event_target_value(&ev))>
                                    <option value="" selected=move || syringe_kind.get().is_empty()>
                                        "Select..."
                                    </option>
                                    <For
                                        each=move || SYRINGE_KINDS.to_vec()
                                        key=|kind| syringe_kind_label(kind).to_string()
                                        children=move |kind| {
                                            let label = syringe_kind_label(&kind).to_string();
                                            let value = label.clone();
                                            view! {
                                                <option value=value.clone() selected=move || syringe_kind.get() == value>
                                                    {label}
                                                </option>
                                            }
                                        }
                                    />
                                </select>
                            </label>

                            <label>
                                "Needle length (optional)"
                                <input
                                    type="text"
                                    placeholder="e.g., 4mm or 1\""
                                    on:input=move |ev| needle_length.set(event_target_value(&ev))
                                    prop:value=move || needle_length.get()
                                />
                            </label>
                            <label>
                                "Needle gauge (optional)"
                                <input
                                    type="text"
                                    placeholder="e.g., 32g or 30G"
                                    on:input=move |ev| needle_gauge.set(event_target_value(&ev))
                                    prop:value=move || needle_gauge.get()
                                />
                            </label>
                        </Show>

                        <Show when=move || !schedule_only && estrogen_method.get() == "oral">
                            <label>
                                "Pill quantity"
                                <input
                                    type="text"
                                    min="1"
                                    step="any"
                                    inputmode="decimal"
                                    on:input=move |ev| estrogen_pill_qty.set(event_target_value(&ev))
                                    prop:value=move || estrogen_pill_qty.get()
                                />
                                <p class="muted">
                                    "Total = "
                                    <strong>{move || fmt(estrogen_pill_total.get(), 2)}</strong>
                                    " mg"
                                </p>
                            </label>
                        </Show>

                        <Show when=move || mode.get() == "schedule">
                            <label>
                                "Started (regimen start)"
                                <input
                                    type="date"
                                    on:input=move |ev| estrogen_start.set(event_target_value(&ev))
                                    prop:value=move || estrogen_start.get()
                                />
                            </label>
                            <label class="toggle toggle-wide">
                                <input
                                    type="checkbox"
                                    aria-label="Fill in estrogen doses from the start date"
                                    on:change=move |ev| backfill_estrogen.set(event_target_checked(&ev))
                                    prop:checked=move || backfill_estrogen.get()
                                />
                                <span class="toggle-track" aria-hidden="true"></span>
                                <span class="toggle-label">"Fill in doses from the start date"</span>
                            </label>
                            <Show when=move || backfill_estrogen.get() && !estrogen_start.get().is_empty()>
                                <p class="muted">
                                    {move || {
                                        let count = estrogen_backfill_dates.get().len();
                                        if count == 0 {
                                            "No missing doses to add.".to_string()
                                        } else {
                                            format!("Will add {count} dose(s) up to today.")
                                        }
                                    }}
                                </p>
                            </Show>
                            <label>
                                "Next Dose Date"
                                <input
                                    type="datetime-local"
                                    on:input=move |ev| estrogen_next_date.set(event_target_value(&ev))
                                    prop:value=move || estrogen_next_date.get()
                                />
                            </label>
                        </Show>

                        <div class="calc-grid">
                            <label>
                                "Type"
                                <select
                                    on:change=move |ev| {
                                        if estrogen_method.get() == "injection" {
                                            injectable_type.set(event_target_value(&ev));
                                        } else {
                                            oral_type.set(event_target_value(&ev));
                                        }
                                    }
                                >
                                    <For
                                        each=move || {
                                            if estrogen_method.get() == "injection" {
                                                INJECTABLE_OPTIONS
                                                    .iter()
                                                    .map(|kind| injectable_label(kind).to_string())
                                                    .collect::<Vec<_>>()
                                            } else {
                                                ORAL_OPTIONS
                                                    .iter()
                                                    .map(|kind| oral_label(kind).to_string())
                                                    .collect::<Vec<_>>()
                                            }
                                        }
                                        key=|label| label.clone()
                                        children=move |label| {
                                            let value = label.clone();
                                            view! {
                                                <option
                                                    value=value.clone()
                                                    selected=move || {
                                                        if estrogen_method.get() == "injection" {
                                                            injectable_type.get() == value
                                                        } else {
                                                            oral_type.get() == value
                                                        }
                                                    }
                                                >
                                                    {label}
                                                </option>
                                            }
                                        }
                                    />
                                </select>
                            </label>

                            <Show when=move || mode.get() == "schedule">
                                <label>
                                    "Frequency (days)"
                                    <input
                                        type="text"
                                        step="any"
                                        inputmode="decimal"
                                        aria-invalid=move || if estrogen_frequency_error.get().is_some() { "true" } else { "false" }
                                        aria-describedby="estrogen-frequency-error"
                                        on:input=move |ev| {
                                            estrogen_frequency_error.set(None);
                                            if estrogen_method.get() == "injection" {
                                                injection_frequency.set(event_target_value(&ev));
                                            } else {
                                                oral_frequency.set(event_target_value(&ev));
                                            }
                                        }
                                        prop:value=move || {
                                            if estrogen_method.get() == "injection" {
                                                injection_frequency.get()
                                            } else {
                                                oral_frequency.get()
                                            }
                                        }
                                    />
                                </label>
                            </Show>

                            <label>
                                {move || {
                                    if estrogen_method.get() != "injection" {
                                        "Dose"
                                    } else if estrogen_dose_in_iu.get() {
                                        "Dose (IU)"
                                    } else {
                                        "Dose (mg)"
                                    }
                                }}
                                <span class="unit-field">
                                    <input
                                        type="text"
                                        step="any"
                                        inputmode="decimal"
                                        aria-invalid=move || if estrogen_dose_error.get().is_some() { "true" } else { "false" }
                                        aria-describedby="estrogen-dose-error"
                                        on:input=move |ev| {
                                            estrogen_dose_error.set(None);
                                            estrogen_dose.set(event_target_value(&ev));
                                        }
                                        prop:value=move || estrogen_dose.get()
                                    />
                                    <span class="unit">
                                        {move || dose_unit_suffix(
                                            estrogen_method.get() == "injection" && estrogen_dose_in_iu.get(),
                                        )}
                                    </span>
                                </span>
                            </label>
                            <Show when=move || estrogen_dose_error.get().is_some()>
                                <p class="field-error" id="estrogen-dose-error" role="alert">
                                    {move || estrogen_dose_error.get().unwrap_or_default()}
                                </p>
                            </Show>
                            <Show when=move || estrogen_frequency_error.get().is_some()>
                                <p class="field-error" id="estrogen-frequency-error" role="alert">
                                    {move || estrogen_frequency_error.get().unwrap_or_default()}
                                </p>
                            </Show>

                            <Show
                                when=move || {
                                    estrogen_method.get() == "injection" && estrogen_dose_in_iu.get()
                                }
                            >
                                <p class="muted">
                                    "Stored as "
                                    <strong>
                                        {move || {
                                            estrogen_dose_as_mg
                                                .get()
                                                .map(|dose| format!("{} mg", fmt(dose, 3)))
                                                .unwrap_or_else(|| "—".to_string())
                                        }}
                                    </strong>
                                </p>
                            </Show>

                            <Show
                                when=move || {
                                    estrogen_method.get() == "injection"
                                        && settings.get().displayInjectableInIU.unwrap_or(false)
                                        && !estrogen_dose_in_iu.get()
                                }
                            >
                                <p class="muted">
                                    "To enter injectable doses in IU, select a vial with concentration."
                                </p>
                            </Show>

                        </div>
                        </Show>

                        

                        <Show when=move || !schedule_only>
                            <Show when=move || estrogen_logged_today.get().is_some()>
                                <p class="inline-note warn">
                                    {move || estrogen_logged_today.get().unwrap_or_default()}
                                </p>
                            </Show>
                            <p class="card-summary-line">
                                {move || {
                                    if let Some(logged) = estrogen_logged.get() {
                                        return logged;
                                    }
                                    if estrogen_record_dose.get().is_none() {
                                        return "Choose a dose first".to_string();
                                    }
                                    format!(
                                        "{} · {}",
                                        estrogen_dose_label(),
                                        if estrogen_method.get() == "injection" { "injection" } else { "oral" }
                                    )
                                }}
                            </p>
                        </Show>
                        </Show>
                    </div>
                        </section>

                    <section class="med-row" class:open=move || open_row.get() == Some(MedRow::Antiandrogen)>
                        <div class="med-row-head">
                            <button
                                type="button"
                                class="med-row-toggle"
                                aria-expanded=move || if open_row.get() == Some(MedRow::Antiandrogen) { "true" } else { "false" }
                                aria-controls="row-antiandrogen"
                                on:click=move |_| toggle_row(MedRow::Antiandrogen)
                            >
                                <span class="med-row-name">"Antiandrogen"</span>
                                <span class="med-row-meta">{move || aa_row_meta.get()}</span>
                                <span class="med-row-chev" aria-hidden="true">"⌄"</span>
                            </button>
                            <Show when=move || schedule_only && schedule_aa.get()>
                                <button
                                    type="button"
                                    class="chip"
                                    on:click=move |_| schedule_aa.set(false)
                                >
                                    "Remove regimen"
                                </button>
                            </Show>
                        </div>
                        <Show when=move || schedule_only && !schedule_aa.get()>
                            <p class="muted">"No antiandrogen regimen yet."</p>
                            <button
                                type="button"
                                class="chip"
                                on:click=move |_| schedule_aa.set(true)
                            >
                                "+ Add antiandrogen regimen"
                            </button>
                        </Show>
                        <div class="med-row-body" id="row-antiandrogen" hidden=move || open_row.get() != Some(MedRow::Antiandrogen)>
                        <Show when=move || !schedule_only || schedule_aa.get()>
                        <Show when=move || !schedule_only || schedule_estrogen.get()>
                        <Show when=move || !schedule_only>
                            <button
                                type="button"
                                class="chip"
                                aria-expanded=move || if aa_note_open.get() { "true" } else { "false" }
                                on:click=move |_| aa_note_open.set(!aa_note_open.get())
                            >
                                "+ note"
                            </button>
                            <Show when=move || aa_note_open.get()>
                                <label>
                                    "Note (optional)"
                                    <textarea
                                        rows="2"
                                        on:input=move |ev| aa_note.set(event_target_value(&ev))
                                        prop:value=move || aa_note.get()
                                    ></textarea>
                                </label>
                            </Show>
                        </Show>

                        <Show when=move || mode.get() == "schedule" && !aa_type.get().is_empty()>
                            <label>
                                "Started (regimen start)"
                                <input
                                    type="date"
                                    on:input=move |ev| aa_start.set(event_target_value(&ev))
                                    prop:value=move || aa_start.get()
                                />
                            </label>
                            <label class="toggle toggle-wide">
                                <input
                                    type="checkbox"
                                    aria-label="Fill in antiandrogen doses from the start date"
                                    on:change=move |ev| backfill_aa.set(event_target_checked(&ev))
                                    prop:checked=move || backfill_aa.get()
                                />
                                <span class="toggle-track" aria-hidden="true"></span>
                                <span class="toggle-label">"Fill in doses from the start date"</span>
                            </label>
                            <Show when=move || backfill_aa.get() && !aa_start.get().is_empty()>
                                <p class="muted">
                                    {move || {
                                        let count = aa_backfill_dates.get().len();
                                        if count == 0 {
                                            "No missing doses to add.".to_string()
                                        } else {
                                            format!("Will add {count} dose(s) up to today.")
                                        }
                                    }}
                                </p>
                            </Show>
                            <label>
                                "Next Dose Date"
                                <input
                                    type="datetime-local"
                                    on:input=move |ev| aa_next_date.set(event_target_value(&ev))
                                    prop:value=move || aa_next_date.get()
                                />
                            </label>
                        </Show>

                        <div class="calc-grid">
                            <label>
                                "Type"
                                <select on:change=move |ev| aa_type.set(event_target_value(&ev))>
                                    <For
                                        each=move || {
                                            ANTIANDROGEN_OPTIONS
                                                .iter()
                                                .map(|kind| antiandrogen_label(kind).to_string())
                                                .collect::<Vec<_>>()
                                        }
                                        key=|label| label.clone()
                                        children=move |label| {
                                            let value = label.clone();
                                            view! {
                                                <option value=value.clone() selected=move || aa_type.get() == value>
                                                    {label}
                                                </option>
                                            }
                                        }
                                    />
                                </select>
                            </label>

                            <Show when=move || mode.get() == "schedule" && !aa_type.get().is_empty()>
                                <label>
                                    "Frequency (days)"
                                    <input
                                        type="text"
                                        step="any"
                                        inputmode="decimal"
                                        aria-invalid=move || if aa_frequency_error.get().is_some() { "true" } else { "false" }
                                        aria-describedby="aa-frequency-error"
                                        on:input=move |ev| {
                                            aa_frequency_error.set(None);
                                            aa_frequency.set(event_target_value(&ev));
                                        }
                                        prop:value=move || aa_frequency.get()
                                    />
                                </label>
                            </Show>

                            <label>
                                "Dose"
                                <span class="unit-field">
                                    <input
                                        type="text"
                                        step="any"
                                        inputmode="decimal"
                                        aria-invalid=move || if aa_dose_error.get().is_some() { "true" } else { "false" }
                                        aria-describedby="aa-dose-error"
                                        on:input=move |ev| {
                                            aa_dose_error.set(None);
                                            aa_dose.set(event_target_value(&ev));
                                        }
                                        prop:value=move || aa_dose.get()
                                    />
                                    <span class="unit">"mg"</span>
                                </span>
                            </label>
                            <Show when=move || aa_dose_error.get().is_some()>
                                <p class="field-error" id="aa-dose-error" role="alert">
                                    {move || aa_dose_error.get().unwrap_or_default()}
                                </p>
                            </Show>
                            <Show when=move || aa_frequency_error.get().is_some()>
                                <p class="field-error" id="aa-frequency-error" role="alert">
                                    {move || aa_frequency_error.get().unwrap_or_default()}
                                </p>
                            </Show>
                        </div>
                        </Show>

                        <Show when=move || !schedule_only>
                            <Show when=move || aa_logged_today.get().is_some()>
                                <p class="inline-note warn">
                                    {move || aa_logged_today.get().unwrap_or_default()}
                                </p>
                            </Show>
                            <p class="card-summary-line">
                                    {move || {
                                        if let Some(logged) = aa_logged.get() {
                                            return logged;
                                        }
                                        match aa_record_dose.get() {
                                            None => "Choose a dose first".to_string(),
                                            Some(dose) => format!(
                                                "{} · {} mg",
                                                antiandrogen_label(&antiandrogen_from_label(&aa_type.get())),
                                                fmt(dose, 2)
                                            ),
                                        }
                                    }}
                            </p>
                        </Show>
                        </Show>
                    </div>
                        </section>

                    <section class="med-row" class:open=move || open_row.get() == Some(MedRow::Progesterone)>
                        <div class="med-row-head">
                            <button
                                type="button"
                                class="med-row-toggle"
                                aria-expanded=move || if open_row.get() == Some(MedRow::Progesterone) { "true" } else { "false" }
                                aria-controls="row-progesterone"
                                on:click=move |_| toggle_row(MedRow::Progesterone)
                            >
                                <span class="med-row-name">"Progesterone"</span>
                                <span class="med-row-meta">{move || prog_row_meta.get()}</span>
                                <span class="med-row-chev" aria-hidden="true">"⌄"</span>
                            </button>
                            <Show when=move || schedule_only && prog_card_open.get()>
                                <span class="status-chip">{move || prog_regimen_short.get()}</span>
                                <button
                                    type="button"
                                    class="chip"
                                    on:click=move |_| {
                                        schedule_prog.set(false);
                                        prog_total.set(String::new());
                                        prog_start.set(String::new());
                                    }
                                >
                                    "Remove regimen"
                                </button>
                            </Show>
                        </div>
                        <Show when=move || schedule_only && !prog_card_open.get()>
                            <p class="muted">"No progesterone regimen yet."</p>
                            <button
                                type="button"
                                class="chip"
                                on:click=move |_| start_prog_regimen.with_value(|start| start())
                            >
                                "+ Add progesterone regimen"
                            </button>
                        </Show>
                        <div class="med-row-body" id="row-progesterone" hidden=move || open_row.get() != Some(MedRow::Progesterone)>
                        <Show when=move || !schedule_only || prog_card_open.get()>
                        <Show when=move || schedule_only || prog_card_open.get()>
                        // Dosing is chosen with chips; one legal progesterone type
                        // (micronised) means there is no type select on this card.
                        <div class="chip-row" role="group" aria-label="Progesterone dose">
                                {[("50", "50 mg"), ("100", "100 mg"), ("200", "200 mg")]
                                    .into_iter()
                                    .map(|(value, label)| {
                                        let dose_value = value.to_string();
                                        let active_value = dose_value.clone();
                                        view! {
                                            <button
                                                type="button"
                                                class="chip"
                                                class:active=move || prog_total.get() == active_value
                                                on:click=move |_| prog_total.set(dose_value.clone())
                                            >
                                                {label}
                                            </button>
                                        }
                                    })
                                    .collect_view()}
                            </div>
                            <label>
                                "Other dose"
                                <span class="unit-field">
                                    <input
                                        type="text"
                                        inputmode="decimal"
                                        placeholder="e.g. 275"
                                        aria-invalid=move || if prog_dose_error.get().is_some() { "true" } else { "false" }
                                        aria-describedby="prog-dose-error"
                                        on:input=move |ev| {
                                            prog_dose_error.set(None);
                                            prog_total.set(event_target_value(&ev));
                                        }
                                        prop:value=move || prog_total.get()
                                    />
                                    <span class="unit">"mg"</span>
                                </span>
                            </label>
                            <div class="chip-row" role="group" aria-label="Progesterone route">
                                {[ProgesteroneRoutes::Oral, ProgesteroneRoutes::Boofed]
                                    .into_iter()
                                    .map(|route| {
                                        let label = progesterone_route_chip_label(&route).to_string();
                                        let value = label.clone();
                                        let active_value = label.clone();
                                        view! {
                                            <button
                                                type="button"
                                                class="chip"
                                                class:active=move || prog_route.get() == active_value
                                                on:click=move |_| prog_route.set(value.clone())
                                            >
                                                {label}
                                            </button>
                                        }
                                    })
                                    .collect_view()}
                            </div>
                            <p class="muted">
                                {move || match prog_record_total.get().map(progesterone_dose_amounts) {
                                    Some((dose, quantity)) if quantity > 1.0 => {
                                        format!("{} × {} mg = {} mg", fmt(quantity, 2), fmt(dose, 2), fmt(dose * quantity, 2))
                                    }
                                    Some((dose, _)) => format!("{} mg", fmt(dose, 2)),
                                    None => "Choose a dose first".to_string(),
                                }}
                            </p>
                            <Show when=move || prog_dose_error.get().is_some()>
                                <p class="field-error" id="prog-dose-error" role="alert">
                                    {move || prog_dose_error.get().unwrap_or_default()}
                                </p>
                            </Show>
                        <Show when=move || !schedule_only>
                            <button
                                type="button"
                                class="chip"
                                aria-expanded=move || if prog_note_open.get() { "true" } else { "false" }
                                on:click=move |_| prog_note_open.set(!prog_note_open.get())
                            >
                                "+ note"
                            </button>
                            <Show when=move || prog_note_open.get()>
                                <label>
                                    "Note (optional)"
                                    <textarea
                                        rows="2"
                                        on:input=move |ev| prog_note.set(event_target_value(&ev))
                                        prop:value=move || prog_note.get()
                                    ></textarea>
                                </label>
                            </Show>
                        </Show>

                        <Show when=move || schedule_only>
                            <div class="regimen-block">
                                <div class="chip-row" role="group" aria-label="How often progesterone is taken">
                                    {REGIMEN_INTERVALS
                                        .into_iter()
                                        .map(|(days, label)| {
                                            let value = days.to_string();
                                            let active_value = value.clone();
                                            view! {
                                                <button
                                                    type="button"
                                                    class="chip"
                                                    class:active=move || prog_frequency.get() == active_value
                                                    on:click=move |_| prog_frequency.set(value.clone())
                                                >
                                                    {label}
                                                </button>
                                            }
                                        })
                                        .collect_view()}
                                    <label class="chip-field">
                                        "or every"
                                        <input
                                            type="text"
                                            inputmode="decimal"
                                            aria-label="Custom interval in days"
                                            aria-invalid=move || if prog_frequency_error.get().is_some() { "true" } else { "false" }
                                            aria-describedby="prog-frequency-error"
                                            on:input=move |ev| {
                                                prog_frequency_error.set(None);
                                                prog_frequency.set(event_target_value(&ev));
                                            }
                                            prop:value=move || prog_frequency.get()
                                        />
                                        "days"
                                    </label>
                                </div>
                                <Show when=move || prog_frequency_error.get().is_some()>
                                    <p class="field-error" id="prog-frequency-error" role="alert">
                                        {move || prog_frequency_error.get().unwrap_or_default()}
                                    </p>
                                </Show>

                                <div class="calc-grid">
                                    <label>
                                        "Started (first dose)"
                                        <input
                                            type="date"
                                            on:input=move |ev| prog_start.set(event_target_value(&ev))
                                            prop:value=move || prog_start.get()
                                        />
                                    </label>
                                    <label>
                                        "Time of day"
                                        <input
                                            type="time"
                                            on:input=move |ev| prog_time.set(event_target_value(&ev))
                                            prop:value=move || prog_time.get()
                                        />
                                    </label>
                                </div>

                                <label class="toggle toggle-wide">
                                    <input
                                        type="checkbox"
                                        aria-label="Fill in progesterone doses since the start date"
                                        on:change=move |ev| backfill_prog.set(event_target_checked(&ev))
                                        prop:checked=move || backfill_prog.get()
                                    />
                                    <span class="toggle-track" aria-hidden="true"></span>
                                    <span class="toggle-label">"Fill in the doses I have already taken"</span>
                                </label>
                                <Show when=move || backfill_prog.get() && !prog_start.get().is_empty()>
                                    <p class="muted">
                                        {move || {
                                            let count = prog_backfill_dates.get().len();
                                            if count == 0 {
                                                "No missing doses to add.".to_string()
                                            } else {
                                                format!("Will add {count} dose(s) up to today.")
                                            }
                                        }}
                                    </p>
                                </Show>

                                <p class="inline-note">{move || prog_regimen_summary.get()}</p>
                            </div>
                        </Show>
                        </Show>

                        


                        <Show when=move || !schedule_only>
                            <Show when=move || prog_logged_today.get().is_some()>
                                <p class="inline-note warn">
                                    {move || prog_logged_today.get().unwrap_or_default()}
                                </p>
                            </Show>
                            <p class="card-summary-line">
                                    {move || {
                                        if let Some(logged) = prog_logged.get() {
                                            return logged;
                                        }
                                        match prog_record_total.get().map(progesterone_dose_amounts) {
                                            None => "Choose a dose first".to_string(),
                                            Some((dose, quantity)) if quantity > 1.0 => format!(
                                                "{} × {} mg = {} mg",
                                                fmt(quantity, 2),
                                                fmt(dose, 2),
                                                fmt(dose * quantity, 2)
                                            ),
                                            Some((dose, _)) => format!("{} mg", fmt(dose, 2)),
                                        }
                                    }}
                                </p>
                        </Show>
                        </Show>
                    </div>
                        </section>

                    <Show when=move || schedule_only && form_error.get().is_some()>
                        <p class="error">{move || form_error.get().unwrap_or_default()}</p>
                    </Show>

                    <Show when=move || schedule_only>
                        <div class="form-actions">
                            <button type="submit">"Save schedule"</button>
                            <Show when=move || schedule_feedback.get()>
                                <p class="muted">"Schedule saved!"</p>
                            </Show>
                        </div>
                    </Show>

                    <Show when=move || !schedule_only>
                        <div class="form-actions">
                            <span class="card-summary">
                                {move || {
                                    if open_row.get().is_none() {
                                        return "Pick a medication above to log it".to_string();
                                    }
                                    if !open_row_ready.get() {
                                        return "Nothing to write yet — set a dose first".to_string();
                                    }
                                    format!(
                                        "Writing {}",
                                        open_row_label.get().trim_start_matches("Log ")
                                    )
                                }}
                            </span>
                            <button
                                type="button"
                                on:click=move |_| commit_open_row.with_value(|commit| commit())
                                prop:disabled=move || !open_row_ready.get()
                            >
                                {move || open_row_label.get()}
                            </button>
                        </div>

                        <div class="toast" role="status" hidden=move || toast.get().is_none()>
                            <span>
                                {move || toast.get().map(|state| state.message).unwrap_or_default()}
                            </span>
                            <button class="toast-undo" type="button" on:click=move |_| undo_toast.with_value(|undo| undo())>
                                "Undo"
                            </button>
                        </div>
                    </Show>
                </form>
            </div>
        }
        .into_view(),
    )
}

#[component]
pub fn CreateDosage() -> impl IntoView {
    dosage_editor_page(false)
}

#[component]
pub fn EditSchedulePage() -> impl IntoView {
    dosage_editor_page(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progesterone_dose_chips_map_to_per_pill_amounts() {
        // 200 mg is the two-pill chip; everything else is a single pill.
        assert_eq!(progesterone_dose_amounts(200.0), (100.0, 2.0));
        assert_eq!(progesterone_dose_amounts(50.0), (50.0, 1.0));
        assert_eq!(progesterone_dose_amounts(275.0), (275.0, 1.0));
        assert_eq!(progesterone_dose_amounts(100.0), (100.0, 1.0));
    }

    #[test]
    fn dose_unit_is_a_static_suffix() {
        assert_eq!(dose_unit_suffix(false), "mg");
        assert_eq!(dose_unit_suffix(true), "IU");
    }

    #[test]
    fn rectal_chip_keeps_the_stored_boofed_route() {
        assert_eq!(
            progesterone_route_from_chip("Rectal"),
            ProgesteroneRoutes::Boofed
        );
        assert_eq!(
            progesterone_route_from_chip("Oral"),
            ProgesteroneRoutes::Oral
        );
        assert_eq!(
            progesterone_route_chip_label(&ProgesteroneRoutes::Boofed),
            "Rectal"
        );
    }

    #[test]
    fn progesterone_commit_label_states_the_outcome() {
        assert_eq!(
            progesterone_commit_label(100.0, &ProgesteroneRoutes::Oral),
            "Log progesterone · 100 mg · oral"
        );
        assert_eq!(
            progesterone_commit_label(200.0, &ProgesteroneRoutes::Boofed),
            "Log progesterone · 200 mg · rectal"
        );
    }

    #[test]
    fn entry_dose_text_reads_as_pills_not_debug() {
        let entry = DosageHistoryEntry::Progesterone {
            date: 0,
            id: None,
            kind: Progesterones::Micronized,
            route: ProgesteroneRoutes::Oral,
            dose: 100.0,
            unit: HormoneUnits::Mg,
            pillQuantity: Some(2.0),
            note: None,
        };
        assert_eq!(entry_dose_text(&entry), "2 × 100 mg");
        assert!(!entry_dose_text(&entry).contains("Mg"));
    }

    /// The page's own source. Names are assembled from parts so that the scan
    /// cannot match its own needles.
    fn page_source() -> &'static str {
        include_str!("create_dosage.rs")
    }

    #[test]
    fn record_mode_has_no_record_switches() {
        // The per-medication "Record X Dose" switches are gone: card contents are
        // the only intent, so a progesterone-only card cannot write estrogen.
        let source = page_source();
        for medication in ["estrogen", "aa", "prog"] {
            let switch = format!("record_{medication}");
            assert!(
                !source.contains(&switch),
                "the `{switch}` switch must stay deleted from create_dosage.rs"
            );
        }
    }

    #[test]
    fn progesterone_unit_is_a_suffix_not_a_select() {
        let source = page_source();
        let unit_select_helper = format!("hormone_unit_{}()", "labels");
        assert!(
            !source.contains(&unit_select_helper),
            "dose fields must not build a unit <select>; render a static suffix instead"
        );
        assert!(source.contains("class=\"unit-field\""));
        assert!(source.contains("dose_unit_suffix("));
    }

    #[test]
    fn regimen_start_uses_the_chosen_time_of_day() {
        let hour = 60 * 60 * 1000;
        assert_eq!(clock_offset_ms("22:00", "10:00"), 22 * hour);
        assert_eq!(clock_offset_ms("09:30", "22:00"), 9 * hour + 30 * 60 * 1000);
        // A missing or malformed time falls back to the default, not to midnight.
        assert_eq!(clock_offset_ms("", "22:00"), 22 * hour);
        assert_eq!(clock_offset_ms("99:99", "22:00"), 22 * hour);
        assert_eq!(clock_offset_ms("24:00", "22:00"), 22 * hour);
        // An empty date has no start at all.
        assert_eq!(parse_regimen_start_at("  ", "22:00", "22:00"), None);
    }

    #[test]
    fn next_dose_steps_from_the_start_until_it_is_upcoming() {
        const DAY: i64 = 86_400_000;
        let start = 1_000_000_000_000i64;
        // A start in the future is the next dose.
        assert_eq!(next_dose_at_or_after(start, 1.0, start - DAY), Some(start));
        // Otherwise the grid advances by whole steps, landing at or after now.
        assert_eq!(next_dose_at_or_after(start, 1.0, start + DAY / 2), Some(start + DAY));
        assert_eq!(next_dose_at_or_after(start, 1.0, start + DAY), Some(start + DAY));
        assert_eq!(next_dose_at_or_after(start, 2.0, start + DAY * 5), Some(start + DAY * 6));
        // A frequency that cannot produce a step has no next dose.
        assert_eq!(next_dose_at_or_after(start, 0.0, start), None);
        assert_eq!(next_dose_at_or_after(start, f64::NAN, start), None);
    }

    #[test]
    fn interval_chips_read_as_a_schedule() {
        assert_eq!(interval_phrase(1.0), "every day");
        assert_eq!(interval_phrase(2.0), "every other day");
        assert_eq!(interval_phrase(3.0), "every 3 days");
        assert_eq!(interval_phrase(0.0), "no interval set");
        // The chips offered in the UI are the ones that mean these day counts.
        let days: Vec<&str> = REGIMEN_INTERVALS.iter().map(|(days, _)| *days).collect();
        assert_eq!(days, vec!["1", "2", "3"]);
    }
}
