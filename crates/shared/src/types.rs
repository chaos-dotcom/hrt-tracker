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
        bonusDose: Option<bool>,
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
    pub id: Option<String>,
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

/// Liver and lipid markers that come with a wider blood panel.
///
/// The units are fixed because UK labs report them that way: proteins in g/L,
/// enzymes in U/L, bilirubin in umol/L, lipids in mmol/L. Values are stored as
/// plain numbers so old data and the charts keep working.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct MetabolicPanel {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub totalProtein: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub albumin: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub globulin: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alkalinePhosphatase: Option<f64>,
    /// Alanine transferase (ALT), the liver enzyme a tablet estrogen most
    /// often moves first.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alt: Option<f64>,
    /// Aspartate transferase (AST).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ast: Option<f64>,
    /// Gamma-glutamyl transferase (gamma GT).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ggt: Option<f64>,
    /// Total bilirubin, umol/L.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bilirubin: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub triglyceride: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub totalCholesterol: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hdlCholesterol: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ldlCholesterol: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonHdlCholesterol: Option<f64>,
}

impl MetabolicPanel {
    pub fn is_empty(&self) -> bool {
        MetabolicMarker::ALL
            .iter()
            .all(|marker| marker.get(self).is_none())
    }

    pub fn has_liver_values(&self) -> bool {
        MetabolicMarker::ALL
            .iter()
            .any(|marker| marker.is_liver() && marker.get(self).is_some())
    }

    pub fn has_lipid_values(&self) -> bool {
        MetabolicMarker::ALL
            .iter()
            .any(|marker| !marker.is_liver() && marker.get(self).is_some())
    }
}

/// One marker of a [`MetabolicPanel`], with the name, unit and the reference
/// range a UK lab would print for an adult.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetabolicMarker {
    TotalProtein,
    Albumin,
    Globulin,
    AlkalinePhosphatase,
    Alt,
    Ast,
    Ggt,
    Bilirubin,
    Triglyceride,
    TotalCholesterol,
    HdlCholesterol,
    LdlCholesterol,
    NonHdlCholesterol,
}

impl MetabolicMarker {
    pub const ALL: [MetabolicMarker; 13] = [
        MetabolicMarker::TotalProtein,
        MetabolicMarker::Albumin,
        MetabolicMarker::Globulin,
        MetabolicMarker::AlkalinePhosphatase,
        MetabolicMarker::Alt,
        MetabolicMarker::Ast,
        MetabolicMarker::Ggt,
        MetabolicMarker::Bilirubin,
        MetabolicMarker::Triglyceride,
        MetabolicMarker::TotalCholesterol,
        MetabolicMarker::HdlCholesterol,
        MetabolicMarker::LdlCholesterol,
        MetabolicMarker::NonHdlCholesterol,
    ];

