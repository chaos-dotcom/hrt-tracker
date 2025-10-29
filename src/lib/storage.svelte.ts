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
    icsSecret: '',
    enableBloodTestSchedule: false,
    bloodTestIntervalMonths: 3,
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
      // Autosave disabled to avoid dev server refresh loops. Use hrtData.saveNow() to persist explicitly.
      this.data; // depend to keep Svelte happy, but do nothing
    });
  }

  addBloodTest(test: BloodTest) {
    if (typeof test?.date === 'number' && isFinite(test.date)) {
      try {
        test.date = this.snapToNextInjectionBoundary(test.date);
      } catch {
        // If snapping fails, keep original date
      }
    }
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

  // Snap a timestamp to the next scheduled injectable estradiol boundary (trough day)
  // If an injectable schedule exists, uses its frequency; otherwise derives from the last two injections in history.
  // Returns the original timestamp if no cadence can be determined.
  snapToNextInjectionBoundary(ts: number): number {
    const DAY_MS = 24 * 60 * 60 * 1000;
    const inj = this.data.injectableEstradiol;

    // Determine interval in ms
    let stepMs: number | undefined;
    if (inj && typeof inj.frequency === 'number' && inj.frequency > 0) {
      stepMs = inj.frequency * DAY_MS;
    } else {
      // Derive from history if possible (use the last interval)
      const hist = (this.data.dosageHistory ?? [])
        .filter(
          (d) =>
            d &&
            d.medicationType === 'injectableEstradiol' &&
            typeof d.date === 'number' &&
            isFinite(d.date)
        )
        .sort((a, b) => a.date - b.date);
      if (hist.length >= 2) {
        const last = hist[hist.length - 1].date;
        const prev = hist[hist.length - 2].date;
        const gap = last - prev;
        if (gap > 0) stepMs = gap;
      }
    }
    if (!stepMs) return ts;

    // Choose a reference boundary R on the injection grid
    let reference: number | undefined;
    const lastTakenDates = (this.data.dosageHistory ?? [])
      .filter(
        (d) =>
          d &&
          d.medicationType === 'injectableEstradiol' &&
          typeof d.date === 'number' &&
          isFinite(d.date)
      )
      .map((d) => d.date);
    if (lastTakenDates.length > 0) {
      reference = Math.max(...lastTakenDates);
    } else if (inj && typeof inj.nextDoseDate === 'number' && isFinite(inj.nextDoseDate)) {
      reference = inj.nextDoseDate;
    }
    if (reference === undefined) return ts;

    // Compute the smallest boundary >= ts (ceil towards the next injection day)
    const n = Math.ceil((ts - reference) / stepMs);
    const target = reference + n * stepMs;
    // Always set to morning of that day (10:00 local time)
    const d = new Date(target);
    d.setHours(10, 0, 0, 0);
    return d.getTime();
  }

  async saveNow() {
    try {
      // Do not persist settings in JSON; store them only in YAML via /api/settings
      const { settings: _settings, ...dataWithoutSettings } = this.data as any;
      const dataToSave = JSON.stringify(dataWithoutSettings);

      await fetch('/api/data', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: dataToSave,
      });

      if (this.data?.settings) {
        await fetch('/api/settings', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(this.data.settings),
        });
      }
      return true;
    } catch (error) {
      console.error('Failed to save data:', error);
      return false;
    }
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
        if (!schedule) return;

        const intervalDays = Number(schedule.frequency) > 0 ? schedule.frequency : undefined;
        if (!intervalDays) return;

        const intervalMillis = intervalDays * 24 * 60 * 60 * 1000;
        if (intervalMillis <= 0) return;

        // Derive next dose from either configured nextDoseDate or last recorded dose + frequency
        let nextDoseTime: number | undefined = schedule.nextDoseDate;

        const lastTakenDates = (this.data.dosageHistory ?? [])
            .filter((d) => d && d.medicationType === medicationType && typeof d.date === 'number' && isFinite(d.date))
            .map((d) => d.date);

        if (lastTakenDates.length > 0) {
            const lastTaken = Math.max(...lastTakenDates);
            const nextAfterLast = lastTaken + intervalMillis;
            if (!Number.isFinite(nextDoseTime as number) || (nextDoseTime as number) < nextAfterLast) {
                nextDoseTime = nextAfterLast;
            }
        }

        if (!Number.isFinite(nextDoseTime as number)) return;

        // Advance to at least today (ignore time-of-day so "today" is kept)
        const today = new Date();
        today.setHours(0, 0, 0, 0);
        const todayStartMs = today.getTime();

        const getDayStart = (ms: number) => {
            const d = new Date(ms);
            d.setHours(0, 0, 0, 0);
            return d.getTime();
        };

        while (getDayStart(nextDoseTime as number) < todayStartMs) {
            nextDoseTime = (nextDoseTime as number) + intervalMillis;
        }

        schedule.nextDoseDate = nextDoseTime as number;
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
