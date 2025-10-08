import { browser } from "$app/environment";
import type { HRTData, DosageHistoryEntry, BloodTest, Measurement, UnixTime } from "./types";

const defaultData: HRTData = {
  // injectableEstradiol: undefined,
  // oralEstradiol: undefined,
  // antiandrogen: undefined,
  bloodTests: [],
  dosageHistory: [],
  measurements: [],
};

class hrtStore {
  data = $state({ ...defaultData });
  #initialized = false;
  #debounceTimeout: ReturnType<typeof setTimeout> | undefined;

  init(initialData: HRTData) {
    if (this.#initialized || !browser) return;
    this.data = initialData ? { ...defaultData, ...initialData } : { ...defaultData };
    this.backfillScheduledDoses();
    this.#initialized = true;

    $effect(() => {
      // This is a dependency on data.
      const dataToSave = JSON.stringify(this.data);

      if (this.#debounceTimeout) {
        clearTimeout(this.#debounceTimeout);
      }

      this.#debounceTimeout = setTimeout(async () => {
        try {
          await fetch('/api/data', {
            method: 'POST',
            headers: {
              'Content-Type': 'application/json',
            },
            body: dataToSave,
          });
        } catch (error) {
          console.error('Failed to save data:', error);
        }
      }, 500);
    });
  }

  addBloodTest(test: BloodTest) {
    this.data.bloodTests.push(test);
  }

  addDosageRecord(rec: DosageHistoryEntry) {
    this.data.dosageHistory.push(rec);
  }

  deleteBloodTest(test: BloodTest) {
    this.data.bloodTests = this.data.bloodTests.filter((t) => t !== test);
  }

  deleteDosageRecord(rec: DosageHistoryEntry) {
    this.data.dosageHistory = this.data.dosageHistory.filter((r) => r !== rec);
  }

  addMeasurement(measurement: Measurement) {
    this.data.measurements.push(measurement);
  }

  deleteMeasurement(measurement: Measurement) {
    this.data.measurements = this.data.measurements.filter((m) => m !== measurement);
  }

  backfillScheduledDoses() {
    const now = Date.now();

    const createDoseEntry = (
        medicationType: DosageHistoryEntry['medicationType'],
        schedule: any,
        date: number
    ): DosageHistoryEntry => {
        const base = {
            date,
            type: schedule.type,
            dose: schedule.dose,
            unit: schedule.unit,
        };
        switch (medicationType) {
            case 'injectableEstradiol':
                return { ...base, medicationType: 'injectableEstradiol' };
            case 'oralEstradiol':
                return { ...base, medicationType: 'oralEstradiol' };
            case 'antiandrogen':
                return { ...base, medicationType: 'antiandrogen' };
            case 'progesterone':
                return { ...base, medicationType: 'progesterone', route: schedule.route };
        }
    };

    const processSchedule = (
        schedule: { frequency: number; nextDoseDate?: UnixTime; [key: string]: any } | undefined,
        medicationType: DosageHistoryEntry['medicationType']
    ) => {
        if (!schedule || !schedule.frequency || !schedule.nextDoseDate) return;

        const intervalDays = schedule.frequency;

        const intervalMillis = intervalDays * 24 * 60 * 60 * 1000;
        if (intervalMillis <= 0) return;

        let nextDoseTime = schedule.nextDoseDate;

        while (nextDoseTime <= now) {
            const doseExists = this.data.dosageHistory.some(d => d.medicationType === medicationType && d.date === nextDoseTime);
            if (!doseExists) {
                const newDose = createDoseEntry(medicationType, schedule, nextDoseTime);
                this.data.dosageHistory.push(newDose);
            }
            nextDoseTime += intervalMillis;
        }
        
        schedule.nextDoseDate = nextDoseTime;
    };

    processSchedule(this.data.injectableEstradiol, "injectableEstradiol");
    processSchedule(this.data.oralEstradiol, "oralEstradiol");
    processSchedule(this.data.antiandrogen, "antiandrogen");
    processSchedule(this.data.progesterone, "progesterone");
  }
  constructor() {
    // $effect has been moved to init() to avoid effect_orphan error.
  }
}
export const hrtData = new hrtStore();
