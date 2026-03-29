//! Wasm-bindgen tests for leptos component rendering.
//!
//! Run with: wasm-pack test --headless --chrome crates/web

#![cfg(target_arch = "wasm32")]

use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

use leptos::*;

/// Helper: mount a leptos view into a fresh DOM section and return the container.
fn mount_test<F, V>(f: F) -> web_sys::HtmlElement
where
    F: FnOnce() -> V + 'static,
    V: IntoView,
{
    let document = web_sys::window().unwrap().document().unwrap();
    let container: web_sys::HtmlElement = document
        .create_element("section")
        .unwrap()
        .unchecked_into();
    document.body().unwrap().append_child(&container).unwrap();
    let _ = leptos::mount_to(container.clone(), f);
    container
}

/// Test that the App component renders the top-bar brand.
#[wasm_bindgen_test]
async fn app_renders_brand_title() {
    let container = mount_test(hrt_web::App);
    // Wait a tick for reactivity
    gloo_timers::future::TimeoutFuture::new(50).await;
    let html = container.inner_html();
    assert!(
        html.contains("HRT Tracker"),
        "Should render brand title, got: {}",
        &html[..html.len().min(500)]
    );
}

/// Test that the App component renders navigation links.
#[wasm_bindgen_test]
async fn app_renders_nav_links() {
    let container = mount_test(hrt_web::App);
    gloo_timers::future::TimeoutFuture::new(50).await;
    let html = container.inner_html();
    assert!(html.contains("View"), "Should have View nav link");
    assert!(html.contains("Stats"), "Should have Stats nav link");
    assert!(html.contains("Estrannaise"), "Should have Estrannaise nav link");
    assert!(html.contains("New Dose"), "Should have New Dose nav link");
    assert!(html.contains("Calculator"), "Should have Calculator nav link");
    assert!(html.contains("Vials"), "Should have Vials nav link");
}

/// Test that the App component renders the tagline.
#[wasm_bindgen_test]
async fn app_renders_tagline() {
    let container = mount_test(hrt_web::App);
    gloo_timers::future::TimeoutFuture::new(50).await;
    let html = container.inner_html();
    assert!(
        html.contains("Get Absolutely Estrogen"),
        "Should render tagline"
    );
}
