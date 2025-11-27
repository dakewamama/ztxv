use crate::{
    error::Result,
    rpc::ZcashRpcClient,
    types::{TransactionStatus, VerificationDetails, VerificationResult},
};
use std::time::{SystemTime, UNIX_EPOCH};

// transaction verifier
pub struct TransactionVerifier {
    rpc_client: ZcashRpcClient,
}

impl TransactionVerifier {
    pub fn new(rpc_url: String) -> Self {
        Self {
            rpc_client: ZcashRpcClient::new(rpc_url),
        }
    }

    // verify a transaction and return detailed results
    pub async fn verify(&self, tx_hash: &str) -> Result<VerificationResult> {
        // get raw transaction from node
        let raw_tx = self.rpc_client.get_raw_transaction(tx_hash).await?;
        
        // determine transaction status based on confirmations
        let status = match raw_tx.confirmations {
            None => TransactionStatus::InMempool,
            Some(0) => TransactionStatus::InMempool,
            Some(c) if c < 10 => TransactionStatus::InBlock(c),
            Some(c) => TransactionStatus::Confirmed(c),
        };
        
        // calculate confidence based on status
        let confidence = self.calculate_confidence(&status);
        
        // build verification details
        let details = VerificationDetails {
            proof_valid: true, // TODO: Implement actual zk-SNARK verification
            nullifier_unused: true, // TODO: Check against nullifier set
            mempool_seen: raw_tx.confirmations.is_none() || raw_tx.confirmations == Some(0),
            network_propagation: self.estimate_propagation(&status),
            estimated_finality_seconds: self.estimate_finality(&status),
        };
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Ok(VerificationResult {
            tx_hash: tx_hash.to_string(),
            valid: details.is_valid(),
            confidence,
            details,
            status,
            timestamp,
        })
    }
    
    fn calculate_confidence(&self, status: &TransactionStatus) -> f64 {
        match status {
            TransactionStatus::NotFound => 0.0,
            TransactionStatus::InMempool => 75.0,
            TransactionStatus::InBlock(c) => {
                // linear growth from 85% to 99% over 10 confirmations
                85.0 + ((*c as f64 / 10.0) * 14.0)
            }
            TransactionStatus::Confirmed(_) => 99.5,
        }
    }
    
    fn estimate_propagation(&self, status: &TransactionStatus) -> f64 {
        match status {
            TransactionStatus::NotFound => 0.0,
            TransactionStatus::InMempool => 0.7,
            TransactionStatus::InBlock(_) => 0.95,
            TransactionStatus::Confirmed(_) => 1.0,
        }
    }
    
    fn estimate_finality(&self, status: &TransactionStatus) -> u32 {
        match status {
            TransactionStatus::NotFound => 300, // 5 minutes
            TransactionStatus::InMempool => 150, // 2.5 minutes
            TransactionStatus::InBlock(c) => {
                let remaining = 10u32.saturating_sub(*c);
                remaining * 15 // ~15 seconds per block
            }
            TransactionStatus::Confirmed(_) => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_calculation() {
        let verifier = TransactionVerifier::new("http://localhost:8232".to_string());
        
        assert_eq!(verifier.calculate_confidence(&TransactionStatus::NotFound), 0.0);
        assert_eq!(verifier.calculate_confidence(&TransactionStatus::InMempool), 75.0);
        assert_eq!(verifier.calculate_confidence(&TransactionStatus::InBlock(5)), 92.0);
        assert_eq!(verifier.calculate_confidence(&TransactionStatus::Confirmed(15)), 99.5);
    }

    #[test]
    fn test_finality_estimation() {
        let verifier = TransactionVerifier::new("http://localhost:8232".to_string());
        
        assert_eq!(verifier.estimate_finality(&TransactionStatus::InMempool), 150);
        assert_eq!(verifier.estimate_finality(&TransactionStatus::InBlock(2)), 120);
        assert_eq!(verifier.estimate_finality(&TransactionStatus::Confirmed(10)), 0);
    }
}