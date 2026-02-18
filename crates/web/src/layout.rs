use chrono::{Local, TimeZone};
use gloo_timers::callback::Interval;
use js_sys::Date;
use leptos::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::store::use_store;

pub fn page_layout(title: &'static str, body: View) -> impl IntoView {
    let store = use_store();
    let is_loading = store.is_loading;
    let is_saving = store.is_saving;
    let is_dirty = store.is_dirty;
    let last_saved = store.last_saved;
    let error = store.last_error;
    let stopwatch_open = create_rw_signal(false);
    let stopwatch_start = create_rw_signal(None::<f64>);
    let stopwatch_accum = create_rw_signal(0.0);
    let stopwatch_tick = create_rw_signal(Date::now() as i64);
    let interval_handle: Rc<RefCell<Option<Interval>>> = Rc::new(RefCell::new(None));

    let is_running = move || stopwatch_start.get().is_some();
    let elapsed_ms = move || {
        let _tick = stopwatch_tick.get();
        let base = stopwatch_accum.get();
        if let Some(start) = stopwatch_start.get() {
            base + (Date::now() - start)
        } else {
            base
        }
    };

    let format_stopwatch = |ms: f64| {
        let total = (ms / 1000.0).floor().max(0.0) as i64;
        let hours = total / 3600;
        let minutes = (total % 3600) / 60;
        let seconds = total % 60;
        format!("{hours:02}:{minutes:02}:{seconds:02}")
    };

    let toggle_running = {
        let stopwatch_start = stopwatch_start;
        let stopwatch_accum = stopwatch_accum;
        let stopwatch_tick = stopwatch_tick;
        move |_| {
            if let Some(start) = stopwatch_start.get() {
                let now = Date::now();
                stopwatch_accum.set(stopwatch_accum.get() + (now - start));
                stopwatch_start.set(None);
            } else {
                stopwatch_start.set(Some(Date::now()));
                stopwatch_tick.set(Date::now() as i64);
            }
        }
    };

    let reset_stopwatch = {
        let stopwatch_start = stopwatch_start;
        let stopwatch_accum = stopwatch_accum;
        let stopwatch_tick = stopwatch_tick;
        move |_| {
            stopwatch_start.set(None);
            stopwatch_accum.set(0.0);
            stopwatch_tick.set(Date::now() as i64);
        }
    };

    create_effect({
        let interval_handle = interval_handle.clone();
        let stopwatch_start = stopwatch_start;
        let stopwatch_tick = stopwatch_tick;
        move |_| {
            if stopwatch_start.get().is_some() {
                if interval_handle.borrow().is_none() {
                    let tick = stopwatch_tick;
                    let handle = Interval::new(250, move || {
                        tick.set(Date::now() as i64);
                    });
                    interval_handle.replace(Some(handle));
                }
            } else {
                interval_handle.borrow_mut().take();
            }
        }
    });

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
                <div class="page-header-actions">
                    <button
                        type="button"
                        class="ghost-button"
                        on:click=move |_| stopwatch_open.set(!stopwatch_open.get())
                    >
                        {move || {
                            if is_running() {
                                format!("Stopwatch · {}", format_stopwatch(elapsed_ms()))
                            } else {
                                "Stopwatch".to_string()
                            }
                        }}
                    </button>
                </div>
            </header>
            <div class="stopwatch-panel" class:open=move || stopwatch_open.get()>
                <div class="stopwatch-card">
                    <div class="stopwatch-time">{move || format_stopwatch(elapsed_ms())}</div>
                    <div class="stopwatch-controls">
                        <button type="button" on:click=toggle_running>
                            {move || if is_running() { "Stop" } else { "Start" }}
                        </button>
                        <button type="button" class="ghost-button" on:click=reset_stopwatch>
                            "Reset"
                        </button>
                    </div>
                </div>
            </div>
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
