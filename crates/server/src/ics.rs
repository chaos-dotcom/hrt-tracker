use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::Response;
use chrono::{Datelike, TimeZone, Timelike};
use serde::Deserialize;
use serde_json::Value;

use crate::storage::{read_data_value, read_settings_value};

const DAY_MS: i64 = 24 * 60 * 60 * 1000;

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct IcsQuery {
    pub horizonDays: Option<String>,
    pub includePast: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct IcsOptions {
    pub horizon_days: i64,
    pub include_past: bool,
    pub now_ms: i64,
}

pub async fn get_public_ics(Query(query): Query<IcsQuery>) -> Response {
    let mut conf = serde_json::json!({});
    if let Ok(Some(value)) = read_settings_value().await {
        conf = value;
    }

    if conf
        .get("icsSecret")
        .and_then(|v| v.as_str())
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false)
    {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body("Not found".into())
            .unwrap();
    }

    build_ics(query, conf).await
}

pub async fn get_secret_ics(Path(secret): Path<String>, Query(query): Query<IcsQuery>) -> Response {
    let mut conf = serde_json::json!({});
    if let Ok(Some(value)) = read_settings_value().await {
        conf = value;
    }

    let configured = conf
        .get("icsSecret")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();

    if configured.is_empty() || secret.trim() != configured {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body("Not found".into())
            .unwrap();
    }

    build_ics(query, conf).await
}

async fn build_ics(query: IcsQuery, conf: Value) -> Response {
    let horizon_days = parse_horizon_days(query.horizonDays.as_deref());
    let include_past = parse_include_past(query.includePast.as_deref());
    let now_ms = chrono::Utc::now().timestamp_millis();

    let options = IcsOptions {
        horizon_days,
        include_past,
        now_ms,
    };

    let data = match read_data_value().await {
        Ok(Some(value)) => value,
        Ok(None) => serde_json::json!({}),
        Err(_) => serde_json::json!({}),
    };

    let calendar = generate_ics(&data, &conf, options);

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/calendar; charset=utf-8")
        .header("Cache-Control", "no-cache, no-store, must-revalidate")
        .header("Pragma", "no-cache")
        .header("Expires", "0")
        .body(calendar.into())
        .unwrap()
}

fn parse_horizon_days(raw: Option<&str>) -> i64 {
    match raw.and_then(|v| v.parse::<f64>().ok()) {
        Some(value) if value > 0.0 => value as i64,
        _ => 365,
    }
}

fn parse_include_past(raw: Option<&str>) -> bool {
    match raw {
        None => true,
        Some(value) => value != "0",
    }
}

