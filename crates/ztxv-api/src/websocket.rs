use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use tokio::time::{interval, Duration};
use tokio::sync::mpsc;
use ztxv_core::VerificationResult;

use crate::AppState;

#[derive(Serialize)]
struct WsMessage {
    #[serde(rename = "type")]
    msg_type: String,
    data: Option<WsData>,
    error: Option<String>,
}

#[derive(Serialize)]
#[serde(untagged)]
enum WsData {
    Verification(VerificationResult),
    Consensus(ConsensusData),
    Mempool(MempoolData),
}

#[derive(Serialize)]
struct ConsensusData {
    tx_hash: String,
    confirmations: u32,
    confidence: f64,
    consensus_reached: bool,
    agreement_ratio: f64,
    node_responses: Vec<NodeInfo>,
}

#[derive(Serialize)]
struct NodeInfo {
    name: String,
    confirmations: u32,
    response_time_ms: u64,
}

#[derive(Serialize)]
struct MempoolData {
    tx_hash: String,
    detected_at: u64,
    initial_confidence: f64,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<WsMessage>();
    
    let mut mempool_rx = state.tx_broadcast.subscribe();
    let tx_clone = tx.clone();

    let mempool_task = tokio::spawn(async move {
        loop {
            match mempool_rx.recv().await {
                Ok(event) => {
                    let msg = WsMessage {
                        msg_type: "mempool".to_string(),
                        data: Some(WsData::Mempool(MempoolData {
                            tx_hash: event.tx_hash,
                            detected_at: event.detected_at,
                            initial_confidence: event.initial_confidence,
                        })),
                        error: None,
                    };
                    if tx_clone.send(msg).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    let state_clone = state.clone();
    let receive_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            if let Ok(Message::Text(text)) = msg {
                let tx_hash = text.trim().to_string();
                let tx = tx.clone();
                let verifier = state_clone.consensus_verifier.clone();
                
                tokio::spawn(async move {
                    let mut tick = interval(Duration::from_secs(3));
                    
                    loop {
                        tick.tick().await;
                        
                        match verifier.verify(&tx_hash).await {
                            Ok(result) => {
                                let confidence = calculate_confidence(result.confirmations);
                                
                                let msg = WsMessage {
                                    msg_type: "consensus".to_string(),
                                    data: Some(WsData::Consensus(ConsensusData {
                                        tx_hash: tx_hash.clone(),
                                        confirmations: result.confirmations,
                                        confidence,
                                        consensus_reached: result.consensus_reached,
                                        agreement_ratio: result.agreement_ratio,
                                        node_responses: result.responses.iter().map(|r| NodeInfo {
                                            name: r.node_name.clone(),
                                            confirmations: r.confirmations,
                                            response_time_ms: r.response_time_ms,
                                        }).collect(),
                                    })),
                                    error: None,
                                };
                                
                                if tx.send(msg).is_err() {
                                    break;
                                }
                                
                                if result.confirmations >= 24 {
                                    break;
                                }
                            }
                            Err(e) => {
                                let msg = WsMessage {
                                    msg_type: "error".to_string(),
                                    data: None,
                                    error: Some(e.to_string()),
                                };
                                let _ = tx.send(msg);
                                break;
                            }
                        }
                    }
                });
            }
        }
    });

    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender
                .send(Message::Text(serde_json::to_string(&msg).unwrap()))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = receive_task => {},
        _ = mempool_task => {},
    }
}

fn calculate_confidence(confirmations: u32) -> f64 {
    match confirmations {
        0 => 0.2,
        1 => 0.75,
        2 => 0.85,
        3..=9 => 0.95,
        10..=23 => 0.98,
        _ => 0.999,
    }
}