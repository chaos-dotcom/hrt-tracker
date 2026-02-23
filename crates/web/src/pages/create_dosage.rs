use gloo_timers::callback::Timeout;
use js_sys::Date;
use leptos::window;
use leptos::*;
use leptos_router::{use_navigate, A};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsValue;

use crate::layout::page_layout;
use crate::store::use_store;
use crate::utils::{
    hormone_unit_label, injectable_dose_from_iu, injectable_iu_from_dose, parse_decimal,
    parse_decimal_or_nan, parse_hormone_unit,
};
use hrt_shared::logic::backfill_scheduled_doses;
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

const PROGESTERONE_ROUTE_OPTIONS: [ProgesteroneRoutes; 2] =
    [ProgesteroneRoutes::Oral, ProgesteroneRoutes::Boofed];

const HORMONE_UNITS: [HormoneUnits; 9] = [
    HormoneUnits::E2PgMl,
    HormoneUnits::E2PmolL,
    HormoneUnits::TNgDl,
    HormoneUnits::TNmolL,
    HormoneUnits::Mg,
    HormoneUnits::NgMl,
    HormoneUnits::MIuMl,
    HormoneUnits::MIuL,
    HormoneUnits::UL,
];

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

fn progesterone_route_label(kind: &ProgesteroneRoutes) -> &'static str {
    match kind {
        ProgesteroneRoutes::Oral => "Oral",
        ProgesteroneRoutes::Boofed => "Boofed",
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

fn progesterone_route_from_label(value: &str) -> ProgesteroneRoutes {
    PROGESTERONE_ROUTE_OPTIONS
        .iter()
        .find(|kind| progesterone_route_label(kind) == value)
        .cloned()
        .unwrap_or(ProgesteroneRoutes::Oral)
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

fn hormone_unit_labels() -> Vec<String> {
    HORMONE_UNITS
        .iter()
        .map(|unit| hormone_unit_label(unit).to_string())
        .collect()
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
                    navigate("/edit/schedule", Default::default());
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
    let estrogen_dose = create_rw_signal("0".to_string());
    let estrogen_unit = create_rw_signal(hormone_unit_label(&HormoneUnits::Mg).to_string());
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

    let aa_type = create_rw_signal(String::new());
    let aa_dose = create_rw_signal("0".to_string());
    let aa_unit = create_rw_signal(hormone_unit_label(&HormoneUnits::Mg).to_string());
    let aa_frequency = create_rw_signal("1".to_string());
    let aa_next_date = create_rw_signal(String::new());

    let prog_type = create_rw_signal(String::new());
    let prog_dose = create_rw_signal("0".to_string());
    let prog_unit = create_rw_signal(hormone_unit_label(&HormoneUnits::Mg).to_string());
    let prog_route =
        create_rw_signal(progesterone_route_label(&ProgesteroneRoutes::Oral).to_string());
    let prog_frequency = create_rw_signal("1".to_string());
    let prog_next_date = create_rw_signal(String::new());

    let record_estrogen = create_rw_signal(true);
    let record_aa = create_rw_signal(false);
    let record_prog = create_rw_signal(false);
    let bonus_dose = create_rw_signal(false);

    let estrogen_note = create_rw_signal(String::new());
    let aa_note = create_rw_signal(String::new());
    let prog_note = create_rw_signal(String::new());
    let estrogen_pill_qty = create_rw_signal("1".to_string());
    let prog_pill_qty = create_rw_signal("1".to_string());

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

            let value = parse_optional_num(&estrogen_dose.get());
            let Some(current_value) = value else {
                if !target_iu_mode {
                    estrogen_dose_in_iu.set(false);
                }
                return;
            };

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
            let converted = if target_iu_mode {
                injectable_iu_from_dose(
                    &data_value,
                    current_value,
                    &HormoneUnits::Mg,
                    selected_vial_id,
                    schedule_vial_id,
                )
            } else {
                injectable_dose_from_iu(
                    &data_value,
                    current_value,
                    selected_vial_id,
                    schedule_vial_id,
                )
            };

            if let Some(next_value) = converted {
                estrogen_dose.set(if target_iu_mode {
                    fmt(next_value, 0)
                } else {
                    fmt(next_value, 3)
                });
                estrogen_dose_in_iu.set(target_iu_mode);
            } else if target_iu_mode {
                estrogen_dose_in_iu.set(false);
            } else {
                estrogen_dose.set(String::new());
                estrogen_dose_in_iu.set(false);
            }
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

    let initialized = create_rw_signal(false);
    create_effect({
        let store = store.clone();
        move |_| {
            if initialized.get() {
                return;
            }
            let data = store.data.get();
            if let Some(inj) = data.injectableEstradiol.as_ref() {
                estrogen_method.set("injection".to_string());
                injectable_type.set(injectable_label(&inj.kind).to_string());
                let dose_in_iu = settings.get().displayInjectableInIU.unwrap_or(false)
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
                    .map(|iu| fmt(iu, 0))
                    .unwrap_or_else(|| format!("{:.3}", inj.dose))
                } else {
                    format!("{:.3}", inj.dose)
                };
                estrogen_dose.set(dose_label);
                estrogen_dose_in_iu.set(dose_in_iu);
                estrogen_unit.set(hormone_unit_label(&HormoneUnits::Mg).to_string());
                injection_frequency.set(format!("{:.2}", inj.frequency));
                estrogen_next_date.set(
                    inj.nextDoseDate
                        .map(to_local_input_value)
                        .unwrap_or_default(),
                );
                selected_vial_id.set(inj.vialId.clone().unwrap_or_default());
                selected_sub_vial_id.set(inj.subVialId.clone().unwrap_or_default());
                syringe_kind.set(inj.syringeKind.clone().unwrap_or_default());
                needle_length.set(inj.needleLength.clone().unwrap_or_default());
                needle_gauge.set(inj.needleGauge.clone().unwrap_or_default());
            } else if let Some(oral) = data.oralEstradiol.as_ref() {
                estrogen_method.set("oral".to_string());
                oral_type.set(oral_label(&oral.kind).to_string());
                estrogen_dose.set(format!("{:.3}", oral.dose));
                estrogen_dose_in_iu.set(false);
                estrogen_unit.set(hormone_unit_label(&oral.unit).to_string());
                oral_frequency.set(format!("{:.2}", oral.frequency));
                estrogen_next_date.set(
                    oral.nextDoseDate
                        .map(to_local_input_value)
                        .unwrap_or_default(),
                );
                selected_vial_id.set(String::new());
                selected_sub_vial_id.set(String::new());
                syringe_kind.set(String::new());
                needle_length.set(String::new());
                needle_gauge.set(String::new());
            }

            if let Some(aa) = data.antiandrogen.as_ref() {
                aa_type.set(antiandrogen_label(&aa.kind).to_string());
                aa_dose.set(format!("{:.3}", aa.dose));
                aa_unit.set(hormone_unit_label(&aa.unit).to_string());
                aa_frequency.set(format!("{:.2}", aa.frequency));
                aa_next_date.set(
                    aa.nextDoseDate
                        .map(to_local_input_value)
                        .unwrap_or_default(),
                );
            }

            if let Some(prog) = data.progesterone.as_ref() {
                prog_type.set(progesterone_label(&prog.kind).to_string());
                prog_dose.set(format!("{:.3}", prog.dose));
                prog_unit.set(hormone_unit_label(&prog.unit).to_string());
                prog_route.set(progesterone_route_label(&prog.route).to_string());
                prog_frequency.set(format!("{:.2}", prog.frequency));
                prog_next_date.set(
                    prog.nextDoseDate
                        .map(to_local_input_value)
                        .unwrap_or_default(),
                );
            }
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

    let prog_pill_total = create_memo(move |_| {
        let dose = parse_num(&prog_dose.get());
        let qty = parse_num(&prog_pill_qty.get());
        if dose.is_finite() && qty.is_finite() {
            dose * qty
        } else {
            0.0
        }
    });

    let on_submit = {
        let store = store.clone();
        let navigate = navigate.clone();
        let feedback_timeout = feedback_timeout.clone();
        move |ev: leptos::ev::SubmitEvent| {
            ev.prevent_default();
            let mode_value = mode.get();
            let estrogen_method_value = estrogen_method.get();
            let estrogen_input_dose = parse_num(&estrogen_dose.get());
            let estrogen_unit_value = if estrogen_method_value == "injection" {
                HormoneUnits::Mg
            } else {
                parse_hormone_unit(&estrogen_unit.get()).unwrap_or(HormoneUnits::Mg)
            };
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

            if mode_value == "record" {
                let record_ms = parse_datetime_local(&record_date_time.get());
                store.data.update(|data| {
                    if record_estrogen.get() {
                        if estrogen_method_value == "injection" {
                            let kind = injectable_from_label(&injectable_type.get());
                            let record = DosageHistoryEntry::InjectableEstradiol {
                                date: record_ms,
                                id: None,
                                kind,
                                dose: estrogen_dose_value,
                                unit: estrogen_unit_value.clone(),
                                note: if estrogen_note.get().trim().is_empty() {
                                    None
                                } else {
                                    Some(estrogen_note.get())
                                },
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
                            };
                            data.dosageHistory.push(record);
                        } else {
                            let kind = oral_from_label(&oral_type.get());
                            let pill_qty = parse_optional_num(&estrogen_pill_qty.get())
                                .filter(|value| *value > 0.0);
                            let record = DosageHistoryEntry::OralEstradiol {
                                date: record_ms,
                                id: None,
                                kind,
                                dose: estrogen_dose_value,
                                unit: estrogen_unit_value.clone(),
                                pillQuantity: pill_qty,
                                note: if estrogen_note.get().trim().is_empty() {
                                    None
                                } else {
                                    Some(estrogen_note.get())
                                },
                            };
                            data.dosageHistory.push(record);
                        }
                    }

                    if record_aa.get() && !aa_type.get().trim().is_empty() {
                        let kind = antiandrogen_from_label(&aa_type.get());
                        let record = DosageHistoryEntry::Antiandrogen {
                            date: record_ms,
                            id: None,
                            kind,
                            dose: parse_num(&aa_dose.get()),
                            unit: parse_hormone_unit(&aa_unit.get()).unwrap_or(HormoneUnits::Mg),
                            note: if aa_note.get().trim().is_empty() {
                                None
                            } else {
                                Some(aa_note.get())
                            },
                        };
                        data.dosageHistory.push(record);
                    }

                    if record_prog.get() && !prog_type.get().trim().is_empty() {
                        let kind = progesterone_from_label(&prog_type.get());
                        let pill_qty =
                            parse_optional_num(&prog_pill_qty.get()).filter(|value| *value > 0.0);
                        let record = DosageHistoryEntry::Progesterone {
                            date: record_ms,
                            id: None,
                            kind,
                            route: progesterone_route_from_label(&prog_route.get()),
                            dose: parse_num(&prog_dose.get()),
                            unit: parse_hormone_unit(&prog_unit.get()).unwrap_or(HormoneUnits::Mg),
                            pillQuantity: pill_qty,
                            note: if prog_note.get().trim().is_empty() {
                                None
                            } else {
                                Some(prog_note.get())
                            },
                        };
                        data.dosageHistory.push(record);
                    }
                });
                store.mark_dirty();
                navigate("/view", Default::default());
                return;
            }

            store.data.update(|data| {
                if estrogen_method_value == "injection" {
                    let schedule = InjectableSchedule {
                        kind: injectable_from_label(&injectable_type.get()),
                        dose: estrogen_dose_value,
                        unit: estrogen_unit_value.clone(),
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
                    };
                    data.injectableEstradiol = Some(schedule);
                    data.oralEstradiol = None;
                } else {
                    let schedule = OralSchedule {
                        kind: oral_from_label(&oral_type.get()),
                        dose: estrogen_dose_value,
                        unit: estrogen_unit_value.clone(),
                        frequency: parse_num(&oral_frequency.get()).max(1.0),
                        nextDoseDate: parse_optional_datetime(&estrogen_next_date.get()),
                    };
                    data.oralEstradiol = Some(schedule);
                    data.injectableEstradiol = None;
                }

                if aa_type.get().trim().is_empty() {
                    data.antiandrogen = None;
                } else {
                    let schedule = AntiandrogenSchedule {
                        kind: antiandrogen_from_label(&aa_type.get()),
                        dose: parse_num(&aa_dose.get()),
                        unit: parse_hormone_unit(&aa_unit.get()).unwrap_or(HormoneUnits::Mg),
                        frequency: parse_num(&aa_frequency.get()).max(1.0),
                        nextDoseDate: parse_optional_datetime(&aa_next_date.get()),
                    };
                    data.antiandrogen = Some(schedule);
                }

                if prog_type.get().trim().is_empty() {
                    data.progesterone = None;
                } else {
                    let schedule = ProgesteroneSchedule {
                        kind: progesterone_from_label(&prog_type.get()),
                        route: progesterone_route_from_label(&prog_route.get()),
                        dose: parse_num(&prog_dose.get()),
                        unit: parse_hormone_unit(&prog_unit.get()).unwrap_or(HormoneUnits::Mg),
                        frequency: parse_num(&prog_frequency.get()).max(1.0),
                        nextDoseDate: parse_optional_datetime(&prog_next_date.get()),
                    };
                    data.progesterone = Some(schedule);
                }

                backfill_scheduled_doses(data);
            });

            store.mark_dirty();
            store.save();

            schedule_feedback.set(true);
            if let Some(existing) = feedback_timeout.borrow_mut().take() {
                drop(existing);
            }
            let schedule_feedback = schedule_feedback.clone();
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
                        <h2>{if schedule_only { "Edit schedule" } else { "Record dosage" }}</h2>
                        <p class="muted">
                            {if schedule_only {
                                "Configure recurring schedules for each medication."
                            } else {
                                "Record one-off doses for your regimen."
                            }}
                        </p>
                    </div>
                    <div class="header-actions">
                        <A href="/view">"View dosage history"</A>
                        <Show when=move || schedule_only>
                            <A href="/create/dosage">"Record dose"</A>
                        </Show>
                    </div>
                </div>

                <form class="form-wide" on:submit=on_submit>
                    <Show when=move || estrogen_method.get() == "injection">
                        <section class="card">
                            <h3>"Injection helper"</h3>
                            <div class="calc-grid">
                                <div class="calc-block">
                                    <h4>"Dose and Concentration to Volume"</h4>
                                    <div class="inline-equal">
                                        <label>
                                            {move || {
                                                if settings.get().displayInjectableInIU.unwrap_or(false) {
                                                    "Dose (IU)"
                                                } else {
                                                    "Dose (mg)"
                                                }
                                            }}
                                            <input
                                                type="text"
                                                step="any"
                                                on:input=move |ev| inj_conv_dose_mg.set(event_target_value(&ev))
                                                prop:value=move || inj_conv_dose_mg.get()
                                            />
                                        </label>
                                        <label>
                                            "Concentration (mg/mL)"
                                            <input
                                                type="text"
                                                step="any"
                                                on:input=move |ev| inj_conv_conc_mg_ml.set(event_target_value(&ev))
                                                prop:value=move || inj_conv_conc_mg_ml.get()
                                            />
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
                                            "Volume (mL)"
                                            <input
                                                type="text"
                                                step="any"
                                                on:input=move |ev| inj_conv_vol2_ml.set(event_target_value(&ev))
                                                prop:value=move || inj_conv_vol2_ml.get()
                                            />
                                        </label>
                                        <label>
                                            "Concentration (mg/mL)"
                                            <input
                                                type="text"
                                                step="any"
                                                on:input=move |ev| inj_conv_conc2_mg_ml.set(event_target_value(&ev))
                                                prop:value=move || inj_conv_conc2_mg_ml.get()
                                            />
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

                    <section class="card dose-card">
                        <h3>"Estrogen"</h3>
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

                        <Show when=move || mode.get() == "record">
                            <label class="toggle toggle-wide">
                                <input
                                    type="checkbox"
                                    on:change=move |ev| record_estrogen.set(event_target_checked(&ev))
                                    prop:checked=move || record_estrogen.get()
                                />
                                <span class="toggle-track" aria-hidden="true"></span>
                                <span class="toggle-label">"Record Estrogen Dose"</span>
                            </label>
                        </Show>

                        <Show when=move || mode.get() == "record" && record_estrogen.get()>
                            <label>
                                "Note (optional)"
                                <textarea
                                    rows="2"
                                    on:input=move |ev| estrogen_note.set(event_target_value(&ev))
                                    prop:value=move || estrogen_note.get()
                                ></textarea>
                            </label>
                        </Show>

                        <Show when=move || mode.get() == "record" && record_estrogen.get() && estrogen_method.get() == "injection">
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

                        <Show when=move || mode.get() == "record" && record_estrogen.get() && estrogen_method.get() == "injection">
                            <label>
                                "Injection site (optional)"
                                <select
                                    on:change=move |ev| injection_site.set(event_target_value(&ev))
                                    prop:value=move || injection_site.get()
                                >
                                    <option value="">"Select injection site"</option>
                                    <For
                                        each=move || INJECTION_SITES.to_vec()
                                        key=|site| injection_site_label(site).to_string()
                                        children=move |site| {
                                            let label = injection_site_label(&site).to_string();
                                            view! { <option value=label.clone()>{label}</option> }
                                        }
                                    />
                                </select>
                            </label>
                        </Show>

                        <Show when=move || estrogen_method.get() == "injection">
                            <label>
                                "Vial (optional)"
                                <div class="inline-equal">
                                    <select
                                        on:change=move |ev| selected_vial_id.set(event_target_value(&ev))
                                        prop:value=move || selected_vial_id.get()
                                    >
                                        <option value="">"None"</option>
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
                                                view! { <option value=vial.id.clone()>{label}</option> }
                                            }
                                        />
                                    </select>
                                    <A class="pill-button" href="/vials/create">"New..."</A>
                                </div>
                            </label>

                            <Show when=move || !selected_vial_id.get().is_empty()>
                                <label>
                                    "Sub-vial / cartridge (optional)"
                                    <select
                                        on:change=move |ev| selected_sub_vial_id.set(event_target_value(&ev))
                                        prop:value=move || selected_sub_vial_id.get()
                                    >
                                        <option value="">"None"</option>
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
                                                view! { <option value=sub.id.clone()>{label}</option> }
                                            }
                                        />
                                    </select>
                                </label>
                            </Show>

                            <label>
                                "Syringe kind (optional)"
                                <select
                                    on:change=move |ev| syringe_kind.set(event_target_value(&ev))
                                    prop:value=move || syringe_kind.get()
                                >
                                    <option value="">"Select..."</option>
                                    <For
                                        each=move || SYRINGE_KINDS.to_vec()
                                        key=|kind| syringe_kind_label(kind).to_string()
                                        children=move |kind| {
                                            let label = syringe_kind_label(&kind).to_string();
                                            view! { <option value=label.clone()>{label}</option> }
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

                        <Show when=move || mode.get() == "record" && record_estrogen.get() && estrogen_method.get() == "oral">
                            <label>
                                "Pill quantity"
                                <input
                                    type="text"
                                    min="1"
                                    step="any"
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
                                    prop:value=move || {
                                        if estrogen_method.get() == "injection" {
                                            injectable_type.get()
                                        } else {
                                            oral_type.get()
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
                                            view! { <option value=label.clone()>{label}</option> }
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
                                        on:input=move |ev| {
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
                                <input
                                    type="text"
                                    step="any"
                                    on:input=move |ev| estrogen_dose.set(event_target_value(&ev))
                                    prop:value=move || estrogen_dose.get()
                                />
                            </label>

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

                            <Show when=move || estrogen_method.get() != "injection">
                                <label>
                                    "Unit"
                                    <select
                                        on:change=move |ev| estrogen_unit.set(event_target_value(&ev))
                                        prop:value=move || estrogen_unit.get()
                                    >
                                        <For
                                            each=move || hormone_unit_labels()
                                            key=|label| label.clone()
                                            children=move |label| {
                                                view! { <option value=label.clone()>{label}</option> }
                                            }
                                        />
                                    </select>
                                </label>
                            </Show>
                        </div>
                    </section>

                    <section class="card dose-card">
                        <h3>"Antiandrogen"</h3>
                        <Show when=move || mode.get() == "record">
                            <label class="toggle toggle-wide">
                                <input
                                    type="checkbox"
                                    on:change=move |ev| record_aa.set(event_target_checked(&ev))
                                    prop:checked=move || record_aa.get()
                                />
                                <span class="toggle-track" aria-hidden="true"></span>
                                <span class="toggle-label">"Record Antiandrogen Dose"</span>
                            </label>
                            <Show when=move || record_aa.get()>
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
                                <select
                                    on:change=move |ev| aa_type.set(event_target_value(&ev))
                                    prop:value=move || aa_type.get()
                                >
                                    <option value="">"None"</option>
                                    <For
                                        each=move || {
                                            ANTIANDROGEN_OPTIONS
                                                .iter()
                                                .map(|kind| antiandrogen_label(kind).to_string())
                                                .collect::<Vec<_>>()
                                        }
                                        key=|label| label.clone()
                                        children=move |label| view! { <option value=label.clone()>{label}</option> }
                                    />
                                </select>
                            </label>

                            <Show when=move || mode.get() == "schedule" && !aa_type.get().is_empty()>
                                <label>
                                    "Frequency (days)"
                                    <input
                                        type="text"
                                        step="any"
                                        on:input=move |ev| aa_frequency.set(event_target_value(&ev))
                                        prop:value=move || aa_frequency.get()
                                    />
                                </label>
                            </Show>

                            <Show when=move || !aa_type.get().is_empty()>
                                <label>
                                    "Dose"
                                    <input
                                        type="text"
                                        step="any"
                                        on:input=move |ev| aa_dose.set(event_target_value(&ev))
                                        prop:value=move || aa_dose.get()
                                    />
                                </label>
                                <label>
                                    "Unit"
                                    <select
                                        on:change=move |ev| aa_unit.set(event_target_value(&ev))
                                        prop:value=move || aa_unit.get()
                                    >
                                        <For
                                            each=move || hormone_unit_labels()
                                            key=|label| label.clone()
                                            children=move |label| view! { <option value=label.clone()>{label}</option> }
                                        />
                                    </select>
                                </label>
                            </Show>
                        </div>
                    </section>

                    <section class="card dose-card">
                        <h3>"Progesterone"</h3>
                        <Show when=move || mode.get() == "record">
                            <label class="toggle toggle-wide">
                                <input
                                    type="checkbox"
                                    on:change=move |ev| record_prog.set(event_target_checked(&ev))
                                    prop:checked=move || record_prog.get()
                                />
                                <span class="toggle-track" aria-hidden="true"></span>
                                <span class="toggle-label">"Record Progesterone Dose"</span>
                            </label>
                            <Show when=move || record_prog.get()>
                                <label>
                                    "Note (optional)"
                                    <textarea
                                        rows="2"
                                        on:input=move |ev| prog_note.set(event_target_value(&ev))
                                        prop:value=move || prog_note.get()
                                    ></textarea>
                                </label>
                                <label>
                                    "Pill quantity"
                                    <input
                                        type="text"
                                        min="1"
                                        step="any"
                                        on:input=move |ev| prog_pill_qty.set(event_target_value(&ev))
                                        prop:value=move || prog_pill_qty.get()
                                    />
                                    <p class="muted">
                                        "Total = "
                                        <strong>{move || fmt(prog_pill_total.get(), 2)}</strong>
                                        " mg"
                                    </p>
                                </label>
                            </Show>
                        </Show>

                        <Show when=move || mode.get() == "schedule" && !prog_type.get().is_empty()>
                            <label>
                                "Next Dose Date"
                                <input
                                    type="datetime-local"
                                    on:input=move |ev| prog_next_date.set(event_target_value(&ev))
                                    prop:value=move || prog_next_date.get()
                                />
                            </label>
                        </Show>

                        <div class="calc-grid">
                            <label>
                                "Type"
                                <select
                                    on:change=move |ev| prog_type.set(event_target_value(&ev))
                                    prop:value=move || prog_type.get()
                                >
                                    <option value="">"None"</option>
                                    <For
                                        each=move || {
                                            PROGESTERONE_OPTIONS
                                                .iter()
                                                .map(|kind| progesterone_label(kind).to_string())
                                                .collect::<Vec<_>>()
                                        }
                                        key=|label| label.clone()
                                        children=move |label| view! { <option value=label.clone()>{label}</option> }
                                    />
                                </select>
                            </label>

                            <Show when=move || !prog_type.get().is_empty()>
                                <Show when=move || mode.get() == "schedule">
                                    <label>
                                        "Frequency (days)"
                                        <input
                                            type="text"
                                            step="any"
                                            on:input=move |ev| prog_frequency.set(event_target_value(&ev))
                                            prop:value=move || prog_frequency.get()
                                        />
                                    </label>
                                </Show>
                                <label>
                                    "Dose"
                                    <input
                                        type="text"
                                        step="any"
                                        on:input=move |ev| prog_dose.set(event_target_value(&ev))
                                        prop:value=move || prog_dose.get()
                                    />
                                </label>
                                <label>
                                    "Unit"
                                    <select
                                        on:change=move |ev| prog_unit.set(event_target_value(&ev))
                                        prop:value=move || prog_unit.get()
                                    >
                                        <For
                                            each=move || hormone_unit_labels()
                                            key=|label| label.clone()
                                            children=move |label| view! { <option value=label.clone()>{label}</option> }
                                        />
                                    </select>
                                </label>
                                <label>
                                    "Route"
                                    <select
                                        on:change=move |ev| prog_route.set(event_target_value(&ev))
                                        prop:value=move || prog_route.get()
                                    >
                                        <For
                                            each=move || {
                                                PROGESTERONE_ROUTE_OPTIONS
                                                    .iter()
                                                    .map(|route| progesterone_route_label(route).to_string())
                                                    .collect::<Vec<_>>()
                                            }
                                            key=|label| label.clone()
                                            children=move |label| view! { <option value=label.clone()>{label}</option> }
                                        />
                                    </select>
                                </label>
                            </Show>
                        </div>
                    </section>

                    <div class="form-actions">
                        <button type="submit">
                            {move || if mode.get() == "record" { "Record dosage" } else { "Save schedule" }}
                        </button>
                        <Show when=move || schedule_feedback.get()>
                            <p class="muted">"Schedule saved!"</p>
                        </Show>
                    </div>
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
