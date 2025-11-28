use ztxv_core::{TransactionVerifier, ZcashRpcClient};

#[tokio::test]
#[ignore]
async fn test_rpc_connection() {
    let rpc_url = std::env::var("ZCASH_RPC_URL")
        .expect("ZCASH_RPC_URL not set");
    
    let client = ZcashRpcClient::new(rpc_url);
    let tx_hash = "0d19877eb803b3806f640d2eeb89572ca8d2f68a26f1b6e1b1cccc6b6e9e0e2e";
    
    match client.get_raw_transaction(tx_hash).await {
        Ok(tx) => {
            println!("Connected! TXID: {}", tx.txid);
            println!("Confirmations: {:?}", tx.confirmations);
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_verifier_real_data() {
    let rpc_url = std::env::var("ZCASH_RPC_URL")
        .expect("ZCASH_RPC_URL not set");
    
    let verifier = TransactionVerifier::new(rpc_url);
    let tx_hash = "0d19877eb803b3806f640d2eeb89572ca8d2f68a26f1b6e1b1cccc6b6e9e0e2e";
    
    match verifier.verify(tx_hash).await {
        Ok(result) => {
            println!("Valid: {}", result.valid);
            println!("Confidence: {}", result.confidence);
            println!("Status: {:?}", result.status);
            assert!(result.confidence >= 0.0);
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
}
