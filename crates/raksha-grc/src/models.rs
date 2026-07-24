//! GRC domain models.
//!
//! Defines the core data structures for risk items, policies, controls,
//! and their cross-framework mappings.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier type (matches raksha-core convention).
pub type Id = Uuid;

// ============================================================
// Risk Models
// ============================================================

/// Risk categories for classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskCategory {
    Technical,
    Operational,
    Compliance,
    Financial,
    Reputational,
    Strategic,
    ThirdParty,
}

impl std::fmt::Display for RiskCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Technical => write!(f, "technical"),
            Self::Operational => write!(f, "operational"),
            Self::Compliance => write!(f, "compliance"),
            Self::Financial => write!(f, "financial"),
            Self::Reputational => write!(f, "reputational"),
            Self::Strategic => write!(f, "strategic"),
            Self::ThirdParty => write!(f, "third_party"),
        }
    }
}

/// Risk lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskStatus {
    Identified,
    Assessed,
    Mitigated,
    Accepted,
    Closed,
}

impl std::fmt::Display for RiskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Identified => write!(f, "identified"),
            Self::Assessed => write!(f, "assessed"),
            Self::Mitigated => write!(f, "mitigated"),
            Self::Accepted => write!(f, "accepted"),
            Self::Closed => write!(f, "closed"),
        }
    }
}

/// A risk register item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskItem {
    pub id: Id,
    pub tenant_id: Id,
    pub title: String,
    pub description: String,
    pub category: RiskCategory,
    /// Likelihood score (1-5).
    pub likelihood: u8,
    /// Impact score (1-5).
    pub impact: u8,
    /// Calculated risk score: likelihood * impact (1-25).
    pub risk_score: u8,
    /// Owner user ID.
    pub owner: Id,
    pub status: RiskStatus,
    pub mitigation_plan: Option<String>,
    /// Next scheduled review date.
    pub review_date: NaiveDate,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl RiskItem {
    /// Calculate risk score from likelihood and impact.
    pub fn calculate_score(likelihood: u8, impact: u8) -> u8 {
        likelihood.clamp(1, 5) * impact.clamp(1, 5)
    }

    /// Returns the risk level label based on the score.
    pub fn risk_level(&self) -> &'static str {
        match self.risk_score {
            1..=4 => "low",
            5..=9 => "medium",
            10..=15 => "high",
            16..=25 => "critical",
            _ => "unknown",
        }
    }
}

// ============================================================
// Policy Models
// ============================================================

/// Policy lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyStatus {
    Draft,
    Active,
    Archived,
}

impl std::fmt::Display for PolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "draft"),
            Self::Active => write!(f, "active"),
            Self::Archived => write!(f, "archived"),
        }
    }
}

/// An organizational policy document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub id: Id,
    pub tenant_id: Id,
    pub title: String,
    /// Semantic version (e.g., "1.0", "2.1").
    pub version: String,
    /// Full policy content (Markdown).
    pub content: String,
    pub status: PolicyStatus,
    /// User who approved this version.
    pub approved_by: Option<Id>,
    /// Date when the policy becomes effective.
    pub effective_date: Option<NaiveDate>,
    /// Days between mandatory reviews.
    pub review_cycle_days: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Tracks a user's acknowledgment of a specific policy version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyAcknowledgment {
    pub id: Id,
    pub tenant_id: Id,
    pub policy_id: Id,
    pub user_id: Id,
    pub acknowledged_at: DateTime<Utc>,
    /// The policy version that was acknowledged.
    pub version_acknowledged: String,
}

// ============================================================
// Control & Framework Models
// ============================================================

/// Supported compliance frameworks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Framework {
    #[serde(rename = "CIS")]
    Cis,
    #[serde(rename = "NIST")]
    Nist,
    #[serde(rename = "PCI-DSS")]
    PciDss,
    #[serde(rename = "ISO-27001")]
    Iso27001,
    #[serde(rename = "SOC2")]
    Soc2,
    #[serde(rename = "HIPAA")]
    Hipaa,
}