    pub fn label(self) -> &'static str {
        match self {
            MetabolicMarker::TotalProtein => "Total protein",
            MetabolicMarker::Albumin => "Albumin",
            MetabolicMarker::Globulin => "Globulin",
            MetabolicMarker::AlkalinePhosphatase => "Alkaline phosphatase",
            MetabolicMarker::Alt => "ALT",
            MetabolicMarker::Ast => "AST",
            MetabolicMarker::Ggt => "Gamma GT",
            MetabolicMarker::Bilirubin => "Bilirubin",
            MetabolicMarker::Triglyceride => "Triglyceride",
            MetabolicMarker::TotalCholesterol => "Total cholesterol",
            MetabolicMarker::HdlCholesterol => "HDL cholesterol",
            MetabolicMarker::LdlCholesterol => "LDL cholesterol",
            MetabolicMarker::NonHdlCholesterol => "Non-HDL cholesterol",
        }
    }

    /// Long name as a report prints it, for the form hint.
    pub fn full_name(self) -> &'static str {
        match self {
            MetabolicMarker::Alt => "Alanine transferase",
            MetabolicMarker::Ast => "Aspartate transferase",
            MetabolicMarker::Ggt => "Gamma-glutamyl transferase",
            other => other.label(),
        }
    }

    pub fn unit(self) -> &'static str {
        match self {
            MetabolicMarker::TotalProtein
            | MetabolicMarker::Albumin
            | MetabolicMarker::Globulin => "g/L",
            MetabolicMarker::AlkalinePhosphatase
            | MetabolicMarker::Alt
            | MetabolicMarker::Ast
            | MetabolicMarker::Ggt => "U/L",
            MetabolicMarker::Bilirubin => "umol/L",
            MetabolicMarker::Triglyceride
            | MetabolicMarker::TotalCholesterol
            | MetabolicMarker::HdlCholesterol
            | MetabolicMarker::LdlCholesterol
            | MetabolicMarker::NonHdlCholesterol => "mmol/L",
        }
    }

    /// True for the liver panel, which matters while estradiol is taken as a
    /// tablet: it is swallowed, so it reaches the liver before the blood.
    pub fn is_liver(self) -> bool {
        !matches!(
            self,
            MetabolicMarker::Triglyceride
                | MetabolicMarker::TotalCholesterol
                | MetabolicMarker::HdlCholesterol
                | MetabolicMarker::LdlCholesterol
                | MetabolicMarker::NonHdlCholesterol
        )
    }

    /// Reference range for an adult, as a UK lab prints it. Where labs differ
    /// this is the lower (female) limit, and the UI labels the range as
    /// typical rather than yours: the report carries its own.
    pub fn typical_range(self) -> (Option<f64>, Option<f64>) {
        match self {
            MetabolicMarker::TotalProtein => (Some(60.0), Some(80.0)),
            MetabolicMarker::Albumin => (Some(35.0), Some(50.0)),
            MetabolicMarker::Globulin => (Some(21.0), Some(35.0)),
            MetabolicMarker::AlkalinePhosphatase => (Some(30.0), Some(130.0)),
            MetabolicMarker::Alt => (Some(0.0), Some(33.0)),
            MetabolicMarker::Ast => (Some(0.0), Some(32.0)),
            MetabolicMarker::Ggt => (Some(0.0), Some(40.0)),
            MetabolicMarker::Bilirubin => (Some(0.0), Some(21.0)),
            MetabolicMarker::Triglyceride => (Some(0.0), Some(1.7)),
            MetabolicMarker::TotalCholesterol => (Some(0.0), Some(5.0)),
            MetabolicMarker::HdlCholesterol => (Some(1.0), None),
            MetabolicMarker::LdlCholesterol => (Some(0.0), Some(3.0)),
            MetabolicMarker::NonHdlCholesterol => (Some(0.0), Some(4.0)),
        }
    }

    /// `None` when the value is inside the typical range, `Some(true)` when it
    /// is above it and `Some(false)` when it is below it.
    pub fn above_range(self, value: f64) -> Option<bool> {
        let (low, high) = self.typical_range();
        if high.is_some_and(|high| value > high) {
            Some(true)
        } else if low.is_some_and(|low| value < low) {
            Some(false)
        } else {
            None
        }
    }

    pub fn get(self, panel: &MetabolicPanel) -> Option<f64> {
        match self {
            MetabolicMarker::TotalProtein => panel.totalProtein,
            MetabolicMarker::Albumin => panel.albumin,
            MetabolicMarker::Globulin => panel.globulin,
            MetabolicMarker::AlkalinePhosphatase => panel.alkalinePhosphatase,
            MetabolicMarker::Alt => panel.alt,
            MetabolicMarker::Ast => panel.ast,
            MetabolicMarker::Ggt => panel.ggt,
            MetabolicMarker::Bilirubin => panel.bilirubin,
            MetabolicMarker::Triglyceride => panel.triglyceride,
            MetabolicMarker::TotalCholesterol => panel.totalCholesterol,
            MetabolicMarker::HdlCholesterol => panel.hdlCholesterol,
            MetabolicMarker::LdlCholesterol => panel.ldlCholesterol,
            MetabolicMarker::NonHdlCholesterol => panel.nonHdlCholesterol,
        }
    }

    pub fn set(self, panel: &mut MetabolicPanel, value: Option<f64>) {
        match self {
            MetabolicMarker::TotalProtein => panel.totalProtein = value,
            MetabolicMarker::Albumin => panel.albumin = value,
            MetabolicMarker::Globulin => panel.globulin = value,
            MetabolicMarker::AlkalinePhosphatase => panel.alkalinePhosphatase = value,
            MetabolicMarker::Alt => panel.alt = value,
            MetabolicMarker::Ast => panel.ast = value,
            MetabolicMarker::Ggt => panel.ggt = value,
            MetabolicMarker::Bilirubin => panel.bilirubin = value,
            MetabolicMarker::Triglyceride => panel.triglyceride = value,
            MetabolicMarker::TotalCholesterol => panel.totalCholesterol = value,
            MetabolicMarker::HdlCholesterol => panel.hdlCholesterol = value,
            MetabolicMarker::LdlCholesterol => panel.ldlCholesterol = value,
            MetabolicMarker::NonHdlCholesterol => panel.nonHdlCholesterol = value,
        }
    }
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pdfFiles: Option<Vec<String>>,
    /// Liver and lipid markers from a wider kit; absent on hormone-only tests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metabolic: Option<MetabolicPanel>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initialIu: Option<f64>,
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
    /// How to show testosterone results: nmol/L (the default) or ng/dL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub displayTestosteroneUnit: Option<HormoneUnits>,
    /// How to show progesterone results: nmol/L (the default) or ng/mL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub displayProgesteroneUnit: Option<HormoneUnits>,
    /// How to show FSH and LH results: mIU/mL (the default) or mIU/L.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub displayFshUnit: Option<HormoneUnits>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub displayLhUnit: Option<HormoneUnits>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub displayInjectableInIU: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub braSizeSystem: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pdfPassword: Option<String>,
    /// Progesterone is usually a daily tablet, so its doses fill a calendar feed
    /// fast. Off unless the user asks for it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icsIncludeProgesterone: Option<bool>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub startDate: Option<UnixTime>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub startDate: Option<UnixTime>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub startDate: Option<UnixTime>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub startDate: Option<UnixTime>,
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

