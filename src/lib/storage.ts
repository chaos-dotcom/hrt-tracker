import { setContext, getContext } from "svelte";
import { HRT_STORAGE_KEY, type HRTData } from "./types";
const defaultData: HRTData = {
  bloodTests: [],
  dosageHistory: [],
};

function loadFromStorage(): HRTData {
  try {
    const shared = localStorage.getItem(HRT_STORAGE_KEY);
    if (shared !== null) {
      setContext(HRT_STORAGE_KEY, shared);
      return JSON.parse(shared);
    }
    setContext(HRT_STORAGE_KEY, defaultData);
    return defaultData;
  } catch (e) {
    console.error("error loading from localstorage: ", e);
    setContext(HRT_STORAGE_KEY, defaultData);
    return defaultData;
  }
}

function loadFromContext(): HRTData {
  try {
    return getContext<HRTData>(HRT_STORAGE_KEY);
  } catch (e) {
    console.error("error getting from context: ", e);
  }
  return defaultData;
}

function saveToStorage(data: HRTData): void {
  try {
    setContext(HRT_STORAGE_KEY, data);
    localStorage.setItem(HRT_STORAGE_KEY, JSON.stringify(data));
  } catch (e) {
    console.error("error saving data: ", e);
  }
}

export function updateHRTData(updaterFunction: (data: HRTData) => void): void {
  const currentData = loadFromContext();
  updaterFunction(currentData);
  saveToStorage(currentData);
}

export function addBloodTest(test: HRTData["bloodTests"][0]): void {
  updateHRTData((data) => {
    if (!data.bloodTests) data.bloodTests = [];
    data.bloodTests.push(test);
  });
}

export function addDosageRecord(record: HRTData["dosageHistory"][0]): void {
  updateHRTData((data) => {
    if (!data.dosageHistory) data.dosageHistory = [];
    data.dosageHistory.push(record);
  });
}

export function initHRTStorage(): HRTData {
  return loadFromStorage();
}
