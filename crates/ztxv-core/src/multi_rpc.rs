use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::{Duration, Instant};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConsensusError {
    #[error("No consensus: only {0}/{1} nodes agreed")]
    NoConsensus(usize, usize),
    
    #[error("All nodes failed")]
    AllNodesFailed,
}

#[derive(Debug, Clone)]
pub struct RpcNode {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeResponse {
    pub node_name: String,
    pub confirmations: u32,
    pub response_time_ms: u64,
    pub success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConsensusResult {
    pub consensus_reached: bool,
    pub agreement_ratio: f64,
    pub confirmations: u32,
    pub responses: Vec<NodeResponse>,
    pub total_time_ms: u64,
}

pub struct MultiRpcVerifier {
    nodes: Vec<RpcNode>,
    client: Client,
}

impl MultiRpcVerifier {
    pub fn new() -> Self {
        let nodes = vec![
            RpcNode {
                name: "GetBlock".to_string(),
                url: std::env::var("GETBLOCK_API_KEY").unwrap_or_default(),
            },
            RpcNode {
                name: "NowNodes".to_string(),
                url: std::env::var("NOWNODES_API_KEY").unwrap_or_default(),
            },
        ];

        Self {
            nodes,
            client: Client::builder()
                .timeout(Duration::from_secs(8))
                .build()
                .unwrap(),
        }
    }

    pub async fn verify(&self, tx_hash: &str) -> Result<ConsensusResult, ConsensusError> {
        let start = Instant::now();
        let mut tasks = vec![];

        for node in &self.nodes {
            let client = self.client.clone();
            let node = node.clone();
            let tx = tx_hash.to_string();

            tasks.push(tokio::spawn(async move {
                Self::query_node(client, node, tx).await
            }));
        }

        let mut responses = vec![];
        for task in tasks {
            match task.await {
                Ok(Ok(resp)) => responses.push(resp),
                Ok(Err(e)) => eprintln!("Node error: {}", e),
                Err(e) => eprintln!("Task error: {}", e),
            }
        }

        if responses.is_empty() {
            return Err(ConsensusError::AllNodesFailed);
        }

        let successful = responses.iter().filter(|r| r.success).count();
        let total = self.nodes.len();
        let agreement_ratio = successful as f64 / total as f64;

        if agreement_ratio < 0.50 {
            return Err(ConsensusError::NoConsensus(successful, total));
        }

        let mut confs: Vec<u32> = responses
            .iter()
            .filter(|r| r.success)
            .map(|r| r.confirmations)
            .collect();
        confs.sort();
        let confirmations = confs.get(confs.len() / 2).copied().unwrap_or(0);

        Ok(ConsensusResult {
            consensus_reached: true,
            agreement_ratio,
            confirmations,
            responses,
            total_time_ms: start.elapsed().as_millis() as u64,
        })
    }

    async fn query_node(
        client: Client,
        node: RpcNode,
        tx_hash: String,
    ) -> Result<NodeResponse, Box<dyn std::error::Error + Send + Sync>> {
        let start = Instant::now();

        let resp = client
            .post(&node.url)
            .json(&json!({
                "jsonrpc": "2.0",
                "method": "getrawtransaction",
                "params": [tx_hash, 1],
                "id": 1
            }))
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        eprintln!("Node {} response: {}", node.name, resp);

        if let Some(error) = resp.get("error") {
            if !error.is_null() {
                return Err(format!("RPC error: {}", error).into());
            }
        }

        let result = resp.get("result")
            .ok_or("No result field in response")?;
        
        if result.is_null() {
            return Err("Result is null".into());
        }

        let success = true;
        let confirmations = result["confirmations"]
            .as_u64()
            .unwrap_or(0) as u32;

        Ok(NodeResponse {
            node_name: node.name,
            confirmations,
            response_time_ms: start.elapsed().as_millis() as u64,
            success,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_multi_rpc_verify() {
        dotenv::dotenv().ok();
        
        let verifier = MultiRpcVerifier::new();
        let tx_hash = "2916ab4fad38bafc0632716ed431d24e91213e23233ce99583962ed82ffed10e";
        
        match verifier.verify(tx_hash).await {
            Ok(result) => {
                println!("Consensus: {}", result.consensus_reached);
                println!("Agreement: {:.0}%", result.agreement_ratio * 100.0);
                println!("Confirmations: {}", result.confirmations);
                println!("Time: {}ms", result.total_time_ms);
                for resp in &result.responses {
                    println!("  {} - {} - {}ms", 
                        resp.node_name, 
                        if resp.success { "success" } else { "failed" },
                        resp.response_time_ms
                    );
                }
            }
            Err(e) => println!("Error: {}", e),
        }
    }
}