type Base = 'g' | 'mol' | 'L';

export type Hormone =
  | 'Cholesterol'
  | 'Testosterone'
  | 'Dihydrotestosterone'
  | 'Dehydroepiandrosterone'
  | 'Estrone'
  | 'Estradiol'
  | 'Estriol'
  | 'Estetrol'
  | 'Progesterone'
  | 'Aldosterone'
  | 'Androstenedione'
  | 'Cortisol'
  | 'Gonadorelin'
  | 'Follicle-stimulating hormone'
  | 'Luteinising hormone'
  | 'Thyroid-stimulating hormone'
  | 'Sex hormone-binding globulin'
  | 'Prolactin'
  | 'Thyroxine'
  | 'Triiodothyronine'
  | 'Vitamin D3'
  | 'Vitamin B12';

const MOLAR_MASS_G_PER_MOL: Record<Hormone, number> = {
  Cholesterol: 386.65,
  Testosterone: 288.431,
  Dihydrotestosterone: 290.447,
  Dehydroepiandrosterone: 288.424,
  Estrone: 270.336,
  Estradiol: 272.38,
  Estriol: 288.387,
  Estetrol: 304.386,
  Progesterone: 314.469,
  Aldosterone: 360.45,
  Androstenedione: 286.415,
  Cortisol: 362.46,
  Gonadorelin: 1182.311,
  'Follicle-stimulating hormone': 30000,
  'Luteinising hormone': 33000,
  'Thyroid-stimulating hormone': 28000,
  'Sex hormone-binding globulin': 43700,
  Prolactin: 22892,
  Thyroxine: 776.87,
  Triiodothyronine: 650.977,
  'Vitamin D3': 384.64,
  'Vitamin B12': 1355.388
};

const PREFIX_EXP: Record<string, number> = {
  y: -24, z: -21, a: -18, f: -15, p: -12, n: -9, 'µ': -6, u: -6,
  m: -3, c: -2, d: -1, '': 0, da: 1, h: 2, k: 3, M: 6, G: 9, T: 12, P: 15, E: 18, Z: 21, Y: 24
};

interface UnitSingle {
  base: Base;
  exp: number; // SI exponent for the prefix, e.g. 'm' => -3
}
interface UnitRatio {
  numerator: UnitSingle;
  denominator: UnitSingle;
}

function normalizeToken(token: string): string {
  return token
    .trim()
    .replace(/\s+/g, '')
    .replace(/ℓ|l/g, 'L'); // accept L, l, ℓ
}

function parseUnitSingle(token: string): UnitSingle {
  const t = normalizeToken(token);

  // Determine base first to avoid confusing 'M' (mega) with 'mol'
  let base: Base | null = null;
  let baseLen = 0;

  if (t.endsWith('mol')) {
    base = 'mol';
    baseLen = 3;
  } else if (t.endsWith('g')) {
    base = 'g';
    baseLen = 1;
  } else if (t.endsWith('L')) {
    base = 'L';
    baseLen = 1;
  } else {
    throw new Error(`Unsupported unit token "${token}"`);
  }

  const prefix = t.slice(0, t.length - baseLen);
  if (!(prefix in PREFIX_EXP)) {
    throw new Error(`Unsupported prefix "${prefix}" in "${token}"`);
  }
  const exp = PREFIX_EXP[prefix];

  return { base, exp };
}

function parseUnitRatio(unit: string): UnitRatio {
  const [numRaw, denRaw] = unit.split('/');
  if (!numRaw || !denRaw) throw new Error(`Expected a ratio like "pg/mL", got "${unit}"`);
  const numerator = parseUnitSingle(numRaw);
  const denominator = parseUnitSingle(denRaw);

  // Only mass/volume or mol/volume are supported
  if (!((numerator.base === 'g' || numerator.base === 'mol') && denominator.base === 'L')) {
    throw new Error(`Only mass/volume or mol/volume units are supported (got "${unit}")`);
  }

  return { numerator, denominator };
}

/**
 * Core conversion mirroring the Rust compute_result math:
 * value_out = value_in * 10^(in.num + out.den - in.den - out.num) * (mass swap factor)
 */
function convertCore(value: number, hormone: Hormone, from: UnitRatio, to: UnitRatio): number {
  const prefixCalc =
    (from.numerator.exp + to.denominator.exp) -
    (from.denominator.exp + to.numerator.exp);

  let baseFactor = 1;
  const M = MOLAR_MASS_G_PER_MOL[hormone];

  // Base swap factor between numerator bases
  const fromBase = from.numerator.base;
  const toBase = to.numerator.base;

  if (fromBase === 'mol' && toBase === 'g') baseFactor = M;
  else if (fromBase === 'g' && toBase === 'mol') baseFactor = 1 / M;
  else if ((fromBase === 'mol' && toBase === 'mol') || (fromBase === 'g' && toBase === 'g')) baseFactor = 1;
  else throw new Error(`Unsupported base conversion from ${fromBase} to ${toBase}`);

  // Use fixed exponent to avoid accumulating FP rounding in the prefix part
  const tenPow = Math.pow(10, prefixCalc);
  return value * tenPow * baseFactor;
}

/**
 * Public API
 * Example: convertHormone(100, 'Estradiol', 'pg/mL', 'pmol/L')
 */
export function convertHormone(value: number, hormone: Hormone, fromUnit: string, toUnit: string): number {
  const from = parseUnitRatio(fromUnit);
  const to = parseUnitRatio(toUnit);
  const out = convertCore(value, hormone, from, to);
  return out;
}

// Convenience helpers for common HRT analytes
export function convertEstradiol(value: number, fromUnit: 'pg/mL' | 'pmol/L', toUnit: 'pg/mL' | 'pmol/L'): number {
  return convertHormone(value, 'Estradiol', fromUnit, toUnit);
}
export function convertTestosterone(value: number, fromUnit: 'ng/dL' | 'nmol/L', toUnit: 'ng/dL' | 'nmol/L'): number {
  return convertHormone(value, 'Testosterone', fromUnit, toUnit);
}
export function convertProgesterone(value: number, fromUnit: 'ng/mL' | 'nmol/L', toUnit: 'ng/mL' | 'nmol/L'): number {
  return convertHormone(value, 'Progesterone', fromUnit, toUnit);
}
