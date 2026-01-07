use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde_json::json;
use std::env;
use tower_http::trace::TraceLayer;
use tracing::{info, info_span};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    let cell_id = env::var("CELL_ID").unwrap_or_else(|_| "unknown".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());

    info!(
        cell_id = %cell_id,
        port = %port,
        service = "cell-service",
        "cell start"
    );

    let app = Router::new()
        .route("/", get(root_handler))
        .route("/health", get(health_handler))
        .layer(middleware::from_fn(inject_cell_header))
        .layer(middleware::from_fn(request_tracing))
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();

    info!(
        cell_id = %cell_id,
        addr = %listener.local_addr().unwrap(),
        "cell ready"
    );

    axum::serve(listener, app).await.unwrap();
}

async fn request_tracing(mut req: Request, next: Next) -> Response {
    let request_id = Uuid::new_v4().to_string();
    let cell_id = env::var("CELL_ID").unwrap_or_else(|_| "unknown".to_string());

    let span = info_span!(
        "request",
        request_id = %request_id,
        cell_id = %cell_id,
        method = %req.method(),
        path = %req.uri().path(),
    );

    let _guard = span.enter();

    info!("request received");

    req.headers_mut()
        .insert("x-request-id", request_id.parse().unwrap());

    let response = next.run(req).await;

    info!(
        status = %response.status(),
        "request processed"
    );

    response
}

async fn inject_cell_header(req: Request, next: Next) -> Response {
    let cell_id = env::var("CELL_ID").unwrap_or_else(|_| "unknown".to_string());
    let mut response = next.run(req).await;

    response
        .headers_mut()
        .insert("X-TechGarden-Cell", cell_id.parse().unwrap());

    response
}

async fn root_handler(headers: HeaderMap) -> impl IntoResponse {
    let cell_id = env::var("CELL_ID").unwrap_or_else(|_| "unknown".to_string());
    let request_id = headers
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("no-request-id");

    info!(
        handler = "root",
        cell_id = %cell_id,
        request_id = %request_id,
        "processing... root request"
    );

    Json(json!({
        "message": "Tech Garden - Cellular Lab Environment Fabric",
        "cell_id": cell_id,
        "request_id": request_id,
        "phase": "1-foundation",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

async fn health_handler() -> impl IntoResponse {
    let cell_id = env::var("CELL_ID").unwrap_or_else(|_| "unknown".to_string());

    info!(
        handler = "health",
        cell_id = %cell_id,
        "health check, checked, me healthy"
    );

    (
        StatusCode::OK,
        Json(json!({
            "status": "healthy",
            "cell_id": cell_id
        })),
    )
}
