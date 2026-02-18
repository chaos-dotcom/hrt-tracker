use axum::http::{HeaderValue, Method};
use axum::routing::{get, post};
use axum::Router;
use hrt_server::{api, ics};
use tower_http::cors::{AllowOrigin, Any, CorsLayer};

#[tokio::main]
async fn main() {
    // Get allowed origins from environment variable or use defaults
    let origins_str = std::env::var("HRT_ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "http://127.0.0.1:4100,http://127.0.0.1:3000,http://localhost:4100,http://localhost:3000".to_string());

    let origins: Vec<HeaderValue> = origins_str
        .split(',')
        .filter_map(|s| s.trim().parse::<HeaderValue>().ok())
        .collect();

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([Method::GET, Method::POST, Method::DELETE])
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/api/data", get(api::get_data).post(api::post_data))
        .route(
            "/api/settings",
            get(api::get_settings).post(api::post_settings),
        )
        .route("/api/convert", post(api::convert))
        .route("/api/ics", get(ics::get_public_ics))
        .route("/api/ics/:secret", get(ics::get_secret_ics))
        .route(
            "/api/dosage-photo/:entry_id",
            post(api::upload_dosage_photo),
        )
        .route(
            "/api/dosage-photo/:entry_id/:filename",
            get(api::get_dosage_photo).delete(api::delete_dosage_photo),
        )
        .layer(cors);

    let addr = std::env::var("HRT_SERVER_ADDR").unwrap_or_else(|_| "127.0.0.1:4200".to_string());
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind");
    println!("Server listening on http://{addr}");
    axum::serve(listener, app).await.expect("server error");
}
