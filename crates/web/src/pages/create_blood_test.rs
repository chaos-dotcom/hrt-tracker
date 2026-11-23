use gloo_net::http::Request;
use gloo_timers::callback::Timeout;
use js_sys::{Date, Object, Reflect};
use leptos::*;
use leptos_router::A;
use serde::Deserialize;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{FileReader, FormData, HtmlInputElement};

use crate::layout::page_layout;
use crate::store::use_store;
use crate::utils::{compute_fudge_factor, hormone_unit_label, parse_decimal, parse_hormone_unit};
use hrt_shared::logic::{latest_estrogen_route, predict_e2_pg_ml};
use hrt_shared::types::{BloodTest, HormoneUnits, MetabolicMarker, MetabolicPanel};

#[derive(Clone, PartialEq)]
struct UnitOption {
    label: String,
}

const UNIT_OPTIONS: [HormoneUnits; 9] = [
    HormoneUnits::E2PgMl,
    HormoneUnits::E2PmolL,
    HormoneUnits::TNgDl,
    HormoneUnits::TNmolL,
    HormoneUnits::Mg,
    HormoneUnits::NgMl,
    HormoneUnits::MIuMl,
    HormoneUnits::MIuL,
    HormoneUnits::UL,
];

#[derive(Clone, Debug)]
struct OcrValue {
    value: String,
    unit: Option<String>,
}

#[derive(Clone, Debug, Default)]
struct OcrExtraction {
    estradiol: Option<OcrValue>,
    testosterone: Option<OcrValue>,
    progesterone: Option<OcrValue>,
    fsh: Option<OcrValue>,
    lh: Option<OcrValue>,
    prolactin: Option<OcrValue>,
    shbg: Option<OcrValue>,
    fai: Option<OcrValue>,
    sample_date: Option<String>,
    metabolic: MetabolicPanel,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PdfUploadFile {
    filename: String,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    extract_error: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct PdfUploadResponse {
    files: Vec<PdfUploadFile>,
}

/// The liver and lipid inputs of the blood-test form, one signal per marker.
#[derive(Clone, Copy)]
struct MetabolicInputs {
    total_protein: RwSignal<String>,
    albumin: RwSignal<String>,
    globulin: RwSignal<String>,
    alkaline_phosphatase: RwSignal<String>,
    alt: RwSignal<String>,
    ast: RwSignal<String>,
    ggt: RwSignal<String>,
    bilirubin: RwSignal<String>,
    triglyceride: RwSignal<String>,
    total_cholesterol: RwSignal<String>,
    hdl_cholesterol: RwSignal<String>,
    ldl_cholesterol: RwSignal<String>,
    non_hdl_cholesterol: RwSignal<String>,
}

impl MetabolicInputs {
    fn new() -> Self {
        Self {
            total_protein: create_rw_signal(String::new()),
            albumin: create_rw_signal(String::new()),
            globulin: create_rw_signal(String::new()),
            alkaline_phosphatase: create_rw_signal(String::new()),
            alt: create_rw_signal(String::new()),
            ast: create_rw_signal(String::new()),
            ggt: create_rw_signal(String::new()),
            bilirubin: create_rw_signal(String::new()),
            triglyceride: create_rw_signal(String::new()),
            total_cholesterol: create_rw_signal(String::new()),
            hdl_cholesterol: create_rw_signal(String::new()),
            ldl_cholesterol: create_rw_signal(String::new()),
            non_hdl_cholesterol: create_rw_signal(String::new()),
        }
    }

    fn signal(self, marker: MetabolicMarker) -> RwSignal<String> {
        match marker {
            MetabolicMarker::TotalProtein => self.total_protein,
            MetabolicMarker::Albumin => self.albumin,
            MetabolicMarker::Globulin => self.globulin,
            MetabolicMarker::AlkalinePhosphatase => self.alkaline_phosphatase,
            MetabolicMarker::Alt => self.alt,
            MetabolicMarker::Ast => self.ast,
            MetabolicMarker::Ggt => self.ggt,
            MetabolicMarker::Bilirubin => self.bilirubin,
            MetabolicMarker::Triglyceride => self.triglyceride,
            MetabolicMarker::TotalCholesterol => self.total_cholesterol,
            MetabolicMarker::HdlCholesterol => self.hdl_cholesterol,
            MetabolicMarker::LdlCholesterol => self.ldl_cholesterol,
            MetabolicMarker::NonHdlCholesterol => self.non_hdl_cholesterol,
        }
    }

    /// What to store: `None` when the person filled nothing in.
    fn panel(self) -> Option<MetabolicPanel> {
        let mut panel = MetabolicPanel::default();
        for marker in MetabolicMarker::ALL {
            if let Some(value) = parse_decimal(&self.signal(marker).get()) {
                marker.set(&mut panel, Some(value));
            }
        }
        (!panel.is_empty()).then_some(panel)
    }

    /// Fill the inputs a report gave us and leave the rest alone.
    fn apply(self, panel: &MetabolicPanel) -> usize {
        let mut filled = 0;
        for marker in MetabolicMarker::ALL {
            if let Some(value) = marker.get(panel) {
                self.signal(marker).set(fmt_marker_value(value));
                filled += 1;
            }
        }
        filled
    }
}

fn fmt_marker_value(value: f64) -> String {
    if value.fract().abs() < 1e-9 {
        format!("{value:.0}")
    } else {
        format!("{value}")
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_ocr_extraction(
    extracted: OcrExtraction,
    estradiol_level: RwSignal<String>,
    estradiol_unit: RwSignal<String>,
    test_level: RwSignal<String>,
    test_unit: RwSignal<String>,
    progesterone_level: RwSignal<String>,
    progesterone_unit: RwSignal<String>,
    fsh_level: RwSignal<String>,
    fsh_unit: RwSignal<String>,
    lh_level: RwSignal<String>,
    lh_unit: RwSignal<String>,
    prolactin_level: RwSignal<String>,
    prolactin_unit: RwSignal<String>,
    shbg_level: RwSignal<String>,
    shbg_unit: RwSignal<String>,
    free_androgen_index: RwSignal<String>,
    test_date_time: RwSignal<String>,
    metabolic: MetabolicInputs,
) -> usize {
    let mut filled = 0;
    if let Some(date) = extracted.sample_date {
        test_date_time.set(date);
        filled += 1;
    }
    if let Some(value) = extracted.estradiol {
        estradiol_level.set(value.value);
        if let Some(unit) = value.unit {
            estradiol_unit.set(unit);
        }
        filled += 1;
    }
    if let Some(value) = extracted.testosterone {
        test_level.set(value.value);
        if let Some(unit) = value.unit {
            test_unit.set(unit);
        }
        filled += 1;
    }
    if let Some(value) = extracted.progesterone {
        progesterone_level.set(value.value);
        if let Some(unit) = value.unit {
            progesterone_unit.set(unit);
        }
        filled += 1;
    }
    if let Some(value) = extracted.fsh {
        fsh_level.set(value.value);
        if let Some(unit) = value.unit {
            fsh_unit.set(unit);
        }
        filled += 1;
    }
    if let Some(value) = extracted.lh {
        lh_level.set(value.value);
        if let Some(unit) = value.unit {
            lh_unit.set(unit);
        }
        filled += 1;
    }
    if let Some(value) = extracted.prolactin {
        prolactin_level.set(value.value);
        if let Some(unit) = value.unit {
            prolactin_unit.set(unit);
        }
        filled += 1;
    }
    if let Some(value) = extracted.shbg {
        shbg_level.set(value.value);
        if let Some(unit) = value.unit {
            shbg_unit.set(unit);
        }
        filled += 1;
    }
    if let Some(value) = extracted.fai {
        free_androgen_index.set(value.value);
        filled += 1;
    }
    filled += metabolic.apply(&extracted.metabolic);
    filled
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = Tesseract, js_name = recognize)]
    fn tesseract_recognize(image: JsValue, lang: &str, options: JsValue) -> js_sys::Promise;
}

fn to_local_input_value(ms: i64) -> String {
    let date = Date::new(&JsValue::from_f64(ms as f64));
    let year = date.get_full_year();
    let month = date.get_month() + 1;
    let day = date.get_date();
    let hour = date.get_hours();
    let minute = date.get_minutes();
    format!(
        "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}",
        year = year,
        month = month,
        day = day,
        hour = hour,
        minute = minute
    )
}

fn parse_datetime_local(value: &str) -> i64 {
    if value.trim().is_empty() {
        return Date::now() as i64;
    }
    let parsed = Date::parse(value);
    if parsed.is_nan() {
        Date::now() as i64
    } else {
        parsed as i64
    }
}

fn parse_optional(value: &str) -> Option<f64> {
    parse_decimal(value)
}

fn unit_or_default(value: &str, fallback: HormoneUnits) -> HormoneUnits {
    parse_hormone_unit(value).unwrap_or(fallback)
}

fn parse_ocr_number(token: &str) -> Option<String> {
    let trimmed = token.trim();
    if trimmed.is_empty() {
        return None;
    }
    let trimmed = trimmed.trim_matches(|c: char| c == '<' || c == '>' || c == '=');
    let mut cleaned = String::new();
    for c in trimmed.chars() {
        if c.is_ascii_digit() || c == '.' {
            cleaned.push(c);
        } else if c == ',' {
            cleaned.push('.');
        }
    }
    if cleaned.ends_with('.') {
        cleaned.pop();
    }
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned)
    }
}

fn normalize_ocr_unit(token: &str) -> Option<String> {
    normalize_unit_token(token).map(str::to_string)
}

/// Canonical unit label used across the app ("pmol/L", "mIU/L", ...).
///
/// Reports print the litre as a lowercase `l` ("pmol/l", "mIU/l", "U/l") and
/// OCR renders it as `|`, `1`, `I` or `i`; `0` turns into `O` inside `nmol`.
fn normalize_unit_token(token: &str) -> Option<&'static str> {
    let trimmed = token.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut cleaned: String = trimmed
        .trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '/' && c != '|' && c != '\\')
        .to_lowercase();
    cleaned = cleaned.replace(['\u{00b5}', '\u{03bc}'], "u");
    cleaned = cleaned.replace(['|', '\\'], "/");
    cleaned = cleaned.replace("/l1", "/l").replace("/i1", "/l");
    cleaned = cleaned.replace("/i", "/l").replace("/1", "/l");
    cleaned = cleaned.replace("nm0l", "nmol").replace("pm0l", "pmol");
    cleaned = cleaned.replace("mlu", "miu");
    let cleaned = cleaned
        .trim_matches(|c: char| c == '.' || c == ',' || c == ':')
        .to_string();
    match cleaned.as_str() {
        "pmol/l" => Some("pmol/L"),
        "pg/ml" => Some("pg/mL"),
        "ng/dl" => Some("ng/dL"),
        "nmol/l" => Some("nmol/L"),
        "ng/ml" => Some("ng/mL"),
        "miu/l" | "miu" | "miul" => Some("mIU/L"),
        "miu/ml" => Some("mIU/mL"),
        "u/l" | "iu/l" | "ui/l" | "ul" | "ui" | "iu" => Some("U/L"),
        // protein and the wider panel: g/L, umol/L and mmol/L
        "g/l" | "gl" => Some("g/L"),
        "umol/l" | "umol" => Some("umol/L"),
        "mmol/l" | "mmol" => Some("mmol/L"),
        _ => None,
    }
}

