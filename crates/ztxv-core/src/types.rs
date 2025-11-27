use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub tx_hash: String,
    pub valid: bool,
    pub confidence: f64, // percentage confidence in validity
    pub details: VerificationDetails,
    pub status: TransactionStatus,
    pub timestamp: u64, // Unix timestamp of verification
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VerificationDetails {
    pub proof_valid: bool,
    pub nullifier_unused: bool,
    pub mempool_seen: bool,
    pub network_propagation: f64, // percentage of network nodes that have seen the transaction
    pub estimated_finality_seconds: u32, 
}

impl VerificationDetails {
    pub fn is_valid(&self) -> bool {
        self.proof_valid && self.nullifier_unused 
    } //&& self.mempool_seen
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionStatus {
    NotFound,
    InMempool,
    InBlock(u32), // number of confirmations
    Confirmed(u32), // number of confirmations
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfidenceLevel {
    Low,
    Medium,
    High,
    Confirmed,
}

impl ConfidenceLevel {
    pub fn threshold(&self) -> f64 {
        match self {
            Self::Low => 70.0,
            Self::Medium => 85.0,
            Self::High => 95.0,
            Self::Confirmed => 99.0,
        }
    }

    pub fn from_score(score: f64) -> Self {
        if score >= Self::Confirmed.threshold() {
            Self::Confirmed
        } else if score >= Self::High.threshold() {
            Self::High
        } else if score >= Self::Medium.threshold() {
            Self::Medium
        } else {
            Self::Low
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_level_thresholds() {
        assert_eq!(ConfidenceLevel::Low.threshold(), 70.0);
        assert_eq!(ConfidenceLevel::Medium.threshold(), 85.0);
        assert_eq!(ConfidenceLevel::High.threshold(), 95.0);
        assert_eq!(ConfidenceLevel::Confirmed.threshold(), 99.0);
    }

    #[test]
    fn test_confidence_level_from_score() {
        assert_eq!(ConfidenceLevel::from_score(65.0), ConfidenceLevel::Low);
        assert_eq!(ConfidenceLevel::from_score(80.0), ConfidenceLevel::Low);
        assert_eq!(ConfidenceLevel::from_score(90.0), ConfidenceLevel::Medium);
        assert_eq!(ConfidenceLevel::from_score(97.0), ConfidenceLevel::High);
        assert_eq!(ConfidenceLevel::from_score(99.5), ConfidenceLevel::Confirmed);
    }

    #[test]
    fn test_verification_details_all_valid() {
        // if all checks pass, is_valid should return true
        let details = VerificationDetails {
            proof_valid: true,
            nullifier_unused: true,
            mempool_seen: true,
            network_propagation: 0.95,
            estimated_finality_seconds: 25,
        };

        assert!(details.is_valid());
    }

    #[test] 
    fn test_verification_details_invalid_proof() {
        //returns false if proof is invalid
        let details = VerificationDetails {
            proof_valid: false,
            nullifier_unused: true,
            mempool_seen: true,
            network_propagation: 0.95,
            estimated_finality_seconds: 25,
        };

        assert!(!details.is_valid());
    }

    #[test]
    fn test_transaction_status() {
        let not_found = TransactionStatus::NotFound;
        let in_mempool = TransactionStatus::InMempool;
        let in_block = TransactionStatus::InBlock(1);
        let confirmed = TransactionStatus::Confirmed(10);

        assert!(matches!(not_found, TransactionStatus::NotFound));
        assert!(matches!(in_mempool, TransactionStatus::InMempool));

        if let TransactionStatus::InBlock(confirmations) = in_block {
            assert_eq!(confirmations, 1);
        } 
        
        if let TransactionStatus::Confirmed(confirmations) = confirmed {
            assert_eq!(confirmations, 10);
        } 
    }

    #[test]
    fn test_verification_result_creation() {
        let result = VerificationResult {
            tx_hash: "testhash".to_string(),
            valid: true,
            confidence: 95.5,
            details: VerificationDetails {
                proof_valid: true,
                nullifier_unused: true,
                mempool_seen: true,
                network_propagation: 0.85,
                estimated_finality_seconds: 30,
            },
            status: TransactionStatus::InMempool,
            timestamp: 1625247600,
       
        };
        assert_eq!(result.tx_hash, "testhash");
        assert!(result.valid);
        assert_eq!(result.confidence, 95.5);

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("testhash"));
    }
}