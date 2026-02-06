use chrono::{Local, TimeZone, Timelike};

use crate::estrannaise::e2_multidose_3c;
use crate::types::{
    DosageHistoryEntry, EstrannaiseModel, HormoneUnits, HrtData, InjectableEstradiols,
    ProgesteroneSchedule, UnixTime,
};

const DAY_MS: i64 = 24 * 60 * 60 * 1000;

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

pub fn predict_e2_pg_ml(data: &HrtData, date: i64) -> Option<f64> {
    let mut dose_history: Vec<_> = data
        .dosageHistory
        .iter()
        .filter_map(|entry| match entry {
            DosageHistoryEntry::InjectableEstradiol {
                date, kind, dose, ..
            } => Some((*date, kind.clone(), *dose)),
            _ => None,
        })
        .collect();
    if dose_history.is_empty() {
        return None;
    }
    dose_history.sort_by_key(|(date, _, _)| *date);
    let start_date = dose_history.first().map(|(date, _, _)| *date)?;
    if date < start_date {
        return None;
    }
    let mut time_map = Vec::new();
    let mut dose_map = Vec::new();
    let mut model_map = Vec::new();
    for (dose_date, kind, dose) in dose_history {
        if let Some(model) = map_estrannaise_model(&kind) {
            time_map.push((dose_date - start_date) as f64 / DAY_MS as f64);
            dose_map.push(dose);
            model_map.push(model);
        }
    }
    if model_map.is_empty() {
        return None;
    }
    let t = (date - start_date) as f64 / DAY_MS as f64;
    let predicted = e2_multidose_3c(t, &dose_map, &time_map, &model_map, 1.0, false);
    if predicted.is_finite() && predicted > 0.0 {
        Some(predicted)
    } else {
        None
    }
}

pub fn migrate_blood_tests_fudge_factor(data: &mut HrtData) -> bool {
    if data.bloodTests.is_empty() {
        return false;
    }
    let predicted_values: Vec<Option<f64>> = data
        .bloodTests
        .iter()
        .map(|test| predict_e2_pg_ml(data, test.date))
        .collect();
    let mut migrated = false;
    for (idx, test) in data.bloodTests.iter_mut().enumerate() {
        if test.fudgeFactor.is_some() {
            continue;
        }
        let Some(estradiol_level) = test.estradiolLevel else {
            continue;
        };
        let measured = match test.estradiolUnit {
            Some(HormoneUnits::E2PmolL) => estradiol_level / 3.671,
            _ => estradiol_level,
        };
        if !measured.is_finite() {
            continue;
        }
        let predicted = predicted_values
            .get(idx)
            .and_then(|value| *value)
            .or_else(|| test.estrannaiseNumber);
        let fudge_factor = if let Some(predicted) = predicted.filter(|value| *value > 0.0) {
            let ratio = measured / predicted;
            if ratio.is_finite() {
                (ratio * 1000.0).round() / 1000.0
            } else {
                1.0
            }
        } else {
            1.0
        };
        test.fudgeFactor = Some(fudge_factor);
        migrated = true;
    }
    migrated
}

pub fn snap_to_next_injection_boundary(data: &HrtData, ts: UnixTime) -> UnixTime {
    let inj = match &data.injectableEstradiol {
        Some(inj) => inj,
        None => return ts,
    };

    let step_ms = if inj.frequency > 0.0 {
        (inj.frequency * DAY_MS as f64) as i64
    } else {
        return ts;
    };

    let mut reference: Option<i64> = None;
    let last_taken_dates: Vec<i64> = data
        .dosageHistory
        .iter()
        .filter_map(|d| match d {
            DosageHistoryEntry::InjectableEstradiol { date, .. } => Some(*date),
            _ => None,
        })
        .collect();

    if !last_taken_dates.is_empty() {
        reference = Some(*last_taken_dates.iter().max().unwrap());
    } else if let Some(next) = inj.nextDoseDate {
        reference = Some(next);
    }

    let reference = match reference {
        Some(value) => value,
        None => return ts,
    };

    let n = ((ts - reference) as f64 / step_ms as f64).ceil() as i64;
    let target = reference + n * step_ms;
    let base_dt = Local
        .timestamp_millis_opt(target)
        .single()
        .unwrap_or_else(|| Local.timestamp_millis_opt(ts).unwrap());
    let dt = base_dt
        .with_hour(10)
        .and_then(|d| d.with_minute(0))
        .and_then(|d| d.with_second(0))
        .and_then(|d| d.with_nanosecond(0))
        .unwrap_or(base_dt);
    dt.timestamp_millis()
}

