use axum::routing::get;
use axum::Router;
use tower_http::services::ServeDir;

pub fn serve() {
    let addr = std::env::var("HRT_WEB_ADDR").unwrap_or_else(|_| "127.0.0.1:4100".to_string());

    let app = Router::new()
        .nest_service("/pkg", ServeDir::new("target/site/pkg"))
        .route("/", get(index_handler))
        .fallback(get(index_handler));

    println!("Web UI listening on http://{addr}");
    let runtime = tokio::runtime::Runtime::new().expect("Failed to start runtime");
    runtime.block_on(async move {
        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .expect("Failed to bind web server");
        axum::serve(listener, app).await.expect("web server error");
    });
}

async fn index_handler() -> axum::response::Html<String> {
    axum::response::Html(read_index())
}

fn read_index() -> String {
    let candidates = ["target/site/index.html", "crates/web/index.html"];
    for path in candidates {
        if let Ok(contents) = std::fs::read_to_string(path) {
            return contents;
        }
    }
    "Missing index.html".to_string()
}
