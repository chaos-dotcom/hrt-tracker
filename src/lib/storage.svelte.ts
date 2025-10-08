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
  constructor() {
    // 3) on first load in the browser, hydrate from localStorage
    $effect.root(() => {
      // if (!browser) return;
      const raw = localStorage.getItem(HRT_STORAGE_KEY);
      this.data = raw ? JSON.parse(raw) : defaultData;
      // ^^ hrtData is still undefined bc it's in the class, temporal dead zone. avoid referring to it
      // use this.data instead
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
