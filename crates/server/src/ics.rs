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

/// Progesterone is taken daily for most people, so its events are left out of the
/// feed unless the setting asks for them (`icsIncludeProgesterone: true`).
fn includes_progesterone(conf: &Value) -> bool {
    conf.get("icsIncludeProgesterone")
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
}

pub fn generate_ics(data: &Value, conf: &Value, options: IcsOptions) -> String {
    let include_progesterone = includes_progesterone(conf);
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
                if medication_type == "progesterone" && !include_progesterone {
                    continue;
                }
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
        if key == "progesterone" && !include_progesterone {
            continue;
        }
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
        if step <= 0 {
            // Positive in days but below one millisecond per dose: there is no
            // representable step, and `t += step` would never advance.
            continue;
        }

        let last_taken_dates: Vec<i64> = data
            .get("dosageHistory")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|d| {
                        let is_key = d.get("medicationType").and_then(|v| v.as_str()) == Some(key);
                        // A bonus dose is taken off-schedule; it does not move the
                        // regimen's grid. hrt_shared::logic skips it the same way.
                        let is_bonus =
                            d.get("bonusDose").and_then(|v| v.as_bool()).unwrap_or(false);
                        if is_key && !is_bonus {
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
                        // Occurrence `n` is anchored on the last test's day of the
                        // month and clamped to each target month's length. Feeding
                        // the previous occurrence back in instead eroded the anchor
                        // (the 31st became the 29th after February, then the 28th
                        // for good), so every later test moved a few days earlier.
                        let mut n: i64 = 1;
                        let mut t = set_local_morning_10(add_months_utc(last, interval_months * n));
                        while t <= options.now_ms {
                            n += 1;
                            t = set_local_morning_10(add_months_utc(last, interval_months * n));
                        }
                        while t <= horizon_end {
                            let uid = format!("bloodtest-{}-scheduled@hrt-tracker", t);
                            let summary = "Scheduled Blood Test";
                            let desc =
                                format!("Routine blood test every {} month(s).", interval_months);
                            events.push(make_event(&uid, t, summary, Some(&desc), options.now_ms));
                            n += 1;
                            t = set_local_morning_10(add_months_utc(last, interval_months * n));
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
        .unwrap_or_else(chrono::Utc::now);
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
        .unwrap_or_else(chrono::Utc::now);
    let target_month = dt.month0() as i64 + months;
    let year = dt.year() + (target_month / 12) as i32;
    let month0 = (target_month % 12) as u32;
    // Last day of the *target* month: the day before the first of the month
    // after it. Taking the day before the first of the target month clamped
    // against the previous month, so a shorter target month kept the day,
    // `with_ymd_and_hms` below rejected it, and the walk returned the input
    // unchanged — the caller's `t += step` loop then never advanced.
    let last_day = chrono::NaiveDate::from_ymd_opt(year, month0 + 1, 1)
        .and_then(|first| first.checked_add_months(chrono::Months::new(1)))
        .and_then(|next| next.checked_sub_days(chrono::Days::new(1)))
        .map(|last| last.day())
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
        .unwrap_or_else(chrono::Local::now);
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
        .unwrap_or_else(chrono::Utc::now);
    chrono::Utc
        .with_ymd_and_hms(dt.year(), dt.month(), dt.day(), 0, 0, 0)
        .single()
        .map(|d| d.timestamp_millis())
        .unwrap_or(ms)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn generate_ics_empty_data() {
        let data = json!({});
        let conf = json!({});
        let options = IcsOptions {
            horizon_days: 30,
            include_past: true,
            now_ms: 1700000000000,
        };
        let cal = generate_ics(&data, &conf, options);
        assert!(cal.contains("BEGIN:VCALENDAR"));
        assert!(cal.contains("END:VCALENDAR"));
        assert!(cal.contains("VERSION:2.0"));
        assert!(cal.contains("PRODID:-//HRT Tracker//EN"));
    }

    #[test]
    fn generate_ics_includes_dosage_history() {
        let data = json!({
            "dosageHistory": [{
                "date": 1700000000000_i64,
                "medicationType": "injectableEstradiol",
                "type": "Estradiol Valerate",
                "dose": 4,
                "unit": "mg"
            }]
        });
        let conf = json!({});
        let options = IcsOptions {
            horizon_days: 30,
            include_past: true,
            now_ms: 1700100000000,
        };
        let cal = generate_ics(&data, &conf, options);
        assert!(cal.contains("Injection"), "should contain medication summary");
        assert!(cal.contains("Estradiol Valerate"), "should contain drug name");
        assert!(cal.contains("Recorded dose"), "should contain description");
    }

    #[test]
    fn generate_ics_excludes_history_when_disabled() {
        let data = json!({
            "dosageHistory": [{
                "date": 1700000000000_i64,
                "medicationType": "injectableEstradiol",
                "type": "Estradiol Valerate",
                "dose": 4,
                "unit": "mg"
            }]
        });
        let conf = json!({});
        let options = IcsOptions {
            horizon_days: 30,
            include_past: false,
            now_ms: 1700100000000,
        };
        let cal = generate_ics(&data, &conf, options);
        assert!(!cal.contains("Recorded dose"), "should not contain past events");
    }

    #[test]
    fn generate_ics_includes_scheduled_doses() {
        let now = 1700000000000_i64;
        let data = json!({
            "injectableEstradiol": {
                "type": "Estradiol Valerate",
                "dose": 4,
                "unit": "mg",
                "frequency": 7,
                "nextDoseDate": now + 86400000
            }
        });
        let conf = json!({});
        let options = IcsOptions {
            horizon_days: 30,
            include_past: false,
            now_ms: now,
        };
        let cal = generate_ics(&data, &conf, options);
        assert!(cal.contains("Scheduled Injection"), "should contain scheduled events");
    }

    #[test]
    fn generate_ics_includes_blood_test_schedule() {
        let now = 1700000000000_i64;
        let data = json!({
            "bloodTests": [{"date": now - 86400000 * 90}]
        });
        let conf = json!({
            "enableBloodTestSchedule": true,
            "bloodTestIntervalMonths": 3
        });
        let options = IcsOptions {
            horizon_days: 365,
            include_past: false,
            now_ms: now,
        };
        let cal = generate_ics(&data, &conf, options);
        assert!(cal.contains("Scheduled Blood Test"), "should contain blood test events");
    }

    #[test]
    fn generate_ics_oral_estradiol_summary() {
        let now = 1700000000000_i64;
        let data = json!({
            "dosageHistory": [{
                "date": now,
                "medicationType": "oralEstradiol",
                "type": "Estradiol Hemihydrate",
                "dose": 2,
                "unit": "mg"
            }]
        });
        let conf = json!({});
        let options = IcsOptions {
            horizon_days: 30,
            include_past: true,
            now_ms: now + 1000,
        };
        let cal = generate_ics(&data, &conf, options);
        assert!(cal.contains("Oral Estradiol"), "should use oral estradiol summary");
    }

    #[test]
    fn generate_ics_progesterone_includes_route() {
        let now = 1700000000000_i64;
        let data = json!({
            "progesterone": {
                "type": "Micronized Progesterone",
                "route": "Oral",
                "dose": 200,
                "unit": "mg",
                "frequency": 1,
                "nextDoseDate": now + 86400000
            }
        });
        // Progesterone is opt-in in the feed, so ask for it here.
        let conf = json!({ "icsIncludeProgesterone": true });
        let options = IcsOptions {
            horizon_days: 7,
            include_past: false,
            now_ms: now,
        };
        let cal = generate_ics(&data, &conf, options);
        assert!(cal.contains("Progesterone"), "should contain progesterone events");
        assert!(cal.contains("Oral"), "should include route");
    }

    #[test]
    fn generate_ics_escapes_special_chars() {
        let now = 1700000000000_i64;
        let data = json!({
            "dosageHistory": [{
                "date": now,
                "medicationType": "injectableEstradiol",
                "type": "Test; Drug, Name",
                "dose": 4,
                "unit": "mg",
                "note": "line1\nline2"
            }]
        });
        let conf = json!({});
        let options = IcsOptions {
            horizon_days: 30,
            include_past: true,
            now_ms: now + 1000,
        };
        let cal = generate_ics(&data, &conf, options);
        assert!(cal.contains("\\;"), "should escape semicolons");
        assert!(cal.contains("\\,"), "should escape commas");
    }

    #[test]
    fn generate_ics_multiple_medication_types() {
        let now = 1700000000000_i64;
        let data = json!({
            "injectableEstradiol": {
                "type": "Estradiol Valerate",
                "dose": 4,
                "unit": "mg",
                "frequency": 7,
                "nextDoseDate": now + 86400000
            },
            "antiandrogen": {
                "type": "Cyproterone Acetate",
                "dose": 12.5,
                "unit": "mg",
                "frequency": 1,
                "nextDoseDate": now + 86400000
            }
        });
        let conf = json!({});
        let options = IcsOptions {
            horizon_days: 14,
            include_past: false,
            now_ms: now,
        };
        let cal = generate_ics(&data, &conf, options);
        assert!(cal.contains("Injection"), "should have injection events");
        assert!(cal.contains("Antiandrogen"), "should have antiandrogen events");
    }

    /// Parse every DTSTART in the feed and report it as a local calendar date.
    fn dtstart_local_days(calendar: &str) -> Vec<chrono::NaiveDate> {
        use chrono::TimeZone;
        calendar
            .split("\r\n")
            .filter_map(|line| line.strip_prefix("DTSTART:"))
            .map(|raw| {
                let naive = chrono::NaiveDateTime::parse_from_str(raw, "%Y%m%dT%H%M%SZ")
                    .unwrap_or_else(|err| panic!("bad DTSTART {raw:?}: {err}"));
                chrono::Local
                    .timestamp_millis_opt(chrono::Utc.from_utc_datetime(&naive).timestamp_millis())
                    .single()
                    .expect("valid instant")
                    .date_naive()
            })
            .collect()
    }

    /// A routine blood test recurs every N months *from the last test*, so every
    /// occurrence keeps that test's day of month (clamped only when the target
    /// month is shorter).
    #[test]
    fn blood_test_schedule_keeps_the_last_test_day_of_month() {
        let now: i64 = 1_709_596_800_000; // 2024-03-05T00:00:00Z
        let last_test: i64 = 1_706_691_600_000; // 2024-01-31T09:00:00Z
        let data = json!({ "bloodTests": [{ "date": last_test }] });
        let conf = json!({
            "enableBloodTestSchedule": true,
            "bloodTestIntervalMonths": 2
        });
        let options = IcsOptions {
            horizon_days: 200,
            include_past: false,
            now_ms: now,
        };

        let calendar = generate_ics(&data, &conf, options);

        let expected: Vec<chrono::NaiveDate> = ["2024-03-31", "2024-05-31", "2024-07-31"]
            .iter()
            .map(|raw| chrono::NaiveDate::parse_from_str(raw, "%Y-%m-%d").expect("valid date"))
            .collect();
        assert_eq!(
            dtstart_local_days(&calendar),
            expected,
            "every 2 months from a test on the 31st must stay on the 31st:\n{calendar}"
        );
    }

    /// The same walk must advance even when the target month is shorter than the
    /// last test's day of month; it used to loop forever on its own input.
    #[test]
    fn blood_test_schedule_after_a_test_on_the_31st_terminates() {
        let now: i64 = 1_709_596_800_000; // 2024-03-05T00:00:00Z
        let last_test: i64 = 1_706_691_600_000; // 2024-01-31T09:00:00Z
        let data = json!({ "bloodTests": [{ "date": last_test }] });
        let conf = json!({
            "enableBloodTestSchedule": true,
            "bloodTestIntervalMonths": 1
        });
        let options = IcsOptions {
            horizon_days: 400,
            include_past: false,
            now_ms: now,
        };

        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(generate_ics(&data, &conf, options));
        });

        assert!(
            rx.recv_timeout(std::time::Duration::from_secs(5)).is_ok(),
            "a monthly blood-test reminder after a test on the 31st must return"
        );
    }

    /// A positive frequency in days can still round to a zero-millisecond step;
    /// `t += step` then never advances, so the walk must skip the regimen.
    #[test]
    fn sub_millisecond_frequency_is_skipped_instead_of_spinning() {
        let now: i64 = 1_709_596_800_000; // 2024-03-05T00:00:00Z
        let data = json!({
            "dosageHistory": [{
                "date": now - 30 * 86_400_000_i64,
                "medicationType": "injectableEstradiol",
                "type": "Estradiol Valerate",
                "dose": 4,
                "unit": "mg"
            }],
            "injectableEstradiol": {
                "type": "Estradiol Valerate",
                "dose": 4,
                "unit": "mg",
                "frequency": 1e-9
            }
        });
        let options = IcsOptions {
            horizon_days: 30,
            include_past: false,
            now_ms: now,
        };

        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(generate_ics(&data, &json!({}), options));
        });

        let calendar = rx
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("a regimen with a zero-millisecond step must not spin");
        assert!(
            !calendar.contains("BEGIN:VEVENT"),
            "a zero step cannot schedule a dose:\n{calendar}"
        );
    }

    /// Every DTSTART in the feed, as milliseconds since the epoch.
    fn dtstart_millis(calendar: &str) -> Vec<i64> {
        use chrono::TimeZone;
        calendar
            .split("\r\n")
            .filter_map(|line| line.strip_prefix("DTSTART:"))
            .map(|raw| {
                let naive = chrono::NaiveDateTime::parse_from_str(raw, "%Y%m%dT%H%M%SZ")
                    .unwrap_or_else(|err| panic!("bad DTSTART {raw:?}: {err}"));
                chrono::Utc.from_utc_datetime(&naive).timestamp_millis()
            })
            .collect()
    }

    /// A bonus dose is an extra dose taken off-schedule, so it is not a new
    /// anchor: the shared scheduler ignores it when it derives the next dose
    /// (`hrt_shared::logic::backfill_scheduled_doses`), and the feed must agree.
    #[test]
    fn scheduled_doses_ignore_bonus_doses() {
        const DAY_MS: i64 = 86_400_000;
        const HOUR_MS: i64 = 3_600_000;
        let now: i64 = 1_709_596_800_000; // 2024-03-05T00:00:00Z, a UTC midnight
        let today_start = now;
        let real_dose = today_start - 10 * DAY_MS + 10 * HOUR_MS; // 2024-02-24T10:00Z
        let bonus_dose = today_start - DAY_MS + 10 * HOUR_MS; // 2024-03-04T10:00Z, off-schedule
        let data = json!({
            "dosageHistory": [
                {
                    "date": real_dose,
                    "medicationType": "injectableEstradiol",
                    "type": "Estradiol Valerate",
                    "dose": 4,
                    "unit": "mg"
                },
                {
                    "date": bonus_dose,
                    "medicationType": "injectableEstradiol",
                    "bonusDose": true,
                    "type": "Estradiol Valerate",
                    "dose": 4,
                    "unit": "mg"
                }
            ],
            "injectableEstradiol": {
                "type": "Estradiol Valerate",
                "dose": 4,
                "unit": "mg",
                "frequency": 7
            }
        });
        let options = IcsOptions {
            horizon_days: 30,
            include_past: false,
            now_ms: now,
        };

        let calendar = generate_ics(&data, &json!({}), options);

        // Weekly from the last real dose: 2024-03-02T10:00Z (already past, so
        // skipped) then 2024-03-09T10:00Z.
        let expected_first = today_start + 4 * DAY_MS + 10 * HOUR_MS;
        let starts = dtstart_millis(&calendar);
        assert_eq!(
            starts.first().copied(),
            Some(expected_first),
            "the bonus dose must not push the schedule a week later:\n{calendar}"
        );
    }

    /// Every occurrence keeps the last test's day of month, clamped to the
    /// month it lands in. Walking from the previous occurrence instead let
    /// February shorten the anchor once and for all (31 -> 29 -> 28).
    #[test]
    fn monthly_blood_test_keeps_the_31st_across_a_february() {
        let now: i64 = 1_795_212_000_000; // 2026-11-20T00:00:00Z, well past several Februaries
        let last_test: i64 = 1_706_691_600_000; // 2024-01-31T09:00:00Z
        let data = json!({ "bloodTests": [{ "date": last_test }] });
        let conf = json!({
            "enableBloodTestSchedule": true,
            "bloodTestIntervalMonths": 1
        });
        let options = IcsOptions {
            horizon_days: 400,
            include_past: false,
            now_ms: now,
        };
        let cal = generate_ics(&data, &conf, options);
        let days = dtstart_local_days(&cal);
        assert!(!days.is_empty(), "the feed must schedule blood tests");

        use chrono::Datelike;
        for day in &days {
            let expected = day.with_day(1).and_then(|first| {
                first
                    .checked_add_months(chrono::Months::new(1))
                    .and_then(|next| next.checked_sub_days(chrono::Days::new(1)))
            });
            assert_eq!(
                Some(day.day()),
                expected.map(|last| last.day()),
                "{day} must keep the last test's day of month (the 31st, clamped)"
            );
        }
        // November has 30 days and December has 31: the anchor must still be
        // the last test's 31st, not a drifted 28th.
        assert!(
            days.iter().any(|d| d.day() == 30) && days.iter().any(|d| d.day() == 31),
            "a 31st-anchored monthly walk must clamp to 30 in short months and return to 31: {days:?}"
        );
    }

    /// Progesterone is typically daily; its events stay out of the feed unless
    /// the setting opts in, while every other medication is unaffected.
    #[test]
    fn progesterone_events_are_off_by_default() {
        const DAY_MS: i64 = 86_400_000;
        let now: i64 = 1_709_596_800_000; // 2024-03-05T00:00:00Z
        let data = json!({
            "progesterone": {
                "type": "Micronized Progesterone",
                "route": "Oral",
                "dose": 100,
                "unit": "mg",
                "frequency": 1
            },
            "antiandrogen": {
                "type": "Cyproterone Acetate",
                "dose": 12.5,
                "unit": "mg",
                "frequency": 2
            },
            "dosageHistory": [
                { "date": now - DAY_MS, "medicationType": "progesterone", "type": "Micronized Progesterone", "route": "Oral", "dose": 100, "unit": "mg" },
                { "date": now - 2 * DAY_MS, "medicationType": "antiandrogen", "type": "Cyproterone Acetate", "dose": 12.5, "unit": "mg" }
            ]
        });
        let options = IcsOptions {
            horizon_days: 7,
            include_past: true,
            now_ms: now,
        };

        let without = generate_ics(&data, &json!({}), options);
        assert!(!without.contains("Progesterone"), "default feed must not carry progesterone:\n{without}");
        assert!(!without.contains("progesterone"), "no progesterone UIDs or descriptions either");
        assert!(without.contains("Antiandrogen"), "other medications stay in the feed");

        let with = generate_ics(&data, &json!({ "icsIncludeProgesterone": true }), options);
        assert!(with.contains("Progesterone"), "opting in must bring the scheduled doses back");
        assert!(
            with.matches("BEGIN:VEVENT").count() > without.matches("BEGIN:VEVENT").count(),
            "opting in adds events, not just a label"
        );
        assert!(with.contains("Antiandrogen"), "antiandrogen is unaffected either way");
    }
}
