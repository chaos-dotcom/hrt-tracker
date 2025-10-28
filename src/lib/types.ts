// https://transfemscience.org/misc/injectable-e2-simulator/
// https://transfemscience.org/misc/
export enum InjectableEstradiols {
  Benzoate = "Estradiol Benzoate",
  Cypionate = "Estradiol Cypionate",
  Enanthate = "Estradiol Enanthate",
  Undecylate = "Estradiol Undecylate",
  Valerate = "Estradiol Valerate",
  PolyestradiolPhosphate = "Polyestradiol Phosphate",
}

export enum OralEstradiols {
  Hemihydrate = "Estradiol Hemihydrate",
  Valerate = "Estradiol Valerate",
  Premarin = "Premarin",
}

export enum Antiandrogens {
  CPA = "Cyproterone Acetate",
  Spiro = "Spironolactone",
  Bica = "Bicalutamide",
  Finasteride = "Finasteride",
}

export enum Progesterones {
  Micronized = "Micronized Progesterone",
}

export enum ProgesteroneRoutes {
  Oral = "Oral",
  Rectal = "Rectal",
}

export enum WeightUnit {
  KG = "kg",
  LBS = "lbs",
}

export enum LengthUnit {
  CM = "cm",
  IN = "in",
}

export enum HormoneUnits {
  E2_pg_mL = "pg/mL",
  E2_pmol_L = "pmol/L",
  T_ng_dL = "ng/dL",
  T_nmol_L = "nmol/L",
  mg = "mg",
  ng_mL = "ng/mL",
  mIU_mL = "mIU/mL",
  mIU_L = "mIU/L",
  U_L = "U/L",
}

export type UnixTime = number;

export enum InjectionSites {
  StomachRight = "Stomach right",
  StomachLeft = "Stomach left",
  ThighRight = "Thigh right",
  ThighLeft = "Thigh left",
  ButtockRight = "Buttock right",
  ButtockLeft = "Buttock left",
}

export type DosageHistoryEntry =
  | {
      date: UnixTime;
      medicationType: "injectableEstradiol";
      type: InjectableEstradiols;
      dose: number;
      unit: HormoneUnits;
      note?: string;
      injectionSite?: InjectionSites;
    }
  | {
      date: UnixTime;
      medicationType: "oralEstradiol";
      type: OralEstradiols;
      dose: number;
      unit: HormoneUnits;
      note?: string;
    }
  | {
      date: UnixTime;
      medicationType: "antiandrogen";
      type: Antiandrogens;
      dose: number;
      unit: HormoneUnits;
      note?: string;
    }
  | {
      date: UnixTime;
      medicationType: "progesterone";
      type: Progesterones;
      route: ProgesteroneRoutes;
      dose: number;
      unit: HormoneUnits;
      note?: string;
    };

export type EstrogenType =
  | { route: "injection"; type: InjectableEstradiols }
  | { route: "oral"; type: OralEstradiols };

export interface Measurement {
  date: UnixTime;
  weight?: number;
  weightUnit?: WeightUnit;
  height?: number;
  heightUnit?: LengthUnit;
  underbust?: number;
  bust?: number;
  bideltoid?: number;
  waist?: number;
  hip?: number;
  bodyMeasurementUnit?: LengthUnit;
  braSize?: string;
}

export interface BloodTest {
  date: UnixTime;
  estradiolLevel?: number;
  testLevel?: number;
  estradiolUnit?: HormoneUnits;
  testUnit?: HormoneUnits;
  progesteroneLevel?: number;
  progesteroneUnit?: HormoneUnits;
  fshLevel?: number;
  fshUnit?: HormoneUnits;
  lhLevel?: number;
  lhUnit?: HormoneUnits;
  prolactinLevel?: number;
  prolactinUnit?: HormoneUnits;
  shbgLevel?: number;
  shbgUnit?: HormoneUnits;
  freeAndrogenIndex?: number;
  estrannaiseNumber?: number;
  notes?: string;
  // i have to keep track of oral or injection so we gotta add those as types
  estrogenType?: EstrogenType;
  // basically, i can do { route: "injectable", InjectableEstradiols.Cypionate }
}

export interface DiaryEntry {
  id: string;
  date: UnixTime;
  title?: string;
  content: string;
}

export const HRT_STORAGE_KEY = "hrt-meow-data";
export interface Settings {
  enableAutoBackfill: boolean;
  defaultInjectionFrequencyDays: number;
  defaultOralFrequencyDays: number;
  defaultAntiandrogenFrequencyDays: number;
  defaultProgesteroneFrequencyDays: number;
}
export interface HRTData {
  injectableEstradiol?: {
    type: InjectableEstradiols;
    dose: number;
    unit: HormoneUnits;
    frequency: number; // in days
    nextDoseDate?: UnixTime;
  };
  oralEstradiol?: {
    type: OralEstradiols;
    dose: number;
    unit: HormoneUnits;
    frequency: number; // in days
    nextDoseDate?: UnixTime;
  };
  antiandrogen?: {
    type: Antiandrogens;
    dose: number;
    unit: HormoneUnits;
    frequency: number; // in days
    nextDoseDate?: UnixTime;
  };
  progesterone?: {
    type: Progesterones;
    route: ProgesteroneRoutes;
    dose: number;
    unit: HormoneUnits;
    frequency: number; // in days
    nextDoseDate?: UnixTime;
  };
  bloodTests: BloodTest[];
  dosageHistory: DosageHistoryEntry[];
  measurements: Measurement[];
  notes: DiaryEntry[];
  settings?: Settings;
}
