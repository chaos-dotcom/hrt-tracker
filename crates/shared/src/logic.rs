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
            .or(test.estrannaiseNumber);
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
            DosageHistoryEntry::InjectableEstradiol {
                date, bonusDose, ..
            } => {
                if bonusDose.unwrap_or(false) {
                    None
                } else {
                    Some(*date)
                }
            }
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
        settings.map(|s| s.enableAutoBackfill),
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
                DosageHistoryEntry::InjectableEstradiol {
                    date, bonusDose, ..
                } if medication_type == "injectableEstradiol" => {
                    if bonusDose.unwrap_or(false) {
                        None
                    } else {
                        Some(*date)
                    }
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
            if next_time.is_none_or(|t| t < next_after_last) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    fn make_injectable_entry(date: i64, kind: InjectableEstradiols, dose: f64) -> DosageHistoryEntry {
        DosageHistoryEntry::InjectableEstradiol {
            date,
            id: None,
            kind,
            dose,
            unit: HormoneUnits::Mg,
            note: None,
            bonusDose: None,
            injectionSite: None,
            vialId: None,
            subVialId: None,
            syringeKind: None,
            needleLength: None,
            needleGauge: None,
            photos: None,
        }
    }

    fn make_oral_entry(date: i64) -> DosageHistoryEntry {
        DosageHistoryEntry::OralEstradiol {
            date,
            id: None,
            kind: OralEstradiols::Hemihydrate,
            dose: 2.0,
            unit: HormoneUnits::Mg,
            pillQuantity: None,
            note: None,
        }
    }

    #[test]
    fn predict_e2_returns_none_for_empty_history() {
        let data = HrtData::default();
        assert!(predict_e2_pg_ml(&data, 1700000000000).is_none());
    }

    #[test]
    fn predict_e2_returns_none_for_oral_only() {
        let mut data = HrtData::default();
        data.dosageHistory.push(make_oral_entry(1700000000000));
        assert!(predict_e2_pg_ml(&data, 1700100000000).is_none());
    }

    #[test]
    fn predict_e2_returns_none_before_first_dose() {
        let mut data = HrtData::default();
        data.dosageHistory.push(make_injectable_entry(
            1700000000000,
            InjectableEstradiols::Valerate,
            4.0,
        ));
        // Query before the dose
        assert!(predict_e2_pg_ml(&data, 1699999000000).is_none());
    }

    #[test]
    fn predict_e2_returns_some_after_dose() {
        let mut data = HrtData::default();
        let dose_time = 1700000000000_i64;
        data.dosageHistory.push(make_injectable_entry(
            dose_time,
            InjectableEstradiols::Valerate,
            4.0,
        ));
        // 3 days later
        let result = predict_e2_pg_ml(&data, dose_time + 3 * DAY_MS);
        assert!(result.is_some(), "should predict after dose");
        assert!(result.unwrap() > 0.0, "prediction should be positive");
    }

    #[test]
    fn predict_e2_returns_none_for_polyestradiol_phosphate_only() {
        let mut data = HrtData::default();
        data.dosageHistory.push(make_injectable_entry(
            1700000000000,
            InjectableEstradiols::PolyestradiolPhosphate,
            40.0,
        ));
        // PEP has no estrannaise model
        assert!(predict_e2_pg_ml(&data, 1700100000000).is_none());
    }

    #[test]
    fn predict_e2_higher_dose_gives_higher_level() {
        let dose_time = 1700000000000_i64;
        let query_time = dose_time + 3 * DAY_MS;

        let mut data_low = HrtData::default();
        data_low.dosageHistory.push(make_injectable_entry(
            dose_time,
            InjectableEstradiols::Valerate,
            2.0,
        ));

        let mut data_high = HrtData::default();
        data_high.dosageHistory.push(make_injectable_entry(
            dose_time,
            InjectableEstradiols::Valerate,
            8.0,
        ));

        let low = predict_e2_pg_ml(&data_low, query_time).unwrap();
        let high = predict_e2_pg_ml(&data_high, query_time).unwrap();
        assert!(high > low, "higher dose should give higher level: {high} vs {low}");
    }

    #[test]
    fn migrate_fudge_factor_returns_false_for_no_blood_tests() {
        let mut data = HrtData::default();
        assert!(!migrate_blood_tests_fudge_factor(&mut data));
    }

    #[test]
    fn migrate_fudge_factor_skips_existing() {
        let mut data = HrtData::default();
        data.dosageHistory.push(make_injectable_entry(
            1700000000000,
            InjectableEstradiols::Valerate,
            4.0,
        ));
        data.bloodTests.push(BloodTest {
            date: 1700000000000 + 3 * DAY_MS,
            estradiolLevel: Some(200.0),
            estradiolUnit: Some(HormoneUnits::E2PgMl),
            fudgeFactor: Some(1.5),
            testLevel: None,
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
            notes: None,
            estrogenType: None,
            pdfFiles: None,
        });
        // Already has fudge factor, should not migrate
        assert!(!migrate_blood_tests_fudge_factor(&mut data));
        assert_eq!(data.bloodTests[0].fudgeFactor, Some(1.5));
    }

    #[test]
    fn migrate_fudge_factor_sets_value() {
        let mut data = HrtData::default();
        let dose_time = 1700000000000_i64;
        data.dosageHistory.push(make_injectable_entry(
            dose_time,
            InjectableEstradiols::Valerate,
            4.0,
        ));
        data.bloodTests.push(BloodTest {
            date: dose_time + 3 * DAY_MS,
            estradiolLevel: Some(200.0),
            estradiolUnit: Some(HormoneUnits::E2PgMl),
            fudgeFactor: None,
            testLevel: None,
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
            notes: None,
            estrogenType: None,
            pdfFiles: None,
        });
        assert!(migrate_blood_tests_fudge_factor(&mut data));
        assert!(data.bloodTests[0].fudgeFactor.is_some());
        let ff = data.bloodTests[0].fudgeFactor.unwrap();
        assert!(ff > 0.0, "fudge factor should be positive: {ff}");
    }

    #[test]
    fn migrate_fudge_factor_handles_pmol_l_unit() {
        let mut data = HrtData::default();
        let dose_time = 1700000000000_i64;
        data.dosageHistory.push(make_injectable_entry(
            dose_time,
            InjectableEstradiols::Valerate,
            4.0,
        ));
        data.bloodTests.push(BloodTest {
            date: dose_time + 3 * DAY_MS,
            estradiolLevel: Some(734.26), // ~200 pg/mL in pmol/L
            estradiolUnit: Some(HormoneUnits::E2PmolL),
            fudgeFactor: None,
            testLevel: None,
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
            notes: None,
            estrogenType: None,
            pdfFiles: None,
        });
        assert!(migrate_blood_tests_fudge_factor(&mut data));
        let ff = data.bloodTests[0].fudgeFactor.unwrap();
        assert!(ff > 0.0, "fudge factor should be positive: {ff}");
    }

    #[test]
    fn snap_returns_ts_when_no_injectable() {
        let data = HrtData::default();
        let ts = 1700000000000_i64;
        assert_eq!(snap_to_next_injection_boundary(&data, ts), ts);
    }

    #[test]
    fn snap_returns_ts_when_zero_frequency() {
        let mut data = HrtData::default();
        data.injectableEstradiol = Some(InjectableSchedule {
            kind: InjectableEstradiols::Valerate,
            dose: 4.0,
            unit: HormoneUnits::Mg,
            frequency: 0.0,
            vialId: None,
            subVialId: None,
            syringeKind: None,
            needleLength: None,
            needleGauge: None,
            nextDoseDate: None,
        });
        let ts = 1700000000000_i64;
        assert_eq!(snap_to_next_injection_boundary(&data, ts), ts);
    }

    #[test]
    fn snap_advances_to_next_boundary() {
        let dose_time = 1700000000000_i64;
        let mut data = HrtData::default();
        data.injectableEstradiol = Some(InjectableSchedule {
            kind: InjectableEstradiols::Valerate,
            dose: 4.0,
            unit: HormoneUnits::Mg,
            frequency: 7.0,
            vialId: None,
            subVialId: None,
            syringeKind: None,
            needleLength: None,
            needleGauge: None,
            nextDoseDate: None,
        });
        data.dosageHistory.push(make_injectable_entry(
            dose_time,
            InjectableEstradiols::Valerate,
            4.0,
        ));

        // Query 1 day after dose - should snap to next boundary (dose_time + 7 days, at 10:00 AM)
        let result = snap_to_next_injection_boundary(&data, dose_time + DAY_MS);
        assert!(result > dose_time, "snapped time should be after dose: {result} vs {dose_time}");
        assert!(result >= dose_time + 7 * DAY_MS - DAY_MS, "should be near next boundary");
    }

    #[test]
    fn backfill_skips_when_disabled() {
        let mut data = HrtData::default();
        data.settings = Some(Settings {
            enableAutoBackfill: false,
            icsSecret: None,
            enableBloodTestSchedule: None,
            bloodTestIntervalMonths: None,
            statsBreakdownBySyringeKind: None,
            displayEstradiolUnit: None,
            displayInjectableInIU: None,
            braSizeSystem: None,
            pdfPassword: None,
        });
        data.injectableEstradiol = Some(InjectableSchedule {
            kind: InjectableEstradiols::Valerate,
            dose: 4.0,
            unit: HormoneUnits::Mg,
            frequency: 7.0,
            vialId: None,
            subVialId: None,
            syringeKind: None,
            needleLength: None,
            needleGauge: None,
            nextDoseDate: None,
        });
        backfill_scheduled_doses(&mut data);
        assert!(data.injectableEstradiol.as_ref().unwrap().nextDoseDate.is_none());
    }

    #[test]
    fn backfill_sets_next_dose_from_history() {
        let mut data = HrtData::default();
        let dose_time = chrono::Local::now().timestamp_millis() - 2 * DAY_MS; // 2 days ago
        data.injectableEstradiol = Some(InjectableSchedule {
            kind: InjectableEstradiols::Valerate,
            dose: 4.0,
            unit: HormoneUnits::Mg,
            frequency: 7.0,
            vialId: None,
            subVialId: None,
            syringeKind: None,
            needleLength: None,
            needleGauge: None,
            nextDoseDate: None,
        });
        data.dosageHistory.push(make_injectable_entry(
            dose_time,
            InjectableEstradiols::Valerate,
            4.0,
        ));
        backfill_scheduled_doses(&mut data);
        let next = data.injectableEstradiol.as_ref().unwrap().nextDoseDate;
        assert!(next.is_some(), "should have set nextDoseDate");
        let next = next.unwrap();
        // Should be dose_time + 7 days
        assert!(next > dose_time, "next dose should be after last dose");
    }
}
