use js_sys::Date;

use super::types::{NextDoseCandidate, RegimenKey, DAY_MS};
use crate::utils::format_injectable_dose;
use hrt_shared::types::{DosageHistoryEntry, HrtData, InjectableEstradiols};

fn injectable_model_id(kind: &InjectableEstradiols) -> Option<i64> {
    match kind {
        InjectableEstradiols::Benzoate => Some(0),
        InjectableEstradiols::Valerate => Some(1),
        InjectableEstradiols::Enanthate => Some(2),
        InjectableEstradiols::Cypionate => Some(3),
        InjectableEstradiols::Undecylate => Some(4),
        InjectableEstradiols::PolyestradiolPhosphate => None,
    }
}

fn get_last_dose_date_for_type(data: &HrtData, med_type: RegimenKey) -> Option<i64> {
    let mut dates = Vec::new();
    for entry in &data.dosageHistory {
        match (med_type, entry) {
            (
                RegimenKey::InjectableEstradiol,
                DosageHistoryEntry::InjectableEstradiol {
                    date, bonusDose, ..
                },
            ) => {
                if !bonusDose.unwrap_or(false) {
                    dates.push(*date);
                }
            }
            (RegimenKey::OralEstradiol, DosageHistoryEntry::OralEstradiol { date, .. })
            | (RegimenKey::Antiandrogen, DosageHistoryEntry::Antiandrogen { date, .. })
            | (RegimenKey::Progesterone, DosageHistoryEntry::Progesterone { date, .. }) => {
                dates.push(*date);
            }
            _ => {}
        }
    }
    dates.into_iter().max()
}

fn get_next_scheduled_date_for(data: &HrtData, med_type: RegimenKey) -> Option<i64> {
    let now = Date::now() as i64;
    match med_type {
        RegimenKey::InjectableEstradiol => {
            let cfg = data.injectableEstradiol.as_ref()?;
            let last = get_last_dose_date_for_type(data, RegimenKey::InjectableEstradiol);
            if let (Some(next), Some(last)) = (cfg.nextDoseDate, last) {
                if next > last {
                    return Some(next);
                }
            }
            if let Some(last) = last {
                return Some(last + (cfg.frequency * DAY_MS as f64) as i64);
            }
            Some(now)
        }
        RegimenKey::OralEstradiol => {
            let cfg = data.oralEstradiol.as_ref()?;
            let last = get_last_dose_date_for_type(data, RegimenKey::OralEstradiol);
            if let Some(last) = last {
                return Some(last + (cfg.frequency * DAY_MS as f64) as i64);
            }
            Some(now)
        }
        RegimenKey::Antiandrogen => {
            let cfg = data.antiandrogen.as_ref()?;
            let last = get_last_dose_date_for_type(data, RegimenKey::Antiandrogen);
            if let Some(last) = last {
                return Some(last + (cfg.frequency * DAY_MS as f64) as i64);
            }
            Some(now)
        }
        RegimenKey::Progesterone => {
            let cfg = data.progesterone.as_ref()?;
            let last = get_last_dose_date_for_type(data, RegimenKey::Progesterone);
            if let Some(last) = last {
                return Some(last + (cfg.frequency * DAY_MS as f64) as i64);
            }
            Some(now)
        }
    }
}

