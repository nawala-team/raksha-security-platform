use raksha_core::models::AlertSeverity;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Alert rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub condition: RuleCondition,
    pub severity: AlertSeverity,
    pub enabled: bool,
    pub cooldown_secs: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleCondition {
    pub field: String,
    pub operator: ConditionOperator,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionOperator {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    Contains,
    Regex,
    Threshold,
}

impl AlertRule {
    pub fn evaluate(&self, data: &serde_json::Value) -> bool {
        if !self.enabled {
            return false;
        }

        match &self.condition.operator {
            ConditionOperator::Equals => {
                data.get(&self.condition.field) == Some(&self.condition.value)
            }
            ConditionOperator::GreaterThan => {
                if let (Some(actual), Some(threshold)) = (
                    data.get(&self.condition.field).and_then(|v| v.as_f64()),
                    self.condition.value.as_f64(),
                ) {
                    actual > threshold
                } else {
                    false
                }
            }
            ConditionOperator::LessThan => {
                if let (Some(actual), Some(threshold)) = (
                    data.get(&self.condition.field).and_then(|v| v.as_f64()),
                    self.condition.value.as_f64(),
                ) {
                    actual < threshold
                } else {
                    false
                }
            }
            _ => false, // Other operators can be implemented as needed
        }
    }
}
