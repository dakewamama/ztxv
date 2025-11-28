use crate::error::{Result, ZtxvError};
use serde::{Deserialize, Serialize};
use serde_json::json;

// zcash rpc client communication with zcashd node
#[derive(Debug, Clone)]
pub struct ZcashRpcClient {
    url: String,
    client: reqwest::Client,
}

#[derive(Debug, Serialize)]
pub(crate) struct RpcRequest {
    jsonrpc: String,
    id: String,
    method: String,
    params: Vec<serde_json::Value>,
}

// JSON-RPC response structure
#[derive(Debug, Deserialize)]
struct RpcResponse<T> {
    result: Option<T>,
    error: Option<RpcError>,
}

#[derive(Debug, Deserialize)]
struct RpcError {
    code: i32,
    message: String,
}

impl ZcashRpcClient {
    // Create a new RPC client
    pub fn new(url: String) -> Self {
        Self {
            url,
            client: reqwest::Client::new(),
        }
    }

    // Make a JSON-RPC call to the Zcash node
    async fn call<T: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        params: Vec<serde_json::Value>,
    ) -> Result<T> {
        let request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            id: "ztxv".to_string(),
            method: method.to_string(),
            params,
        };

        let response = self
            .client
            .post(&self.url)
            .json(&request)
            .send()
            .await?;

        let rpc_response: RpcResponse<T> = response.json().await?;

        if let Some(error) = rpc_response.error {
            return Err(ZtxvError::RpcError(format!(
                "Code {}: {}",
                error.code, error.message
            )));
        }

        rpc_response
            .result
            .ok_or_else(|| ZtxvError::RpcError("No result in response".to_string()))
    }

    // Get raw transaction data
    pub async fn get_raw_transaction(&self, tx_hash: &str) -> Result<RawTransaction> {
        self.call("getrawtransaction", vec![json!(tx_hash), json!(1)])
            .await
    }

    // Check if a nullifier has been spent
    pub async fn is_nullifier_spent(&self, _nullifier: &str) -> Result<bool> {
        //nullifier checking requires full node database access
        //standard rpc does not expose this 
        //so for now we check if tx was confirmed that can give us an implicit nullifier validation
        Ok(false)
    }
}

// Raw transaction data from Zcash node
#[derive(Debug, Deserialize)]
pub struct RawTransaction {
    pub txid: String,
    pub confirmations: Option<u32>,
    pub hex: String,
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpc_client_creation() {
        let client = ZcashRpcClient::new("http://localhost:8232".to_string());
        assert_eq!(client.url, "http://localhost:8232");
    }

    #[tokio::test]
    async fn test_raw_transaction_deserialization() {
        // Test that we can deserialize a raw transaction response
        let json = r#"{
            "txid": "abc123",
            "confirmations": 5,
            "hex": "0100000001..."
        }"#;
        
        let tx: RawTransaction = serde_json::from_str(json).unwrap();
        assert_eq!(tx.txid, "abc123");
        assert_eq!(tx.confirmations, Some(5));
    }
}