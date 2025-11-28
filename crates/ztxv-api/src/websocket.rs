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
use ztxv_core::VerificationResult;

use crate::AppState;

#[derive(Serialize)]
struct WsMessage {
    #[serde(rename = "type")]
    msg_type: String,
    data: Option<VerificationResult>,
    error: Option<String>,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    while let Some(msg) = receiver.next().await {
        if let Ok(Message::Text(text)) = msg {
            let tx_hash = text.trim().to_string();
            
            let mut tick = interval(Duration::from_millis(500));
            
            for _ in 0..20 {
                tick.tick().await;
                
                match state.verifier.verify(&tx_hash).await {
                    Ok(result) => {
                        let confidence = result.confidence;
                        let msg = WsMessage {
                            msg_type: "update".to_string(),
                            data: Some(result),
                            error: None,
                        };
                        
                        if sender
                            .send(Message::Text(serde_json::to_string(&msg).unwrap()))
                            .await
                            .is_err()
                        {
                            break;
                        }
                        
                        if confidence >= 99.0 {
                            break;
                        }
                    }
                    Err(e) => {
                        let msg = WsMessage {
                            msg_type: "error".to_string(),
                            data: None,
                            error: Some(e.to_string()),
                        };
                        let _ = sender
                            .send(Message::Text(serde_json::to_string(&msg).unwrap()))
                            .await;
                        break;
                    }
                }
            }
        }
    }
}