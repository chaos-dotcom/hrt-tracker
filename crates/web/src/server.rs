use axum::extract::OriginalUri;
use axum::routing::{get, post};
use axum::{
    Router,
    body::Body,
    http::{Request, Response, StatusCode},
};
use std::path::Path;
use tower_http::services::ServeDir;

pub fn serve() {
    let addr = std::env::var("HRT_WEB_ADDR").unwrap_or_else(|_| "127.0.0.1:4100".to_string());
    let backend_addr = std::env::var("HRT_SERVER_ADDR").unwrap_or_else(|_| "127.0.0.1:4200".to_string());

    let ocr_dir = if Path::new("target/site/ocr").exists() {
        "target/site/ocr"
    } else {
        "static/ocr"
    };
    let app = Router::new()
        .nest_service("/pkg", ServeDir::new("target/site/pkg"))
        .nest_service("/ocr", ServeDir::new(ocr_dir))
        .route("/", get(index_handler))
        .fallback(get(index_handler))
        .nest("/api", api_proxy_router(backend_addr));

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
    let candidates = ["target/site/index.html", "index.html"];
    for path in candidates {
        if let Ok(contents) = std::fs::read_to_string(path) {
            return contents;
        }
    }
    "Missing index.html".to_string()
}

fn api_proxy_router(backend_addr: String) -> Router {
    let backend_url = format!("http://{}", backend_addr);
    
    Router::new()
        .route("/health", get(proxy_handler))
        .route("/data", get(proxy_handler).post(proxy_handler))
        .route("/settings", get(proxy_handler).post(proxy_handler))
        .route("/convert", post(proxy_handler))
        .route("/ics", get(proxy_handler))
        .route("/ics/:secret", get(proxy_handler))
        .route("/dosage-photo/:entry_id", post(proxy_handler))
        .route("/dosage-photo/:entry_id/:filename", get(proxy_handler).delete(proxy_handler))
        .fallback(proxy_handler)
        .layer(axum::middleware::from_fn(move |req, next| {
            let backend_url = backend_url.clone();
            async move { proxy_middleware(req, next, backend_url).await }
        }))
}

async fn proxy_handler(req: Request<Body>) -> Result<Response<Body>, axum::response::ErrorResponse> {
    let backend_url = format!(
        "http://{}",
        std::env::var("HRT_SERVER_ADDR").unwrap_or_else(|_| "127.0.0.1:4200".to_string())
    );
    proxy_request(req, &backend_url).await
}

async fn proxy_middleware(
    req: Request<Body>,
    _next: axum::middleware::Next,
    backend_url: String,
) -> Result<Response<Body>, axum::response::ErrorResponse> {
    proxy_request(req, &backend_url).await
}

async fn proxy_request(
    req: Request<Body>,
    backend_url: &str,
) -> Result<Response<Body>, axum::response::ErrorResponse> {
    let original_uri = req
        .extensions()
        .get::<OriginalUri>()
        .map(|uri| uri.0.clone());
    let path_and_query = original_uri
        .as_ref()
        .unwrap_or_else(|| req.uri())
        .path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or("");
    let target_url = format!("{}{}", backend_url, path_and_query);

    let client = reqwest::Client::new();
    let mut request_builder = client.request(req.method().clone(), &target_url);

    for (name, value) in req.headers() {
        request_builder = request_builder.header(name, value);
    }

    let body_bytes = match axum::body::to_bytes(req.into_body(), usize::MAX).await {
        Ok(bytes) => bytes,
        Err(_) => {
            return Ok(proxy_error(
                StatusCode::BAD_REQUEST,
                "Invalid request body",
            ))
        }
    };
    request_builder = request_builder.body(body_bytes);

    let response = match request_builder.send().await {
        Ok(response) => response,
        Err(_) => return Ok(proxy_error(StatusCode::BAD_GATEWAY, "Bad gateway")),
    };

    let status = response.status();
    let mut builder = Response::builder().status(status);

    for (name, value) in response.headers() {
        builder = builder.header(name, value);
    }

    let response_body = match response.bytes().await {
        Ok(bytes) => bytes,
        Err(_) => return Ok(proxy_error(StatusCode::BAD_GATEWAY, "Bad gateway")),
    };

    let response = match builder.body(Body::from(response_body)) {
        Ok(response) => response,
        Err(_) => return Ok(proxy_error(StatusCode::INTERNAL_SERVER_ERROR, "Proxy error")),
    };

    Ok(response)
}

fn proxy_error(status: StatusCode, message: &str) -> Response<Body> {
    let mut response = Response::builder().status(status);
    response = response.header("Content-Type", "application/json");
    let safe_message = message.replace('"', "\\\"");
    let body = format!("{{\"error\":\"{}\"}}", safe_message);
    response
        .body(Body::from(body))
        .unwrap_or_else(|_| Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Body::from("{}")).unwrap())
}