pub fn backfill_scheduled_doses(data: &mut HrtData) {
    let settings = data.settings.as_ref();
    if matches!(
        settings.and_then(|s| Some(s.enableAutoBackfill)),
        Some(false)
    ) {
        return;
    }

    fn process_schedule<T>(
        schedule: &mut Option<T>,
        medication_type: &str,
        dosage_history: &[DosageHistoryEntry],
    ) where
        T: ScheduleFields,
    {
        let schedule = match schedule.as_mut() {
            Some(value) => value,
            None => return,
        };
        let interval_days = schedule.frequency();
        if interval_days <= 0.0 {
            return;
        }
        let interval_ms = (interval_days * DAY_MS as f64) as i64;
        if interval_ms <= 0 {
            return;
        }

        let mut next_time = schedule.next_dose_date();

        let last_taken_dates: Vec<i64> = dosage_history
            .iter()
            .filter_map(|d| match d {
                DosageHistoryEntry::InjectableEstradiol { date, .. }
                    if medication_type == "injectableEstradiol" =>
                {
                    Some(*date)
                }
                DosageHistoryEntry::OralEstradiol { date, .. }
                    if medication_type == "oralEstradiol" =>
                {
                    Some(*date)
                }
                DosageHistoryEntry::Antiandrogen { date, .. }
                    if medication_type == "antiandrogen" =>
                {
                    Some(*date)
                }
                DosageHistoryEntry::Progesterone { date, .. }
                    if medication_type == "progesterone" =>
                {
                    Some(*date)
                }
                _ => None,
            })
            .collect();

        if !last_taken_dates.is_empty() {
            let last_taken = *last_taken_dates.iter().max().unwrap();
            let next_after_last = last_taken + interval_ms;
            if next_time.map_or(true, |t| t < next_after_last) {
                next_time = Some(next_after_last);
            }
        }

        let mut next_time = match next_time {
            Some(value) => value,
            None => return,
        };

        let today_start = Local::now().date_naive().and_hms_opt(0, 0, 0).unwrap();
        let today_start_ms = Local
            .from_local_datetime(&today_start)
            .single()
            .map(|d| d.timestamp_millis())
            .unwrap_or_else(|| Local::now().timestamp_millis());

        while day_start_ms(next_time) < today_start_ms {
            next_time += interval_ms;
        }

        schedule.set_next_dose_date(next_time);
    }

    process_schedule(
        &mut data.injectableEstradiol,
        "injectableEstradiol",
        &data.dosageHistory,
    );
    process_schedule(
        &mut data.oralEstradiol,
        "oralEstradiol",
        &data.dosageHistory,
    );
    process_schedule(&mut data.antiandrogen, "antiandrogen", &data.dosageHistory);
    process_schedule(&mut data.progesterone, "progesterone", &data.dosageHistory);
}

fn day_start_ms(ms: i64) -> i64 {
    let dt = Local.timestamp_millis_opt(ms).single();
    let dt = match dt {
        Some(value) => value,
        None => return ms,
    };
    let start = dt
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .and_then(|d| Local.from_local_datetime(&d).single());
    start.map(|d| d.timestamp_millis()).unwrap_or(ms)
}

trait ScheduleFields {
    fn frequency(&self) -> f64;
    fn next_dose_date(&self) -> Option<i64>;
    fn set_next_dose_date(&mut self, ts: i64);
}

impl ScheduleFields for crate::types::InjectableSchedule {
    fn frequency(&self) -> f64 {
        self.frequency
    }
    fn next_dose_date(&self) -> Option<i64> {
        self.nextDoseDate
    }
    fn set_next_dose_date(&mut self, ts: i64) {
        self.nextDoseDate = Some(ts);
    }
}

impl ScheduleFields for crate::types::OralSchedule {
    fn frequency(&self) -> f64 {
        self.frequency
    }
    fn next_dose_date(&self) -> Option<i64> {
        self.nextDoseDate
    }
    fn set_next_dose_date(&mut self, ts: i64) {
        self.nextDoseDate = Some(ts);
    }
}

impl ScheduleFields for crate::types::AntiandrogenSchedule {
    fn frequency(&self) -> f64 {
        self.frequency
    }
    fn next_dose_date(&self) -> Option<i64> {
        self.nextDoseDate
    }
    fn set_next_dose_date(&mut self, ts: i64) {
        self.nextDoseDate = Some(ts);
    }
}

impl ScheduleFields for ProgesteroneSchedule {
    fn frequency(&self) -> f64 {
        self.frequency
    }
    fn next_dose_date(&self) -> Option<i64> {
        self.nextDoseDate
    }
    fn set_next_dose_date(&mut self, ts: i64) {
        self.nextDoseDate = Some(ts);
    }
}
