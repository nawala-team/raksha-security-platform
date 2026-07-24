use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents an anomaly detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyResult {
    pub id: Uuid,
    pub source_event_id: Uuid,
    pub score: f64,
    pub is_anomaly: bool,
    pub features: Vec<FeatureContribution>,
    pub model_version: String,
    pub detected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureContribution {
    pub name: String,
    pub value: f64,
    pub contribution: f64,
}

/// Anomaly detection engine
#[derive(Clone)]
pub struct AnomalyDetector {
    threshold: f64,
    model_version: String,
}

impl AnomalyDetector {
    pub fn new(threshold: f64) -> Self {
        Self {
            threshold,
            model_version: "v0.1.0-baseline".to_string(),
        }
    }

    /// Score a set of features for anomaly detection
    /// In production this would call an ML model; for now uses statistical heuristics
    pub fn detect(&self, features: &[(&str, f64)]) -> AnomalyResult {
        let mut total_score = 0.0;
        let mut contributions = Vec::new();

        for (name, value) in features {
            // Simple z-score based heuristic (placeholder for real ML model)
            let contribution = (value.abs() / 100.0).min(1.0);
            total_score += contribution;

            contributions.push(FeatureContribution {
                name: name.to_string(),
                value: *value,
                contribution,
            });
        }

        let normalized_score = if features.is_empty() {
            0.0
        } else {
            (total_score / features.len() as f64).min(1.0)
        };

        AnomalyResult {
            id: Uuid::now_v7(),
            source_event_id: Uuid::nil(),
            score: normalized_score,
            is_anomaly: normalized_score > self.threshold,
            features: contributions,
            model_version: self.model_version.clone(),
            detected_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_traffic() {
        let detector = AnomalyDetector::new(0.7);
        let result = detector.detect(&[("requests_per_min", 50.0), ("error_rate", 2.0)]);
        assert!(!result.is_anomaly);
    }

    #[test]
    fn test_anomalous_traffic() {
        let detector = AnomalyDetector::new(0.5);
        let result = detector.detect(&[("requests_per_min", 500.0), ("error_rate", 90.0)]);
        assert!(result.is_anomaly);
    }
}
