use std::collections::HashMap;

use crate::types::Hormone;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BaseUnit {
    G,
    Mol,
    L,
}

#[derive(Debug, Clone, Copy)]
struct UnitSingle {
    base: BaseUnit,
    exp: i32,
}

#[derive(Debug, Clone, Copy)]
struct UnitRatio {
    numerator: UnitSingle,
    denominator: UnitSingle,
}

fn molar_masses() -> HashMap<Hormone, f64> {
    use Hormone::*;
    HashMap::from([
        (Cholesterol, 386.65),
        (Testosterone, 288.431),
        (Dihydrotestosterone, 290.447),
        (Dehydroepiandrosterone, 288.424),
        (Estrone, 270.336),
        (Estradiol, 272.38),
        (Estriol, 288.387),
        (Estetrol, 304.386),
        (Progesterone, 314.469),
        (Aldosterone, 360.45),
        (Androstenedione, 286.415),
        (Cortisol, 362.46),
        (Gonadorelin, 1182.311),
        (FollicleStimulatingHormone, 30000.0),
        (LuteinisingHormone, 33000.0),
        (ThyroidStimulatingHormone, 28000.0),
        (SexHormoneBindingGlobulin, 43700.0),
        (Prolactin, 22892.0),
        (Thyroxine, 776.87),
        (Triiodothyronine, 650.977),
        (VitaminD3, 384.64),
        (VitaminB12, 1355.388),
    ])
}

fn prefix_exponents() -> HashMap<&'static str, i32> {
    HashMap::from([
        ("y", -24),
        ("z", -21),
        ("a", -18),
        ("f", -15),
        ("p", -12),
        ("n", -9),
        ("µ", -6),
        ("u", -6),
        ("m", -3),
        ("c", -2),
        ("d", -1),
        ("", 0),
        ("da", 1),
        ("h", 2),
        ("k", 3),
        ("M", 6),
        ("G", 9),
        ("T", 12),
        ("P", 15),
        ("E", 18),
        ("Z", 21),
        ("Y", 24),
    ])
}

fn normalize_token(token: &str) -> String {
    token
        .trim()
        .replace('ℓ', "L")
        .replace('l', "L")
        .replace(' ', "")
}

fn parse_unit_single(token: &str) -> Result<UnitSingle, String> {
    let t = normalize_token(token);

    let (base, base_len) = if t.ends_with("mol") {
        (BaseUnit::Mol, 3)
    } else if t.ends_with('g') {
        (BaseUnit::G, 1)
    } else if t.ends_with('L') {
        (BaseUnit::L, 1)
    } else {
        return Err(format!("Unsupported unit token \"{}\"", token));
    };

    let prefix = &t[..t.len() - base_len];
    let exponents = prefix_exponents();
    let prefix_exp = exponents
        .get(prefix)
        .ok_or_else(|| format!("Unsupported prefix \"{}\" in \"{}\"", prefix, token))?;

    Ok(UnitSingle {
        base,
        exp: *prefix_exp,
    })
}

fn parse_unit_ratio(unit: &str) -> Result<UnitRatio, String> {
    let mut iter = unit.split('/');
    let num_raw = iter.next().unwrap_or("");
    let den_raw = iter.next().unwrap_or("");
    if num_raw.is_empty() || den_raw.is_empty() {
        return Err(format!("Expected a ratio like \"pg/mL\", got \"{}\"", unit));
    }

    let numerator = parse_unit_single(num_raw)?;
    let denominator = parse_unit_single(den_raw)?;

    let num_ok = matches!(numerator.base, BaseUnit::G | BaseUnit::Mol);
    let den_ok = matches!(denominator.base, BaseUnit::L);
    if !(num_ok && den_ok) {
        return Err(format!(
            "Only mass/volume or mol/volume units are supported (got \"{}\")",
            unit
        ));
    }

    Ok(UnitRatio {
        numerator,
        denominator,
    })
}

fn convert_core(
    value: f64,
    hormone: Hormone,
    from: UnitRatio,
    to: UnitRatio,
) -> Result<f64, String> {
    let prefix_calc =
        (from.numerator.exp + to.denominator.exp) - (from.denominator.exp + to.numerator.exp);

    let mass = molar_masses()
        .get(&hormone)
        .copied()
        .ok_or_else(|| format!("Missing molar mass for {:?}", hormone))?;

    let base_factor = match (from.numerator.base, to.numerator.base) {
        (BaseUnit::Mol, BaseUnit::G) => mass,
        (BaseUnit::G, BaseUnit::Mol) => 1.0 / mass,
        (BaseUnit::Mol, BaseUnit::Mol) => 1.0,
        (BaseUnit::G, BaseUnit::G) => 1.0,
        _ => {
            return Err(format!(
                "Unsupported base conversion from {:?} to {:?}",
                from.numerator.base, to.numerator.base
            ))
        }
    };

    let ten_pow = 10_f64.powi(prefix_calc);
    Ok(value * ten_pow * base_factor)
}

pub fn convert_hormone(
    value: f64,
    hormone: Hormone,
    from_unit: &str,
    to_unit: &str,
) -> Result<f64, String> {
    let from = parse_unit_ratio(from_unit)?;
    let to = parse_unit_ratio(to_unit)?;
    convert_core(value, hormone, from, to)
}

pub fn convert_estradiol(value: f64, from_unit: &str, to_unit: &str) -> Result<f64, String> {
    convert_hormone(value, Hormone::Estradiol, from_unit, to_unit)
}

pub fn convert_testosterone(value: f64, from_unit: &str, to_unit: &str) -> Result<f64, String> {
    convert_hormone(value, Hormone::Testosterone, from_unit, to_unit)
}

pub fn convert_progesterone(value: f64, from_unit: &str, to_unit: &str) -> Result<f64, String> {
    convert_hormone(value, Hormone::Progesterone, from_unit, to_unit)
}
