use chrono::{Local, TimeZone, Timelike};

use crate::types::{DosageHistoryEntry, HormoneUnits, HrtData, ProgesteroneSchedule, UnixTime};

const DAY_MS: i64 = 24 * 60 * 60 * 1000;

pub fn migrate_blood_tests_fudge_factor(data: &mut HrtData) -> bool {
    let mut migrated = false;
    for test in &mut data.bloodTests {
        if test.fudgeFactor.is_some() {
            continue;
        }
        let (estradiol_level, estrannaise) = match (test.estradiolLevel, test.estrannaiseNumber) {
            (Some(level), Some(number)) if number > 0.0 => (level, number),
            _ => continue,
        };
        let measured = match test.estradiolUnit {
            Some(HormoneUnits::E2PmolL) => estradiol_level / 3.671,
            _ => estradiol_level,
        };
        if measured.is_finite() {
            test.fudgeFactor = Some((measured / estrannaise * 1000.0).round() / 1000.0);
            migrated = true;
        }
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