#[cfg(test)]
mod metabolic_tests {
    use super::{BloodTest, MetabolicMarker, MetabolicPanel};

    fn panel_with(marker: MetabolicMarker, value: f64) -> MetabolicPanel {
        let mut panel = MetabolicPanel::default();
        marker.set(&mut panel, Some(value));
        panel
    }

    #[test]
    fn a_panel_knows_which_half_it_holds() {
        let liver = panel_with(MetabolicMarker::Alt, 23.0);
        assert!(liver.has_liver_values());
        assert!(!liver.has_lipid_values());

        let lipids = panel_with(MetabolicMarker::TotalCholesterol, 4.4);
        assert!(lipids.has_lipid_values());
        assert!(!lipids.has_liver_values());
        assert!(!MetabolicPanel::default().is_empty() == false);
    }

    #[test]
    fn typical_ranges_flag_only_what_sits_outside_them() {
        assert_eq!(MetabolicMarker::Alt.above_range(23.0), None);
        assert_eq!(MetabolicMarker::Alt.above_range(31.0), None);
        assert_eq!(MetabolicMarker::Alt.above_range(60.0), Some(true));
        assert_eq!(MetabolicMarker::Albumin.above_range(30.0), Some(false));
        // HDL has a floor and no ceiling: 0.9 is low, 2.0 is fine.
        assert_eq!(MetabolicMarker::HdlCholesterol.above_range(0.9), Some(false));
        assert_eq!(MetabolicMarker::HdlCholesterol.above_range(2.0), None);
    }

    #[test]
    fn markers_carry_their_own_unit() {
        assert_eq!(MetabolicMarker::TotalProtein.unit(), "g/L");
        assert_eq!(MetabolicMarker::Ggt.unit(), "U/L");
        assert_eq!(MetabolicMarker::Bilirubin.unit(), "umol/L");
        assert_eq!(MetabolicMarker::LdlCholesterol.unit(), "mmol/L");
        assert_eq!(MetabolicMarker::ALL.len(), 13);
        assert_eq!(
            MetabolicMarker::ALL.iter().filter(|m| m.is_liver()).count(),
            8
        );
    }

    #[test]
    fn a_test_without_a_panel_still_deserialises() {
        // Data files written before the wider panel existed must keep loading.
        let json = r#"{"date":1700000000000,"estradiolLevel":200.0,"estradiolUnit":"pg/mL"}"#;
        let test: BloodTest = serde_json::from_str(json).expect("old blood test parses");
        assert_eq!(test.metabolic, None);
    }

    #[test]
    fn a_panel_round_trips_through_json() {
        let mut panel = MetabolicPanel::default();
        MetabolicMarker::Alt.set(&mut panel, Some(23.0));
        MetabolicMarker::NonHdlCholesterol.set(&mut panel, Some(2.6));
        let json = serde_json::to_string(&panel).unwrap();
        assert!(json.contains("\"alt\":23.0"), "{json}");
        let back: MetabolicPanel = serde_json::from_str(&json).unwrap();
        assert_eq!(back, panel);
    }
}
