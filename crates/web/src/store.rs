use gloo_net::http::Request;
use gloo_timers::callback::Timeout;
use gloo_timers::future::TimeoutFuture;
use hrt_shared::logic::{backfill_scheduled_doses, migrate_blood_tests_fudge_factor};
use hrt_shared::types::{HormoneUnits, HrtData, Settings};
use leptos::*;
use serde_json::Value;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct AppStore {
    pub data: RwSignal<HrtData>,
    pub settings: RwSignal<Settings>,
    pub is_loading: RwSignal<bool>,
    pub is_saving: RwSignal<bool>,
    pub is_dirty: RwSignal<bool>,
    pub last_saved: RwSignal<Option<i64>>,
    pub last_error: RwSignal<Option<String>>,
    autosave_handle: Rc<RefCell<Option<Timeout>>>,
}

impl AppStore {
    pub fn new() -> Self {
        Self {
            data: create_rw_signal(HrtData::default()),
            settings: create_rw_signal(default_settings()),
            is_loading: create_rw_signal(false),
            is_saving: create_rw_signal(false),
            is_dirty: create_rw_signal(false),
            last_saved: create_rw_signal(None),
            last_error: create_rw_signal(None),
            autosave_handle: Rc::new(RefCell::new(None)),
        }
    }

    pub fn load(&self) {
        let data = self.data;
        let settings = self.settings;
        let is_loading = self.is_loading;
        let is_dirty = self.is_dirty;
        let last_error = self.last_error;
        let api_base = api_base();
        is_loading.set(true);
        last_error.set(None);
        spawn_local(async move {
            let resp = Request::get(&format!("{}/api/data", api_base)).send().await;
            match resp {
                Ok(resp) => match resp.json::<HrtData>().await {
                    Ok(mut loaded) => {
                        migrate_blood_tests_fudge_factor(&mut loaded);
                        backfill_scheduled_doses(&mut loaded);
                        data.set(loaded);
                        is_dirty.set(false);
                    }
                    Err(err) => last_error.set(Some(format!("Failed to parse data: {}", err))),
                },
                Err(err) => last_error.set(Some(format!("Failed to load data: {}", err))),
            }

            let mut last_settings_error: Option<String> = None;
            for attempt in 0..3 {
                match fetch_settings(&api_base).await {
                    Ok(parsed) => {
                        settings.set(parsed);
                        last_settings_error = None;
                        break;
                    }
                    Err(err) => {
                        last_settings_error = Some(err);
                        if attempt < 2 {
                            TimeoutFuture::new(250 * (attempt + 1) as u32).await;
                        }
                    }
                }
            }
            if let Some(err) = last_settings_error {
                last_error.set(Some(err));
            }
            is_loading.set(false);
        });
    }

    pub fn mark_dirty(&self) {
        self.is_dirty.set(true);
        self.schedule_autosave();
    }

    fn schedule_autosave(&self) {
        let mut handle = self.autosave_handle.borrow_mut();
        if let Some(existing) = handle.take() {
            existing.cancel();
        }
        let store = self.clone();
        *handle = Some(Timeout::new(900, move || {
            if store.is_dirty.get() && !store.is_saving.get() {
                store.save();
            }
        }));
    }

    pub fn save(&self) {
        let settings_value = self.settings.get();
        if settings_value.enableAutoBackfill {
            self.data.update(|data| backfill_scheduled_doses(data));
        }
        let data_value = self.data.get();
        let is_saving = self.is_saving;
        let is_dirty = self.is_dirty;
        let last_saved = self.last_saved;
        let last_error = self.last_error;
        let api_base = api_base();
        spawn_local(async move {
            is_saving.set(true);
            last_error.set(None);
            let payload = serde_json::to_string(&data_value).unwrap_or_else(|_| "{}".to_string());
            let resp = Request::post(&format!("{}/api/data", api_base))
                .header("Content-Type", "application/json")
                .body(payload);
            let mut failed = false;
            match resp {
                Ok(request) => {
                    if let Err(err) = request.send().await {
                        last_error.set(Some(format!("Failed to save data: {}", err)));
                        failed = true;
                    }
                }
                Err(err) => {
                    last_error.set(Some(format!("Failed to save data: {}", err)));
                    failed = true;
                }
            }

            let settings_payload =
                serde_json::to_string(&settings_value).unwrap_or_else(|_| "{}".to_string());
            let settings_resp = Request::post(&format!("{}/api/settings", api_base))
                .header("Content-Type", "application/json")
                .body(settings_payload);
            match settings_resp {
                Ok(request) => {
                    if let Err(err) = request.send().await {
                        last_error.set(Some(format!("Failed to save settings: {}", err)));
                        failed = true;
                    }
                }
                Err(err) => {
                    last_error.set(Some(format!("Failed to save settings: {}", err)));
                    failed = true;
                }
            }

            if !failed {
                is_dirty.set(false);
                last_saved.set(Some(js_sys::Date::now() as i64));
            }
            is_saving.set(false);
        });
    }
}