impl std::fmt::Display for Framework {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cis => write!(f, "CIS"),
            Self::Nist => write!(f, "NIST"),
            Self::PciDss => write!(f, "PCI-DSS"),
            Self::Iso27001 => write!(f, "ISO-27001"),
            Self::Soc2 => write!(f, "SOC2"),
            Self::Hipaa => write!(f, "HIPAA"),
        }
    }
}

/// Control implementation status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlStatus {
    Implemented,
    Partial,
    NotImplemented,
    NotApplicable,
}

impl std::fmt::Display for ControlStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Implemented => write!(f, "implemented"),
            Self::Partial => write!(f, "partial"),
            Self::NotImplemented => write!(f, "not_implemented"),
            Self::NotApplicable => write!(f, "not_applicable"),
        }
    }
}

/// An internal security control.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Control {
    pub id: Id,
    pub tenant_id: Id,
    pub title: String,
    pub description: String,
    /// Primary framework this control references.
    pub framework: Framework,
    /// Framework-specific control ID (e.g., "CIS 1.1", "NIST AC-1").
    pub control_ref: String,
    pub status: ControlStatus,
    /// Evidence of implementation (URL, description, or artifact reference).
    pub evidence: Option<String>,
    /// When this control was last assessed.
    pub last_assessed: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Maps a single internal control to one or more framework requirements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlMapping {
    pub id: Id,
    pub control_id: Id,
    pub framework: Framework,
    /// The framework-specific requirement ID.
    pub framework_ref: String,
    /// Optional description of how the control satisfies the requirement.
    pub rationale: Option<String>,
}

// ============================================================
// API Request/Response DTOs
// ============================================================

/// Request to create or update a risk item.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateRiskRequest {
    pub title: String,
    pub description: String,
    pub category: RiskCategory,
    pub likelihood: u8,
    pub impact: u8,
    pub owner: Id,
    pub mitigation_plan: Option<String>,
    pub review_date: NaiveDate,
}

/// Request to create or update a policy.
#[derive(Debug, Clone, Deserialize)]
pub struct CreatePolicyRequest {
    pub title: String,
    pub version: String,
    pub content: String,
    pub effective_date: Option<NaiveDate>,
    pub review_cycle_days: Option<i32>,
}

/// Request to create a control.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateControlRequest {
    pub title: String,
    pub description: String,
    pub framework: Framework,
    pub control_ref: String,
    pub evidence: Option<String>,
}

/// Request to add a control mapping.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateControlMappingRequest {
    pub control_id: Id,
    pub framework: Framework,
    pub framework_ref: String,
    pub rationale: Option<String>,
}

/// Request to acknowledge a policy.
#[derive(Debug, Clone, Deserialize)]
pub struct AcknowledgePolicyRequest {
    pub policy_id: Id,
    pub user_id: Id,
}

/// GRC dashboard summary statistics.
#[derive(Debug, Clone, Serialize)]
pub struct GrcDashboard {
    pub total_risks: u64,
    pub critical_risks: u64,
    pub high_risks: u64,
    pub overdue_reviews: u64,
    pub active_policies: u64,
    pub pending_acknowledgments: u64,
    pub total_controls: u64,
    pub implemented_controls: u64,
    pub partial_controls: u64,
    pub not_implemented_controls: u64,
}

/// 5x5 risk heatmap cell.
#[derive(Debug, Clone, Serialize)]
pub struct HeatmapCell {
    pub likelihood: u8,
    pub impact: u8,
    pub count: u64,
    pub risk_ids: Vec<Id>,
}

/// Risk heatmap response (5x5 grid).
#[derive(Debug, Clone, Serialize)]
pub struct RiskHeatmap {
    pub cells: Vec<HeatmapCell>,
    pub total_risks: u64,
}

/// Framework coverage report.
#[derive(Debug, Clone, Serialize)]
pub struct FrameworkCoverage {
    pub framework: Framework,
    pub total_requirements: u64,
    pub implemented: u64,
    pub partial: u64,
    pub not_implemented: u64,
    pub not_applicable: u64,
    pub coverage_percent: f64,
}