pub fn generate_ics(data: &Value, conf: &Value, options: IcsOptions) -> String {
    let horizon_end = options.now_ms + options.horizon_days * DAY_MS;
    let mut events: Vec<String> = Vec::new();

    if options.include_past {
        if let Some(history) = data.get("dosageHistory").and_then(|v| v.as_array()) {
            for d in history {
                let date = match d.get("date").and_then(|v| v.as_i64()) {
                    Some(value) => value,
                    None => continue,
                };
                let medication_type = d
                    .get("medicationType")
                    .and_then(|v| v.as_str())
                    .unwrap_or("medication");
                let name = d.get("type").and_then(|v| v.as_str()).unwrap_or("");
                let qty = d.get("dose").map(|v| v.to_string()).unwrap_or_default();
                let unit = d.get("unit").and_then(|v| v.as_str()).unwrap_or("mg");
                let site = d
                    .get("injectionSite")
                    .and_then(|v| v.as_str())
                    .map(|s| format!("; Site: {}", s))
                    .unwrap_or_default();
                let note = d
                    .get("note")
                    .and_then(|v| v.as_str())
                    .map(|s| format!("; Note: {}", s))
                    .unwrap_or_default();
                let summary = format!(
                    "{}: {} {} {}",
                    summary_for_medication(medication_type),
                    name,
                    qty,
                    unit
                )
                .trim()
                .to_string();
                let desc = format!("Recorded dose{}{}", site, note).trim().to_string();
                let uid = format!("{}-{}-history@hrt-tracker", medication_type, date);
                events.push(make_event(
                    &uid,
                    date,
                    &summary,
                    Some(&desc),
                    options.now_ms,
                ));
            }
        }
    }

    let regimen_keys = [
        "injectableEstradiol",
        "oralEstradiol",
        "antiandrogen",
        "progesterone",
    ];
    for key in regimen_keys {
        let sched = match data.get(key) {
            Some(value) => value,
            None => continue,
        };
        let freq_days = sched
            .get("frequency")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        if freq_days <= 0.0 {
            continue;
        }
        let step = (freq_days * DAY_MS as f64) as i64;

        let last_taken_dates: Vec<i64> = data
            .get("dosageHistory")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|d| {
                        if d.get("medicationType").and_then(|v| v.as_str()) == Some(key) {
                            d.get("date").and_then(|v| v.as_i64())
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        let mut t = if !last_taken_dates.is_empty() {
            last_taken_dates.into_iter().max().unwrap() + step
        } else if let Some(next) = sched.get("nextDoseDate").and_then(|v| v.as_i64()) {
            next
        } else {
            continue;
        };

        let today_start_utc = utc_day_start(options.now_ms);
        while t < today_start_utc {
            t += step;
        }

        while t <= horizon_end {
            let name = sched.get("type").and_then(|v| v.as_str()).unwrap_or("");
            let qty = sched.get("dose").map(|v| v.to_string()).unwrap_or_default();
            let unit = sched.get("unit").and_then(|v| v.as_str()).unwrap_or("mg");
            let route = if key == "progesterone" {
                sched
                    .get("route")
                    .and_then(|v| v.as_str())
                    .map(|r| format!(" ({})", r))
                    .unwrap_or_default()
            } else {
                String::new()
            };
            let summary = format!(
                "Scheduled {}{}: {} {} {}",
                summary_for_medication(key),
                route,
                name,
                qty,
                unit
            )
            .trim()
            .to_string();
            let desc = format!("Scheduled per regimen; every {} day(s).", freq_days);
            let uid = format!("{}-{}-scheduled@hrt-tracker", key, t);
            events.push(make_event(&uid, t, &summary, Some(&desc), options.now_ms));
            t += step;
        }
    }

    if let Some(true) = conf
        .get("enableBloodTestSchedule")
        .and_then(|v| v.as_bool())
    {
        if let Some(interval_months) = conf.get("bloodTestIntervalMonths").and_then(|v| v.as_i64())
        {
            if interval_months > 0 {
                if let Some(blood_tests) = data.get("bloodTests").and_then(|v| v.as_array()) {
                    let last_dates: Vec<i64> = blood_tests
                        .iter()
                        .filter_map(|b| b.get("date").and_then(|v| v.as_i64()))
                        .collect();
                    if let Some(last) = last_dates.into_iter().max() {
                        let mut t = add_months_utc(last, interval_months);
                        t = set_local_morning_10(t);
                        while t <= options.now_ms {
                            t = add_months_utc(t, interval_months);
                            t = set_local_morning_10(t);
                        }
                        while t <= horizon_end {
                            let uid = format!("bloodtest-{}-scheduled@hrt-tracker", t);
                            let summary = "Scheduled Blood Test";
                            let desc =
                                format!("Routine blood test every {} month(s).", interval_months);
                            events.push(make_event(&uid, t, summary, Some(&desc), options.now_ms));
                            t = add_months_utc(t, interval_months);
                            t = set_local_morning_10(t);
                        }
                    }
                }
            }
        }
    }

    let mut lines = vec![
        "BEGIN:VCALENDAR".to_string(),
        "PRODID:-//HRT Tracker//EN".to_string(),
        "VERSION:2.0".to_string(),
        "CALSCALE:GREGORIAN".to_string(),
        "METHOD:PUBLISH".to_string(),
        "X-WR-CALNAME:HRT Doses".to_string(),
        "X-WR-TIMEZONE:UTC".to_string(),
    ];
    lines.extend(events);
    lines.push("END:VCALENDAR".to_string());

    lines.join("\r\n")
}

fn summary_for_medication(medication_type: &str) -> &str {
    match medication_type {
        "injectableEstradiol" => "Injection",
        "oralEstradiol" => "Oral Estradiol",
        "antiandrogen" => "Antiandrogen",
        "progesterone" => "Progesterone",
        _ => "Medication",
    }
}

fn make_event(
    uid: &str,
    start_ms: i64,
    summary: &str,
    description: Option<&str>,
    now_ms: i64,
) -> String {
    let mut lines = vec![
        "BEGIN:VEVENT".to_string(),
        format!("UID:{}", uid),
        format!("DTSTAMP:{}", to_ics_date_time(now_ms)),
        format!("DTSTART:{}", to_ics_date_time(start_ms)),
        "DURATION:PT5M".to_string(),
        format!("SUMMARY:{}", escape_text(summary)),
        "CATEGORIES:HRT".to_string(),
        "TRANSP:OPAQUE".to_string(),
        "END:VEVENT".to_string(),
    ];
    if let Some(desc) = description {
        lines.insert(6, format!("DESCRIPTION:{}", escape_text(desc)));
    }
    lines.join("\r\n")
}

fn escape_text(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace(';', "\\;")
        .replace(',', "\\,")
        .replace("\r\n", "\\n")
        .replace('\n', "\\n")
}

fn to_ics_date_time(ms: i64) -> String {
    let dt = chrono::Utc
        .timestamp_millis_opt(ms)
        .single()
        .unwrap_or_else(|| chrono::Utc::now());
    format!(
        "{:04}{:02}{:02}T{:02}{:02}{:02}Z",
        dt.year(),
        dt.month(),
        dt.day(),
        dt.hour(),
        dt.minute(),
        dt.second()
    )
}

fn add_months_utc(ms: i64, months: i64) -> i64 {
    let dt = chrono::Utc
        .timestamp_millis_opt(ms)
        .single()
        .unwrap_or_else(|| chrono::Utc::now());
    let target_month = dt.month0() as i64 + months;
    let year = dt.year() + (target_month / 12) as i32;
    let month0 = (target_month % 12) as u32;
    let last_day = chrono::Utc
        .with_ymd_and_hms(year, month0 + 1, 1, dt.hour(), dt.minute(), dt.second())
        .single()
        .and_then(|d| d.checked_sub_signed(chrono::Duration::days(1)))
        .map(|d| d.day())
        .unwrap_or(dt.day());
    let day = dt.day().min(last_day);
    chrono::Utc
        .with_ymd_and_hms(year, month0 + 1, day, dt.hour(), dt.minute(), dt.second())
        .single()
        .map(|d| d.timestamp_millis())
        .unwrap_or(ms)
}

fn set_local_morning_10(ms: i64) -> i64 {
    let dt = chrono::Local
        .timestamp_millis_opt(ms)
        .single()
        .unwrap_or_else(|| chrono::Local::now());
    let dt = dt
        .with_hour(10)
        .and_then(|d| d.with_minute(0))
        .and_then(|d| d.with_second(0))
        .and_then(|d| d.with_nanosecond(0))
        .unwrap_or(dt);
    dt.timestamp_millis()
}

fn utc_day_start(ms: i64) -> i64 {
    let dt = chrono::Utc
        .timestamp_millis_opt(ms)
        .single()
        .unwrap_or_else(|| chrono::Utc::now());
    chrono::Utc
        .with_ymd_and_hms(dt.year(), dt.month(), dt.day(), 0, 0, 0)
        .single()
        .map(|d| d.timestamp_millis())
        .unwrap_or(ms)
}
