use chrono::Local;
use leptos::window;
use leptos::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use crate::layout::page_layout;
use crate::store;
use crate::store::use_store;
use crate::utils::parse_decimal;
use hrt_shared::types::{HormoneUnits, Settings};

#[component]
pub fn BackupPage() -> impl IntoView {
    let store = use_store();
    let settings = store.settings;

    let ics_secret = create_rw_signal(settings.get().icsSecret.unwrap_or_default());
    let blood_test_interval_months = create_rw_signal(
        settings
            .get()
            .bloodTestIntervalMonths
            .map(|v| v.to_string())
            .unwrap_or_default(),
    );
    let ics_url = create_memo({
        let ics_secret = ics_secret;
        move |_| {
            let base = store::api_base();
            let secret = ics_secret.get();
            if secret.trim().is_empty() {
                format!("{}/api/ics?horizonDays=365&includePast=1", base)
            } else {
                format!(
                    "{}/api/ics/{}?horizonDays=365&includePast=1",
                    base,
                    urlencoding::encode(secret.trim())
                )
            }
        }
    });

    let on_copy_ics = move |_| {
        let url = ics_url.get();
        let _ = window().navigator().clipboard().write_text(&url);
    };

    let on_save_settings = {
        let store = store.clone();
        let ics_secret = ics_secret;
        let blood_test_interval_months = blood_test_interval_months;
        move |_: leptos::ev::MouseEvent| {
            let secret = ics_secret.get();
            let interval = parse_decimal(&blood_test_interval_months.get());
            store.settings.update(|s| {
                s.icsSecret = if secret.trim().is_empty() {
                    None
                } else {
                    Some(secret)
                };
                s.bloodTestIntervalMonths = interval;
            });
            store.mark_dirty();
        }
    };

    let on_restore = {
        let store = store.clone();
        move |ev: leptos::ev::Event| {
            let input: web_sys::HtmlInputElement = event_target(&ev);
            let Some(files) = input.files() else {
                return;
            };
            let Some(file) = files.get(0) else {
                return;
            };
            let reader = match web_sys::FileReader::new() {
                Ok(reader) => reader,
                Err(_) => return,
            };
            let reader_clone = reader.clone();
            let store = store.clone();
            let onload = Closure::wrap(Box::new(move |_ev: web_sys::ProgressEvent| {
                let Ok(result) = reader_clone.result() else {
                    return;
                };
                let Some(text) = result.as_string() else {
                    return;
                };
                if let Ok(payload) = serde_json::from_str::<serde_json::Value>(&text) {
                    let data_value = if payload.get("data").is_some() {
                        payload.get("data").cloned().unwrap_or(payload.clone())
                    } else {
                        payload.clone()
                    };
                    if let Ok(parsed) =
                        serde_json::from_value::<hrt_shared::types::HrtData>(data_value)
                    {
                        store.data.set(parsed);
                        store.mark_dirty();
                    }
                    if let Some(settings_value) = payload.get("settings") {
                        if let Ok(parsed) =
                            serde_json::from_value::<Settings>(settings_value.clone())
                        {
                            store.settings.set(parsed);
                            store.mark_dirty();
                        }
                    }
                    if let Some(document) = window().document() {
                        if let Some(element) = document.get_element_by_id("restore-file") {
                            if let Ok(input) = element.dyn_into::<web_sys::HtmlInputElement>() {
                                input.set_value("");
                            }
                        }
                    }
                }
            }) as Box<dyn FnMut(_)>);
            reader.set_onload(Some(onload.as_ref().unchecked_ref()));
            onload.forget();
            let _ = reader.read_as_text(&file);
        }
    };

    let on_export = {
        let store = store.clone();
        move |_| {
            let payload = serde_json::json!({
                "data": store.data.get(),
                "settings": store.settings.get(),
            });
            if let Ok(json) = serde_json::to_string_pretty(&payload) {
                let parts = js_sys::Array::new();
                parts.push(&json.into());
                if let Ok(blob) = web_sys::Blob::new_with_str_sequence(&parts) {
                    if let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) {
                        if let Some(document) = window().document() {
                            if let Ok(link) = document.create_element("a") {
                                let link: web_sys::HtmlAnchorElement = link.unchecked_into();
                                link.set_href(&url);
                                link.set_download(&format!(
                                    "hrt-data-backup-{}.json",
                                    Local::now().format("%Y-%m-%d")
                                ));
                                let _ = link.click();
                                let _ = web_sys::Url::revoke_object_url(&url);
                            }
                        }
                    }
                }
            }
        }
    };

    let auto_backfill = move || settings.get().enableAutoBackfill;

    page_layout(
        "Settings & Backup",
        view! {
            <div class="view-layout">
                <div class="view-header">
                    <div>
                        <h2>"Settings & Backup"</h2>
                        <p class="muted">"Manage settings, calendar feeds, and backup/restore."</p>
                    </div>
                </div>

                <div class="settings-stack">
                    <div class="card">
                        <h3>"Settings"</h3>
                        <div class="settings-toggles">
                            <label class="toggle toggle-wide">
                                <input
                                    type="checkbox"
                                    on:change={
                                        let store = store.clone();
                                        move |ev| {
                                            let enabled = event_target_checked(&ev);
                                            store.settings.update(|s| s.enableAutoBackfill = enabled);
                                            store.mark_dirty();
                                        }
                                    }
                                    prop:checked=move || auto_backfill()
                                />
                                <span class="toggle-track" aria-hidden="true"></span>
                                <span class="toggle-label">"Enable auto backfill"</span>
                            </label>
                            <label class="toggle toggle-wide">
                                <input
                                    type="checkbox"
                                    on:change={
                                        let store = store.clone();
                                        move |ev| {
                                            let enabled = event_target_checked(&ev);
                                            store.settings.update(|s| s.enableBloodTestSchedule = Some(enabled));
                                            store.mark_dirty();
                                        }
                                    }
                                    prop:checked=move || store.settings.get().enableBloodTestSchedule.unwrap_or(false)
                                />
                                <span class="toggle-track" aria-hidden="true"></span>
                                <span class="toggle-label">"Enable blood test schedule"</span>
                            </label>
                            <label class="toggle toggle-wide">
                                <input
                                    type="checkbox"
                                    on:change={
                                        let store = store.clone();
                                        move |ev| {
                                            let enabled = event_target_checked(&ev);
                                            store.settings.update(|s| s.displayInjectableInIU = Some(enabled));
                                            store.mark_dirty();
                                        }
                                    }
                                    prop:checked=move || store.settings.get().displayInjectableInIU.unwrap_or(false)
                                />
                                <span class="toggle-track" aria-hidden="true"></span>
                                <span class="toggle-label">"Show injectable doses in IU"</span>
                            </label>
                        </div>
                        <label>"Blood test interval (months)"</label>
                        <input
                            type="text"
                            step="any"
                            min="1"
                            on:input=move |ev| blood_test_interval_months.set(event_target_value(&ev))
                            on:blur={
                                let store = store.clone();
                                let blood_test_interval_months = blood_test_interval_months;
                                move |_| {
                                    let parsed = parse_decimal(&blood_test_interval_months.get());
                                    store.settings.update(|s| s.bloodTestIntervalMonths = parsed);
                                    store.mark_dirty();
                                }
                            }
                            prop:value=move || blood_test_interval_months.get()
                        />
                        <label>"ICS URL secret (optional)"</label>
                        <input
                            type="text"
                            placeholder="my-private-feed"
                            on:input=move |ev| ics_secret.set(event_target_value(&ev))
                            prop:value=move || ics_secret.get()
                        />
                        <label>"Estradiol display unit"</label>
                        <select
                            on:change={
                                let store = store.clone();
                                move |ev| {
                                    let value = event_target_value(&ev);
                                    let unit = match value.as_str() {
                                        "pmol/L" => Some(HormoneUnits::E2PmolL),
                                        "pg/mL" => Some(HormoneUnits::E2PgMl),
                                        _ => None,
                                    };
                                    store.settings.update(|s| s.displayEstradiolUnit = unit);
                                    store.mark_dirty();
                                }
                            }
                        >
                            <option value="pmol/L" selected=move || settings.get().displayEstradiolUnit == Some(HormoneUnits::E2PmolL)>
                                "pmol/L"
                            </option>
                            <option value="pg/mL" selected=move || settings.get().displayEstradiolUnit == Some(HormoneUnits::E2PgMl)>
                                "pg/mL"
                            </option>
                        </select>
                        <label>"Bra size system"</label>
                        <select
                            on:change={
                                let store = store.clone();
                                move |ev| {
                                    let value = event_target_value(&ev);
                                    store.settings.update(|s| s.braSizeSystem = Some(value));
                                    store.mark_dirty();
                                }
                            }
                        >
                            <option value="us" selected=move || settings.get().braSizeSystem.as_deref() == Some("us")>
                                "US or CA"
                            </option>
                            <option value="uk" selected=move || settings.get().braSizeSystem.as_deref() == Some("uk")>
                                "UK"
                            </option>
                            <option value="eu" selected=move || settings.get().braSizeSystem.as_deref() == Some("eu")>
                                "EU under EN 13402"
                            </option>
                            <option value="fr" selected=move || settings.get().braSizeSystem.as_deref() == Some("fr")>
                                "FR, BE, or ES"
                            </option>
                            <option value="au" selected=move || settings.get().braSizeSystem.as_deref() == Some("au")>
                                "Australia or New Zealand"
                            </option>
                            <option value="us-plus4" selected=move || settings.get().braSizeSystem.as_deref() == Some("us-plus4")>
                                "US or CA with underbust +4"
                            </option>
                            <option value="uk-plus4" selected=move || settings.get().braSizeSystem.as_deref() == Some("uk-plus4")>
                                "UK with underbust +4"
                            </option>
                            <option value="uk-dress" selected=move || settings.get().braSizeSystem.as_deref() == Some("uk-dress")>
                                "UK using dress code"
                            </option>
                        </select>
                        <div class="primary-actions">
                            <button type="button" on:click=on_save_settings>
                                "Save settings"
                            </button>
                        </div>
                    </div>

                    <div class="card">
                        <h3>"ICS Calendar"</h3>
                        <p class="muted">"Subscribe in your calendar app using this URL."</p>
                        <input type="text" readonly prop:value=move || ics_url.get() />
                        <div class="primary-actions">
                            <a href=move || ics_url.get() target="_blank" rel="noopener noreferrer">"Open"</a>
                            <button type="button" on:click=on_copy_ics>"Copy"</button>
                        </div>
                    </div>

                    <div class="card">
                        <h3>"Backup"</h3>
                        <p class="muted">"Export your full data + settings bundle for safekeeping."</p>
                        <div class="primary-actions">
                            <button type="button" on:click=on_export>"Export to JSON"</button>
                        </div>
                    </div>

                    <div class="card">
                        <h3>"Restore"</h3>
                        <p class="muted">"Restore from a JSON backup file (overwrites current data)."</p>
                        <input
                            id="restore-file"
                            class="file-input"
                            type="file"
                            accept=".json"
                            on:change=on_restore
                        />
                    </div>
                </div>
            </div>
        }
        .into_view(),
    )
}
