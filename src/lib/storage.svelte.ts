import { browser } from "$app/environment";
import type { HRTData, DosageHistoryEntry, BloodTest, Measurement, UnixTime, DiaryEntry } from "./types";

const defaultData: HRTData = {
  // injectableEstradiol: undefined,
  // oralEstradiol: undefined,
  // antiandrogen: undefined,
  bloodTests: [],
  dosageHistory: [],
  measurements: [],
  notes: [],
  settings: {
    enableAutoBackfill: true,
    defaultInjectionFrequencyDays: 7,
    defaultOralFrequencyDays: 1,
    defaultAntiandrogenFrequencyDays: 1,
    defaultProgesteroneFrequencyDays: 1,
  },
};

class hrtStore {
  data = $state({ ...defaultData });
  #initialized = false;
  #debounceTimeout: ReturnType<typeof setTimeout> | undefined;

  init(initialData: HRTData) {
    if (this.#initialized || !browser) return;
    this.data = initialData ? { ...defaultData, ...initialData } : { ...defaultData };

    // Load settings from server-side YAML if available
    ;(async () => {
      try {
        const resp = await fetch('/api/settings');
        if (resp.ok) {
          const s = await resp.json();
          if (s && typeof s === 'object') {
            this.data.settings = { ...this.data.settings, ...s };
          }
        }
      } catch (err) {
        console.warn('Unable to load settings from /api/settings, using defaults:', err);
      }
    })();

    // One-time migration of localStorage notes ("hrt.notes") to centralized store
    try {
      if (browser) {
        const raw = localStorage.getItem("hrt.notes");
        if (raw && (!this.data.notes || this.data.notes.length === 0)) {
          const parsed = JSON.parse(raw);
          if (Array.isArray(parsed)) {
            const normalized: DiaryEntry[] = parsed
              .filter((n: any) => n && typeof n.content === "string")
              .map((n: any) => ({
                id:
                  typeof n.id === "string" && n.id
                    ? n.id
                    : (globalThis.crypto?.randomUUID?.() ?? String(n.date ?? Date.now())),
                date:
                  typeof n.date === "number"
                    ? n.date
                    : new Date(n.date || Date.now()).getTime(),
                title: typeof n.title === "string" ? n.title : "",
                content: n.content,
              }));
            if (normalized.length) {
              this.data.notes = normalized;
              localStorage.removeItem("hrt.notes");
            }
          }
        }
      }
    } catch {
      // ignore migration errors
    }

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
          // Persist settings YAML
          if (this.data?.settings) {
            await fetch('/api/settings', {
              method: 'POST',
              headers: {
                'Content-Type': 'application/json',
              },
              body: JSON.stringify(this.data.settings),
            });
          }
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
    const s = this.data.settings;
    if (s && s.enableAutoBackfill === false) return;

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
        if (!schedule || !schedule.nextDoseDate) return;

        const intervalDays =
          Number(schedule.frequency) > 0
            ? schedule.frequency
            : (medicationType === 'injectableEstradiol'
                ? (this.data.settings?.defaultInjectionFrequencyDays ?? 7)
                : medicationType === 'oralEstradiol'
                ? (this.data.settings?.defaultOralFrequencyDays ?? 1)
                : medicationType === 'antiandrogen'
                ? (this.data.settings?.defaultAntiandrogenFrequencyDays ?? 1)
                : (this.data.settings?.defaultProgesteroneFrequencyDays ?? 1));

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
