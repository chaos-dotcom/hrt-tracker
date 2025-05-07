import { setContext, getContext } from "svelte";
import { HRT_STORAGE_KEY, type HRTData } from "./types";
const defaultData: HRTData = {
  bloodTests: [],
  dosageHistory: [],
};

export function loadHRTData(): HRTData {
  // 1. try getting from context.
  // if not there, get from localstorage.
  // if not there, create default, save to localstorage and context
  let contextData: HRTData | undefined = undefined;
  try {
    contextData = getContext<HRTData>(HRT_STORAGE_KEY);
  } catch (e) {
    contextData = undefined;
  }
  if (contextData != null || contextData !== undefined) {
    return contextData;
  }
  console.log("creating data...");
  let data: HRTData;
  try {
    console.log("getting from local storage");
    const raw = localStorage.getItem(HRT_STORAGE_KEY);
    data = raw ? JSON.parse(raw) : defaultData;
  } catch {
    console.log("defaulting");
    data = defaultData;
  }
  console.log(JSON.stringify(data));
  localStorage.setItem(HRT_STORAGE_KEY, JSON.stringify(data));
  setContext(HRT_STORAGE_KEY, data);
  return data;
}

export function saveHRTData(data: HRTData): void {
  try {
    setContext(HRT_STORAGE_KEY, data);
    localStorage.setItem(HRT_STORAGE_KEY, JSON.stringify(data));
  } catch (e) {
    console.error("error saving data: ", e);
  }
}

export function updateHRTData(updaterFunction: (data: HRTData) => void): void {
  const currentData = loadHRTData();
  updaterFunction(currentData);
  saveHRTData(currentData);
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
    console.log("meow");
    data.dosageHistory.push(record);
  });
}
