use serde_json::json;

use hrt_server::ics::{generate_ics, IcsOptions};

#[test]
fn parity_includes_history_and_schedule() {
    let data = json!({
        "dosageHistory": [
            {
                "date": 1700000000000_i64,
                "medicationType": "injectableEstradiol",
                "type": "Estradiol Valerate",
                "dose": 5,
                "unit": "mg"
            }
        ],
        "injectableEstradiol": {
            "type": "Estradiol Valerate",
            "dose": 5,
            "unit": "mg",
            "frequency": 7,
            "nextDoseDate": 1700000000000_i64
        }
    });
    let conf = json!({
        "enableBloodTestSchedule": false
    });
    let options = IcsOptions {
        horizon_days: 30,
        include_past: true,
        now_ms: 1700000000000_i64,
    };

    let calendar = generate_ics(&data, &conf, options);

    assert!(calendar.contains("BEGIN:VCALENDAR"));
    assert!(calendar.contains("SUMMARY:Injection: Estradiol Valerate 5 mg"));
    assert!(calendar.contains("Scheduled Injection"));
    assert!(calendar.contains("END:VCALENDAR"));
}

#[test]
fn parity_omits_history_when_disabled() {
    let data = json!({
        "dosageHistory": [
            {
                "date": 1700000000000_i64,
                "medicationType": "oralEstradiol",
                "type": "Estradiol Hemihydrate",
                "dose": 2,
                "unit": "mg"
            }
        ]
    });
    let conf = json!({});
    let options = IcsOptions {
        horizon_days: 30,
        include_past: false,
        now_ms: 1700000000000_i64,
    };

    let calendar = generate_ics(&data, &conf, options);

    assert!(!calendar.contains("Recorded dose"));
}
