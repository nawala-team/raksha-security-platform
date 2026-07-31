use chrono::{DateTime, Utc};
use raksha_core::models::{ComplianceStandard, Id};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ComplianceRule {
    pub id: Id,
    pub standard: String,
    pub control_id: String,
    pub title: String,
    pub description: Option<String>,
    pub category: String,
    pub severity: String,
    pub automated: bool,
    pub check_query: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "compliance_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ComplianceStatus {
    Compliant,
    NonCompliant,
    PartiallyCompliant,
    NotApplicable,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub id: Id,
    pub standard: ComplianceStandard,
    pub overall_score: f64,
    pub total_controls: u32,
    pub compliant: u32,
    pub non_compliant: u32,
    pub partially_compliant: u32,
    pub not_applicable: u32,
    pub findings: Vec<ComplianceFinding>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFinding {
    pub rule_id: Id,
    pub control_id: String,
    pub status: ComplianceStatus,
    pub evidence: Option<String>,
    pub remediation: Option<String>,
    pub checked_at: DateTime<Utc>,
}
