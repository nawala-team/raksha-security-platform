use chrono::{DateTime, Utc};
use raksha_core::models::{AlertSeverity, AlertStatus, Id};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: Id,
    pub title: String,
    pub description: String,
    pub severity: AlertSeverity,
    pub status: AlertStatus,
    pub source: String,
    pub source_id: Option<String>,
    pub agent_id: Option<Id>,
    pub assigned_to: Option<Id>,
    pub rule_id: Option<Id>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateAlert {
    pub title: String,
    pub description: String,
    pub severity: AlertSeverity,
    pub source: String,
    pub source_id: Option<String>,
    pub agent_id: Option<Id>,
    pub rule_id: Option<Id>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AlertFilter {
    pub severity: Option<AlertSeverity>,
    pub status: Option<AlertStatus>,
    pub source: Option<String>,
    pub agent_id: Option<Id>,
    pub assigned_to: Option<Id>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
}
