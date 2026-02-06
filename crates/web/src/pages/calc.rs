use leptos::*;

use crate::layout::page_layout;

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
    value
        .trim()
        .replace(',', ".")
        .parse::<f64>()
        .unwrap_or(f64::NAN)
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
    let tfs_dose_mg = create_rw_signal("4".to_string());
    let tfs_conc_mg_ml = create_rw_signal("40".to_string());
    let tfs_vol_ml = create_memo(move |_| {
        let dose = parse_num(&tfs_dose_mg.get());
        let conc = parse_num(&tfs_conc_mg_ml.get());
        if dose.is_finite() && conc.is_finite() && conc > 0.0 {
            dose / conc
        } else {
            f64::NAN
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
    let inj_vol_ml = create_memo(move |_| {
        let dose = parse_num(&cafe_dose_mg.get());
        let conc = parse_num(&cafe_conc_mg_ml.get());
        if dose.is_finite() && conc.is_finite() && conc > 0.0 {
            dose / conc
        } else {
            f64::NAN
        }
    });

    let gear_results = create_memo(move |_| {
        let dose = parse_num(&cafe_dose_mg.get());
        let freq = parse_num(&cafe_freq_days.get());
        let vial_ml = parse_num(&cafe_vial_ml.get());
        let conc = parse_num(&cafe_conc_mg_ml.get());
        GEARS
            .iter()
            .map(|gear| calc_for(*gear, dose, freq, vial_ml, conc))
            .collect::<Vec<_>>()
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
                                    "Dose (mg)"
                                    <input
                                        type="number"
                                        min="0"
                                        step="any"
                                        on:input=move |ev| tfs_dose_mg.set(event_target_value(&ev))
                                        prop:value=move || tfs_dose_mg.get()
                                    />
                                </label>
                                <label class="calc-field">
                                    "Concentration (mg/mL)"
                                    <input
                                        type="number"
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
                                        type="number"
                                        min="0"
                                        step="any"
                                        on:input=move |ev| tfs_vol2_ml.set(event_target_value(&ev))
                                        prop:value=move || tfs_vol2_ml.get()
                                    />
                                </label>
                                <label class="calc-field">
                                    "Concentration (mg/mL)"
                                    <input
                                        type="number"
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
                            "I am injecting (mg)"
                            <input
                                type="number"
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
                                type="number"
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
                                type="number"
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
                                type="number"
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
