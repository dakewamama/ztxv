mod websocket;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use ztxv_core::{TransactionVerifier, VerificationResult};
use tower_http::cors::CorsLayer;


#[derive(Clone)]
struct AppState {
    verifier: Arc<TransactionVerifier>,
}

#[derive(Deserialize)]
struct VerifyRequest {
    tx_hash: String,
}

#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let rpc_url = std::env::var("ZCASH_RPC_URL")
        .unwrap_or_else(|_| "http://localhost:8232".to_string());

    let verifier = Arc::new(TransactionVerifier::new(rpc_url));
    let state = AppState { verifier };

    let app = Router::new()
    .route("/", get(root))
    .route("/health", get(health))
    .route("/verify/:tx_hash", get(verify_tx))
    .route("/verify", post(verify_tx_post))
    .route("/ws", get(websocket::ws_handler))
    .layer(CorsLayer::permissive())
    .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    tracing::info!("Server running on http://0.0.0.0:3000");

    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "ztxv API >>> Zcash Transaction verification from stauroX"
}

async fn health() -> impl IntoResponse {
    Json(ApiResponse {
        success: true,
        data: Some("Okay"),
        error: None,
    })
}

async fn verify_tx(
    State(state): State<AppState>,
    Path(tx_hash): Path<String>,
) -> impl IntoResponse {
    match state.verifier.verify(&tx_hash).await {
        Ok(result) => (
            StatusCode::OK,
            Json(ApiResponse {
                success: true,
                data: Some(result),
                error: None,
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<VerificationResult> {
                success: false,
                data: None,
                error: Some(e.to_string()),
            }),
        ),
    }
}

async fn verify_tx_post(
    State(state): State<AppState>,
    Json(payload): Json<VerifyRequest>,
) -> impl IntoResponse {
    match state.verifier.verify(&payload.tx_hash).await {
        Ok(result) => (
            StatusCode::OK,
            Json(ApiResponse {
                success: true,
                data: Some(result),
                error: None,
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<VerificationResult> {
                success: false,
                data: None,
                error: Some(e.to_string()),
            }),
        ),
    }
}