fn default_settings() -> Settings {
    Settings {
        enableAutoBackfill: true,
        icsSecret: None,
        enableBloodTestSchedule: Some(false),
        bloodTestIntervalMonths: Some(3.0),
        statsBreakdownBySyringeKind: Some(false),
        displayEstradiolUnit: Some(HormoneUnits::E2PmolL),
        displayInjectableInIU: Some(false),
        braSizeSystem: Some("uk".to_string()),
        pdfPassword: None,
    }
}

fn merge_settings(base: &mut Settings, incoming: Settings) {
    base.enableAutoBackfill = incoming.enableAutoBackfill;
    if incoming.icsSecret.is_some() {
        base.icsSecret = incoming.icsSecret;
    }
    if incoming.enableBloodTestSchedule.is_some() {
        base.enableBloodTestSchedule = incoming.enableBloodTestSchedule;
    }
    if incoming.bloodTestIntervalMonths.is_some() {
        base.bloodTestIntervalMonths = incoming.bloodTestIntervalMonths;
    }
    if incoming.statsBreakdownBySyringeKind.is_some() {
        base.statsBreakdownBySyringeKind = incoming.statsBreakdownBySyringeKind;
    }
    if incoming.displayEstradiolUnit.is_some() {
        base.displayEstradiolUnit = incoming.displayEstradiolUnit;
    }
    if incoming.displayInjectableInIU.is_some() {
        base.displayInjectableInIU = incoming.displayInjectableInIU;
    }
    if incoming.braSizeSystem.is_some() {
        base.braSizeSystem = incoming.braSizeSystem;
    }
    if incoming.pdfPassword.is_some() {
        base.pdfPassword = incoming.pdfPassword;
    }
}

async fn fetch_settings(api_base: &str) -> Result<Settings, String> {
    let resp = Request::get(&format!("{}/api/settings", api_base))
        .send()
        .await
        .map_err(|err| format!("Failed to load settings: {}", err))?;

    if !resp.ok() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        let detail = if text.trim().is_empty() {
            status.to_string()
        } else {
            format!("{}: {}", status, text.trim())
        };
        return Err(format!("Failed to load settings: {}", detail));
    }

    let text = resp
        .text()
        .await
        .map_err(|err| format!("Failed to read settings: {}", err))?;

    if text.trim().is_empty() {
        return Ok(default_settings());
    }

    let value: Value =
        serde_json::from_str(&text).map_err(|err| format!("Failed to parse settings: {}", err))?;
    let mut parsed = default_settings();
    if let Ok(incoming) = serde_json::from_value::<Settings>(value) {
        merge_settings(&mut parsed, incoming);
    }
    Ok(parsed)
}

#[component]
pub fn StoreProvider(children: Children) -> impl IntoView {
    let store = AppStore::new();
    provide_context(store.clone());
    store.load();

    view! { <>{children()}</> }
}

pub fn use_store() -> AppStore {
    use_context::<AppStore>().expect("AppStore missing in context")
}

pub fn api_base() -> String {
    // Use relative paths for single-port proxy setup, fallback to absolute URL for development
    let base = std::option_env!("HRT_API_BASE")
        .unwrap_or("")
        .trim()
        .to_string();
    if base.is_empty() {
        return String::new();
    }
    let trimmed = base.trim_end_matches('/');
    if trimmed.ends_with("/api") {
        trimmed.trim_end_matches("/api").to_string()
    } else {
        trimmed.to_string()
    }
}
