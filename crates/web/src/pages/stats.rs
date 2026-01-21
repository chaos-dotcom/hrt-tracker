use leptos::*;

use crate::layout::page_layout;
use crate::store::use_store;
use hrt_shared::types::{DosageHistoryEntry, HormoneUnits, ProgesteroneRoutes};

#[component]
pub fn StatsPage() -> impl IntoView {
    let store = use_store();
    let data = store.data;
    let settings = store.settings;
    const DAY_MS: i64 = 24 * 60 * 60 * 1000;
    let hist = move || data.get().dosageHistory;
    let vials = move || data.get().vials;

    let total_days_since_start = move || {
        let hist_value = hist();
        let min_date = hist_value
            .iter()
            .map(|d| match d {
                DosageHistoryEntry::InjectableEstradiol { date, .. }
                | DosageHistoryEntry::OralEstradiol { date, .. }
                | DosageHistoryEntry::Antiandrogen { date, .. }
                | DosageHistoryEntry::Progesterone { date, .. } => *date,
            })
            .min();
        min_date.map(|value| ((js_sys::Date::now() as i64 - value) / DAY_MS).max(0))
    };

    let estrogen_records = move || {
        hist()
            .into_iter()
            .filter(|d| {
                matches!(
                    d,
                    DosageHistoryEntry::InjectableEstradiol { .. }
                        | DosageHistoryEntry::OralEstradiol { .. }
                )
            })
            .collect::<Vec<_>>()
    };

    let injectable_records = move || {
        hist()
            .into_iter()
            .filter(|d| matches!(d, DosageHistoryEntry::InjectableEstradiol { .. }))
            .collect::<Vec<_>>()
    };

    let total_injectable_estradiol_mg = move || {
        injectable_records()
            .iter()
            .map(|d| match d {
                DosageHistoryEntry::InjectableEstradiol { dose, unit, .. } => {
                    if *unit == HormoneUnits::Mg {
                        *dose
                    } else {
                        0.0
                    }
                }
                _ => 0.0,
            })
            .sum::<f64>()
    };

    let total_estrogen_mg = move || {
        estrogen_records()
            .iter()
            .map(|d| match d {
                DosageHistoryEntry::OralEstradiol {
                    dose,
                    unit,
                    pillQuantity,
                    ..
                } => {
                    if *unit != HormoneUnits::Mg {
                        0.0
                    } else {
                        let qty = pillQuantity.unwrap_or(1.0);
                        dose * qty
                    }
                }
                DosageHistoryEntry::InjectableEstradiol { dose, unit, .. } => {
                    if *unit == HormoneUnits::Mg {
                        *dose
                    } else {
                        0.0
                    }
                }
                _ => 0.0,
            })
            .sum::<f64>()
    };

    let total_injection_ml = move || {
        let vials_value = vials();
        injectable_records()
            .iter()
            .map(|d| match d {
                DosageHistoryEntry::InjectableEstradiol {
                    dose, unit, vialId, ..
                } => {
                    if *unit != HormoneUnits::Mg {
                        return 0.0;
                    }
                    let dose_value = *dose;
                    if !dose_value.is_finite() || dose_value <= 0.0 {
                        return 0.0;
                    }
                    let Some(vial_id) = vialId else {
                        return 0.0;
                    };
                    let conc = vials_value
                        .iter()
                        .find(|v| v.id == *vial_id)
                        .and_then(|v| v.concentrationMgPerMl)
                        .unwrap_or(0.0);
                    if conc > 0.0 {
                        dose_value / conc
                    } else {
                        0.0
                    }
                }
                _ => 0.0,
            })
            .sum::<f64>()
    };

    let oral_estradiol_records = move || {
        hist()
            .into_iter()
            .filter(|d| matches!(d, DosageHistoryEntry::OralEstradiol { .. }))
            .collect::<Vec<_>>()
    };

    let progesterone_records = move || {
        hist()
            .into_iter()
            .filter(|d| matches!(d, DosageHistoryEntry::Progesterone { .. }))
            .collect::<Vec<_>>()
    };

    let total_oral_pills_count = move || {
        oral_estradiol_records()
            .iter()
            .map(|d| match d {
                DosageHistoryEntry::OralEstradiol { pillQuantity, .. } => {
                    let qty = pillQuantity.unwrap_or(1.0);
                    if qty > 0.0 {
                        qty
                    } else {
                        1.0
                    }
                }
                _ => 0.0,
            })
            .sum::<f64>()
    };

    let total_oral_estradiol_mg = move || {
        oral_estradiol_records()
            .iter()
            .map(|d| match d {
                DosageHistoryEntry::OralEstradiol {
                    dose,
                    unit,
                    pillQuantity,
                    ..
                } => {
                    if *unit != HormoneUnits::Mg {
                        0.0
                    } else {
                        let qty = pillQuantity.unwrap_or(1.0);
                        dose * qty
                    }
                }
                _ => 0.0,
            })
            .sum::<f64>()
    };

    let total_progesterone_mg = move || {
        progesterone_records()
            .iter()
            .map(|d| match d {
                DosageHistoryEntry::Progesterone {
                    dose,
                    unit,
                    pillQuantity,
                    ..
                } => {
                    if *unit != HormoneUnits::Mg {
                        0.0
                    } else {
                        let qty = pillQuantity.unwrap_or(1.0);
                        dose * qty
                    }
                }
                _ => 0.0,
            })
            .sum::<f64>()
    };

    let boofed_progesterone_records = move || {
        progesterone_records()
            .into_iter()
            .filter(|d| {
                matches!(
                    d,
                    DosageHistoryEntry::Progesterone {
                        route: ProgesteroneRoutes::Boofed,
                        ..
                    }
                )
            })
            .collect::<Vec<_>>()
    };

    let boofed_progesterone_count = move || {
        boofed_progesterone_records()
            .iter()
            .map(|d| match d {
                DosageHistoryEntry::Progesterone { pillQuantity, .. } => {
                    let qty = pillQuantity.unwrap_or(1.0);
                    if qty > 0.0 {
                        qty
                    } else {
                        1.0
                    }
                }
                _ => 0.0,
            })
            .sum::<f64>()
    };

    let boofed_progesterone_mg = move || {
        boofed_progesterone_records()
            .iter()
            .map(|d| match d {
                DosageHistoryEntry::Progesterone {
                    dose,
                    unit,
                    pillQuantity,
                    ..
                } => {
                    if *unit != HormoneUnits::Mg {
                        0.0
                    } else {
                        let qty = pillQuantity.unwrap_or(1.0);
                        dose * qty
                    }
                }
                _ => 0.0,
            })
            .sum::<f64>()
    };

    let total_pills_count = move || total_oral_pills_count() + boofed_progesterone_count();
    let total_pills_mg_combined = move || total_oral_estradiol_mg() + boofed_progesterone_mg();

    let fmt = |value: f64, decimals: usize| {
        if !value.is_finite() {
            "-".to_string()
        } else {
            let formatted = format!("{value:.decimals$}");
            formatted
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string()
        }
    };

    let fmt_iu_from_ml = |ml: f64| {
        if !ml.is_finite() {
            "-".to_string()
        } else {
            format!("{}", (ml * 100.0).round() as i64)
        }
    };

    let parse_needle_length_mm = |raw: &str| {
        let cleaned = raw.trim().to_lowercase();
        if cleaned.is_empty() {
            return None;
        }
        if let Some(match_val) = cleaned.split_whitespace().find_map(|part| {
            if part.ends_with("mm") {
                part.trim_end_matches("mm").parse::<f64>().ok()
            } else {
                None
            }
        }) {
            return if match_val > 0.0 {
                Some(match_val)
            } else {
                None
            };
        }
        if let Some(match_val) = cleaned.split_whitespace().find_map(|part| {
            if part.ends_with("cm") {
                part.trim_end_matches("cm").parse::<f64>().ok()
            } else {
                None
            }
        }) {
            return if match_val > 0.0 {
                Some(match_val * 10.0)
            } else {
                None
            };
        }
        if let Some(inch_token) = cleaned.split_whitespace().find(|part| {
            part.ends_with("in")
                || part.ends_with("inch")
                || part.ends_with("inches")
                || part.ends_with('"')
        }) {
            let token = inch_token
                .trim_end_matches("in")
                .trim_end_matches("inch")
                .trim_end_matches("inches")
                .trim_end_matches('"');
            let value = if token.contains('/') {
                let parts: Vec<&str> = token.split_whitespace().collect();
                if parts.len() == 2 && parts[1].contains('/') {
                    let whole = parts[0].parse::<f64>().ok()?;
                    let frac: Vec<&str> = parts[1].split('/').collect();
                    if frac.len() != 2 {
                        return None;
                    }
                    let num = frac[0].parse::<f64>().ok()?;
                    let den = frac[1].parse::<f64>().ok()?;
                    if den == 0.0 {
                        return None;
                    }
                    Some(whole + num / den)
                } else if parts.len() == 1 {
                    let frac: Vec<&str> = parts[0].split('/').collect();
                    if frac.len() != 2 {
                        return None;
                    }
                    let num = frac[0].parse::<f64>().ok()?;
                    let den = frac[1].parse::<f64>().ok()?;
                    if den == 0.0 {
                        return None;
                    }
                    Some(num / den)
                } else {
                    None
                }
            } else {
                token.parse::<f64>().ok()
            }?;
            return if value > 0.0 {
                Some(value * 25.4)
            } else {
                None
            };
        }
        let numbers: Vec<f64> = cleaned
            .split(|c: char| !c.is_ascii_digit() && c != '.')
            .filter(|part| !part.is_empty())
            .filter_map(|part| part.parse::<f64>().ok())
            .collect();
        numbers.last().copied().filter(|val| *val > 0.0)
    };

    let norm_syringe_kind = |kind: &Option<String>| match kind.as_deref() {
        Some(value) if value == "Regular syringe" => value.to_string(),
        Some(value) if value == "Low waste syringe" => value.to_string(),
        Some(value) if value == "Low waste needle" => value.to_string(),
        Some(value) if value == "Insulin syringe" => value.to_string(),
        Some(value) if value == "Insulin pen" => value.to_string(),
        Some(value) if !value.trim().is_empty() => value.to_string(),
        _ => "Other".to_string(),
    };

    let deadspace_ul_for = |kind: &Option<String>| match kind.as_deref() {
        Some("Regular syringe") => Some(92.0),
        Some("Low waste syringe") => Some(59.0),
        Some("Low waste needle") => Some(17.0),
        Some("Insulin syringe") => Some(3.0),
        Some("Insulin pen") => Some(3.0),
        _ => None,
    };

    let by_kind_agg = move || {
        let vials_value = vials();
        let mut acc: std::collections::HashMap<String, (usize, f64, f64, f64, f64, f64)> =
            std::collections::HashMap::new();
        for entry in injectable_records() {
            let (syringe_kind, needle_length, dose, unit, vial_id) = match entry {
                DosageHistoryEntry::InjectableEstradiol {
                    syringeKind,
                    needleLength,
                    dose,
                    unit,
                    vialId,
                    ..
                } => (syringeKind, needleLength, dose, unit, vialId),
                _ => continue,
            };
            let key = norm_syringe_kind(&syringe_kind);
            let record = acc.entry(key).or_insert((0, 0.0, 0.0, 0.0, 0.0, 0.0));
            record.0 += 1;
            if let Some(mm) = needle_length
                .as_deref()
                .and_then(|value| parse_needle_length_mm(value))
            {
                if mm.is_finite() && mm > 0.0 {
                    record.1 += mm;
                }
            }
            if let Some(ds_ul) = deadspace_ul_for(&syringe_kind) {
                let ds_ml = ds_ul / 1000.0;
                record.2 += ds_ml;
                if let Some(vial_id) = vial_id {
                    if let Some(conc) = vials_value
                        .iter()
                        .find(|v| v.id == *vial_id)
                        .and_then(|v| v.concentrationMgPerMl)
                    {
                        if conc > 0.0 {
                            record.3 += conc * ds_ml;
                            if unit == HormoneUnits::Mg && dose > 0.0 {
                                let dose_ml = dose / conc;
                                record.4 += ds_ml;
                                record.5 += ds_ml + dose_ml;
                            }
                        }
                    }
                }
            }
        }
        acc
    };

    let wastage_agg = move || {
        let vials_value = vials();
        let mut total_ml = 0.0;
        let mut total_mg = 0.0;
        let mut skipped_no_kind = 0;
        let mut skipped_no_conc = 0;
        let mut counted = 0;
        let mut dead_for_pct_ml = 0.0;
        let mut drawn_for_pct_ml = 0.0;
        for entry in injectable_records() {
            let (syringe_kind, dose, unit, vial_id) = match entry {
                DosageHistoryEntry::InjectableEstradiol {
                    syringeKind,
                    dose,
                    unit,
                    vialId,
                    ..
                } => (syringeKind, dose, unit, vialId),
                _ => continue,
            };
            let Some(ds_ul) = deadspace_ul_for(&syringe_kind) else {
                skipped_no_kind += 1;
                continue;
            };
            let ds_ml = ds_ul / 1000.0;
            total_ml += ds_ml;
            counted += 1;
            if let Some(vial_id) = vial_id {
                if let Some(conc) = vials_value
                    .iter()
                    .find(|v| v.id == *vial_id)
                    .and_then(|v| v.concentrationMgPerMl)
                {
                    if conc > 0.0 {
                        total_mg += conc * ds_ml;
                        if unit == HormoneUnits::Mg && dose > 0.0 {
                            let dose_ml = dose / conc;
                            dead_for_pct_ml += ds_ml;
                            drawn_for_pct_ml += ds_ml + dose_ml;
                        }
                    }
                } else {
                    skipped_no_conc += 1;
                }
            }
        }
        (
            total_ml,
            total_mg,
            skipped_no_kind,
            skipped_no_conc,
            counted,
            dead_for_pct_ml,
            drawn_for_pct_ml,
        )
    };

    let wastage_pct = move || {
        let (_, _, _, _, _, dead_for_pct_ml, drawn_for_pct_ml) = wastage_agg();
        if drawn_for_pct_ml > 0.0 {
            (100.0 * dead_for_pct_ml) / drawn_for_pct_ml
        } else {
            f64::NAN
        }
    };

    let needle_agg = move || {
        let mut sum_mm = 0.0;
        let mut skipped = 0;
        for entry in injectable_records() {
            let needle = match entry {
                DosageHistoryEntry::InjectableEstradiol { needleLength, .. } => {
                    needleLength.clone()
                }
                _ => None,
            };
            let Some(value) = needle else {
                skipped += 1;
                continue;
            };
            if value.trim().is_empty() {
                skipped += 1;
                continue;
            }
            if let Some(mm) = parse_needle_length_mm(&value) {
                if mm > 0.0 {
                    sum_mm += mm;
                } else {
                    skipped += 1;
                }
            } else {
                skipped += 1;
            }
        }
        (sum_mm, skipped)
    };

    let stats_breakdown = move || settings.get().statsBreakdownBySyringeKind.unwrap_or(false);

    page_layout(
        "Stats",
        view! {
            <div class="view-layout">
                <div class="view-header">
                    <div>
                        <h2>"Stats"</h2>
                        <p class="muted">
                            "Totals and usage stats based on your recorded doses."
                        </p>
                    </div>
                </div>

                <div class="card-grid">
                    <div class="card">
                        <h3>"Total Estrogen Taken"</h3>
                        <p class="muted">"Injectable total"</p>
                        <p><strong>{move || fmt(total_injectable_estradiol_mg(), 2)}</strong> " mg"</p>
                        <p class="muted">"Oral total"</p>
                        <p><strong>{move || fmt(total_oral_estradiol_mg(), 2)}</strong> " mg"</p>
                        <p class="muted">"Combined"</p>
                        <p><strong>{move || fmt(total_estrogen_mg(), 2)}</strong> " mg"</p>
                        <Show when=move || { total_injection_ml() > 0.0 }>
                            <p class="muted">"Injection volume"</p>
                            <p>
                                <strong>{move || fmt(total_injection_ml(), 3)}</strong>
                                " mL (" <strong>{move || fmt_iu_from_ml(total_injection_ml())}</strong> " IU)"
                            </p>
                        </Show>
                    </div>
                    <div class="card">
                        <h3>"Pills"</h3>
                        <p>
                            "Estradiol pills:" <strong>{move || fmt(total_oral_pills_count(), 0)}</strong>
                            <Show when=move || { total_oral_pills_count() > 0.0 }>
                                " (" <strong>{move || fmt(total_oral_estradiol_mg(), 2)}</strong> " mg)"
                            </Show>
                        </p>
                        <p>
                            "Progesterone boofed:" <strong>{move || fmt(boofed_progesterone_count(), 0)}</strong>
                            <Show when=move || { boofed_progesterone_count() > 0.0 }>
                                " (" <strong>{move || fmt(boofed_progesterone_mg(), 2)}</strong> " mg)"
                            </Show>
                        </p>
                        <p>
                            "All pills combined:" <strong>{move || fmt(total_pills_count(), 0)}</strong>
                            <Show when=move || { total_pills_count() > 0.0 }>
                                " (" <strong>{move || fmt(total_pills_mg_combined(), 2)}</strong> " mg)"
                            </Show>
                        </p>
                        <Show when=move || { total_progesterone_mg() > 0.0 }>
                            <p class="muted">
                                "Total progesterone: "<strong>{move || fmt(total_progesterone_mg(), 2)}</strong> " mg"
                            </p>
                        </Show>
                    </div>
                    <div class="card">
                        <h3>"Days Since Starting"</h3>
                        <Show when=move || total_days_since_start().is_some() fallback=move || view! {
                            <p class="muted">"No doses recorded yet."</p>
                        }>
                            <p><strong>{move || total_days_since_start().unwrap_or(0)}</strong> " days"</p>
                        </Show>
                    </div>
                    <div class="card">
                        <h3>"Needle Usage"</h3>
                        <p>
                            "Total needle length: "
                            <strong>{move || fmt(needle_agg().0, 1)}</strong> " mm ("
                            <strong>{move || fmt(needle_agg().0 / 25.4, 2)}</strong> " in)"
                        </p>
                        <p>
                                "Wastage from dead space: "
                            <strong>{move || fmt(wastage_agg().0, 3)}</strong> " mL ("
                            <strong>{move || fmt_iu_from_ml(wastage_agg().0)}</strong> " IU)"
                            <Show when=move || { wastage_agg().1 > 0.0 }>
                                " · about " <strong>{move || fmt(wastage_agg().1, 2)}</strong> " mg"
                            </Show>

                            <Show when=move || wastage_pct().is_finite()>
                                " · " <strong>{move || fmt(wastage_pct(), 1)}</strong> "% of drawn volume"
                            </Show>
                        </p>
                        <Show when=move || { wastage_agg().2 > 0 || wastage_agg().3 > 0 }>
                            <p class="muted">
                                {move || {
                                    let skipped_kind = wastage_agg().2;
                                    let skipped_conc = wastage_agg().3;
                                    if skipped_kind > 0 && skipped_conc > 0 {
                                        format!("Skipped {} injection(s) without syringe kind. No mg estimate for {} injection(s) lacking vial concentration.", skipped_kind, skipped_conc)
                                    } else if skipped_kind > 0 {
                                        format!("Skipped {} injection(s) without a syringe kind.", skipped_kind)
                                    } else if skipped_conc > 0 {
                                        format!("No mg estimate for {} injection(s) lacking vial concentration.", skipped_conc)
                                    } else {
                                        "".to_string()
                                    }
                                }}
                            </p>
                        </Show>
                        <Show when=move || { needle_agg().1 > 0 }>
                            <p class="muted">
                                {move || format!("Skipped {} injection(s) without a parsable needle length.", needle_agg().1)}
                            </p>
                        </Show>
                    </div>
                </div>

                <div class="card">
                    <div class="view-header">
                        <div>
                            <h3>"Syringe Breakdown"</h3>
                            <p class="muted">"Enable to see needle and wastage details by syringe kind."</p>
                        </div>
                        <label class="muted">
                            <input
                                type="checkbox"
                                on:change={
                                    let store = store.clone();
                                    move |ev| {
                                        store.settings.update(|s| {
                                            s.statsBreakdownBySyringeKind = Some(event_target_checked(&ev));
                                        });
                                        store.mark_dirty();
                                    }
                                }
                                prop:checked=stats_breakdown
                            />
                            " Break down by syringe kind"
                        </label>
                    </div>
                    <Show when=move || stats_breakdown()>
                        <div class="card-grid">
                            <For
                                each=move || { by_kind_agg().into_iter().collect::<Vec<_>>() }
                                key=|(kind, _)| kind.clone()
                                children=move |(kind, values)| {
                                    let (count, sum_mm, dead_ml, total_mg, dead_pct_ml, drawn_pct_ml) = values;
                                    let pct = if drawn_pct_ml > 0.0 {
                                        (100.0 * dead_pct_ml) / drawn_pct_ml
                                    } else {
                                        f64::NAN
                                    };
                                    view! {
                                        <div class="mini-card">
                                            <h4>{kind}</h4>
                                            <p class="muted">{format!("Count: {}", count)}</p>
                                            <Show when=move || { sum_mm > 0.0 }>
                                                <p>{format!("Needle length: {} mm ({} in)", fmt(sum_mm, 1), fmt(sum_mm / 25.4, 2))}</p>
                                            </Show>
                                            <Show when=move || { dead_ml > 0.0 }>
                                                <p>{format!("Wastage: {} mL ({} IU)", fmt(dead_ml, 3), fmt_iu_from_ml(dead_ml))}</p>
                                            </Show>
                                            <Show when=move || { total_mg > 0.0 }>
                                                <p>{format!("about {} mg", fmt(total_mg, 2))}</p>
                                            </Show>
                                            <Show when=move || pct.is_finite()>
                                                <p>{format!("{}% of drawn volume", fmt(pct, 1))}</p>
                                            </Show>
                                        </div>
                                    }
                                }
                            />
                        </div>
                    </Show>
                </div>
            </div>
        }
        .into_view(),
    )
}
