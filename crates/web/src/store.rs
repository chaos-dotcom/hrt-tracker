use gloo_net::http::Request;
use gloo_timers::callback::Timeout;
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
        spawn_local(async move {
            is_loading.set(true);
            last_error.set(None);
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

            let settings_resp = Request::get(&format!("{}/api/settings", api_base))
                .send()
                .await;
            match settings_resp {
                Ok(resp) => match resp.json::<Value>().await {
                    Ok(value) => {
                        let mut parsed = default_settings();
                        if let Ok(incoming) = serde_json::from_value::<Settings>(value) {
                            merge_settings(&mut parsed, incoming);
                        }
                        settings.set(parsed);
                    }
                    Err(err) => last_error.set(Some(format!("Failed to parse settings: {}", err))),
                },
                Err(err) => last_error.set(Some(format!("Failed to load settings: {}", err))),
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
        let data_value = self.data.get();
        let settings_value = self.settings.get();
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
        braSizeSystem: Some("uk".to_string()),
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
    if incoming.braSizeSystem.is_some() {
        base.braSizeSystem = incoming.braSizeSystem;
    }
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
    std::option_env!("HRT_API_BASE")
        .unwrap_or("/api")
        .to_string()
}