pub(super) fn get_next_scheduled_candidate(
    data: &HrtData,
    use_iu: bool,
) -> Option<NextDoseCandidate> {
    let mut options: Vec<(RegimenKey, i64, String)> = Vec::new();
    if let Some(cfg) = data.injectableEstradiol.as_ref() {
        if let Some(date) = get_next_scheduled_date_for(data, RegimenKey::InjectableEstradiol) {
            let dose_label = format_injectable_dose(
                data,
                cfg.dose,
                &cfg.unit,
                cfg.vialId.as_ref(),
                cfg.vialId.as_ref(),
                use_iu,
            );
            options.push((
                RegimenKey::InjectableEstradiol,
                date,
                format!("Injection: {:?}, {dose_label}", cfg.kind),
            ));
        }
    }
    if let Some(cfg) = data.oralEstradiol.as_ref() {
        if let Some(date) = get_next_scheduled_date_for(data, RegimenKey::OralEstradiol) {
            options.push((
                RegimenKey::OralEstradiol,
                date,
                format!(
                    "Oral Estradiol: {:?}, {:.2} {:?}",
                    cfg.kind, cfg.dose, cfg.unit
                ),
            ));
        }
    }
    if let Some(cfg) = data.antiandrogen.as_ref() {
        if let Some(date) = get_next_scheduled_date_for(data, RegimenKey::Antiandrogen) {
            options.push((
                RegimenKey::Antiandrogen,
                date,
                format!(
                    "Antiandrogen: {:?}, {:.2} {:?}",
                    cfg.kind, cfg.dose, cfg.unit
                ),
            ));
        }
    }
    if let Some(cfg) = data.progesterone.as_ref() {
        if let Some(date) = get_next_scheduled_date_for(data, RegimenKey::Progesterone) {
            options.push((
                RegimenKey::Progesterone,
                date,
                format!(
                    "Progesterone ({:?}): {:?}, {:.2} {:?}",
                    cfg.route, cfg.kind, cfg.dose, cfg.unit
                ),
            ));
        }
    }
    if options.is_empty() {
        return None;
    }
    let now = Date::now() as i64;
    let mut future: Vec<_> = options
        .iter()
        .cloned()
        .filter(|(_, date, _)| *date >= now)
        .collect();
    if !future.is_empty() {
        future.sort_by_key(|(_, date, _)| *date);
        let (med_type, _, label) = future[0].clone();
        return Some(NextDoseCandidate { med_type, label });
    }
    options.sort_by(|a, b| b.1.cmp(&a.1));
    let (med_type, _, label) = options[0].clone();
    Some(NextDoseCandidate { med_type, label })
}

pub(super) fn generate_estrannaise_url(
    data: &HrtData,
    fudge_factor: Option<f64>,
) -> Option<String> {
    let regimen = data.injectableEstradiol.as_ref();
    let mut historical: Vec<(i64, InjectableEstradiols, f64)> = data
        .dosageHistory
        .iter()
        .filter_map(|entry| match entry {
            DosageHistoryEntry::InjectableEstradiol {
                date, kind, dose, ..
            } => Some((*date, kind.clone(), *dose)),
            _ => None,
        })
        .collect();
    historical.sort_by_key(|(date, _, _)| *date);

    if historical.is_empty() && regimen.is_none() {
        return None;
    }

    let mut all_doses = historical.clone();
    let mut last_dose_date = Date::now() as i64;
    let mut total_duration_days = 0.0;

    if !historical.is_empty() {
        let first_date = historical
            .first()
            .map(|(date, _, _)| *date)
            .unwrap_or(last_dose_date);
        last_dose_date = historical
            .last()
            .map(|(date, _, _)| *date)
            .unwrap_or(last_dose_date);
        total_duration_days = (last_dose_date - first_date) as f64 / DAY_MS as f64;
    } else if let Some(reg) = regimen {
        all_doses.push((last_dose_date, reg.kind.clone(), reg.dose));
    }

    if let Some(reg) = regimen {
        if total_duration_days < 80.0 {
            let frequency_ms = (reg.frequency * DAY_MS as f64) as i64;
            let mut next_dose = last_dose_date + frequency_ms;
            while total_duration_days < 80.0 {
                all_doses.push((next_dose, reg.kind.clone(), reg.dose));
                let first_date = all_doses
                    .first()
                    .map(|(date, _, _)| *date)
                    .unwrap_or(next_dose);
                total_duration_days = (next_dose - first_date) as f64 / DAY_MS as f64;
                next_dose += frequency_ms;
            }
        }
    }

    if all_doses.is_empty() {
        return None;
    }

    let mut dose_strings = Vec::new();
    let mut last_date_for_interval: Option<i64> = None;
    for (date, kind, dose) in all_doses {
        if let Some(model_id) = injectable_model_id(&kind) {
            let time_days = if let Some(last) = last_date_for_interval {
                (date - last) as f64 / DAY_MS as f64
            } else {
                0.0
            };
            last_date_for_interval = Some(date);
            dose_strings.push(format!("{},{:.3},{}", dose, time_days, model_id));
        }
    }

    if dose_strings.is_empty() {
        return None;
    }

    let custom = dose_strings
        .iter()
        .enumerate()
        .map(|(idx, dose_string)| {
            if idx == 0 {
                format!("cu,{dose_string}")
            } else {
                dose_string.clone()
            }
        })
        .collect::<Vec<_>>()
        .join("-");

    let suffix = fudge_factor
        .filter(|value| value.is_finite())
        .map(|value| format!("__{value}"))
        .unwrap_or_else(|| "_".to_string());

    Some(format!("https://estrannai.se/#i_{custom}{suffix}"))
}
