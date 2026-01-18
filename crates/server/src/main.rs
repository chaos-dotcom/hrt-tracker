use axum::routing::{get, post};
use axum::Router;
use hrt_server::{api, ics};

#[tokio::main]
async fn main() {
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
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind");
    axum::serve(listener, app).await.expect("server error");
}
