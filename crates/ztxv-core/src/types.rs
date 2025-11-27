#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
}