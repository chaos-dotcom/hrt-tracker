use leptos::*;

use crate::layout::page_layout;
use crate::store::use_store;
use crate::utils::parse_decimal_or_nan;

#[derive(Clone, Copy)]
struct Gear {
    name: &'static str,
    dead_ul: f64,
}

#[derive(Clone, PartialEq)]
struct GearResult {
    name: &'static str,
    doses: i64,
    days: i64,
    pct_waste: f64,
    dead_ul: f64,
}

const GEARS: [Gear; 4] = [
    Gear {
        name: "Regular Syringe",
        dead_ul: 92.0,
    },
    Gear {
        name: "Low Waste Syringe",
        dead_ul: 59.0,
    },
    Gear {
        name: "Low Waste Needle",
        dead_ul: 17.0,
    },
    Gear {
        name: "Insulin Syringe (SubQ Only)",
        dead_ul: 3.0,
    },
];

fn parse_num(value: &str) -> f64 {
    parse_decimal_or_nan(value)
}

fn fmt(value: f64, decimals: usize) -> String {
    if !value.is_finite() {
        return "—".to_string();
    }
    let formatted = format!("{value:.decimals$}");
    formatted
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

fn fmt_iu_from_ml(ml: f64) -> String {
    if !ml.is_finite() {
        return "—".to_string();
    }
    format!("{}", (ml * 100.0).round() as i64)
}

fn fmt_pct(value: f64) -> String {
    if !value.is_finite() {
        return "—".to_string();
    }
    if value < 2.0 {
        let rounded = (value * 10.0).round() / 10.0;
        let formatted = format!("{rounded:.1}");
        formatted
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    } else {
        format!("{}", value.round() as i64)
    }
}

#[allow(clippy::neg_cmp_op_on_partial_ord)]
fn calc_for(gear: Gear, dose: f64, freq: f64, vial_ml: f64, conc: f64) -> GearResult {
    if !(vial_ml > 0.0) || !(conc > 0.0) || !(dose > 0.0) || !(freq > 0.0) {
        return GearResult {
            name: gear.name,
            doses: 0,
            days: 0,
            pct_waste: f64::NAN,
            dead_ul: gear.dead_ul,
        };
    }
    let dead_ml = gear.dead_ul / 1000.0;
    let dose_ml = dose / conc;
    let drawn_ml = dose_ml + dead_ml;
    let raw_count = vial_ml / drawn_ml;
    let doses = raw_count.round() as i64;
    let days = (raw_count * freq).round() as i64;
    let pct_waste = 100.0 * (dead_ml / drawn_ml);

    GearResult {
        name: gear.name,
        doses,
        days,
        pct_waste,
        dead_ul: gear.dead_ul,
    }
}

#[component]
pub fn CalcPage() -> impl IntoView {
    let store = use_store();
    let settings = store.settings;

    let tfs_dose_mg = create_rw_signal("4".to_string());
    let tfs_conc_mg_ml = create_rw_signal("40".to_string());
    let tfs_vol_ml = create_memo({
        let settings = settings;
        move |_| {
            let dose = parse_num(&tfs_dose_mg.get());
            let conc = parse_num(&tfs_conc_mg_ml.get());
            let dose_mg = if settings.get().displayInjectableInIU.unwrap_or(false) {
                if dose.is_finite() && conc.is_finite() && conc > 0.0 {
                    (dose / 100.0) * conc
                } else {
                    f64::NAN
                }
            } else {
                dose
            };
            if dose_mg.is_finite() && conc.is_finite() && conc > 0.0 {
                dose_mg / conc
            } else {
                f64::NAN
            }
        }
    });

    let tfs_vol2_ml = create_rw_signal("0.1".to_string());
    let tfs_conc2_mg_ml = create_rw_signal("40".to_string());
    let tfs_dose2_mg = create_memo(move |_| {
        let vol = parse_num(&tfs_vol2_ml.get());
        let conc = parse_num(&tfs_conc2_mg_ml.get());
        if vol.is_finite() && conc.is_finite() && conc > 0.0 {
            vol * conc
        } else {
            f64::NAN
        }
    });

    let cafe_dose_mg = create_rw_signal("4".to_string());
    let cafe_freq_days = create_rw_signal("7".to_string());
    let cafe_vial_ml = create_rw_signal("10".to_string());
    let cafe_conc_mg_ml = create_rw_signal("40".to_string());
    let inj_vol_ml = create_memo({
        let settings = settings;
        move |_| {
            let dose = parse_num(&cafe_dose_mg.get());
            let conc = parse_num(&cafe_conc_mg_ml.get());
            let dose_mg = if settings.get().displayInjectableInIU.unwrap_or(false) {
                if dose.is_finite() && conc.is_finite() && conc > 0.0 {
                    (dose / 100.0) * conc
                } else {
                    f64::NAN
                }
            } else {
                dose
            };
            if dose_mg.is_finite() && conc.is_finite() && conc > 0.0 {
                dose_mg / conc
            } else {
                f64::NAN
            }
        }
    });

    let gear_results = create_memo({
        let settings = settings;
        move |_| {
            let dose = parse_num(&cafe_dose_mg.get());
            let freq = parse_num(&cafe_freq_days.get());
            let vial_ml = parse_num(&cafe_vial_ml.get());
            let conc = parse_num(&cafe_conc_mg_ml.get());
            let dose_mg = if settings.get().displayInjectableInIU.unwrap_or(false) {
                if dose.is_finite() && conc.is_finite() && conc > 0.0 {
                    (dose / 100.0) * conc
                } else {
                    f64::NAN
                }
            } else {
                dose
            };
            GEARS
                .iter()
                .map(|gear| calc_for(*gear, dose_mg, freq, vial_ml, conc))
                .collect::<Vec<_>>()
        }
    });

    page_layout(
        "Calculators",
        view! {
            <div class="view-layout">
                <section class="card">
                    <h2>"Dose, Volume, and Concentration Conversion (from Transfeminine Science)"</h2>
                    <div class="calc-grid">
                        <div class="calc-block">
                            <h3>"Dose and Concentration to Volume"</h3>
                            <div class="calc-grid">
                                <label class="calc-field">
                                    {move || {
                                        if settings.get().displayInjectableInIU.unwrap_or(false) {
                                            "Dose (IU)"
                                        } else {
                                            "Dose (mg)"
                                        }
                                    }}
                                    <input
                                        type="text"
                                        min="0"
                                        step="any"
                                        on:input=move |ev| tfs_dose_mg.set(event_target_value(&ev))
                                        prop:value=move || tfs_dose_mg.get()
                                    />
                                </label>
                                <label class="calc-field">
                                    "Concentration (mg/mL)"
                                    <input
                                        type="text"
                                        min="0"
                                        step="any"
                                        on:input=move |ev| tfs_conc_mg_ml.set(event_target_value(&ev))
                                        prop:value=move || tfs_conc_mg_ml.get()
                                    />
                                </label>
                            </div>
                            <p class="muted">
                                "Volume = Dose ÷ Concentration = "
                                <strong>{move || fmt(tfs_vol_ml.get(), 3)}</strong>
                                " mL"
                                <Show when=move || tfs_vol_ml.get().is_finite()>
                                    <span>
                                        " ("
                                        <strong>{move || fmt_iu_from_ml(tfs_vol_ml.get())}</strong>
                                        " IU)"
                                    </span>
                                </Show>
                            </p>
                        </div>
                        <div class="calc-block">
                            <h3>"Volume and Concentration to Dose"</h3>
                            <div class="calc-grid">
                                <label class="calc-field">
                                    "Volume (mL)"
                                    <input
                                        type="text"
                                        min="0"
                                        step="any"
                                        on:input=move |ev| tfs_vol2_ml.set(event_target_value(&ev))
                                        prop:value=move || tfs_vol2_ml.get()
                                    />
                                </label>
                                <label class="calc-field">
                                    "Concentration (mg/mL)"
                                    <input
                                        type="text"
                                        min="0"
                                        step="any"
                                        on:input=move |ev| tfs_conc2_mg_ml.set(event_target_value(&ev))
                                        prop:value=move || tfs_conc2_mg_ml.get()
                                    />
                                </label>
                            </div>
                            <p class="muted">
                                "Volume ≈ "
                                <strong>{move || fmt_iu_from_ml(parse_num(&tfs_vol2_ml.get()))}</strong>
                                " IU"
                            </p>
                            <p class="muted">
                                "Dose = Volume × Concentration = "
                                <strong>{move || fmt(tfs_dose2_mg.get(), 3)}</strong>
                                " mg"
                            </p>
                        </div>
                    </div>
                    <details>
                        <summary>"Notes"</summary>
                        <ol>
                            <li>"Volume is meaningless without concentration (for understanding dose)."</li>
                            <li>"State what you use in terms of dose; it’s more interpretable."</li>
                        </ol>
                    </details>
                </section>

                <section class="card">
                    <h2>"Vial Life & Dose Calculator (from HRT Cafe)"</h2>
                    <div class="calc-grid">
                        <label class="calc-field">
                            {move || {
                                if settings.get().displayInjectableInIU.unwrap_or(false) {
                                    "I am injecting (IU)"
                                } else {
                                    "I am injecting (mg)"
                                }
                            }}
                            <input
                                type="text"
                                min="0"
                                step="any"
                                inputmode="decimal"
                                on:input=move |ev| cafe_dose_mg.set(event_target_value(&ev))
                                prop:value=move || cafe_dose_mg.get()
                            />
                        </label>
                        <label class="calc-field">
                            "Every (days)"
                            <input
                                type="text"
                                min="0"
                                step="any"
                                inputmode="decimal"
                                on:input=move |ev| cafe_freq_days.set(event_target_value(&ev))
                                prop:value=move || cafe_freq_days.get()
                            />
                        </label>
                        <label class="calc-field">
                            "My vial is (mL)"
                            <input
                                type="text"
                                min="0"
                                step="any"
                                inputmode="decimal"
                                on:input=move |ev| cafe_vial_ml.set(event_target_value(&ev))
                                prop:value=move || cafe_vial_ml.get()
                            />
                        </label>
                        <label class="calc-field">
                            "At concentration (mg/mL)"
                            <input
                                type="text"
                                min="0"
                                step="any"
                                inputmode="decimal"
                                on:input=move |ev| cafe_conc_mg_ml.set(event_target_value(&ev))
                                prop:value=move || cafe_conc_mg_ml.get()
                            />
                        </label>
                    </div>
                    <p class="muted">
                        "Inject a volume of "
                        <strong>{move || fmt(inj_vol_ml.get(), 3)}</strong>
                        " mL"
                        <Show when=move || inj_vol_ml.get().is_finite()>
                            <span>
                                " ("
                                <strong>{move || fmt_iu_from_ml(inj_vol_ml.get())}</strong>
                                " IU)"
                            </span>
                        </Show>
                    </p>
                    <h3>"Estimated Vial Lifetime"</h3>
                    <div class="card-grid">
                        {move || {
                            gear_results
                                .get()
                                .into_iter()
                                .map(|result| {
                                    view! {
                                        <div class="mini-card">
                                            <h4>{result.name}</h4>
                                            <p><strong>{result.doses}</strong> " doses"</p>
                                            <p><strong>{result.days}</strong> " days"</p>
                                            <p><strong>{fmt_pct(result.pct_waste)}</strong> " pct waste"</p>
                                            <p>{format!("{} uL dead space", result.dead_ul.round() as i64)}</p>
                                        </div>
                                    }
                                })
                                .collect_view()
                        }}
                    </div>
                    <details>
                        <summary>"Notes"</summary>
                        <ul>
                            <li>"Waste% = dead space / (dead space + drawn dose) per injection."</li>
                            <li>"Doses ≈ round(vial mL / (dose mL + dead space mL))."</li>
                            <li>"Days ≈ round((vial mL / (dose mL + dead space mL)) × frequency days)."</li>
                            <li>"Estimates only; keep a spare vial and use sterile technique."</li>
                        </ul>
                    </details>
                </section>
            </div>
        }
        .into_view(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fmt_trims_trailing_zeros() {
        assert_eq!(fmt(1.0, 3), "1");
        assert_eq!(fmt(1.5, 3), "1.5");
        assert_eq!(fmt(1.500, 3), "1.5");
        assert_eq!(fmt(0.125, 3), "0.125");
        assert_eq!(fmt(1234.0, 2), "1234");
    }

    #[test]
    fn fmt_non_finite() {
        assert_eq!(fmt(f64::NAN, 3), "\u{2014}");
        assert_eq!(fmt(f64::INFINITY, 3), "\u{2014}");
    }

    #[test]
    fn fmt_iu_from_ml_basic() {
        assert_eq!(fmt_iu_from_ml(0.1), "10");
        assert_eq!(fmt_iu_from_ml(0.25), "25");
        assert_eq!(fmt_iu_from_ml(1.0), "100");
    }

    #[test]
    fn fmt_iu_from_ml_non_finite() {
        assert_eq!(fmt_iu_from_ml(f64::NAN), "\u{2014}");
    }

    #[test]
    fn fmt_pct_small_values() {
        assert_eq!(fmt_pct(0.5), "0.5");
        assert_eq!(fmt_pct(1.0), "1");
        assert_eq!(fmt_pct(1.5), "1.5");
        assert_eq!(fmt_pct(1.99), "2");
    }

    #[test]
    fn fmt_pct_large_values() {
        assert_eq!(fmt_pct(5.0), "5");
        assert_eq!(fmt_pct(10.7), "11");
        assert_eq!(fmt_pct(50.0), "50");
    }

    #[test]
    fn fmt_pct_non_finite() {
        assert_eq!(fmt_pct(f64::NAN), "\u{2014}");
        assert_eq!(fmt_pct(f64::INFINITY), "\u{2014}");
    }

    #[test]
    fn calc_for_regular_syringe_known_values() {
        let gear = Gear { name: "Test", dead_ul: 92.0 };
        let result = calc_for(gear, 4.0, 7.0, 10.0, 40.0);
        // dose_ml = 4/40 = 0.1, dead_ml = 0.092, drawn = 0.192
        // raw_count = 10/0.192 = 52.08..., doses = 52
        // days = 52.08 * 7 = 364.58 -> 365
        // pct_waste = 100 * 0.092 / 0.192 = 47.9%
        assert_eq!(result.doses, 52);
        assert_eq!(result.days, 365);
        assert!((result.pct_waste - 47.9).abs() < 1.0, "got {}", result.pct_waste);
    }

    #[test]
    fn calc_for_insulin_syringe_known_values() {
        let gear = Gear { name: "Insulin", dead_ul: 3.0 };
        let result = calc_for(gear, 4.0, 7.0, 10.0, 40.0);
        // dose_ml = 0.1, dead_ml = 0.003, drawn = 0.103
        // raw_count = 10/0.103 = 97.08..., doses = 97
        // days = 97.08 * 7 = 679.6 -> 680
        assert_eq!(result.doses, 97);
        assert_eq!(result.days, 680);
    }

    #[test]
    fn calc_for_zero_dose_returns_empty() {
        let gear = Gear { name: "Test", dead_ul: 92.0 };
        let result = calc_for(gear, 0.0, 7.0, 10.0, 40.0);
        assert_eq!(result.doses, 0);
        assert_eq!(result.days, 0);
        assert!(result.pct_waste.is_nan());
    }

    #[test]
    fn calc_for_zero_conc_returns_empty() {
        let gear = Gear { name: "Test", dead_ul: 92.0 };
        let result = calc_for(gear, 4.0, 7.0, 10.0, 0.0);
        assert_eq!(result.doses, 0);
    }

    #[test]
    fn calc_for_nan_input_returns_empty() {
        let gear = Gear { name: "Test", dead_ul: 92.0 };
        let result = calc_for(gear, f64::NAN, 7.0, 10.0, 40.0);
        assert_eq!(result.doses, 0);
    }

    #[test]
    fn calc_for_all_gears_positive() {
        for gear in GEARS {
            let result = calc_for(gear, 4.0, 7.0, 10.0, 40.0);
            assert!(result.doses > 0, "{} produced 0 doses", gear.name);
            assert!(result.days > 0, "{} produced 0 days", gear.name);
            assert!(result.pct_waste > 0.0, "{} produced 0% waste", gear.name);
        }
    }

    #[test]
    fn lower_dead_volume_means_more_doses() {
        let regular = calc_for(GEARS[0], 4.0, 7.0, 10.0, 40.0);  // 92 uL dead
        let insulin = calc_for(GEARS[3], 4.0, 7.0, 10.0, 40.0);  // 3 uL dead
        assert!(insulin.doses > regular.doses, "insulin {} vs regular {}", insulin.doses, regular.doses);
        assert!(insulin.pct_waste < regular.pct_waste);
    }
}
