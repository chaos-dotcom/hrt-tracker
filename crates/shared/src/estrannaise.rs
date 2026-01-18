use std::collections::HashMap;

use crate::types::EstrannaiseModel;

pub type PKParams = (f64, f64, f64, f64);

pub fn pk_parameters() -> HashMap<EstrannaiseModel, PKParams> {
    use EstrannaiseModel::*;
    HashMap::from([
        (EvIm, (478.0, 0.236, 4.85, 1.24)),
        (EEnIm, (191.4, 0.119, 0.601, 0.402)),
        (EcIm, (246.0, 0.0825, 3.57, 0.669)),
        (EbIm, (1893.1, 0.67, 61.5, 4.34)),
        (EUnIm, (471.5, 0.01729, 6.528, 2.285)),
        (EUnCasubq, (16.15, 0.046, 0.022, 0.101)),
        (PatchTw, (16.792, 0.283, 5.592, 4.3)),
        (PatchOw, (59.481, 0.107, 7.842, 5.193)),
    ])
}

fn e2_steady_state_3c(t: f64, dose: f64, t_cycle: f64, d: f64, k1: f64, k2: f64, k3: f64) -> f64 {
    let frac = t - t_cycle * (t / t_cycle).floor();
    dose * d
        * k1
        * k2
        * (f64::exp(-k1 * frac) / (1.0 - f64::exp(-k1 * t_cycle)) / (k1 - k2) / (k1 - k3)
            - f64::exp(-k2 * frac) / (1.0 - f64::exp(-k2 * t_cycle)) / (k1 - k2) / (k2 - k3)
            + f64::exp(-k3 * frac) / (1.0 - f64::exp(-k3 * t_cycle)) / (k1 - k3) / (k2 - k3))
}

fn e2_curve_3c(
    t: f64,
    dose: f64,
    d: f64,
    k1: f64,
    k2: f64,
    k3: f64,
    ds: f64,
    d2: f64,
    steady_state: bool,
    t_cycle: f64,
) -> f64 {
    if steady_state {
        return e2_steady_state_3c(t, dose, t_cycle, d, k1, k2, k3);
    }

    if t < 0.0 {
        return 0.0;
    }

    let mut ret = 0.0;

    if d2 > 0.0 {
        ret += d2 * f64::exp(-k3 * t);
    }

    if ds > 0.0 {
        if (k2 - k3).abs() < f64::EPSILON {
            ret += ds * k2 * t * f64::exp(-k2 * t);
        } else {
            ret += ds * k2 / (k2 - k3) * (f64::exp(-k3 * t) - f64::exp(-k2 * t));
        }
    }

    if dose > 0.0 && d > 0.0 {
        if (k1 - k2).abs() < f64::EPSILON && (k2 - k3).abs() < f64::EPSILON {
            ret += (dose * d * k1 * k1 * t * t * f64::exp(-k1 * t)) / 2.0;
        } else if (k1 - k2).abs() < f64::EPSILON && (k2 - k3).abs() >= f64::EPSILON {
            ret += (dose
                * d
                * k1
                * k1
                * (f64::exp(-k3 * t) - f64::exp(-k1 * t) * (1.0 + (k1 - k3) * t)))
                / (k1 - k3)
                / (k1 - k3);
        } else if (k1 - k3).abs() < f64::EPSILON && (k1 - k2).abs() >= f64::EPSILON {
            ret += (dose
                * d
                * k1
                * k2
                * (f64::exp(-k2 * t) - f64::exp(-k1 * t) * (1.0 + (k1 - k2) * t)))
                / (k1 - k2)
                / (k1 - k2);
        } else if (k2 - k3).abs() < f64::EPSILON && (k1 - k2).abs() >= f64::EPSILON {
            ret += (dose
                * d
                * k1
                * k2
                * (f64::exp(-k1 * t) - f64::exp(-k2 * t) * (1.0 - (k1 - k2) * t)))
                / (k1 - k2)
                / (k1 - k2);
        } else {
            ret += dose
                * d
                * k1
                * k2
                * (f64::exp(-k1 * t) / (k1 - k2) / (k1 - k3)
                    - f64::exp(-k2 * t) / (k1 - k2) / (k2 - k3)
                    + f64::exp(-k3 * t) / (k1 - k3) / (k2 - k3));
        }
    }

    if ret.is_nan() {
        0.0
    } else {
        ret
    }
}

fn pk_functions(conversion_factor: f64) -> HashMap<EstrannaiseModel, Box<dyn Fn(f64, f64) -> f64>> {
    let params = pk_parameters();
    let mut map: HashMap<EstrannaiseModel, Box<dyn Fn(f64, f64) -> f64>> = HashMap::new();

    for (model, (d, k1, k2, k3)) in params {
        map.insert(
            model,
            Box::new(move |t: f64, dose: f64| {
                e2_curve_3c(
                    t,
                    conversion_factor * dose,
                    d,
                    k1,
                    k2,
                    k3,
                    0.0,
                    0.0,
                    false,
                    1.0,
                )
            }),
        );
    }

    map
}

pub fn e2_multidose_3c(
    t: f64,
    doses: &[f64],
    times: &[f64],
    models: &[EstrannaiseModel],
    conversion_factor: f64,
    intervals: bool,
) -> f64 {
    let mut computed_times = times.to_vec();
    if intervals && !times.is_empty() {
        let mut sum = -times[0];
        computed_times = times
            .iter()
            .map(|v| {
                sum += v;
                sum
            })
            .collect();
    }

    let mut total = 0.0;
    let functions = pk_functions(conversion_factor);

    for (idx, dose) in doses.iter().enumerate() {
        let model = match models.get(idx) {
            Some(m) => m,
            None => continue,
        };
        let t_offset = match computed_times.get(idx) {
            Some(value) => t - *value,
            None => continue,
        };
        if let Some(func) = functions.get(model) {
            total += func(t_offset, *dose);
        }
    }

    total
}
