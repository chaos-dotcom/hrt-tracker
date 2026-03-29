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
    let t = token.trim().replace(' ', "");
    // Only replace l/ℓ → L when it's a liter suffix, not part of "mol"
    if t.ends_with("mol") {
        t
    } else {
        t.replace(['ℓ', 'l'], "L")
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Hormone;

    #[test]
    fn estradiol_pg_ml_to_ng_ml() {
        // 1 pg/mL = 0.001 ng/mL (pico to nano = 1e-3)
        let result = convert_estradiol(1000.0, "pg/mL", "ng/mL").unwrap();
        assert!((result - 1.0).abs() < 0.001, "got {result}");
    }

    #[test]
    fn estradiol_ng_ml_to_pg_ml() {
        let result = convert_estradiol(1.0, "ng/mL", "pg/mL").unwrap();
        assert!((result - 1000.0).abs() < 0.01, "got {result}");
    }

    #[test]
    fn estradiol_mass_roundtrip() {
        let original = 250.0;
        let ng = convert_estradiol(original, "pg/mL", "ng/mL").unwrap();
        let back = convert_estradiol(ng, "ng/mL", "pg/mL").unwrap();
        assert!((back - original).abs() < 0.01, "got {back}");
    }

    #[test]
    fn testosterone_ng_dl_to_ng_ml() {
        // 1 ng/dL = 0.01 ng/mL (dL to mL = 1e-2)
        let result = convert_testosterone(100.0, "ng/dL", "ng/mL").unwrap();
        assert!((result - 1.0).abs() < 0.01, "got {result}");
    }

    #[test]
    fn testosterone_ng_ml_to_ng_dl() {
        let result = convert_testosterone(1.0, "ng/mL", "ng/dL").unwrap();
        assert!((result - 100.0).abs() < 0.5, "got {result}");
    }

    #[test]
    fn progesterone_ng_ml_to_ug_l() {
        // ng/mL and ug/L are equivalent (both are 1e-9 g / 1e-3 L)
        let result = convert_progesterone(1.0, "ng/mL", "ug/L").unwrap();
        assert!((result - 1.0).abs() < 0.001, "got {result}");
    }

    #[test]
    fn estradiol_pg_ml_to_pmol_l() {
        // 1 pg/mL = 3.6713 pmol/L for estradiol (molar mass 272.38)
        let result = convert_estradiol(100.0, "pg/mL", "pmol/L").unwrap();
        assert!((result - 367.13).abs() < 0.5, "got {result}");
    }

    #[test]
    fn estradiol_pmol_l_to_pg_ml() {
        let result = convert_estradiol(367.13, "pmol/L", "pg/mL").unwrap();
        assert!((result - 100.0).abs() < 0.5, "got {result}");
    }

    #[test]
    fn estradiol_mol_roundtrip() {
        let original = 250.0;
        let pmol = convert_estradiol(original, "pg/mL", "pmol/L").unwrap();
        let back = convert_estradiol(pmol, "pmol/L", "pg/mL").unwrap();
        assert!((back - original).abs() < 0.01, "got {back}");
    }

    #[test]
    fn testosterone_ng_dl_to_nmol_l() {
        let result = convert_testosterone(100.0, "ng/dL", "nmol/L").unwrap();
        assert!((result - 3.467).abs() < 0.05, "got {result}");
    }

    #[test]
    fn testosterone_nmol_l_to_ng_dl() {
        let result = convert_testosterone(3.467, "nmol/L", "ng/dL").unwrap();
        assert!((result - 100.0).abs() < 1.0, "got {result}");
    }

    #[test]
    fn progesterone_ng_ml_to_nmol_l() {
        let result = convert_progesterone(1.0, "ng/mL", "nmol/L").unwrap();
        assert!((result - 3.18).abs() < 0.05, "got {result}");
    }

    #[test]
    fn same_unit_returns_identity() {
        let result = convert_estradiol(42.0, "pg/mL", "pg/mL").unwrap();
        assert!((result - 42.0).abs() < f64::EPSILON);
    }

    #[test]
    fn invalid_unit_returns_error() {
        assert!(convert_estradiol(1.0, "bananas", "pg/mL").is_err());
        assert!(convert_estradiol(1.0, "pg/mL", "").is_err());
    }

    #[test]
    fn generic_convert_hormone_works() {
        let result = convert_hormone(100.0, Hormone::Estradiol, "pg/mL", "ng/mL").unwrap();
        let specific = convert_estradiol(100.0, "pg/mL", "ng/mL").unwrap();
        assert!((result - specific).abs() < f64::EPSILON);
    }

    #[test]
    fn micro_prefix_variants() {
        // µg/L and ug/L should both work
        let r1 = convert_hormone(1.0, Hormone::Estradiol, "µg/L", "pg/mL").unwrap();
        let r2 = convert_hormone(1.0, Hormone::Estradiol, "ug/L", "pg/mL").unwrap();
        assert!((r1 - r2).abs() < f64::EPSILON);
    }

    #[test]
    fn ng_ml_to_pg_ml_is_thousand() {
        let result = convert_estradiol(1.0, "ng/mL", "pg/mL").unwrap();
        assert!((result - 1000.0).abs() < 0.01, "got {result}");
    }
}
