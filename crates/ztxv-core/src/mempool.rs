use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tokio::time::sleep;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MempoolError {
    #[error("RPC error: {0}")]
    RpcError(String),
    
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MempoolTransaction {
    pub tx_hash: String,
    pub size: u64,
    pub fee: f64,
    pub time: u64,
    pub height: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTxEvent {
    pub tx_hash: String,
    pub detected_at: u64,
    pub initial_confidence: f64,
}

pub struct MempoolListener {
    rpc_url: String,
    client: Client,
    known_txs: Arc<RwLock<HashSet<String>>>,
    tx_sender: mpsc::UnboundedSender<NewTxEvent>,
}

impl MempoolListener {
    pub fn new(rpc_url: String) -> (Self, mpsc::UnboundedReceiver<NewTxEvent>) {
        let (tx_sender, tx_receiver) = mpsc::unbounded_channel();
        
        let listener = Self {
            rpc_url,
            client: Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .unwrap(),
            known_txs: Arc::new(RwLock::new(HashSet::new())),
            tx_sender,
        };
        
        (listener, tx_receiver)
    }
    
    pub async fn start(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(2));
        
        loop {
            interval.tick().await;
            
            if let Err(e) = self.poll_mempool().await {
                eprintln!("Mempool poll error: {}", e);
            }
        }
    }
    
    async fn poll_mempool(&self) -> Result<(), MempoolError> {
        let mempool_txs = self.get_raw_mempool().await?;
        
        let mut known = self.known_txs.write().await;
        
        for tx_hash in mempool_txs {
            if !known.contains(&tx_hash) {
                known.insert(tx_hash.clone());
                
                let event = NewTxEvent {
                    tx_hash: tx_hash.clone(),
                    detected_at: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    initial_confidence: 0.2,
                };
                
                if self.tx_sender.send(event.clone()).is_ok() {
                    println!("New tx detected: {}", tx_hash);
                }
            }
        }
        
        Ok(())
    }
    
    async fn get_raw_mempool(&self) -> Result<Vec<String>, MempoolError> {
        let resp = self.client
            .post(&self.rpc_url)
            .json(&json!({
                "jsonrpc": "2.0",
                "method": "getrawmempool",
                "params": [false],
                "id": 1
            }))
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;
        
        if let Some(error) = resp.get("error") {
            if !error.is_null() {
                return Err(MempoolError::RpcError(error.to_string()));
            }
        }
        
        let result = resp.get("result")
            .ok_or_else(|| MempoolError::RpcError("No result field".to_string()))?;
        
        if let Some(arr) = result.as_array() {
            let txs: Vec<String> = arr
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();
            return Ok(txs);
        }
        
        Ok(vec![])
    }
    
    pub async fn get_mempool_entry(&self, tx_hash: &str) -> Result<MempoolTransaction, MempoolError> {
        let resp = self.client
            .post(&self.rpc_url)
            .json(&json!({
                "jsonrpc": "2.0",
                "method": "getmempoolentry",
                "params": [tx_hash],
                "id": 1
            }))
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;
        
        if let Some(error) = resp.get("error") {
            if !error.is_null() {
                return Err(MempoolError::RpcError(error.to_string()));
            }
        }
        
        let result = resp.get("result")
            .ok_or_else(|| MempoolError::RpcError("No result field".to_string()))?;
        
        Ok(MempoolTransaction {
            tx_hash: tx_hash.to_string(),
            size: result["size"].as_u64().unwrap_or(0),
            fee: result["fee"].as_f64().unwrap_or(0.0),
            time: result["time"].as_u64().unwrap_or(0),
            height: result["height"].as_u64().unwrap_or(0),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mempool_listener() {
        dotenv::dotenv().ok();
        
        let rpc_url = std::env::var("GETBLOCK_API_KEY").unwrap();
        let (listener, mut rx) = MempoolListener::new(rpc_url);
        
        tokio::spawn(async move {
            listener.start().await;
        });
        
        tokio::time::sleep(Duration::from_secs(10)).await;
        
        while let Ok(event) = rx.try_recv() {
            println!("Received new tx: {:?}", event);
        }
    }
}