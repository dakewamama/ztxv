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
use ztxv_core::{TransactionVerifier, VerificationResult, multi_rpc::MultiRpcVerifier, mempool::{MempoolListener, NewTxEvent}};
use tower_http::cors::CorsLayer;
use tokio::sync::broadcast;

#[derive(Clone)]
struct AppState {
    verifier: Arc<TransactionVerifier>,
    consensus_verifier: Arc<MultiRpcVerifier>,
    tx_broadcast: broadcast::Sender<NewTxEvent>,
}

#[derive(Deserialize)]
struct VerifyRequest {
    tx_hash: String,
}

#[derive(Deserialize)]
struct BatchVerifyRequest {
    tx_hashes: Vec<String>,
}

#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

#[derive(Serialize)]
struct BatchVerifyResponse {
    total: usize,
    successful: usize,
    failed: usize,
    results: Vec<BatchTxResult>,
    total_time_ms: u64,
}

#[derive(Serialize)]
struct BatchTxResult {
    tx_hash: String,
    success: bool,
    confirmations: Option<u32>,
    consensus_reached: Option<bool>,
    agreement_ratio: Option<f64>,
    error: Option<String>,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let rpc_url = std::env::var("ZCASH_RPC_URL")
        .unwrap_or_else(|_| std::env::var("GETBLOCK_API_KEY").unwrap());

    let verifier = Arc::new(TransactionVerifier::new(rpc_url.clone()));
    let consensus_verifier = Arc::new(MultiRpcVerifier::new());
    
    let (tx_broadcast, _) = broadcast::channel(1000);
    let (listener, mut rx) = MempoolListener::new(rpc_url);
    
    tokio::spawn(async move {
        listener.start().await;
    });
    
    let broadcast_clone = tx_broadcast.clone();
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            let _ = broadcast_clone.send(event);
        }
    });
    
    let state = AppState { 
        verifier,
        consensus_verifier,
        tx_broadcast,
    };

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/verify/:tx_hash", get(verify_tx))
        .route("/verify", post(verify_tx_post))
        .route("/verify-consensus/:tx_hash", get(verify_consensus))
        .route("/verify-batch", post(verify_batch))
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
    "ztxv API - Zcash Transaction Verification"
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

async fn verify_consensus(
    State(state): State<AppState>,
    Path(tx_hash): Path<String>,
) -> impl IntoResponse {
    match state.consensus_verifier.verify(&tx_hash).await {
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
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some(e.to_string()),
            }),
        ),
    }
}

async fn verify_batch(
    State(state): State<AppState>,
    Json(payload): Json<BatchVerifyRequest>,
) -> impl IntoResponse {
    let start = std::time::Instant::now();
    
    if payload.tx_hashes.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<BatchVerifyResponse> {
                success: false,
                data: None,
                error: Some("Empty batch".to_string()),
            }),
        );
    }
    
    if payload.tx_hashes.len() > 100 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<BatchVerifyResponse> {
                success: false,
                data: None,
                error: Some("Batch size exceeds 100".to_string()),
            }),
        );
    }
    
    let mut tasks = vec![];
    
    for tx_hash in payload.tx_hashes {
        let verifier = state.consensus_verifier.clone();
        let tx = tx_hash.clone();
        
        tasks.push(tokio::spawn(async move {
            match verifier.verify(&tx).await {
                Ok(result) => BatchTxResult {
                    tx_hash: tx,
                    success: true,
                    confirmations: Some(result.confirmations),
                    consensus_reached: Some(result.consensus_reached),
                    agreement_ratio: Some(result.agreement_ratio),
                    error: None,
                },
                Err(e) => BatchTxResult {
                    tx_hash: tx,
                    success: false,
                    confirmations: None,
                    consensus_reached: None,
                    agreement_ratio: None,
                    error: Some(e.to_string()),
                },
            }
        }));
    }
    
    let mut results = vec![];
    for task in tasks {
        if let Ok(result) = task.await {
            results.push(result);
        }
    }
    
    let successful = results.iter().filter(|r| r.success).count();
    let failed = results.len() - successful;
    
    let response = BatchVerifyResponse {
        total: results.len(),
        successful,
        failed,
        results,
        total_time_ms: start.elapsed().as_millis() as u64,
    };
    
    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            data: Some(response),
            error: None,
        }),
    )
}