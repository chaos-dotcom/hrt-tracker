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

export type UnixTime = number;

type DosageHistoryEntry =
  | {
      date: UnixTime;
      medicationType: "injectableEstradiol";
      type: InjectableEstradiols;
      dose: number;
      unit: string;
    }
  | {
      date: UnixTime;
      medicationType: "oralEstradiol";
      type: OralEstradiols;
      dose: number;
      unit: string;
    }
  | {
      date: UnixTime;
      medicationType: "antiandrogen";
      type: Antiandrogens;
      dose: number;
      unit: string;
    };

export type EstrogenType =
  | { route: "injection"; type: InjectableEstradiols }
  | { route: "oral"; type: OralEstradiols };

export interface BloodTest {
  date: UnixTime;
  estradiolLevel?: number;
  testLevel?: number;
  estradiolUnit?: string;
  testUnit?: string;
  notes?: string;
  // i have to keep track of oral or injection so we gotta add those as types
  estrogenType?: EstrogenType;
  // basically, i can do { route: "injectable", InjectableEstradiols.Cypionate }
}

export const HRT_STORAGE_KEY = "hrt-meow-data";
export interface HRTData {
  injectableEstradiol?: {
    type: InjectableEstradiols;
    dose: number;
    unit: string;
    frequency: string;
  };
  oralEstradiol?: {
    type: OralEstradiols;
    dose: number;
    unit: string;
    frequency: string;
  };
  antiandrogen?: {
    type: Antiandrogens;
    dose: number;
    unit: string;
    frequency: string;
  };
  bloodTests: BloodTest[];
  dosageHistory: DosageHistoryEntry[];
}
