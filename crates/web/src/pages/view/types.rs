use serde::Deserialize;

use hrt_shared::types::{InjectionSites, SyringeKinds};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum RegimenKey {
    InjectableEstradiol,
    OralEstradiol,
    Antiandrogen,
    Progesterone,
}

#[derive(Clone, PartialEq)]
pub(super) struct NextDoseCandidate {
    pub(super) med_type: RegimenKey,
    pub(super) label: String,
}

#[derive(Clone, PartialEq)]
pub(super) struct PhotoView {
    pub(super) file: String,
    pub(super) note: String,
}

#[derive(Deserialize)]
pub(super) struct UploadResponse {
    pub(super) filenames: Vec<String>,
}

pub(super) const INJECTION_SITE_OPTIONS: [InjectionSites; 12] = [
    InjectionSites::StomachRight,
    InjectionSites::StomachLeft,
    InjectionSites::ThighRight,
    InjectionSites::ThighLeft,
    InjectionSites::TopThighRight,
    InjectionSites::TopThighLeft,
    InjectionSites::InnerThighRight,
    InjectionSites::InnerThighLeft,
    InjectionSites::OuterThighRight,
    InjectionSites::OuterThighLeft,
    InjectionSites::ButtockRight,
    InjectionSites::ButtockLeft,
];

pub(super) const SYRINGE_KIND_OPTIONS: [SyringeKinds; 5] = [
    SyringeKinds::RegularSyringe,
    SyringeKinds::LowWasteSyringe,
    SyringeKinds::LowWasteNeedle,
    SyringeKinds::InsulinSyringe,
    SyringeKinds::InsulinPen,
];

pub(super) const DAY_MS: i64 = 24 * 60 * 60 * 1000;
