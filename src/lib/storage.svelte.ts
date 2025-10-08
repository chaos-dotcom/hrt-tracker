import { browser } from "$app/environment";
import type { HRTData, DosageHistoryEntry, BloodTest } from "./types";
import { HRT_STORAGE_KEY } from "./types";

const defaultData: HRTData = {
  // injectableEstradiol: undefined,
  // oralEstradiol: undefined,
  // antiandrogen: undefined,
  bloodTests: [],
  dosageHistory: [],
};

class hrtStore {
  data = $state({ ...defaultData });

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
        schedule: { frequency: string; [key: string]: any } | undefined,
        medicationType: DosageHistoryEntry['medicationType']
    ) => {
        if (!schedule || !schedule.frequency) return;

        let intervalDays: number;
        if (schedule.frequency.toLowerCase().trim() === "daily") {
            intervalDays = 1;
        } else {
            const match = schedule.frequency.match(/\d+/);
            if (match) {
                intervalDays = parseInt(match[0], 10);
            } else {
                console.warn(`Could not parse frequency: "${schedule.frequency}"`);
                return;
            }
        }

        const intervalMillis = intervalDays * 24 * 60 * 60 * 1000;
        if (intervalMillis <= 0) return;

        const relevantDoses = this.data.dosageHistory.filter(d => d.medicationType === medicationType);
        if (relevantDoses.length === 0) return;

        const lastDose = relevantDoses.sort((a, b) => b.date - a.date)[0];

        let nextDoseTime = lastDose.date;

        while (nextDoseTime + intervalMillis <= now) {
            nextDoseTime += intervalMillis;
            const newDose = createDoseEntry(medicationType, schedule, nextDoseTime);
            this.data.dosageHistory.push(newDose);
        }
    };

    processSchedule(this.data.injectableEstradiol, "injectableEstradiol");
    processSchedule(this.data.oralEstradiol, "oralEstradiol");
    processSchedule(this.data.antiandrogen, "antiandrogen");
    processSchedule(this.data.progesterone, "progesterone");
  }
  constructor() {
    // 3) on first load in the browser, hydrate from localStorage
    $effect.root(() => {
      // if (!browser) return;
      const raw = localStorage.getItem(HRT_STORAGE_KEY);
      this.data = raw ? JSON.parse(raw) : defaultData;
      // ^^ hrtData is still undefined bc it's in the class, temporal dead zone. avoid referring to it
      // use this.data instead
      if (browser) {
        this.backfillScheduledDoses();
      }
    });

    $effect.root(() => {
      $effect(() => {
        // if (!browser) return;
        localStorage.setItem(HRT_STORAGE_KEY, JSON.stringify(this.data));
      });
    });
  }
}
export const hrtData = new hrtStore();
