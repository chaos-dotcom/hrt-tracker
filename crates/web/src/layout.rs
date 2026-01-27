use chrono::{Local, TimeZone};
use leptos::*;

use crate::store::use_store;

pub fn page_layout(title: &'static str, body: View) -> impl IntoView {
    let store = use_store();
    let is_loading = store.is_loading;
    let is_saving = store.is_saving;
    let is_dirty = store.is_dirty;
    let last_saved = store.last_saved;
    let error = store.last_error;

    let status_text = move || {
        if is_saving.get() {
            "Saving...".to_string()
        } else if let Some(last) = last_saved.get() {
            let stamp = Local
                .timestamp_millis_opt(last)
                .single()
                .map(|d| d.format("%H:%M").to_string())
                .unwrap_or_else(|| "".to_string());
            if stamp.is_empty() {
                "Saved".to_string()
            } else {
                format!("Saved {stamp}")
            }
        } else if is_dirty.get() {
            "Unsaved changes".to_string()
        } else {
            "".to_string()
        }
    };

    view! {
        <section>
            <header>
                <div class="page-title">
                    <h1>{title}</h1>
                    <Show when=move || !status_text().is_empty()>
                        <span class="status-chip">{status_text}</span>
                    </Show>
                </div>
            </header>
            <Show when=move || is_loading.get()>
                <p>"Loading data..."</p>
            </Show>
            <Show when=move || error.get().is_some()>
                <p class="error">{move || error.get().unwrap_or_default()}</p>
            </Show>
            {body}
        </section>
    }
}
