#![allow(non_snake_case)]

use serde::{Deserialize, Serialize};

// https://transfemscience.org/misc/injectable-e2-simulator/
// https://transfemscience.org/misc/
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InjectableEstradiols {
    #[serde(rename = "Estradiol Benzoate")]
    Benzoate,
    #[serde(rename = "Estradiol Cypionate")]
    Cypionate,
    #[serde(rename = "Estradiol Enanthate")]
    Enanthate,
    #[serde(rename = "Estradiol Undecylate")]
    Undecylate,
    #[serde(rename = "Estradiol Valerate")]
    Valerate,
    #[serde(rename = "Polyestradiol Phosphate")]
    PolyestradiolPhosphate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OralEstradiols {
    #[serde(rename = "Estradiol Hemihydrate")]
    Hemihydrate,
    #[serde(rename = "Estradiol Valerate")]
    Valerate,
    #[serde(rename = "Premarin")]
    Premarin,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Antiandrogens {
    #[serde(rename = "Cyproterone Acetate")]
    CPA,
    #[serde(rename = "Spironolactone")]
    Spiro,
    #[serde(rename = "Bicalutamide")]
    Bica,
    #[serde(rename = "Finasteride")]
    Finasteride,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Progesterones {
    #[serde(rename = "Micronized Progesterone")]
    Micronized,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProgesteroneRoutes {
    #[serde(rename = "Oral")]
    Oral,
    #[serde(rename = "Boofed")]
    Boofed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WeightUnit {
    #[serde(rename = "kg")]
    KG,
    #[serde(rename = "lbs")]
    LBS,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LengthUnit {
    #[serde(rename = "cm")]
    CM,
    #[serde(rename = "in")]
    IN,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HormoneUnits {
    #[serde(rename = "pg/mL")]
    E2PgMl,
    #[serde(rename = "pmol/L")]
    E2PmolL,
    #[serde(rename = "ng/dL")]
    TNgDl,
    #[serde(rename = "nmol/L")]
    TNmolL,
    #[serde(rename = "mg")]
    Mg,
    #[serde(rename = "ng/mL")]
    NgMl,
    #[serde(rename = "mIU/mL")]
    MIuMl,
    #[serde(rename = "mIU/L")]
    MIuL,
    #[serde(rename = "U/L")]
    UL,
}

pub type UnixTime = i64;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InjectionSites {
    #[serde(rename = "Stomach right")]
    StomachRight,
    #[serde(rename = "Stomach left")]
    StomachLeft,
    #[serde(rename = "Top thigh right")]
    TopThighRight,
    #[serde(rename = "Top thigh left")]
    TopThighLeft,
    #[serde(rename = "Inner thigh right")]
    InnerThighRight,
    #[serde(rename = "Inner thigh left")]
    InnerThighLeft,
    #[serde(rename = "Outer thigh right")]
    OuterThighRight,
    #[serde(rename = "Outer thigh left")]
    OuterThighLeft,
    #[serde(rename = "Thigh right")]
    ThighRight,
    #[serde(rename = "Thigh left")]
    ThighLeft,
    #[serde(rename = "Buttock right")]
    ButtockRight,
    #[serde(rename = "Buttock left")]
    ButtockLeft,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SyringeKinds {
    #[serde(rename = "Regular syringe")]
    RegularSyringe,
    #[serde(rename = "Low waste syringe")]
    LowWasteSyringe,
    #[serde(rename = "Low waste needle")]
    LowWasteNeedle,
    #[serde(rename = "Insulin syringe")]
    InsulinSyringe,
    #[serde(rename = "Insulin pen")]
    InsulinPen,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "medicationType")]
pub enum DosageHistoryEntry {
    #[serde(rename = "injectableEstradiol")]
    InjectableEstradiol {
        date: UnixTime,
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        #[serde(rename = "type")]
        kind: InjectableEstradiols,
        dose: f64,
        unit: HormoneUnits,
        #[serde(skip_serializing_if = "Option::is_none")]
        note: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        injectionSite: Option<InjectionSites>,
        #[serde(skip_serializing_if = "Option::is_none")]
        vialId: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        subVialId: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        syringeKind: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        needleLength: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        needleGauge: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        photos: Option<Vec<DosagePhoto>>,
    },
    #[serde(rename = "oralEstradiol")]
    OralEstradiol {
        date: UnixTime,
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        #[serde(rename = "type")]
        kind: OralEstradiols,
        dose: f64,
        unit: HormoneUnits,
        #[serde(skip_serializing_if = "Option::is_none")]
        pillQuantity: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        note: Option<String>,
    },
    #[serde(rename = "antiandrogen")]
    Antiandrogen {
        date: UnixTime,
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        #[serde(rename = "type")]
        kind: Antiandrogens,
        dose: f64,
        unit: HormoneUnits,
        #[serde(skip_serializing_if = "Option::is_none")]
        note: Option<String>,
    },
    #[serde(rename = "progesterone")]
    Progesterone {
        date: UnixTime,
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        #[serde(rename = "type")]
        kind: Progesterones,
        route: ProgesteroneRoutes,
        dose: f64,
        unit: HormoneUnits,
        #[serde(skip_serializing_if = "Option::is_none")]
        pillQuantity: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        note: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum DosagePhoto {
    Legacy(String),
    Entry {
        file: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        note: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EstrogenType {
    #[serde(rename = "injection")]
    Injection {
        #[serde(rename = "type")]
        kind: InjectableEstradiols,
    },
    #[serde(rename = "oral")]
    Oral {
        #[serde(rename = "type")]
        kind: OralEstradiols,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Measurement {
    pub date: UnixTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weightUnit: Option<WeightUnit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heightUnit: Option<LengthUnit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub underbust: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bust: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bideltoid: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub waist: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hip: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bodyMeasurementUnit: Option<LengthUnit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub braSize: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BloodTest {
    pub date: UnixTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estradiolLevel: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub testLevel: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estradiolUnit: Option<HormoneUnits>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub testUnit: Option<HormoneUnits>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progesteroneLevel: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progesteroneUnit: Option<HormoneUnits>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fshLevel: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fshUnit: Option<HormoneUnits>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lhLevel: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lhUnit: Option<HormoneUnits>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prolactinLevel: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prolactinUnit: Option<HormoneUnits>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shbgLevel: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shbgUnit: Option<HormoneUnits>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeAndrogenIndex: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estrannaiseNumber: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fudgeFactor: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estrogenType: Option<EstrogenType>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiaryEntry {
    pub id: String,
    pub date: UnixTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubVial {
    pub id: String,
    pub personalNumber: String,
    pub createdAt: UnixTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Vial {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub esterKind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suspensionOil: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub otherIngredients: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batchNumber: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub concentrationMgPerMl: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub isSpent: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spentAt: Option<UnixTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub useBy: Option<UnixTime>,
    pub createdAt: UnixTime,
    #[serde(default)]
    pub subVials: Vec<SubVial>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    pub enableAutoBackfill: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icsSecret: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enableBloodTestSchedule: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bloodTestIntervalMonths: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statsBreakdownBySyringeKind: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub displayEstradiolUnit: Option<HormoneUnits>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InjectableSchedule {
    #[serde(rename = "type")]
    pub kind: InjectableEstradiols,
    pub dose: f64,
    pub unit: HormoneUnits,
    pub frequency: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vialId: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subVialId: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub syringeKind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub needleLength: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub needleGauge: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nextDoseDate: Option<UnixTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OralSchedule {
    #[serde(rename = "type")]
    pub kind: OralEstradiols,
    pub dose: f64,
    pub unit: HormoneUnits,
    pub frequency: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nextDoseDate: Option<UnixTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AntiandrogenSchedule {
    #[serde(rename = "type")]
    pub kind: Antiandrogens,
    pub dose: f64,
    pub unit: HormoneUnits,
    pub frequency: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nextDoseDate: Option<UnixTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProgesteroneSchedule {
    #[serde(rename = "type")]
    pub kind: Progesterones,
    pub route: ProgesteroneRoutes,
    pub dose: f64,
    pub unit: HormoneUnits,
    pub frequency: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nextDoseDate: Option<UnixTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct HrtData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub injectableEstradiol: Option<InjectableSchedule>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oralEstradiol: Option<OralSchedule>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub antiandrogen: Option<AntiandrogenSchedule>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progesterone: Option<ProgesteroneSchedule>,
    #[serde(default)]
    pub bloodTests: Vec<BloodTest>,
    #[serde(default)]
    pub dosageHistory: Vec<DosageHistoryEntry>,
    #[serde(default)]
    pub measurements: Vec<Measurement>,
    #[serde(default)]
    pub notes: Vec<DiaryEntry>,
    #[serde(default)]
    pub vials: Vec<Vial>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<Settings>,
}

pub const HRT_STORAGE_KEY: &str = "hrt-meow-data";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EstrannaiseModel {
    #[serde(rename = "EB im")]
    EbIm,
    #[serde(rename = "EV im")]
    EvIm,
    #[serde(rename = "EEn im")]
    EEnIm,
    #[serde(rename = "EC im")]
    EcIm,
    #[serde(rename = "EUn im")]
    EUnIm,
    #[serde(rename = "EUn casubq")]
    EUnCasubq,
    #[serde(rename = "patch tw")]
    PatchTw,
    #[serde(rename = "patch ow")]
    PatchOw,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Hormone {
    #[serde(rename = "Cholesterol")]
    Cholesterol,
    #[serde(rename = "Testosterone")]
    Testosterone,
    #[serde(rename = "Dihydrotestosterone")]
    Dihydrotestosterone,
    #[serde(rename = "Dehydroepiandrosterone")]
    Dehydroepiandrosterone,
    #[serde(rename = "Estrone")]
    Estrone,
    #[serde(rename = "Estradiol")]
    Estradiol,
    #[serde(rename = "Estriol")]
    Estriol,
    #[serde(rename = "Estetrol")]
    Estetrol,
    #[serde(rename = "Progesterone")]
    Progesterone,
    #[serde(rename = "Aldosterone")]
    Aldosterone,
    #[serde(rename = "Androstenedione")]
    Androstenedione,
    #[serde(rename = "Cortisol")]
    Cortisol,
    #[serde(rename = "Gonadorelin")]
    Gonadorelin,
    #[serde(rename = "Follicle-stimulating hormone")]
    FollicleStimulatingHormone,
    #[serde(rename = "Luteinising hormone")]
    LuteinisingHormone,
    #[serde(rename = "Thyroid-stimulating hormone")]
    ThyroidStimulatingHormone,
    #[serde(rename = "Sex hormone-binding globulin")]
    SexHormoneBindingGlobulin,
    #[serde(rename = "Prolactin")]
    Prolactin,
    #[serde(rename = "Thyroxine")]
    Thyroxine,
    #[serde(rename = "Triiodothyronine")]
    Triiodothyronine,
    #[serde(rename = "Vitamin D3")]
    VitaminD3,
    #[serde(rename = "Vitamin B12")]
    VitaminB12,
}
