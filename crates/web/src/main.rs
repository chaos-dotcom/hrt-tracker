fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    hrt_web::serve();
}
