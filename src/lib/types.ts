// https://transfemscience.org/misc/injectable-e2-simulator/
// https://transfemscience.org/misc/
export enum InjectableEstradiols {
  Benzoate = "Estradiol Benzoate",
  Valerate = "Estradiol Valerate",
  Cypionate = "Estradiol Cypionate",
  Enanthate = "Estradiol Enanthate",
  Undecylate = "Estradiol Undecylate",
  PolyestradiolPhosphate = "Polyestradiol Phosphate",
}

export enum OralEstradiols {
  Valerate = "Estradiol Valerate",
  Hemihydrate = "Estradiol Hemihydrate",
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

export const HRT_STORAGE_KEY = "hrt-meow-data:3";
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
  bloodTests?: Array<{
    date: UnixTime;
    estradiolLevel?: number;
    testLevel?: number;
    estradiolUnit?: string;
    testUnit?: string;
    notes?: string;
    // [key: string]: Any;
  }>;
  dosageHistory?: Array<DosageHistoryEntry>;
}
