use thiserror::Error;

#[derive(Error, Debug)]
pub enum ZtxvError {
    #[error("RPC request failed: {0}")]
    RpcError(String),

    #[error("Invalid transaction hash: {0}")]
    InvalidTxHash(String),

    #[error("Transaction not found: {0}")]
    TxNotFound(String),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, ZtxvError>;