fn extract_ocr_value(text: &str, labels: &[&str]) -> Option<OcrValue> {
    let lower = text.to_lowercase();
    for label in labels {
        if let Some(idx) = lower.find(label) {
            let start = idx + label.len();
            let end = (start + 220).min(lower.len());
            let window = &lower[start..end];
            let tokens: Vec<&str> = window.split_whitespace().collect();
            for (index, token) in tokens.iter().enumerate() {
                if let Some(value) = parse_ocr_number(token) {
                    let mut unit = None;
                    for candidate in tokens.iter().skip(index + 1).take(4) {
                        if let Some(normalized) = normalize_ocr_unit(candidate) {
                            unit = Some(normalized);
                            break;
                        }
                    }
                    return Some(OcrValue { value, unit });
                }
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Banded report reader (Randox Health "Female Hormone" layouts)
// ---------------------------------------------------------------------------
//
// Newer Randox reports give every hormone its own section: the full name on a
// line of its own ("Oestradiol", "Sex Hormone Binding Globulin"), the result as
// one right-aligned number, the unit on a line of its own, and a band row of
// reference ranges and cycle-phase ranges underneath ("Low / Optimal / High /
// Very High"). `lopdf` hands us the words in content-stream order, so the
// result and its unit land either right before or right after the title, and
// the phase ranges arrive glued to their labels ("1.7 - 7.7 Luteal
// Phase25.8 - 134.8 Post Menopausal"). A naive "first number after the label"
// scan therefore picks a page number out of the contents page, a range bound,
// or a number out of the glossary prose.
//
// The reader below:
//   1. accepts a title only where whole lines spell it, so prose cannot open a
//      section;
//   2. pairs every unit token with the number beside it and gives that pair to
//      the nearest title on the side the number sits on;
//   3. falls back, per section, to the one number that is neither part of a
//      range nor glued to a cycle phase ("Oestradiol" -> 1319 pmol/L);
//   4. drops any number that also occurs inside that section's ranges and
//      bands, so thresholds such as "<102" or ">700" are never results.

const BANDED_HORMONE_COUNT: usize = 8;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum BandedHormone {
    Estradiol,
    Testosterone,
    Progesterone,
    Fsh,
    Lh,
    Prolactin,
    Shbg,
    Fai,
}

const BANDED_TITLES: [(&str, BandedHormone); 10] = [
    ("oestradiol", BandedHormone::Estradiol),
    ("estradiol", BandedHormone::Estradiol),
    ("progesterone", BandedHormone::Progesterone),
    ("prolactin", BandedHormone::Prolactin),
    ("testosterone", BandedHormone::Testosterone),
    ("sex hormone binding globulin", BandedHormone::Shbg),
    ("free androgen index", BandedHormone::Fai),
    ("follicle stimulating hormone", BandedHormone::Fsh),
    ("luteinising hormone", BandedHormone::Lh),
    ("luteinizing hormone", BandedHormone::Lh),
];

/// Words that introduce a cycle-phase reference range; a number next to one of
/// them belongs to the range row, never to the result.
const PHASE_WORDS: [&str; 7] = [
    "follicular",
    "midcycle",
    "mid",
    "luteal",
    "post",
    "menopausal",
    "phase",
];

impl BandedHormone {
    fn index(self) -> usize {
        match self {
            BandedHormone::Estradiol => 0,
            BandedHormone::Testosterone => 1,
            BandedHormone::Progesterone => 2,
            BandedHormone::Fsh => 3,
            BandedHormone::Lh => 4,
            BandedHormone::Prolactin => 5,
            BandedHormone::Shbg => 6,
            BandedHormone::Fai => 7,
        }
    }

    /// Unit a banded report reports this hormone in when the unit line was not
    /// recovered (OCR drops it), except Free Androgen Index, which is unitless.
    fn default_unit(self) -> Option<&'static str> {
        match self {
            BandedHormone::Estradiol => Some("pmol/L"),
            BandedHormone::Testosterone | BandedHormone::Progesterone | BandedHormone::Shbg => {
                Some("nmol/L")
            }
            BandedHormone::Fsh | BandedHormone::Lh => Some("U/L"),
            BandedHormone::Prolactin => Some("mIU/L"),
            BandedHormone::Fai => None,
        }
    }
}

#[derive(Clone, Debug)]
struct LayoutToken {
    text: String,
    line: usize,
}

#[derive(Clone, Copy, Debug)]
struct TitleHit {
    hormone: BandedHormone,
    first_line: usize,
    last_line: usize,
}

/// Collapse every run of whitespace (newlines included) to a single space.
fn collapse_whitespace(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut pending_space = false;
    for ch in text.chars() {
        if ch.is_whitespace() {
            pending_space = !out.is_empty();
        } else {
            if pending_space {
                out.push(' ');
                pending_space = false;
            }
            out.push(ch);
        }
    }
    out
}

/// ASCII-case-insensitive `find`, returning a byte offset into `haystack`.
fn find_ci(haystack: &str, needle: &str) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    let hay = haystack.as_bytes();
    let ndl = needle.as_bytes();
    (0..=hay.len() - ndl.len()).find(|&index| hay[index..index + ndl.len()].eq_ignore_ascii_case(ndl))
}

/// At most `max_chars` characters of `text` starting at `start`.
fn take_chars(text: &str, start: usize, max_chars: usize) -> &str {
    let mut end = start.min(text.len());
    for (count, (offset, ch)) in text[start.min(text.len())..].char_indices().enumerate() {
        if count >= max_chars {
            break;
        }
        end = start + offset + ch.len_utf8();
    }
    &text[start.min(text.len())..end]
}

fn layout_tokens(lines: &[&str]) -> Vec<LayoutToken> {
    let mut tokens = Vec::new();
    for (line, text) in lines.iter().enumerate() {
        for word in text.split_whitespace() {
            // "0.291 -1.67" is a range whose dash lost its trailing space, so
            // the dash is split off and the bound stays a number.
            if let Some(first) = word.chars().next() {
                let (dash, rest) = word.split_at(first.len_utf8());
                if is_dash_token(dash) && text_is_number(rest) {
                    tokens.push(LayoutToken {
                        text: dash.to_string(),
                        line,
                    });
                    tokens.push(LayoutToken {
                        text: rest.to_string(),
                        line,
                    });
                    continue;
                }
            }
            tokens.push(LayoutToken {
                text: word.to_string(),
                line,
            });
        }
    }
    tokens
}

fn title_hormone(phrase: &str) -> Option<BandedHormone> {
    let phrase = phrase.trim().trim_end_matches([':', '.', ',']);
    let phrase: String = phrase
        .chars()
        .map(|ch| {
            if ch.is_whitespace() {
                ' '
            } else {
                ch.to_ascii_lowercase()
            }
        })
        .collect();
    let phrase = collapse_whitespace(&phrase);
    BANDED_TITLES
        .iter()
        .find(|(name, _)| *name == phrase)
        .map(|(_, hormone)| *hormone)
}

/// A title is only accepted where one line, or a few consecutive short lines,
/// spells exactly the hormone's name: this keeps "high testosterone can lead
/// to ..." and the glossary headings from opening sections.
fn match_title_at(lines: &[&str], start: usize) -> Option<(BandedHormone, usize)> {
    let mut phrase = String::new();
    let mut words = 0;
    let mut line = start;
    while line < lines.len() && words < 6 {
        let trimmed = lines[line].trim();
        let line_number = line;
        line += 1;
        if trimmed.is_empty() {
            continue;
        }
        if !phrase.is_empty() {
            phrase.push(' ');
        }
        phrase.push_str(trimmed);
        words += 1;
        if phrase.chars().count() > 40 {
            return None;
        }
        if let Some(hormone) = title_hormone(&phrase) {
            return Some((hormone, line_number));
        }
    }
    None
}

fn find_title_hits(lines: &[&str]) -> Vec<TitleHit> {
    let mut hits = Vec::new();
    let mut line = 0;
    while line < lines.len() {
        match match_title_at(lines, line) {
            Some((hormone, last_line)) => {
                hits.push(TitleHit {
                    hormone,
                    first_line: line,
                    last_line,
                });
                line = last_line + 1;
            }
            None => line += 1,
        }
    }
    hits
}

/// Line index of the last "Test Glossary" heading, if the report has one. The
/// glossary repeats every hormone name and is full of prose numbers, so the
/// sections are read from above it.
fn glossary_cut(lines: &[&str]) -> Option<usize> {
    lines.iter().rposition(|line| {
        let lower = line.trim().to_lowercase();
        lower == "test glossary" || lower.starts_with("test glossary ")
    })
}

/// Digits of a numeric token, plus whether it carried a threshold symbol
/// ("<102", ">700", "≤0.29"). OCR confusions inside numbers are repaired here.
fn numeric_token(token: &str) -> Option<(String, bool)> {
    let mut rest = token.trim();
    let mut threshold = false;
    while let Some(ch) = rest.chars().next() {
        if matches!(ch, '<' | '>' | '≤' | '≥' | '~' | '=') {
            threshold = true;
            rest = &rest[ch.len_utf8()..];
        } else {
            break;
        }
    }
    if rest.is_empty() {
        return if threshold {
            Some((String::new(), true))
        } else {
            None
        };
    }
    let single = rest.chars().count() == 1;
    let has_digit = rest.chars().any(|ch| ch.is_ascii_digit());
    let mut digits = String::new();
    for ch in rest.chars() {
        match ch {
            '0'..='9' => digits.push(ch),
            '.' | ',' => digits.push('.'),
            // OCR reads 0 as O and 1 as l/I often enough to be worth repairing,
            // but only inside something that already looks like a number.
            'O' | 'o' if has_digit || single => digits.push('0'),
            'l' | 'I' if has_digit || single => digits.push('1'),
            _ => return None,
        }
    }
    if !digits.chars().any(|ch| ch.is_ascii_digit()) {
        return None;
    }
    Some((digits, threshold))
}

fn token_has_digits(token: &str) -> bool {
    numeric_token(token)
        .map(|(digits, _)| !digits.is_empty())
        .unwrap_or(false)
}

/// A maximal run of number-like pieces starting at `start`: OCR splits results
/// ("1 31 9") and spaced decimals ("< O . 3") across several tokens, so a run
/// joins at most one multi-digit token.
fn number_run_after(tokens: &[LayoutToken], start: usize) -> Option<(usize, usize)> {
    if start >= tokens.len() {
        return None;
    }
    let mut end = start;
    let mut pieces = 0;
    let mut multi_digit = 0;
    while end < tokens.len() && pieces < 4 {
        let text = tokens[end].text.as_str();
        if text == "." {
            pieces += 1;
            end += 1;
            continue;
        }
        let Some((digits, threshold)) = numeric_token(text) else {
            break;
        };
        if threshold && pieces > 0 {
            break;
        }
        if digits.chars().filter(char::is_ascii_digit).count() >= 2 {
            multi_digit += 1;
            if multi_digit > 1 {
                break;
            }
        }
        pieces += 1;
        end += 1;
    }
    if pieces == 0 {
        None
    } else {
        Some((start, end - 1))
    }
}

/// The run of number-like pieces that ends right before `end` (exclusive),
/// e.g. the "889" in "mIU/l 889 Prolactin".
fn number_run_before(tokens: &[LayoutToken], end: usize) -> Option<(usize, usize)> {
    if end == 0 || end > tokens.len() {
        return None;
    }
    let window_start = end.saturating_sub(5);
    (window_start..end).find_map(|start| {
        let run = number_run_after(tokens, start)?;
        (run.1 + 1 == end).then_some(run)
    })
}

fn text_is_number(token: &str) -> bool {
    token == "." || numeric_token(token).is_some()
}

/// The number a run spells, with threshold symbols stripped and split digits
/// joined: ["1", "31", "9"] -> "1319", ["<", "O", ".", "3"] -> "0.3".
fn run_digits(tokens: &[LayoutToken], start: usize, end: usize) -> Option<(String, bool)> {
    let mut digits = String::new();
    let mut threshold = false;
    for index in start..=end {
        let text = tokens[index].text.as_str();
        if text == "." {
            digits.push('.');
            continue;
        }
        let (piece, piece_threshold) = numeric_token(text)?;
        threshold |= piece_threshold;
        digits.push_str(&piece);
    }
    if digits.matches('.').count() > 1 {
        return None;
    }
    let digits = digits.trim_matches('.').to_string();
    if !digits.chars().any(|ch| ch.is_ascii_digit()) {
        return None;
    }
    Some((digits, threshold))
}

fn is_dash_token(token: &str) -> bool {
    matches!(token.trim(), "-" | "–" | "—")
}

/// True when the run sits inside a reference range ("0.291 - 1.67").
fn is_range_piece(tokens: &[LayoutToken], start: usize, end: usize) -> bool {
    if start >= 2 && is_dash_token(&tokens[start - 1].text) && token_has_digits(&tokens[start - 2].text)
    {
        return true;
    }
    if end + 2 < tokens.len()
        && is_dash_token(&tokens[end + 1].text)
        && token_has_digits(&tokens[end + 2].text)
    {
        return true;
    }
    false
}

/// True when the run belongs to a cycle-phase row: either it is followed by a
/// phase word ("<505 Post Menopausal") or it is glued to one ("Phase25.8").
fn is_phase_piece(tokens: &[LayoutToken], start: usize, end: usize) -> bool {
    if let Some(next) = tokens.get(end + 1) {
        let word: String = next
            .text
            .chars()
            .filter(|ch| ch.is_ascii_alphabetic())
            .collect::<String>()
            .to_lowercase();
        if PHASE_WORDS.iter().any(|phase| word.starts_with(phase)) {
            return true;
        }
    }
    if start > 0 {
        let previous = tokens[start - 1].text.to_lowercase();
        if previous.chars().any(|ch| ch.is_ascii_digit()) {
            let alpha = previous.trim_end_matches(|ch: char| ch.is_ascii_digit() || ch == '.');
            if PHASE_WORDS.iter().any(|phase| alpha.ends_with(phase)) {
                return true;
            }
        }
    }
    false
}

/// True when the section carries the Low/Optimal/High band labels, whose edges
/// are written as thresholds ("<102", "≤0.29") and would look like results.
fn section_has_band_labels(tokens: &[LayoutToken], start: usize, end: usize) -> bool {
    (start..end).any(|index| {
        let word: String = tokens[index]
            .text
            .chars()
            .filter(char::is_ascii_alphabetic)
            .collect::<String>()
            .to_lowercase();
        matches!(word.as_str(), "low" | "optimal" | "high")
    })
}

/// True when the section carries a reference range ("102 - 496"), which is
/// where a threshold number without a decimal point comes from.
fn section_has_ranges(tokens: &[LayoutToken], start: usize, end: usize) -> bool {
    (start..end).any(|index| {
        is_dash_token(&tokens[index].text)
            && index > 0
            && index + 1 < end
            && token_has_digits(&tokens[index - 1].text)
            && token_has_digits(&tokens[index + 1].text)
    })
}

fn nearest_title_before(hits: &[TitleHit], line: usize) -> Option<usize> {
    hits.iter()
        .enumerate()
        .filter(|(_, hit)| hit.last_line <= line)
        .map(|(index, _)| index)
        .next_back()
}

fn nearest_title_after(hits: &[TitleHit], line: usize) -> Option<usize> {
    hits.iter().position(|hit| hit.first_line >= line)
}

/// Token range of the body of the section starting at `hit_index`.
fn section_tokens(tokens: &[LayoutToken], hits: &[TitleHit], hit_index: usize, last_line: usize) -> (usize, usize) {
    let first = hits[hit_index].last_line + 1;
    let last = hits
        .get(hit_index + 1)
        .map(|next| next.first_line)
        .unwrap_or(last_line);
    let start = tokens.iter().position(|token| token.line >= first).unwrap_or(tokens.len());
    let end = tokens
        .iter()
        .position(|token| token.line >= last)
        .unwrap_or(tokens.len());
    (start, end.max(start))
}

/// True when the same number also appears somewhere else in the section: a
/// result is never also a band edge, so "<102" next to "102 - 496" is not one.
fn appears_in_section(
    tokens: &[LayoutToken],
    hits: &[TitleHit],
    hit_index: usize,
    digits: &str,
    skip: (usize, usize),
    last_line: usize,
) -> bool {
    let Some(target) = parse_decimal(digits) else {
        return false;
    };
    let (start, end) = section_tokens(tokens, hits, hit_index, last_line);
    for index in start..end {
        if index >= skip.0 && index <= skip.1 {
            continue;
        }
        if let Some((other, _)) = numeric_token(&tokens[index].text) {
            if parse_decimal(&other) == Some(target) {
                return true;
            }
        }
    }
    false
}

#[derive(Clone, Copy, Debug)]
enum UnitAnchor {
    Measure(&'static str),
    /// The bare dash some reports print instead of a unit (Free Androgen Index).
    Dash,
}

fn unit_anchor(token: &str) -> Option<UnitAnchor> {
    if is_dash_token(token) {
        return Some(UnitAnchor::Dash);
    }
    normalize_unit_token(token).map(UnitAnchor::Measure)
}

fn consider(slot: &mut Option<(u8, OcrValue)>, score: u8, value: OcrValue) {
    let better = slot.as_ref().map(|(best, _)| *best < score).unwrap_or(true);
    if better {
        *slot = Some((score, value));
    }
}

fn extract_banded_values(text: &str) -> [Option<OcrValue>; BANDED_HORMONE_COUNT] {
    let mut best: [Option<(u8, OcrValue)>; BANDED_HORMONE_COUNT] = std::array::from_fn(|_| None);
    let all_lines: Vec<&str> = text.lines().collect();
    let lines: Vec<&str> = match glossary_cut(&all_lines) {
        Some(cut) if find_title_hits(&all_lines[..cut]).len() >= 2 => all_lines[..cut].to_vec(),
        _ => all_lines,
    };
    let hits = find_title_hits(&lines);
    if hits.is_empty() {
        return std::array::from_fn(|_| None);
    }
    let tokens = layout_tokens(&lines);
    let mut claimed = vec![false; tokens.len()];

    // Pass 1: the unit line anchors the result. In `lopdf` order the pair sits
    // either just above the title ("mIU/l 889 Prolactin") or just below the
    // phase rows ("1319 pmol/l" / "<0.30 U/l").
    for index in 0..tokens.len() {
        let Some(anchor) = unit_anchor(&tokens[index].text) else {
            continue;
        };
        let dash = matches!(anchor, UnitAnchor::Dash);
        // a dash between two numbers is a range separator, not a unit
        if dash && index >= 1 && token_has_digits(&tokens[index - 1].text) {
            continue;
        }
        let after = number_run_after(&tokens, index + 1);
        let before = if dash {
            None
        } else {
            number_run_before(&tokens, index)
        };
        let (run_start, run_end, value_before_unit) = match (after, before) {
            (Some(run), _) => (run.0, run.1, false),
            (None, Some(run)) => (run.0, run.1, true),
            (None, None) => continue,
        };
        if is_range_piece(&tokens, run_start, run_end) || is_phase_piece(&tokens, run_start, run_end) {
            continue;
        }
        let Some((digits, threshold)) = run_digits(&tokens, run_start, run_end) else {
            continue;
        };
        let owner = if value_before_unit {
            nearest_title_before(&hits, tokens[run_start].line)
        } else {
            nearest_title_after(&hits, tokens[run_end].line)
        };
        let Some(owner) = owner else {
            continue;
        };
        let hormone = hits[owner].hormone;
        if (dash && hormone != BandedHormone::Fai) || (!dash && hormone == BandedHormone::Fai) {
            continue;
        }
        if appears_in_section(&tokens, &hits, owner, &digits, (run_start, run_end), lines.len()) {
            continue;
        }
        let unit = match anchor {
            UnitAnchor::Measure(label) => Some(label.to_string()),
            UnitAnchor::Dash => None,
        };
        let score = if threshold { 2 } else { 3 };
        consider(&mut best[hormone.index()], score, OcrValue { value: digits, unit });
        for claimed_index in run_start..=run_end {
            claimed[claimed_index] = true;
        }
    }

    // Pass 2: whatever pass 1 missed is read out of the section body, taking
    // the one number that is neither a range bound nor a threshold glued to a
    // cycle phase. A result on a line of its own scores higher.
    for (hit_index, hit) in hits.iter().enumerate() {
        if best[hit.hormone.index()].is_some() {
            continue;
        }
        let (start, end) = section_tokens(&tokens, &hits, hit_index, lines.len());
        if start >= end {
            continue;
        }
        let section_unit = (start..end).find_map(|index| normalize_unit_token(&tokens[index].text));
        let mut index = start;
        while index < end {
            if claimed[index] {
                index += 1;
                continue;
            }
            let Some((run_start, run_end)) = number_run_after(&tokens, index) else {
                index += 1;
                continue;
            };
            index = run_end + 1;
            if run_end >= end || tokens[run_start].line != tokens[run_end].line {
                continue;
            }
            if is_range_piece(&tokens, run_start, run_end) || is_phase_piece(&tokens, run_start, run_end)
            {
                continue;
            }
            let Some((digits, threshold)) = run_digits(&tokens, run_start, run_end) else {
                continue;
            };
            if appears_in_section(
                &tokens,
                &hits,
                hit_index,
                &digits,
                (run_start, run_end),
                lines.len(),
            ) {
                continue;
            }
            // Inside a section body a threshold number is a band edge or a
            // phase bound, not a result: the real results that carry one are
            // printed next to their unit and were picked up by pass 1.
            if threshold
                && (section_has_band_labels(&tokens, start, end)
                    || (section_has_ranges(&tokens, start, end) && !digits.contains('.')))
            {
                continue;
            }
            let line = tokens[run_start].line;
            let alone = (run_start..=run_end).all(|other| tokens[other].line == line)
                && tokens.iter().filter(|token| token.line == line).count() == run_end - run_start + 1;
            let score = if threshold { 1 } else { 2 } + if alone { 1 } else { 0 };
            let unit = section_unit
                .or_else(|| hit.hormone.default_unit())
                .map(str::to_string);
            consider(
                &mut best[hit.hormone.index()],
                score,
                OcrValue {
                    value: digits,
                    unit,
                },
            );
        }
    }

    let mut values: [Option<OcrValue>; BANDED_HORMONE_COUNT] = std::array::from_fn(|_| None);
    for (slot, entry) in best.into_iter().enumerate() {
        values[slot] = entry.map(|(_, value)| value);
    }
    values
}

// ---------------------------------------------------------------------------
// Table reader (Sapphic Bison style result sheets)
// ---------------------------------------------------------------------------
//
// Other providers print a plain table: the marker name starts a line, then its
// result, then the unit, then the reference ranges and a flag:
//
//     Testosterone
//     15.2
//     nmol/L
//     Male
//     8.64 – 29.00
//     Female
//     0.29 – 1.67
//     High
//
// The reader takes the first number that is not part of a range and has a
// recognised unit within three tokens of it, and it stops at the next marker
// name, so a row whose result is missing leaves the field empty instead of
// borrowing the next row's number.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum TableTarget {
    Estradiol,
    Testosterone,
    Progesterone,
    Prolactin,
    Fsh,
    Lh,
    Shbg,
    Fai,
    Metabolic(MetabolicMarker),
}

const TABLE_MARKER_NAMES: [(&str, TableTarget); 32] = [
    ("17-beta oestradiol", TableTarget::Estradiol),
    ("17-beta estradiol", TableTarget::Estradiol),
    ("17 beta oestradiol", TableTarget::Estradiol),
    ("oestradiol", TableTarget::Estradiol),
    ("estradiol", TableTarget::Estradiol),
    ("testosterone", TableTarget::Testosterone),
    ("progesterone", TableTarget::Progesterone),
    ("prolactin", TableTarget::Prolactin),
    ("follicle stimulating hormone", TableTarget::Fsh),
    ("luteinising hormone", TableTarget::Lh),
    ("luteinizing hormone", TableTarget::Lh),
    ("sex hormone binding globulin", TableTarget::Shbg),
    ("free androgen index", TableTarget::Fai),
    (
        "total protein",
        TableTarget::Metabolic(MetabolicMarker::TotalProtein),
    ),
    ("albumin", TableTarget::Metabolic(MetabolicMarker::Albumin)),
    ("globulin", TableTarget::Metabolic(MetabolicMarker::Globulin)),
    (
        "alkaline phosphatase",
        TableTarget::Metabolic(MetabolicMarker::AlkalinePhosphatase),
    ),
    (
        "alanine transferase",
        TableTarget::Metabolic(MetabolicMarker::Alt),
    ),
    ("alt", TableTarget::Metabolic(MetabolicMarker::Alt)),
    (
        "aspartate transferase",
        TableTarget::Metabolic(MetabolicMarker::Ast),
    ),
    ("ast", TableTarget::Metabolic(MetabolicMarker::Ast)),
    ("gamma gt", TableTarget::Metabolic(MetabolicMarker::Ggt)),
    (
        "gamma-glutamyl transferase",
        TableTarget::Metabolic(MetabolicMarker::Ggt),
    ),
    ("ggt", TableTarget::Metabolic(MetabolicMarker::Ggt)),
    ("total bilirubin", TableTarget::Metabolic(MetabolicMarker::Bilirubin)),
    ("bilirubin", TableTarget::Metabolic(MetabolicMarker::Bilirubin)),
    (
        "non-hdl cholesterol",
        TableTarget::Metabolic(MetabolicMarker::NonHdlCholesterol),
    ),
    (
        "hdl cholesterol",
        TableTarget::Metabolic(MetabolicMarker::HdlCholesterol),
    ),
    (
        "ldl cholesterol",
        TableTarget::Metabolic(MetabolicMarker::LdlCholesterol),
    ),
    (
        "total cholesterol",
        TableTarget::Metabolic(MetabolicMarker::TotalCholesterol),
    ),
    (
        "triglycerides",
        TableTarget::Metabolic(MetabolicMarker::Triglyceride),
    ),
    (
        "triglyceride",
        TableTarget::Metabolic(MetabolicMarker::Triglyceride),
    ),
];

/// The longest marker name that starts `line`, so "Non-HDL cholesterol" wins
/// over "HDL cholesterol" and "Sex Hormone Binding Globulin" is not read as
/// "Globulin".
fn table_marker_at(line: &str) -> Option<TableTarget> {
    let trimmed = line.trim_start();
    let bytes = trimmed.as_bytes();
    let mut best: Option<(usize, TableTarget)> = None;
    for (name, target) in TABLE_MARKER_NAMES {
        if bytes.len() < name.len() || !bytes[..name.len()].eq_ignore_ascii_case(name.as_bytes()) {
            continue;
        }
        let rest = &trimmed[name.len()..];
        let boundary = rest.chars().next().is_none_or(|ch| !ch.is_alphanumeric());
        if !boundary {
            continue;
        }
        if best.is_none_or(|(length, _)| name.len() > length) {
            best = Some((name.len(), target));
        }
    }
    best.map(|(_, target)| target)
}

fn unit_within(tokens: &[LayoutToken], from: usize, count: usize) -> Option<String> {
    let end = (from + count).min(tokens.len());
    (from..end).find_map(|index| normalize_unit_token(&tokens[index].text).map(str::to_string))
}

/// Result and unit of the row that starts at `line_index`.
///
/// The first number after the name is the result: if no unit follows it, the
/// row has no result at all and the next row's number is never borrowed.
fn table_value_after(
    tokens: &[LayoutToken],
    lines: &[&str],
    line_index: usize,
    allow_unitless: bool,
) -> Option<OcrValue> {
    let start = tokens.iter().position(|token| token.line >= line_index)?;
    let mut index = start;
    let mut steps = 0;
    while index < tokens.len() && steps < 20 {
        if tokens[index].line > line_index && table_marker_at(lines[tokens[index].line]).is_some() {
            return None;
        }
        steps += 1;
        let Some((run_start, run_end)) = number_run_after(tokens, index) else {
            index += 1;
            continue;
        };
        let unit = unit_within(tokens, run_end + 1, 3);
        if unit.is_none() {
            // Free Androgen Index is printed without a unit; everything else
            // needs one to tell a result from a reference range.
            if !allow_unitless || tokens[run_start].line > line_index + 1 {
                return None;
            }
        }
        let (digits, threshold) = run_digits(tokens, run_start, run_end)?;
        if unit.is_none() && threshold {
            // "<0.3" next to "0.31 - 5.62" is a band edge, not a result.
            return None;
        }
        return Some(OcrValue {
            value: digits,
            unit,
        });
    }
    None
}

fn fill_missing_from_table(out: &mut OcrExtraction, table: &OcrExtraction) {
    if out.estradiol.is_none() {
        out.estradiol = table.estradiol.clone();
    }
    if out.testosterone.is_none() {
        out.testosterone = table.testosterone.clone();
    }
    if out.progesterone.is_none() {
        out.progesterone = table.progesterone.clone();
    }
    if out.fsh.is_none() {
        out.fsh = table.fsh.clone();
    }
    if out.lh.is_none() {
        out.lh = table.lh.clone();
    }
    if out.prolactin.is_none() {
        out.prolactin = table.prolactin.clone();
    }
    if out.shbg.is_none() {
        out.shbg = table.shbg.clone();
    }
    if out.fai.is_none() {
        out.fai = table.fai.clone();
    }
}

fn extract_marker_table(text: &str) -> OcrExtraction {
    let mut out = OcrExtraction::default();
    let lines: Vec<&str> = text.lines().collect();
    let tokens = layout_tokens(&lines);
    for (line_index, line) in lines.iter().enumerate() {
        let Some(target) = table_marker_at(line) else {
            continue;
        };
        if let TableTarget::Metabolic(marker) = target {
            if marker.get(&out.metabolic).is_some() {
                continue;
            }
        }
        let Some(value) = table_value_after(
            &tokens,
            &lines,
            line_index,
            matches!(target, TableTarget::Fai),
        ) else {
            continue;
        };
        if matches!(target, TableTarget::Fai) && value.unit.is_some() {
            continue;
        }
        let TableTarget::Metabolic(marker) = target else {
            let Some(slot) = table_hormone_slot(&mut out, target) else {
                continue;
            };
            if slot.is_none() {
                *slot = Some(value);
            }
            continue;
        };
        // A liver or lipid marker is only believed in the unit it is reported
        // in; anything else would need a conversion this app does not carry.
        if value.unit.as_deref() != Some(marker.unit()) {
            continue;
        }
        if let Some(number) = parse_decimal(&value.value) {
            marker.set(&mut out.metabolic, Some(number));
        }
    }
    out
}

fn table_hormone_slot(out: &mut OcrExtraction, target: TableTarget) -> Option<&mut Option<OcrValue>> {
    let slot = match target {
        TableTarget::Estradiol => &mut out.estradiol,
        TableTarget::Testosterone => &mut out.testosterone,
        TableTarget::Progesterone => &mut out.progesterone,
        TableTarget::Prolactin => &mut out.prolactin,
        TableTarget::Fsh => &mut out.fsh,
        TableTarget::Lh => &mut out.lh,
        TableTarget::Shbg => &mut out.shbg,
        TableTarget::Fai => &mut out.fai,
        TableTarget::Metabolic(_) => return None,
    };
    Some(slot)
}

fn table_hormone_count(extraction: &OcrExtraction) -> usize {
    [
        &extraction.estradiol,
        &extraction.testosterone,
        &extraction.progesterone,
        &extraction.prolactin,
        &extraction.fsh,
        &extraction.lh,
        &extraction.shbg,
        &extraction.fai,
    ]
    .iter()
    .filter(|slot| slot.is_some())
    .count()
}

fn month_from_name(name: &str) -> Option<u32> {
    match name {
        "jan" | "january" => Some(1),
        "feb" | "february" => Some(2),
        "mar" | "march" => Some(3),
        "apr" | "april" => Some(4),
        "may" => Some(5),
        "jun" | "june" => Some(6),
        "jul" | "july" => Some(7),
        "aug" | "august" => Some(8),
        "sep" | "september" => Some(9),
        "oct" | "october" => Some(10),
        "nov" | "november" => Some(11),
        "dec" | "december" => Some(12),
        _ => None,
    }
}

fn parse_time_token(token: &str) -> Option<(u32, u32)> {
    let (hours, minutes) = token.trim().split_once(':')?;
    let hours: u32 = hours.trim().parse().ok()?;
    let minutes: u32 = minutes.trim().parse().ok()?;
    if hours < 24 && minutes < 60 {
        Some((hours, minutes))
    } else {
        None
    }
}

fn valid_ymd(year: u32, month: u32, day: u32) -> Option<(u32, u32, u32)> {
    if (1900..2200).contains(&year) && (1..=12).contains(&month) && (1..=31).contains(&day) {
        Some((year, month, day))
    } else {
        None
    }
}

/// One token holding the whole date: DD/MM/YYYY, DD-MM-YYYY, YYYY-MM-DD,
/// DD.MM.YYYY or the day-first DD-Mon-YYYY this report uses ("26-Oct-2026").
fn parse_joined_date(token: &str) -> Option<(u32, u32, u32)> {
    let trimmed = token.trim().trim_end_matches(|c: char| c == ',' || c == '.' || c == ';');
    for separator in ['/', '-', '.'] {
        let parts: Vec<&str> = trimmed.split(separator).filter(|part| !part.is_empty()).collect();
        if parts.len() != 3 {
            continue;
        }
        if let Some(month) = month_from_name(&parts[1].to_lowercase()) {
            let (Ok(day), Ok(year)) = (parts[0].parse::<u32>(), parts[2].parse::<u32>()) else {
                continue;
            };
            if let Some(date) = valid_ymd(year, month, day) {
                return Some(date);
            }
            continue;
        }
        let numbers: Option<Vec<u32>> = parts.iter().map(|part| part.parse::<u32>().ok()).collect();
        let Some(numbers) = numbers else {
            continue;
        };
        let (first, middle, last) = (numbers[0], numbers[1], numbers[2]);
        let candidate = if last > 100 {
            // DD/MM/YYYY
            valid_ymd(last, middle, first)
        } else if first > 100 {
            // YYYY/MM/DD
            valid_ymd(first, middle, last)
        } else {
            None
        };
        if candidate.is_some() {
            return candidate;
        }
    }
    None
}

/// The date at the head of `tokens`, written either as one token
/// ("26-Oct-2026", "15/03/2025") or as day, month name, year ("15 Mar 2025").
fn parse_date_parts(tokens: &[&str]) -> Option<(u32, u32, u32)> {
    let first = tokens.first()?.trim();
    if let Some(date) = parse_joined_date(first) {
        return Some(date);
    }
    if tokens.len() >= 3 {
        let Ok(day) = tokens[0].trim().parse::<u32>() else {
            return None;
        };
        let month = month_from_name(&tokens[1].trim().to_lowercase())?;
        let year: u32 = tokens[2]
            .trim()
            .trim_end_matches(|c: char| !c.is_ascii_digit())
            .parse()
            .ok()?;
        return valid_ymd(year, month, day);
    }
    None
}

/// `YYYY-MM-DDTHH:MM` for the datetime-local input. The time comes from the
/// report when it prints one (the banded layout prints "26-Oct-2026 09:15")
/// and is noon otherwise, which is what the parser has always filled.
fn try_parse_date(tokens: &[&str]) -> Option<String> {
    let (year, month, day) = parse_date_parts(tokens)?;
    let (hours, minutes) = tokens
        .iter()
        .skip(1)
        .find_map(|token| parse_time_token(token))
        .unwrap_or((12, 0));
    Some(format!(
        "{year:04}-{month:02}-{day:02}T{hours:02}:{minutes:02}"
    ))
}

fn extract_sample_date(text: &str) -> Option<String> {
    // The banded layout splits the label over several lines ("Sample" /
    // "Collection" / "Date"), so match against whitespace-collapsed text.
    let collapsed = collapse_whitespace(text);
    let labels = [
        "sample collection date",
        "collection date",
        "date collected",
        "date of collection",
        "sample date",
        "specimen collection date",
        "specimen date",
        "collected on",
        "collected date",
        "date of sample",
        "collected",
    ];
    for label in labels {
        let Some(index) = find_ci(&collapsed, label) else {
            continue;
        };
        let window = take_chars(&collapsed, index + label.len(), 80);
        let trimmed = window.trim_start_matches([':', ' ', '\t']);
        let tokens: Vec<&str> = trimmed.split_whitespace().collect();
        if let Some(date) = try_parse_date(&tokens) {
            return Some(date);
        }
    }
    None
}

fn extract_ocr_values(text: &str) -> OcrExtraction {
    let cleaned = text.replace('\r', "\n");
    let sample_date = extract_sample_date(&cleaned);
    let banded = extract_banded_values(&cleaned);
    let banded_found = banded.iter().any(|value| value.is_some());
    let table = extract_marker_table(&cleaned);
    let table_hormones = table_hormone_count(&table);
    if !table.metabolic.is_empty() && table_hormones >= 1 {
        // A wider kit report is a result table: its rows are the source of
        // truth, and the banded reader would give a row's value to whichever
        // title happens to sit above it.
        return OcrExtraction {
            sample_date,
            ..table
        };
    }
    if banded_found {
        // A banded report is read section by section. The flat "first number
        // after the label" scan would happily take a phase range, a band edge
        // or a page number out of the prose, so it is not used at all here;
        // the table reader only fills what the sections missed.
        let mut out = OcrExtraction {
            estradiol: banded[BandedHormone::Estradiol.index()].clone(),
            testosterone: banded[BandedHormone::Testosterone.index()].clone(),
            progesterone: banded[BandedHormone::Progesterone.index()].clone(),
            fsh: banded[BandedHormone::Fsh.index()].clone(),
            lh: banded[BandedHormone::Lh.index()].clone(),
            prolactin: banded[BandedHormone::Prolactin.index()].clone(),
            shbg: banded[BandedHormone::Shbg.index()].clone(),
            fai: banded[BandedHormone::Fai.index()].clone(),
            sample_date,
            metabolic: table.metabolic.clone(),
        };
        fill_missing_from_table(&mut out, &table);
        return out;
    }
    if table_hormones >= 3 || (!table.metabolic.is_empty() && table_hormones == 0) {
        // A result table: its rows are trusted over the flat scan, which can
        // wander into the reference ranges printed next to the result.
        return OcrExtraction {
            sample_date,
            ..table
        };
    }
    let mut out = OcrExtraction {
        estradiol: extract_ocr_value(&cleaned, &["oestradiol", "estradiol"]),
        testosterone: extract_ocr_value(&cleaned, &["testosterone"]),
        progesterone: extract_ocr_value(&cleaned, &["progesterone"]),
        fsh: extract_ocr_value(
            &cleaned,
            &[
                "follicle stimulating hormone",
                "follicle-stimulating hormone",
                "fsh",
            ],
        ),
        lh: extract_ocr_value(
            &cleaned,
            &["luteinising hormone", "luteinizing hormone", "lh"],
        ),
        prolactin: extract_ocr_value(&cleaned, &["prolactin"]),
        shbg: extract_ocr_value(&cleaned, &["sex hormone binding globulin", "shbg"]),
        fai: extract_ocr_value(&cleaned, &["free androgen index", "fai"]),
        sample_date,
        metabolic: table.metabolic.clone(),
    };
    fill_missing_from_table(&mut out, &table);
    out
}

#[component]
pub fn CreateBloodTest() -> impl IntoView {
    let store = use_store();
    let unit_options = UNIT_OPTIONS
        .iter()
        .map(|unit| UnitOption {
            label: hormone_unit_label(unit).to_string(),
        })
        .collect::<Vec<_>>();

    let test_date_time = create_rw_signal(to_local_input_value(Date::now() as i64));
    let estradiol_level = create_rw_signal("0".to_string());
    let default_e2_unit = store
        .settings
        .get()
        .displayEstradiolUnit
        .unwrap_or(HormoneUnits::E2PmolL);
    let estradiol_unit = create_rw_signal(hormone_unit_label(&default_e2_unit).to_string());
    let estrannaise_number = create_rw_signal("0".to_string());
    let estrannaise_unit = create_rw_signal(hormone_unit_label(&HormoneUnits::E2PgMl).to_string());
    let test_level = create_rw_signal("0".to_string());
    let test_unit = create_rw_signal(hormone_unit_label(&HormoneUnits::TNmolL).to_string());
    let progesterone_level = create_rw_signal("0".to_string());
    let progesterone_unit = create_rw_signal(hormone_unit_label(&HormoneUnits::TNmolL).to_string());
    let fsh_level = create_rw_signal("0".to_string());
    let fsh_unit = create_rw_signal(hormone_unit_label(&HormoneUnits::UL).to_string());
    let lh_level = create_rw_signal("0".to_string());
    let lh_unit = create_rw_signal(hormone_unit_label(&HormoneUnits::UL).to_string());
    let prolactin_level = create_rw_signal("0".to_string());
    let prolactin_unit = create_rw_signal(hormone_unit_label(&HormoneUnits::MIuL).to_string());
    let shbg_level = create_rw_signal("0".to_string());
    let shbg_unit = create_rw_signal(hormone_unit_label(&HormoneUnits::TNmolL).to_string());
    let free_androgen_index = create_rw_signal("0".to_string());
    let notes = create_rw_signal(String::new());
    let metabolic = MetabolicInputs::new();
    let ocr_busy = create_rw_signal(false);
    let ocr_status = create_rw_signal(String::new());
    let ocr_error = create_rw_signal(None::<String>);
    let ocr_input_ref: NodeRef<html::Input> = create_node_ref();
    let pdf_busy = create_rw_signal(false);
    let pdf_status = create_rw_signal(String::new());
    let pdf_error = create_rw_signal(None::<String>);
    let pdf_input_ref: NodeRef<html::Input> = create_node_ref();
    let attached_pdf_files = create_rw_signal(Vec::<String>::new());
    let show_feedback = create_rw_signal(false);
    let feedback_timeout: Rc<RefCell<Option<Timeout>>> = Rc::new(RefCell::new(None));

    let open_ocr_picker = {
        let ocr_input_ref = ocr_input_ref;
        move |_| {
            if ocr_busy.get() {
                return;
            }
            if let Some(input) = ocr_input_ref.get() {
                input.click();
            }
        }
    };

    let open_pdf_picker = {
        let pdf_input_ref = pdf_input_ref;
        move |_| {
            if pdf_busy.get() {
                return;
            }
            if let Some(input) = pdf_input_ref.get() {
                input.click();
            }
        }
    };

    let on_ocr_change = {
        let estradiol_level = estradiol_level;
        let estradiol_unit = estradiol_unit;
        let test_level = test_level;
        let test_unit = test_unit;
        let progesterone_level = progesterone_level;
        let progesterone_unit = progesterone_unit;
        let fsh_level = fsh_level;
        let fsh_unit = fsh_unit;
        let lh_level = lh_level;
        let lh_unit = lh_unit;
        let prolactin_level = prolactin_level;
        let prolactin_unit = prolactin_unit;
        let shbg_level = shbg_level;
        let shbg_unit = shbg_unit;
        let free_androgen_index = free_androgen_index;
        let test_date_time = test_date_time;
        let metabolic = metabolic;
        move |ev: leptos::ev::Event| {
            if ocr_busy.get() {
                return;
            }
            let input: HtmlInputElement = event_target(&ev);
            let Some(files) = input.files() else {
                return;
            };
            let Some(file) = files.get(0) else {
                return;
            };
            let Ok(reader) = FileReader::new() else {
                ocr_error.set(Some("Could not read the image file.".to_string()));
                ocr_status.set(String::new());
                return;
            };
            let input_clone = input.clone();
            let input_reset = input.clone();
            let reader_clone = reader.clone();
            ocr_busy.set(true);
            ocr_error.set(None);
            ocr_status.set("Reading image...".to_string());
            let onloadend = Closure::wrap(Box::new(move |_ev: web_sys::Event| {
                let input_for_task = input_clone.clone();
                let data_url = reader_clone
                    .result()
                    .ok()
                    .and_then(|value| value.as_string())
                    .unwrap_or_default();
                if data_url.trim().is_empty() {
                    ocr_error.set(Some("Failed to read image data.".to_string()));
                    ocr_status.set(String::new());
                    ocr_busy.set(false);
                    input_clone.set_value("");
                    return;
                }
                let tesseract_loaded =
                    Reflect::has(&js_sys::global(), &JsValue::from_str("Tesseract"))
                        .unwrap_or(false);
                if !tesseract_loaded {
                    ocr_error.set(Some(
                        "OCR library did not load. Try refreshing.".to_string(),
                    ));
                    ocr_status.set(String::new());
                    ocr_busy.set(false);
                    input_clone.set_value("");
                    return;
                }
                ocr_status.set("Running OCR...".to_string());
                spawn_local(async move {
                    let options = Object::new();
                    let _ = Reflect::set(
                        &options,
                        &JsValue::from_str("workerPath"),
                        &JsValue::from_str("/ocr/worker.min.js"),
                    );
                    let _ = Reflect::set(
                        &options,
                        &JsValue::from_str("corePath"),
                        &JsValue::from_str("/ocr/tesseract-core.wasm.js"),
                    );
                    let _ = Reflect::set(
                        &options,
                        &JsValue::from_str("langPath"),
                        &JsValue::from_str("/ocr"),
                    );
                    let result = JsFuture::from(tesseract_recognize(
                        JsValue::from_str(&data_url),
                        "eng",
                        options.into(),
                    ))
                    .await;
                    let value = match result {
                        Ok(value) => value,
                        Err(_) => {
                            ocr_error.set(Some("OCR failed to process the image.".to_string()));
                            ocr_status.set(String::new());
                            ocr_busy.set(false);
                            input_for_task.set_value("");
                            return;
                        }
                    };
                    let text = Reflect::get(&value, &JsValue::from_str("data"))
                        .ok()
                        .and_then(|data| Reflect::get(&data, &JsValue::from_str("text")).ok())
                        .and_then(|value| value.as_string())
                        .unwrap_or_default();
                    if text.trim().is_empty() {
                        ocr_error.set(Some("OCR did not find any text.".to_string()));
                        ocr_status.set(String::new());
                        ocr_busy.set(false);
                        input_for_task.set_value("");
                        return;
                    }
                    let extracted = extract_ocr_values(&text);
                    let filled = apply_ocr_extraction(
                        extracted,
                        estradiol_level,
                        estradiol_unit,
                        test_level,
                        test_unit,
                        progesterone_level,
                        progesterone_unit,
                        fsh_level,
                        fsh_unit,
                        lh_level,
                        lh_unit,
                        prolactin_level,
                        prolactin_unit,
                        shbg_level,
                        shbg_unit,
                        free_androgen_index,
                        test_date_time,
                        metabolic,
                    );
                    if filled == 0 {
                        ocr_error.set(Some("OCR ran, but no lab values were found.".to_string()));
                        ocr_status.set(String::new());
                    } else {
                        ocr_status.set(format!(
                            "Autofilled {filled} fields from OCR. Review before saving."
                        ));
                    }
                    ocr_busy.set(false);
                    input_for_task.set_value("");
                });
            }) as Box<dyn FnMut(_)>);
            reader.set_onloadend(Some(onloadend.as_ref().unchecked_ref()));
            if reader.read_as_data_url(&file).is_err() {
                ocr_error.set(Some("Failed to read the image file.".to_string()));
                ocr_status.set(String::new());
                ocr_busy.set(false);
                input_reset.set_value("");
                return;
            }
            onloadend.forget();
        }
    };

    let on_pdf_change = {
        let attached_pdf_files = attached_pdf_files;
        let estradiol_level = estradiol_level;
        let estradiol_unit = estradiol_unit;
        let test_level = test_level;
        let test_unit = test_unit;
        let progesterone_level = progesterone_level;
        let progesterone_unit = progesterone_unit;
        let fsh_level = fsh_level;
        let fsh_unit = fsh_unit;
        let lh_level = lh_level;
        let lh_unit = lh_unit;
        let prolactin_level = prolactin_level;
        let prolactin_unit = prolactin_unit;
        let shbg_level = shbg_level;
        let shbg_unit = shbg_unit;
        let free_androgen_index = free_androgen_index;
        let test_date_time = test_date_time;
        let metabolic = metabolic;
        move |ev: leptos::ev::Event| {
            if pdf_busy.get() {
                return;
            }
            let input: HtmlInputElement = event_target(&ev);
            let Some(files) = input.files() else {
                return;
            };
            let file_list: Vec<_> = (0..files.length())
                .filter_map(|index| files.get(index))
                .collect();
            if file_list.is_empty() {
                return;
            }
            let input_clone = input.clone();
            pdf_busy.set(true);
            pdf_error.set(None);
            pdf_status.set("Uploading PDF...".to_string());
            spawn_local(async move {
                let form = match FormData::new() {
                    Ok(form) => form,
                    Err(_) => {
                        pdf_error.set(Some("Could not create upload payload.".to_string()));
                        pdf_status.set(String::new());
                        pdf_busy.set(false);
                        input_clone.set_value("");
                        return;
                    }
                };
                for file in file_list {
                    if form
                        .append_with_blob_and_filename("file", &file, &file.name())
                        .is_err()
                    {
                        continue;
                    }
                }
                let request = match Request::post("/api/bloodtest-pdf").body(form) {
                    Ok(request) => request,
                    Err(_) => {
                        pdf_error.set(Some("Could not prepare PDF upload request.".to_string()));
                        pdf_status.set(String::new());
                        pdf_busy.set(false);
                        input_clone.set_value("");
                        return;
                    }
                };
                let response = match request.send().await {
                    Ok(response) => response,
                    Err(_) => {
                        pdf_error.set(Some("PDF upload failed.".to_string()));
                        pdf_status.set(String::new());
                        pdf_busy.set(false);
                        input_clone.set_value("");
                        return;
                    }
                };
                if !response.ok() {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    if body.trim().is_empty() {
                        pdf_error.set(Some(format!("PDF upload failed ({status}).")));
                    } else {
                        pdf_error.set(Some(format!(
                            "PDF upload failed ({status}): {}",
                            body.trim()
                        )));
                    }
                    pdf_status.set(String::new());
                    pdf_busy.set(false);
                    input_clone.set_value("");
                    return;
                }
                let payload = match response.json::<PdfUploadResponse>().await {
                    Ok(payload) => payload,
                    Err(_) => {
                        pdf_error.set(Some("Failed to parse PDF upload response.".to_string()));
                        pdf_status.set(String::new());
                        pdf_busy.set(false);
                        input_clone.set_value("");
                        return;
                    }
                };

                if payload.files.is_empty() {
                    pdf_error.set(Some("No PDF files were uploaded.".to_string()));
                    pdf_status.set(String::new());
                    pdf_busy.set(false);
                    input_clone.set_value("");
                    return;
                }

                let mut files = attached_pdf_files.get();
                let mut uploaded = 0;
                let mut filled = 0;
                let mut extract_failures = 0;
                for item in payload.files {
                    if !files.iter().any(|name| name == &item.filename) {
                        files.push(item.filename.clone());
                        uploaded += 1;
                    }
                    if let Some(text) = item.text {
                        let extracted = extract_ocr_values(&text);
                        filled += apply_ocr_extraction(
                            extracted,
                            estradiol_level,
                            estradiol_unit,
                            test_level,
                            test_unit,
                            progesterone_level,
                            progesterone_unit,
                            fsh_level,
                            fsh_unit,
                            lh_level,
                            lh_unit,
                            prolactin_level,
                            prolactin_unit,
                            shbg_level,
                            shbg_unit,
                            free_androgen_index,
                            test_date_time,
                            metabolic,
                        );
                    }
                    if item.extract_error.is_some() {
                        extract_failures += 1;
                    }
                }
                attached_pdf_files.set(files);

                let mut status_chunks = Vec::new();
                if uploaded > 0 {
                    status_chunks.push(format!(
                        "Attached {uploaded} PDF{}.",
                        if uploaded == 1 { "" } else { "s" }
                    ));
                }
                if filled > 0 {
                    status_chunks.push(format!(
                        "Autofilled {filled} field{} from PDF text.",
                        if filled == 1 { "" } else { "s" }
                    ));
                }
                if extract_failures > 0 {
                    status_chunks.push(format!(
                        "{extract_failures} PDF{} had no extractable text.",
                        if extract_failures == 1 { "" } else { "s" }
                    ));
                }

                if status_chunks.is_empty() {
                    pdf_status.set("PDF upload complete.".to_string());
                } else {
                    pdf_status.set(status_chunks.join(" "));
                }
                pdf_error.set(None);
                pdf_busy.set(false);
                input_clone.set_value("");
            });
        }
    };

    let on_submit = {
        let feedback_timeout = feedback_timeout.clone();
        let attached_pdf_files = attached_pdf_files;
        let metabolic = metabolic;
        move |ev: leptos::ev::SubmitEvent| {
            ev.prevent_default();
            let date = parse_datetime_local(&test_date_time.get());

            let estradiol_value = parse_optional(&estradiol_level.get());
            let estrannaise_value = parse_optional(&estrannaise_number.get());
            let test_value = parse_optional(&test_level.get());
            let progesterone_value = parse_optional(&progesterone_level.get());
            let fsh_value = parse_optional(&fsh_level.get());
            let lh_value = parse_optional(&lh_level.get());
            let prolactin_value = parse_optional(&prolactin_level.get());
            let shbg_value = parse_optional(&shbg_level.get());
            let free_androgen_value = parse_optional(&free_androgen_index.get());

            let default_e2_unit = store
                .settings
                .get()
                .displayEstradiolUnit
                .unwrap_or(HormoneUnits::E2PmolL);
            let estradiol_unit_value = unit_or_default(&estradiol_unit.get(), default_e2_unit);
            let test_unit_value = unit_or_default(&test_unit.get(), HormoneUnits::TNmolL);
            let progesterone_unit_value =
                unit_or_default(&progesterone_unit.get(), HormoneUnits::TNmolL);
            let fsh_unit_value = unit_or_default(&fsh_unit.get(), HormoneUnits::UL);
            let lh_unit_value = unit_or_default(&lh_unit.get(), HormoneUnits::UL);
            let prolactin_unit_value = unit_or_default(&prolactin_unit.get(), HormoneUnits::MIuL);
            let shbg_unit_value = unit_or_default(&shbg_unit.get(), HormoneUnits::TNmolL);

            let estrannaise_unit_value =
                unit_or_default(&estrannaise_unit.get(), HormoneUnits::E2PgMl);
            let measured_e2 = estradiol_value.map(|value| {
                if estradiol_unit_value == HormoneUnits::E2PmolL {
                    value / 3.671
                } else {
                    value
                }
            });
            let predicted_input = estrannaise_value.map(|value| {
                if estrannaise_unit_value == HormoneUnits::E2PmolL {
                    value / 3.671
                } else {
                    value
                }
            });
            let predicted_model = predict_e2_pg_ml(&store.data.get(), date);
            let fudge_factor =
                compute_fudge_factor(measured_e2, predicted_model.or(predicted_input));
            let pdf_files = attached_pdf_files.get();
            let next_pdf_files = pdf_files.clone();
            let metabolic_values = metabolic.panel();
            let estrogen_type = latest_estrogen_route(&store.data.get());

            let entry = BloodTest {
                date,
                estradiolLevel: estradiol_value,
                testLevel: test_value,
                estradiolUnit: Some(estradiol_unit_value),
                testUnit: Some(test_unit_value),
                progesteroneLevel: progesterone_value,
                progesteroneUnit: Some(progesterone_unit_value),
                fshLevel: fsh_value,
                fshUnit: Some(fsh_unit_value),
                lhLevel: lh_value,
                lhUnit: Some(lh_unit_value),
                prolactinLevel: prolactin_value,
                prolactinUnit: Some(prolactin_unit_value),
                shbgLevel: shbg_value,
                shbgUnit: Some(shbg_unit_value),
                freeAndrogenIndex: free_androgen_value,
                estrannaiseNumber: predicted_input,
                fudgeFactor: fudge_factor,
                notes: if notes.get().trim().is_empty() {
                    None
                } else {
                    Some(notes.get())
                },
                estrogenType: estrogen_type,
                pdfFiles: if pdf_files.is_empty() {
                    None
                } else {
                    Some(pdf_files)
                },
                metabolic: metabolic_values,
            };

            let mut removed_pdf_files = Vec::new();
            store.data.update(|d| {
                if let Some(existing) = d.bloodTests.iter_mut().find(|item| item.date == date) {
                    let previous_pdf_files = existing.pdfFiles.clone().unwrap_or_default();
                    removed_pdf_files = previous_pdf_files
                        .iter()
                        .filter(|name| !next_pdf_files.iter().any(|candidate| candidate == *name))
                        .cloned()
                        .collect();
                    let mut replacement = entry.clone();
                    if replacement.metabolic.is_none() {
                        // A blank panel in the form must not erase values an
                        // earlier import recorded.
                        replacement.metabolic = existing.metabolic.clone();
                    }
                    *existing = replacement;
                } else {
                    d.bloodTests.push(entry.clone());
                }
            });
            if !removed_pdf_files.is_empty() {
                spawn_local(async move {
                    for filename in removed_pdf_files {
                        let _ = Request::delete(&format!(
                            "/api/bloodtest-pdf/{}",
                            urlencoding::encode(&filename)
                        ))
                        .send()
                        .await;
                    }
                });
            }
            store.mark_dirty();

            show_feedback.set(true);
            if let Some(existing) = feedback_timeout.borrow_mut().take() {
                drop(existing);
            }
            let show_feedback = show_feedback;
            *feedback_timeout.borrow_mut() = Some(Timeout::new(3000, move || {
                show_feedback.set(false);
            }));
        }
    };

    let render_marker_input = move |marker: MetabolicMarker| {
        let signal = metabolic.signal(marker);
        let (low, high) = marker.typical_range();
        let hint = match (low, high) {
            (Some(low), Some(high)) => format!("typical {low} to {high}"),
            (Some(low), None) => format!("typical above {low}"),
            (None, Some(high)) => format!("typical below {high}"),
            (None, None) => String::new(),
        };
        let label = format!("{} ({})", marker.label(), marker.unit());
        view! {
            <label>
                {label}
                <input
                    type="text"
                    step="any"
                    on:input=move |ev| signal.set(event_target_value(&ev))
                    prop:value=move || signal.get()
                />
                <span class="muted">{hint}</span>
            </label>
        }
    };

    let render_unit_options = move || {
        unit_options
            .clone()
            .into_iter()
            .map(|option| {
                let label = option.label.clone();
                view! { <option value=label.clone()>{label}</option> }
            })
            .collect_view()
    };

    page_layout(
        "Create Blood Test",
        view! {
            <div class="view-layout">
                <div class="view-header">
                    <div>
                        <h2>"Create blood test entry"</h2>
                        <p class="muted">"Record labs and estrannaise data in one place."</p>
                    </div>
                    <div class="header-actions">
                        <A href="/backup">"View all tests"</A>
                    </div>
                </div>

                <form class="form-wide" on:submit=on_submit>
                    <label>
                        "Test date / time"
                        <input
                            type="datetime-local"
                            on:input=move |ev| test_date_time.set(event_target_value(&ev))
                            prop:value=move || test_date_time.get()
                        />
                    </label>

                    <div class="form-section">
                        <h3>"Import lab report"</h3>
                        <div class="inline-equal">
                            <div>
                                <input
                                    type="file"
                                    accept="image/*"
                                    node_ref=ocr_input_ref
                                    on:change=on_ocr_change
                                    class="hidden-input"
                                />
                                <button
                                    type="button"
                                    on:click=open_ocr_picker
                                    prop:disabled=move || ocr_busy.get()
                                >
                                    {move || if ocr_busy.get() { "Running OCR..." } else { "Upload lab screenshot" }}
                                </button>
                                <p class="muted">"PNG/JPEG/WEBP · Cropped results work best."</p>
                            </div>
                            <div>
                                <Show when=move || !ocr_status.get().is_empty()>
                                    <p class="muted">{move || ocr_status.get()}</p>
                                </Show>
                                <Show when=move || ocr_error.get().is_some()>
                                    <p class="muted">{move || ocr_error.get().unwrap_or_default()}</p>
                                </Show>
                            </div>
                        </div>
                        <div class="inline-equal">
                            <div>
                                <input
                                    type="file"
                                    accept="application/pdf,.pdf"
                                    multiple
                                    node_ref=pdf_input_ref
                                    on:change=on_pdf_change
                                    class="hidden-input"
                                />
                                <button
                                    type="button"
                                    on:click=open_pdf_picker
                                    prop:disabled=move || pdf_busy.get()
                                >
                                    {move || if pdf_busy.get() { "Uploading PDF..." } else { "Upload lab PDF" }}
                                </button>
                                <p class="muted">"Encrypted PDFs use the password from Settings & Backup."</p>
                            </div>
                            <div>
                                <Show when=move || !pdf_status.get().is_empty()>
                                    <p class="muted">{move || pdf_status.get()}</p>
                                </Show>
                                <Show when=move || pdf_error.get().is_some()>
                                    <p class="muted">{move || pdf_error.get().unwrap_or_default()}</p>
                                </Show>
                            </div>
                        </div>
                        <Show when=move || !attached_pdf_files.get().is_empty()>
                            <div class="photo-actions">
                                <p class="muted">"Attached report PDFs"</p>
                                <ul class="history-list">
                                    <For
                                        each=move || attached_pdf_files.get()
                                        key=|filename| filename.clone()
                                        children=move |filename| {
                                            let open_href = format!(
                                                "/api/bloodtest-pdf/{}",
                                                urlencoding::encode(&filename)
                                            );
                                            let file_for_remove = filename.clone();
                                            view! {
                                                <li class="history-item">
                                                    <a href=open_href target="_blank" rel="noopener noreferrer">
                                                        {filename.clone()}
                                                    </a>
                                                    <button
                                                        type="button"
                                                        class="action-button"
                                                        on:click=move |_| {
                                                            let remove_name = file_for_remove.clone();
                                                            attached_pdf_files
                                                                .update(|files| files.retain(|item| item != &remove_name));
                                                            spawn_local(async move {
                                                                let _ = Request::delete(&format!(
                                                                    "/api/bloodtest-pdf/{}",
                                                                    urlencoding::encode(&remove_name)
                                                                ))
                                                                .send()
                                                                .await;
                                                            });
                                                        }
                                                    >
                                                        "Remove"
                                                    </button>
                                                </li>
                                            }
                                        }
                                    />
                                </ul>
                            </div>
                        </Show>
                    </div>

                    <div class="form-section">
                        <h3>"Hormone levels"</h3>

                        <div class="inline-equal">
                            <label>
                                "Estradiol level"
                                <input
                                    type="text"
                                    step="any"
                                    on:input=move |ev| estradiol_level.set(event_target_value(&ev))
                                    prop:value=move || estradiol_level.get()
                                />
                            </label>
                            <label>
                                "Estradiol unit"
                                <select
                                    on:change=move |ev| estradiol_unit.set(event_target_value(&ev))
                                    prop:value=move || estradiol_unit.get()
                                >
                                    {render_unit_options()}
                                </select>
                            </label>
                        </div>

                        <div class="inline-equal">
                            <label>
                                "Estrannaise predicted E2"
                                <input
                                    type="text"
                                    step="any"
                                    on:input=move |ev| estrannaise_number.set(event_target_value(&ev))
                                    prop:value=move || estrannaise_number.get()
                                />
                            </label>
                            <label>
                                "Predicted unit"
                                <select
                                    on:change=move |ev| estrannaise_unit.set(event_target_value(&ev))
                                    prop:value=move || estrannaise_unit.get()
                                >
                                    <option value="pg/mL">"pg/mL"</option>
                                    <option value="pmol/L">"pmol/L"</option>
                                </select>
                            </label>
                        </div>

                        <div class="inline-equal">
                            <label>
                                "Testosterone level"
                                <input
                                    type="text"
                                    step="any"
                                    on:input=move |ev| test_level.set(event_target_value(&ev))
                                    prop:value=move || test_level.get()
                                />
                            </label>
                            <label>
                                "Testosterone unit"
                                <select
                                    on:change=move |ev| test_unit.set(event_target_value(&ev))
                                    prop:value=move || test_unit.get()
                                >
                                    {render_unit_options()}
                                </select>
                            </label>
                        </div>

                        <div class="inline-equal">
                            <label>
                                "Progesterone level"
                                <input
                                    type="text"
                                    step="any"
                                    on:input=move |ev| progesterone_level.set(event_target_value(&ev))
                                    prop:value=move || progesterone_level.get()
                                />
                            </label>
                            <label>
                                "Progesterone unit"
                                <select
                                    on:change=move |ev| progesterone_unit.set(event_target_value(&ev))
                                    prop:value=move || progesterone_unit.get()
                                >
                                    {render_unit_options()}
                                </select>
                            </label>
                        </div>

                        <div class="inline-equal">
                            <label>
                                "FSH level"
                                <input
                                    type="text"
                                    step="any"
                                    on:input=move |ev| fsh_level.set(event_target_value(&ev))
                                    prop:value=move || fsh_level.get()
                                />
                            </label>
                            <label>
                                "FSH unit"
                                <select
                                    on:change=move |ev| fsh_unit.set(event_target_value(&ev))
                                    prop:value=move || fsh_unit.get()
                                >
                                    {render_unit_options()}
                                </select>
                            </label>
                        </div>

                        <div class="inline-equal">
                            <label>
                                "LH level"
                                <input
                                    type="text"
                                    step="any"
                                    on:input=move |ev| lh_level.set(event_target_value(&ev))
                                    prop:value=move || lh_level.get()
                                />
                            </label>
                            <label>
                                "LH unit"
                                <select
                                    on:change=move |ev| lh_unit.set(event_target_value(&ev))
                                    prop:value=move || lh_unit.get()
                                >
                                    {render_unit_options()}
                                </select>
                            </label>
                        </div>

                        <div class="inline-equal">
                            <label>
                                "Prolactin level"
                                <input
                                    type="text"
                                    step="any"
                                    on:input=move |ev| prolactin_level.set(event_target_value(&ev))
                                    prop:value=move || prolactin_level.get()
                                />
                            </label>
                            <label>
                                "Prolactin unit"
                                <select
                                    on:change=move |ev| prolactin_unit.set(event_target_value(&ev))
                                    prop:value=move || prolactin_unit.get()
                                >
                                    {render_unit_options()}
                                </select>
                            </label>
                        </div>

                        <div class="inline-equal">
                            <label>
                                "SHBG level"
                                <input
                                    type="text"
                                    step="any"
                                    on:input=move |ev| shbg_level.set(event_target_value(&ev))
                                    prop:value=move || shbg_level.get()
                                />
                            </label>
                            <label>
                                "SHBG unit"
                                <select
                                    on:change=move |ev| shbg_unit.set(event_target_value(&ev))
                                    prop:value=move || shbg_unit.get()
                                >
                                    {render_unit_options()}
                                </select>
                            </label>
                        </div>

                        <div class="inline-equal">
                            <label>
                                "Free Androgen Index"
                                <input
                                    type="text"
                                    step="any"
                                    on:input=move |ev| free_androgen_index.set(event_target_value(&ev))
                                    prop:value=move || free_androgen_index.get()
                                />
                            </label>
                            <div></div>
                        </div>
                    </div>

                    <div class="form-section">
                        <h3>"Liver & lipids"</h3>
                        <p class="muted">
                            "From a wider kit report. The liver markers matter while estradiol is taken as a tablet, because a tablet is swallowed and reaches the liver before the blood; injections skip that first pass. The ranges shown are typical adult ones; your report carries its own."
                        </p>
                        <div class="inline-equal">
                            {MetabolicMarker::ALL
                                .iter()
                                .filter(|marker| marker.is_liver())
                                .map(|marker| render_marker_input(*marker))
                                .collect_view()}
                        </div>
                        <div class="inline-equal">
                            {MetabolicMarker::ALL
                                .iter()
                                .filter(|marker| !marker.is_liver())
                                .map(|marker| render_marker_input(*marker))
                                .collect_view()}
                        </div>
                    </div>

                    <label>
                        "Notes"
                        <textarea
                            rows="3"
                            placeholder="notes..."
                            on:input=move |ev| notes.set(event_target_value(&ev))
                            prop:value=move || notes.get()
                        ></textarea>
                    </label>

                    <div class="form-actions">
                        <button type="submit">"Create test"</button>
                        <Show when=move || show_feedback.get()>
                            <p class="muted">"Blood test added!"</p>
                        </Show>
                    </div>
                </form>
            </div>
        }
        .into_view(),
    )
}
#[cfg(test)]
mod tests {
    use super::{
        extract_ocr_values, normalize_unit_token, parse_ocr_number, table_marker_at,
        BandedHormone, OcrExtraction, TableTarget,
    };
    use hrt_shared::types::MetabolicMarker;

    /// A banded layout the way `lopdf` hands it over: the result and its unit
    /// come *before* the title for the first sections (they are drawn above the
    /// title), phase rows arrive glued to their labels, and the glossary page
    /// repeats the titles in prose. Every number is invented.
    const BANDED_STREAM_ORDER: &str = r#"CID
414784
Sample
ID
TC00196766
Name
Sample Person
DOB
07-Feb-1990
Date
of
Report
31-Oct-2026
Programme
RH Female Hormone
Sample
Collection
Date
26-Oct-2026 09:15
CONTENTS
Hormonal Health
03
Test Glossary 
04
Hormonal Health
A
hormone
is
a
chemical
substance
that
is
produced
in
response
to
changes.
mIU/l
777
Prolactin
102 - 496
<102
497 - 700
>700
High
Optimal
Low
Very High 
nmol/l
0.777
Testosterone 
0.291 - 1.67
≤0.29
>1.67
Optimal
Low
High
nmol/l
77.7
Sex
 
Hormone
 
Binding
 
Globulin
32.41 - 128.0
≤32.4
>128.0
Optimal
Low
High
-
7.77
Free
 
Androgen
 
Index
0.31 - 5.62
≤0.3
>5.62
Optimal
Low
High
Follicle
 
Stimulating
 
Hormone
3.5 - 12.5 Follicular Phase4.7 - 21.5 Mid Cycle1.7 - 7.7 Luteal Phase25.8 - 134.8 Post Menopausal
<0.77
U/l
Oestradiol
45 - 854 Follicular Phase151 - 1461 Midcycle82 - 1251 Luteal Phase<505 Post Menopausal
7777
pmol/l
Luteinising
 
Hormone
2.4 - 12.6 Follicular Phase14 - 95.6 Mid Cycle1 - 11.4 Luteal Phase 7.7 - 58.5 Post Menopausal
<0.17
U/l
Progesterone
<0.616 Follicular Phase0.175 - 13.2 Mid Cycle13.1 - 46.3 Luteal Phase<0.401 Post Menopausal
7.77
nmol/l
Page 3 of 6
Test Glossary 
Follicle Stimulating Hormone
Follicle
stimulating
hormone
(FSH)
peaks
on
day
21
of
a
28
day
cycle.
Progesterone
levels
rise
to
13.1
nmol/l
during
the
luteal
phase
and
Free
Androgen
Index
is
a
ratio
of
9.9
.
Prolactin
may
read
700
mIU/l
.
Got Your
Results?
20 Minute GP Appointment
30 Minute GP Appointment
"#;

    /// The same sections as a screen OCR sees them: the result sits above its
    /// band row, the litre is read as `I`, a decimal point floats free, and one
    /// result is split into single digits. Every number is invented.
    const BANDED_OCR_NOISE: &str = r#"CID 414784
Sample ID TC00196766
Name Sample Person
DOB 07-Feb-1990
Date of Report 31-Oct-2026
Programme RH Female Hormone
Sample Collection Date 26-Oct-2026 09:15
CONTENTS
9 Hormonal Health 03
Q Test Glossary 04
Hormonal Health
A hormone is a chemical substance produced in response to certain changes in the body.
Prolactin
v
777
<102 102 - 496 497 - 700 >700 miU/I
Low Optimal High Very High
Testosterone
0.777
<0.29 0.291 -1.67 >1.67 nmol/l
Low Optimal High
Sex Hormone Binding Globulin
A4
77.7
<324 32.41-128.0 >128.0
Low Optimal High
Free Androgen Index
A4
7.77
<0.3 0.31-5.62 >5.62
Low Optimal High
Follicle Stimulating Hormone
3.5 -12.5 Follicular Phase
4.7 - 21.5 Mid Cycle <0.77
1.7 - 7.7 Luteal Phase U/l
25.8 - 134.8 Post Menopausal
Oestradiol
45 - 854 Follicular Phase
151 - 1461 Midcycle 7 77 7
82 - 1251 Luteal Phase
<505 Post Menopausal pmol/l
Luteinising Hormone
2.4 - 12.6 Follicular Phase
14 - 95.6 Mid Cycle < O . 17
1 - 11.4 Luteal Phase
7.7 - 58.5 Post Menopausal U/I
Progesterone
<0.616 Follicular Phase
0.175 - 13.2 Mid Cycle 7 . 77
13.1 - 46.3 Luteal Phase
<0.401 Post Menopausal nmol/l
Page 3 of 6
"#;

    /// The flat layout the parser already handled: one label and its value on
    /// one line each, units written "pmol/L"/"IU/L".
    const FLAT_LAYOUT: &str = r#"Sample Collection Date: 15/03/2025 09:30
Oestradiol 412 pmol/L
Testosterone 1.2 nmol/L
Sex Hormone Binding Globulin 60 nmol/L
Free Androgen Index 2.1
Follicle Stimulating Hormone 6.4 IU/L
Luteinising Hormone 5.1 IU/L
Prolactin 220 mIU/L
Progesterone 3.1 nmol/L
"#;

    fn value(extracted: &OcrExtraction, hormone: BandedHormone) -> Option<String> {
        let (banded, unit) = match hormone {
            BandedHormone::Estradiol => (&extracted.estradiol, "estradiol"),
            BandedHormone::Testosterone => (&extracted.testosterone, "testosterone"),
            BandedHormone::Progesterone => (&extracted.progesterone, "progesterone"),
            BandedHormone::Fsh => (&extracted.fsh, "fsh"),
            BandedHormone::Lh => (&extracted.lh, "lh"),
            BandedHormone::Prolactin => (&extracted.prolactin, "prolactin"),
            BandedHormone::Shbg => (&extracted.shbg, "shbg"),
            BandedHormone::Fai => (&extracted.fai, "fai"),
        };
        assert!(banded.is_some(), "{unit} was not extracted");
        banded.as_ref().map(|value| value.value.clone())
    }

    fn unit_of(extracted: &OcrExtraction, hormone: BandedHormone) -> Option<String> {
        let value = match hormone {
            BandedHormone::Estradiol => &extracted.estradiol,
            BandedHormone::Testosterone => &extracted.testosterone,
            BandedHormone::Progesterone => &extracted.progesterone,
            BandedHormone::Fsh => &extracted.fsh,
            BandedHormone::Lh => &extracted.lh,
            BandedHormone::Prolactin => &extracted.prolactin,
            BandedHormone::Shbg => &extracted.shbg,
            BandedHormone::Fai => &extracted.fai,
        };
        value.as_ref().and_then(|value| value.unit.clone())
    }

    fn assert_banded_report(extracted: &OcrExtraction) {
        assert_eq!(
            value(extracted, BandedHormone::Estradiol).as_deref(),
            Some("7777"),
            "Oestradiol result, not a phase range"
        );
        assert_eq!(
            unit_of(extracted, BandedHormone::Estradiol).as_deref(),
            Some("pmol/L")
        );
        assert_eq!(
            value(extracted, BandedHormone::Testosterone).as_deref(),
            Some("0.777")
        );
        assert_eq!(
            unit_of(extracted, BandedHormone::Testosterone).as_deref(),
            Some("nmol/L")
        );
        assert_eq!(
            value(extracted, BandedHormone::Progesterone).as_deref(),
            Some("7.77")
        );
        assert_eq!(
            unit_of(extracted, BandedHormone::Progesterone).as_deref(),
            Some("nmol/L")
        );
        assert_eq!(
            value(extracted, BandedHormone::Fsh).as_deref(),
            Some("0.77"),
            "FSH result, not the follicular range"
        );
        assert_eq!(
            unit_of(extracted, BandedHormone::Fsh).as_deref(),
            Some("U/L")
        );
        assert_eq!(
            value(extracted, BandedHormone::Lh).as_deref(),
            Some("0.17"),
            "LH result, not a mid-cycle range"
        );
        assert_eq!(unit_of(extracted, BandedHormone::Lh).as_deref(), Some("U/L"));
        assert_eq!(
            value(extracted, BandedHormone::Prolactin).as_deref(),
            Some("777")
        );
        assert_eq!(
            unit_of(extracted, BandedHormone::Prolactin).as_deref(),
            Some("mIU/L")
        );
        assert_eq!(
            value(extracted, BandedHormone::Shbg).as_deref(),
            Some("77.7")
        );
        assert_eq!(
            unit_of(extracted, BandedHormone::Shbg).as_deref(),
            Some("nmol/L")
        );
        assert_eq!(value(extracted, BandedHormone::Fai).as_deref(), Some("7.77"));
        assert_eq!(
            unit_of(extracted, BandedHormone::Fai),
            None,
            "Free Androgen Index is unitless"
        );
        assert_eq!(
            extracted.sample_date.as_deref(),
            Some("2026-10-26T09:15"),
            "sample collection date with its time"
        );
    }

    #[test]
    fn banded_layout_reads_every_hormone_with_its_unit() {
        let extracted = extract_ocr_values(BANDED_STREAM_ORDER);
        assert_banded_report(&extracted);
    }

    #[test]
    fn banded_layout_survives_ocr_noise() {
        let extracted = extract_ocr_values(BANDED_OCR_NOISE);
        assert_banded_report(&extracted);
    }

    #[test]
    fn flat_layout_still_parses() {
        let extracted = extract_ocr_values(FLAT_LAYOUT);
        assert_eq!(extracted.estradiol.as_ref().map(|v| v.value.as_str()), Some("412"));
        assert_eq!(
            extracted.estradiol.as_ref().and_then(|v| v.unit.as_deref()),
            Some("pmol/L")
        );
        assert_eq!(
            extracted.testosterone.as_ref().map(|v| v.value.as_str()),
            Some("1.2")
        );
        assert_eq!(extracted.shbg.as_ref().map(|v| v.value.as_str()), Some("60"));
        assert_eq!(extracted.fai.as_ref().map(|v| v.value.as_str()), Some("2.1"));
        assert_eq!(extracted.fsh.as_ref().map(|v| v.value.as_str()), Some("6.4"));
        assert_eq!(
            extracted.fsh.as_ref().and_then(|v| v.unit.as_deref()),
            Some("U/L"),
            "IU/L is normalised to U/L"
        );
        assert_eq!(extracted.lh.as_ref().map(|v| v.value.as_str()), Some("5.1"));
        assert_eq!(
            extracted.prolactin.as_ref().map(|v| v.value.as_str()),
            Some("220")
        );
        assert_eq!(
            extracted.progesterone.as_ref().map(|v| v.value.as_str()),
            Some("3.1")
        );
        assert_eq!(
            extracted.sample_date.as_deref(),
            Some("2025-03-15T09:30")
        );
    }

    #[test]
    fn banded_layout_ignores_prose_and_contents_numbers() {
        // Prose numbers appear both before the first section (page numbers in
        // the contents) and after the last one (glossary, marketing page). None
        // of them may become a result.
        let extracted = extract_ocr_values(BANDED_STREAM_ORDER);
        assert_eq!(value(&extracted, BandedHormone::Prolactin).as_deref(), Some("777"));
        assert_eq!(value(&extracted, BandedHormone::Fai).as_deref(), Some("7.77"));
        assert_eq!(
            value(&extracted, BandedHormone::Estradiol).as_deref(),
            Some("7777")
        );
    }

    #[test]
    fn banded_layout_rejects_band_edges_as_results() {
        // The result line was lost by OCR: every remaining number is a band
        // edge, so nothing may be filled in rather than a wrong value.
        let text = "\
Testosterone
0.777
<0.29 0.291 -1.67 >1.67 nmol/l
Low Optimal High
Prolactin
<102 102 - 496 497 - 700 >700 miU/l
Low Optimal High Very High
";
        let extracted = extract_ocr_values(text);
        assert_eq!(
            value(&extracted, BandedHormone::Testosterone).as_deref(),
            Some("0.777")
        );
        assert!(
            extracted.prolactin.is_none(),
            "a band edge (\"<102\", \">700\") must not be read as a result"
        );
    }

    #[test]
    fn banded_ocr_band_edges_are_never_results() {
        // Screen OCR of a banded report drops the right-aligned result for some
        // hormones and leaves only the band row behind, so the thresholds
        // ("<0.29", "<324", a dot-less "<030") must not become results. The
        // results OCR did keep are still read. Every number is invented.
        let text = "\
Testosterone
<0.29 0.291 -1.67 >1.67 nmol/l
Low Optimal High
Sex Hormone Binding Globulin
<324 32.41-128.0 >128.0
Low Optimal High
Free Androgen Index
<0.3 0.31-5.62 >5.62
Low Optimal High
Follicle Stimulating Hormone
3.5 -12.5 Follicular Phase
4.7 - 21.5 Mid Cycle <030
1.7 - 7.7 Luteal Phase U/l
25.8 - 134.8 Post Menopausal
Oestradiol
45 - 854 Follicular Phase
151 - 1461 Midcycle 2 46 8
82 - 1251 Luteal Phase
<505 Post Menopausal pmol/l
";
        let extracted = extract_ocr_values(text);
        assert!(extracted.testosterone.is_none(), "\"<0.29\" is a band edge");
        assert!(extracted.shbg.is_none(), "\"<324\" is a band edge");
        assert!(extracted.fai.is_none(), "\"<0.3\" is a band edge");
        assert!(extracted.fsh.is_none(), "a dot-less \"<030\" is a band edge");
        assert_eq!(
            value(&extracted, BandedHormone::Estradiol).as_deref(),
            Some("2468"),
            "a split result with a decimal point is still recovered"
        );
    }

    #[test]
    fn unit_normalisation_accepts_lowercase_l_and_ocr_confusions() {
        for token in ["pmol/L", "pmol/l", "pmol/1", "pmol|L", "pmol/l."] {
            assert_eq!(normalize_unit_token(token), Some("pmol/L"), "{token}");
        }
        for token in ["nmol/l", "nmol/1", "nmol/I", "nm0l/l"] {
            assert_eq!(normalize_unit_token(token), Some("nmol/L"), "{token}");
        }
        for token in ["mIU/l", "mIU/L", "miU/I", "mlU/l", "mIU/1", "mIUl"] {
            assert_eq!(normalize_unit_token(token), Some("mIU/L"), "{token}");
        }
        for token in ["U/l", "U/L", "u|l", "UI/l", "IU/L", "Ui"] {
            assert_eq!(normalize_unit_token(token), Some("U/L"), "{token}");
        }
        assert_eq!(normalize_unit_token("Low"), None);
        assert_eq!(normalize_unit_token("-"), None);
    }

    #[test]
    fn sample_date_accepts_day_first_month_names_and_times() {
        let day_first = extract_ocr_values("Sample Collection Date 26-Oct-2026 09:15");
        assert_eq!(day_first.sample_date.as_deref(), Some("2026-10-26T09:15"));

        let slash = extract_ocr_values("Sample Collection Date: 26/10/2026");
        assert_eq!(slash.sample_date.as_deref(), Some("2026-10-26T12:00"));

        let spelled = extract_ocr_values("Sample Date: 15 Mar 2025");
        assert_eq!(spelled.sample_date.as_deref(), Some("2025-03-15T12:00"));

        let split_label = extract_ocr_values("Sample\nCollection\nDate\n26-Oct-2026 10:00");
        assert_eq!(split_label.sample_date.as_deref(), Some("2026-10-26T10:00"));

        let report_date_only =
            extract_ocr_values("Date of Report 31-Oct-2026\nSample Collection Date 26-Oct-2026 09:15");
        assert_eq!(
            report_date_only.sample_date.as_deref(),
            Some("2026-10-26T09:15"),
            "the collection date wins over the report date"
        );
    }

    /// A wider-kit result sheet in the shape other providers print it: marker
    /// name on its own line, then result, unit, reference ranges and a flag.
    /// Every number is invented.
    const TABLE_LAYOUT: &str = r#"Blood Test Results
Personal Details
Name:
 Sample Person
Date of Birth:
 1990-01-15
Gender:
 X
Order ID:
 TEST-0001
Specimen Details
Sample Type:
 Capillary
Collected:
 2026-10-26 09:15
Received:
 2026-10-27 14:20
Reported:
 2026-10-28 15:03
Sex Hormones
Test
Result
Units
Reference Range
Flag
Testosterone
12.3
nmol/L
Male
8.64 – 29.00
Female
0.29 – 1.67
High
Progesterone
2.2
nmol/L
0.616 – 46.3
Prolactin
333
mIU/L
102 – 496
17-Beta Oestradiol
555
pmol/L
Male
41.4 – 159
Female
Follicular: 114 – 332
Luteal: 222 – 854
Liver Function
Alanine Transferase
23
U/L
< 33
Aspartate Transferase
26
U/L
< 32
Alkaline Phosphatase
77
U/L
30 – 130
Gamma GT
18
U/L
< 40
Total Bilirubin
8
umol/L
< 21
Total Protein
72
g/L
60 – 80
Albumin
45
g/L
35 – 50
Globulin
27
g/L
21 – 35
Lipids
Triglyceride
1.2
mmol/L
< 1.7
Total Cholesterol
4.4
mmol/L
< 5.0
HDL Cholesterol
1.7
mmol/L
> 1.0
LDL Cholesterol
2.4
mmol/L
< 3.0
Non-HDL Cholesterol
2.6
mmol/L
< 4.0
Understanding Your Results
We've put together a handy resource on reading this results sheet.
"#;

    fn marker(extracted: &OcrExtraction, marker: MetabolicMarker) -> Option<f64> {
        marker.get(&extracted.metabolic)
    }

    #[test]
    fn table_layout_reads_every_hormone_and_unit() {
        let extracted = extract_ocr_values(TABLE_LAYOUT);
        assert_eq!(value(&extracted, BandedHormone::Testosterone).as_deref(), Some("12.3"));
        assert_eq!(
            unit_of(&extracted, BandedHormone::Testosterone).as_deref(),
            Some("nmol/L")
        );
        assert_eq!(value(&extracted, BandedHormone::Progesterone).as_deref(), Some("2.2"));
        assert_eq!(value(&extracted, BandedHormone::Prolactin).as_deref(), Some("333"));
        assert_eq!(
            unit_of(&extracted, BandedHormone::Prolactin).as_deref(),
            Some("mIU/L")
        );
        assert_eq!(value(&extracted, BandedHormone::Estradiol).as_deref(), Some("555"));
        assert_eq!(
            unit_of(&extracted, BandedHormone::Estradiol).as_deref(),
            Some("pmol/L")
        );
        assert_eq!(extracted.sample_date.as_deref(), Some("2026-10-26T09:15"));
    }

    #[test]
    fn table_layout_reads_the_liver_panel() {
        let extracted = extract_ocr_values(TABLE_LAYOUT);
        assert_eq!(marker(&extracted, MetabolicMarker::Alt), Some(23.0));
        assert_eq!(marker(&extracted, MetabolicMarker::Ast), Some(26.0));
        assert_eq!(marker(&extracted, MetabolicMarker::AlkalinePhosphatase), Some(77.0));
        assert_eq!(marker(&extracted, MetabolicMarker::Ggt), Some(18.0));
        assert_eq!(marker(&extracted, MetabolicMarker::Bilirubin), Some(8.0));
        assert_eq!(marker(&extracted, MetabolicMarker::TotalProtein), Some(72.0));
        assert_eq!(marker(&extracted, MetabolicMarker::Albumin), Some(45.0));
        // "Globulin" must come from its own row, not from the squeezed name
        // "Sex Hormone Binding Globulin".
        assert_eq!(marker(&extracted, MetabolicMarker::Globulin), Some(27.0));
        assert!(extracted.metabolic.has_liver_values());
    }

    #[test]
    fn table_layout_reads_the_lipid_panel() {
        let extracted = extract_ocr_values(TABLE_LAYOUT);
        assert_eq!(marker(&extracted, MetabolicMarker::Triglyceride), Some(1.2));
        assert_eq!(marker(&extracted, MetabolicMarker::TotalCholesterol), Some(4.4));
        assert_eq!(marker(&extracted, MetabolicMarker::HdlCholesterol), Some(1.7));
        assert_eq!(marker(&extracted, MetabolicMarker::LdlCholesterol), Some(2.4));
        assert_eq!(marker(&extracted, MetabolicMarker::NonHdlCholesterol), Some(2.6));
        assert!(extracted.metabolic.has_lipid_values());
    }

    #[test]
    fn table_layout_row_without_a_result_borrows_nothing() {
        let text = "\
Liver Function
Alanine Transferase
U/L
< 33
Aspartate Transferase
26
U/L
< 32
";
        let extracted = extract_ocr_values(text);
        assert_eq!(
            marker(&extracted, MetabolicMarker::Alt),
            None,
            "a row with no result must stay empty"
        );
        assert_eq!(marker(&extracted, MetabolicMarker::Ast), Some(26.0));
    }

    #[test]
    fn table_layout_survives_ocr_merged_rows_and_noise() {
        // A screenshot OCR squashes each row onto one line and misreads the
        // litre as "I"; the values must still land.
        let text = "\
Collected: 2026-10-26 09:15
Alanine Transferase 23 U/I < 33
Gamma GT 18 U/I < 40
Total Cholesterol 4.4 mmol/l < 5.0
Non-HDL Cholesterol 2.6 mmol/l < 4.0
";
        let extracted = extract_ocr_values(text);
        assert_eq!(marker(&extracted, MetabolicMarker::Alt), Some(23.0));
        assert_eq!(marker(&extracted, MetabolicMarker::Ggt), Some(18.0));
        assert_eq!(marker(&extracted, MetabolicMarker::TotalCholesterol), Some(4.4));
        assert_eq!(marker(&extracted, MetabolicMarker::NonHdlCholesterol), Some(2.6));
        assert_eq!(extracted.sample_date.as_deref(), Some("2026-10-26T09:15"));
    }

    #[test]
    fn table_layout_skips_units_it_cannot_convert() {
        // mg/dL bilirubin needs a conversion this app does not carry, so the
        // field stays empty instead of storing a number in the wrong unit.
        let text = "\
Total Bilirubin
0.9
mg/dL
< 21
Total Protein
72
g/L
60 – 80
";
        let extracted = extract_ocr_values(text);
        assert_eq!(marker(&extracted, MetabolicMarker::Bilirubin), None);
        assert_eq!(marker(&extracted, MetabolicMarker::TotalProtein), Some(72.0));
    }

    #[test]
    fn table_marker_names_respect_word_boundaries() {
        assert_eq!(
            table_marker_at("Non-HDL Cholesterol"),
            Some(TableTarget::Metabolic(MetabolicMarker::NonHdlCholesterol))
        );
        assert_eq!(
            table_marker_at("HDL Cholesterol"),
            Some(TableTarget::Metabolic(MetabolicMarker::HdlCholesterol))
        );
        assert_eq!(
            table_marker_at("Sex Hormone Binding Globulin"),
            Some(TableTarget::Shbg)
        );
        assert_eq!(
            table_marker_at("Alternatives to consider"),
            None,
            "a word that merely starts with a marker name is not a marker"
        );
        assert_eq!(
            table_marker_at("Test"),
            None,
            "the template's column heading is not testosterone"
        );
    }

    #[test]
    fn oestradiol_means_the_same_as_17_beta_oestradiol() {
        let text = "\
17-Beta Oestradiol
555
pmol/L
Male
41.4 – 159
";
        let extracted = extract_ocr_values(text);
        assert_eq!(value(&extracted, BandedHormone::Estradiol).as_deref(), Some("555"));
    }

    #[test]
    fn numbers_still_strip_threshold_symbols() {
        assert_eq!(parse_ocr_number("<0.30").as_deref(), Some("0.30"));
        assert_eq!(parse_ocr_number("≤0.29").as_deref(), Some("0.29"));
        assert_eq!(parse_ocr_number("1,25").as_deref(), Some("1.25"));
    }

}

