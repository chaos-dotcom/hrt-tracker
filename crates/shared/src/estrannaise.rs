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

#[allow(clippy::too_many_arguments)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::EstrannaiseModel;

    #[test]
    fn pk_parameters_has_all_models() {
        let params = pk_parameters();
        assert!(params.contains_key(&EstrannaiseModel::EvIm));
        assert!(params.contains_key(&EstrannaiseModel::EEnIm));
        assert!(params.contains_key(&EstrannaiseModel::EcIm));
        assert!(params.contains_key(&EstrannaiseModel::EbIm));
        assert!(params.contains_key(&EstrannaiseModel::EUnIm));
        assert!(params.contains_key(&EstrannaiseModel::EUnCasubq));
        assert!(params.contains_key(&EstrannaiseModel::PatchTw));
        assert!(params.contains_key(&EstrannaiseModel::PatchOw));
        assert_eq!(params.len(), 8);
    }

    #[test]
    fn single_dose_ev_peaks_then_decays() {
        // A single 4mg EV injection should peak within first few days then decay
        let doses = vec![4.0];
        let times = vec![0.0];
        let models = vec![EstrannaiseModel::EvIm];

        let at_zero = e2_multidose_3c(0.0, &doses, &times, &models, 1.0, false);
        let at_peak = e2_multidose_3c(2.0, &doses, &times, &models, 1.0, false);
        let at_late = e2_multidose_3c(14.0, &doses, &times, &models, 1.0, false);

        assert!(at_zero < at_peak, "should rise from t=0: {at_zero} vs {at_peak}");
        assert!(at_peak > at_late, "should decay after peak: {at_peak} vs {at_late}");
        assert!(at_peak > 0.0, "peak should be positive");
        assert!(at_late > 0.0, "late value should still be positive");
    }

    #[test]
    fn single_dose_ec_peaks_later_than_ev() {
        // EC (cypionate) has slower kinetics than EV (valerate)
        let doses = vec![5.0];
        let times = vec![0.0];

        let ev_at_1 = e2_multidose_3c(1.0, &doses, &times, &[EstrannaiseModel::EvIm], 1.0, false);
        let ec_at_1 = e2_multidose_3c(1.0, &doses, &times, &[EstrannaiseModel::EcIm], 1.0, false);

        // EV should be higher at day 1 (faster absorption)
        assert!(ev_at_1 > ec_at_1, "EV should rise faster: EV={ev_at_1} EC={ec_at_1}");
    }

    #[test]
    fn multidose_superposition() {
        let model = EstrannaiseModel::EvIm;
        // Two doses at different times
        let single_a = e2_multidose_3c(10.0, &[4.0], &[0.0], std::slice::from_ref(&model), 1.0, false);
        let single_b = e2_multidose_3c(10.0, &[4.0], &[5.0], std::slice::from_ref(&model), 1.0, false);
        let combined = e2_multidose_3c(10.0, &[4.0, 4.0], &[0.0, 5.0], &[model.clone(), model], 1.0, false);

        assert!((combined - (single_a + single_b)).abs() < 0.01,
            "multidose should be sum of individual: {combined} vs {}", single_a + single_b);
    }

    #[test]
    fn conversion_factor_scales_output() {
        let doses = vec![4.0];
        let times = vec![0.0];
        let models = vec![EstrannaiseModel::EvIm];

        let base = e2_multidose_3c(3.0, &doses, &times, &models, 1.0, false);
        let scaled = e2_multidose_3c(3.0, &doses, &times, &models, 2.0, false);

        assert!((scaled - base * 2.0).abs() < 0.01, "factor should scale linearly: {scaled} vs {}", base * 2.0);
    }

    #[test]
    fn intervals_mode_converts_to_absolute() {
        let model = EstrannaiseModel::EvIm;
        // intervals: [7, 7] means doses at day 0 and day 7
        let from_intervals = e2_multidose_3c(10.0, &[4.0, 4.0], &[7.0, 7.0], &[model.clone(), model.clone()], 1.0, true);
        let from_absolute = e2_multidose_3c(10.0, &[4.0, 4.0], &[0.0, 7.0], &[model.clone(), model.clone()], 1.0, false);

        assert!((from_intervals - from_absolute).abs() < 0.01,
            "intervals and absolute should match: {from_intervals} vs {from_absolute}");
    }

    #[test]
    fn empty_doses_returns_zero() {
        let result = e2_multidose_3c(5.0, &[], &[], &[], 1.0, false);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn negative_time_returns_zero() {
        // Before injection, level should be 0
        let result = e2_multidose_3c(-1.0, &[4.0], &[0.0], &[EstrannaiseModel::EvIm], 1.0, false);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn zero_dose_returns_zero() {
        let result = e2_multidose_3c(5.0, &[0.0], &[0.0], &[EstrannaiseModel::EvIm], 1.0, false);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn all_models_produce_positive_values() {
        let params = pk_parameters();
        for model in params.keys() {
            let result = e2_multidose_3c(3.0, &[5.0], &[0.0], std::slice::from_ref(model), 1.0, false);
            assert!(result > 0.0, "{:?} produced non-positive value: {result}", model);
        }
    }
}
