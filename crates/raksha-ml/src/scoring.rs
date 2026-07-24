use serde::{Deserialize, Serialize};

/// Risk score result for an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskScore {
    pub overall: f64,
    pub components: Vec<RiskComponent>,
    pub level: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskComponent {
    pub category: String,
    pub score: f64,
    pub weight: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Critical,
    High,
    Medium,
    Low,
    Minimal,
}

/// Risk scoring engine
#[derive(Clone)]
pub struct RiskScorer {
    weights: Vec<(&'static str, f64)>,
}

impl Default for RiskScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl RiskScorer {
    pub fn new() -> Self {
        Self {
            weights: vec![
                ("vulnerability", 0.3),
                ("compliance", 0.25),
                ("threat_intel", 0.2),
                ("anomaly", 0.15),
                ("exposure", 0.1),
            ],
        }
    }

    /// Calculate composite risk score from category scores
    pub fn calculate(&self, scores: &[(&str, f64)]) -> RiskScore {
        let mut components = Vec::new();
        let mut weighted_sum = 0.0;
        let mut total_weight = 0.0;

        for (category, score) in scores {
            let weight = self
                .weights
                .iter()
                .find(|(c, _)| c == category)
                .map(|(_, w)| *w)
                .unwrap_or(0.1);

            let clamped_score = score.clamp(&0.0, &100.0);

            components.push(RiskComponent {
                category: category.to_string(),
                score: *clamped_score,
                weight,
            });

            weighted_sum += clamped_score * weight;
            total_weight += weight;
        }

        let overall = if total_weight > 0.0 {
            weighted_sum / total_weight
        } else {
            0.0
        };

        let level = match overall as u32 {
            90..=100 => RiskLevel::Critical,
            70..=89 => RiskLevel::High,
            40..=69 => RiskLevel::Medium,
            20..=39 => RiskLevel::Low,
            _ => RiskLevel::Minimal,
        };

        RiskScore {
            overall,
            components,
            level,
        }
    }
}
