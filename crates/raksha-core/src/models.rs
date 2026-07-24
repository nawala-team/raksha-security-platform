//! Raksha Security Platform - Core Domain Models
//!
//! Rust struct definitions matching the PostgreSQL schema.
//! Uses `uuid`, `chrono`, `serde`, `sqlx`, and `serde_json` for
//! database interaction and serialization.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::net::IpAddr;
use uuid::Uuid;

/// Unique identifier type for all entities
pub type Id = Uuid;

/// Generate a new time-sorted UUID v7
pub fn new_id() -> Id {
    Uuid::now_v7()
}

// ============================================================
// Pagination
// ============================================================

/// Pagination parameters
#[derive(Debug, Clone, Deserialize)]
pub struct Pagination {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

fn default_page() -> u32 {
    1
}

fn default_per_page() -> u32 {
    25
}

impl Pagination {
    pub fn offset(&self) -> i64 {
        ((self.page.saturating_sub(1)) * self.per_page) as i64
    }

    pub fn limit(&self) -> i64 {
        self.per_page.min(100) as i64
    }
}

/// Paginated response wrapper
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub data: Vec<T>,
    pub meta: PaginationMeta,
}

#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    pub page: u32,
    pub per_page: u32,
    pub total: i64,
    pub total_pages: u32,
}

// ============================================================
// Enums
// ============================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum UserStatus {
    Active,
    Inactive,
    Suspended,
    PendingVerification,
    Locked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "permission_action", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum PermissionAction {
    Create,
    Read,
    Update,
    Delete,
    Execute,
    Approve,
    Export,
    Manage,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "mfa_method", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum MfaMethod {
    Totp,
    Webauthn,
    Sms,
    Email,
    RecoveryCodes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "audit_action_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AuditActionType {
    Create,
    Read,
    Update,
    Delete,
    Login,
    Logout,
    LoginFailed,
    PermissionChange,
    ConfigChange,
    Export,
    Import,
    Escalation,
    Approval,
    Rejection,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "audit_action_category", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AuditActionCategory {
    Authentication,
    Authorization,
    DataAccess,
    DataModification,
    SystemConfig,
    SecurityEvent,
    Compliance,
    UserManagement,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "audit_risk_level", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AuditRiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "agent_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    Online,
    Offline,
    Degraded,
    Updating,
    Enrolling,
    Decommissioned,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "agent_os", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AgentOs {
    Linux,
    Windows,
    Macos,
    Freebsd,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "agent_arch", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AgentArch {
    X86_64,
    Aarch64,
    Armv7,
    X86,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "metric_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum MetricType {
    Gauge,
    Counter,
    Histogram,
    Summary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "alert_severity", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AlertSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "alert_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AlertStatus {
    Open,
    Acknowledged,
    Investigating,
    Resolved,
    FalsePositive,
    Suppressed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "compliance_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ComplianceStatus {
    Compliant,
    NonCompliant,
    PartiallyCompliant,
    NotAssessed,
    NotApplicable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "check_result", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum CheckResult {
    Pass,
    Fail,
    Warning,
    Error,
    Skip,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "policy_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum PolicyStatus {
    Draft,
    PendingReview,
    Approved,
    Published,
    Deprecated,
    Archived,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "policy_category", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum PolicyCategory {
    AccessControl,
    DataProtection,
    IncidentResponse,
    NetworkSecurity,
    PhysicalSecurity,
    BusinessContinuity,
    RiskManagement,
    Compliance,
    AcceptableUse,
    ChangeManagement,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "indicator_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum IndicatorType {
    IpAddress,
    Domain,
    Url,
    Email,
    FileHashMd5,
    FileHashSha1,
    FileHashSha256,
    Mutex,
    RegistryKey,
    UserAgent,
    Cidr,
    Cve,
    YaraRule,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "threat_severity", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ThreatSeverity {
    Unknown,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "ml_model_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum MlModelType {
    AnomalyDetection,
    Classification,
    Regression,
    Clustering,
    Nlp,
    TimeSeries,
    Reinforcement,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "ml_model_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum MlModelStatus {
    Training,
    Validating,
    Ready,
    Deployed,
    Deprecated,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "monitored_db_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum MonitoredDbType {
    Postgresql,
    Mysql,
    Mongodb,
    Sqlserver,
    Oracle,
    Redis,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "monitor_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum MonitorStatus {
    Active,
    Inactive,
    Error,
    Maintenance,
}

// ============================================================
// Alert Rule Status enum (needed by AlertRule struct)
// ============================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "alert_rule_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AlertRuleStatus {
    Active,
    Disabled,
    Testing,
}

// ============================================================
// Indicator Status enum (needed by ThreatIndicator struct)
// ============================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "indicator_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum IndicatorStatus {
    Active,
    Expired,
    Revoked,
    UnderReview,
}

// ============================================================
// Document Type enums (needed by Document struct)
// ============================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "document_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum DocumentStatus {
    Draft,
    Published,
    Archived,
    Deleted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "document_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum DocumentType {
    Policy,
    Procedure,
    Guideline,
    Standard,
    Report,
    Template,
    Evidence,
    RiskAssessment,
    IncidentReport,
}

// ============================================================
// Struct Models
// ============================================================

/// User account
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub display_name: String,
    pub status: UserStatus,
    pub mfa_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub failed_attempts: i32,
    pub locked_until: Option<DateTime<Utc>>,
}

/// Organization
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Organization {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub domain: Option<String>,
    pub settings: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Role definition
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_system: bool,
    pub priority: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Permission definition
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Permission {
    pub id: Uuid,
    pub resource: String,
    pub action: PermissionAction,
    pub description: Option<String>,
    pub conditions: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// User-role assignment scoped by organization
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserRoleAssignment {
    pub id: Uuid,
    pub user_id: Uuid,
    pub role_id: Uuid,
    pub org_id: Uuid,
    pub granted_by: Option<Uuid>,
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

/// User role enum for RBAC
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    SuperAdmin,
    Admin,
    Analyst,
    Operator,
    Viewer,
}

impl UserRole {
    /// Check if this role has at least the permissions of the required role.
    /// Role hierarchy: SuperAdmin > Admin > Analyst > Operator > Viewer
    pub fn has_permission(&self, required: &UserRole) -> bool {
        self.level() >= required.level()
    }

    fn level(&self) -> u8 {
        match self {
            UserRole::SuperAdmin => 100,
            UserRole::Admin => 80,
            UserRole::Analyst => 60,
            UserRole::Operator => 40,
            UserRole::Viewer => 20,
        }
    }
}

/// Session
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    #[serde(skip_serializing)]
    pub token_hash: String,
    pub ip_address: IpAddr,
    pub user_agent: Option<String>,
    pub device_info: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

/// API key
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ApiKey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub org_id: Option<Uuid>,
    pub name: String,
    pub key_prefix: String,
    #[serde(skip_serializing)]
    pub key_hash: String,
    pub scopes: serde_json::Value,
    pub rate_limit: Option<i32>,
    pub last_used: Option<DateTime<Utc>>,
    pub last_ip: Option<IpAddr>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub revoked: bool,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revoked_by: Option<Uuid>,
}

/// MFA method registration
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserMfa {
    pub id: Uuid,
    pub user_id: Uuid,
    pub method: MfaMethod,
    #[serde(skip_serializing)]
    pub secret_encrypted: String,
    #[serde(skip_serializing)]
    pub backup_codes: Option<Vec<String>>,
    pub verified: bool,
    pub verified_at: Option<DateTime<Utc>>,
    pub last_used: Option<DateTime<Utc>>,
    pub device_name: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Audit trail entry
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditTrail {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub actor_id: Option<Uuid>,
    pub actor_email: Option<String>,
    pub actor_ip: Option<IpAddr>,
    pub action_type: AuditActionType,
    pub action_category: AuditActionCategory,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub changes_before: Option<serde_json::Value>,
    pub changes_after: Option<serde_json::Value>,
    pub metadata: serde_json::Value,
    pub risk_level: AuditRiskLevel,
    pub session_id: Option<Uuid>,
    pub org_id: Option<Uuid>,
    pub integrity_hash: String,
    pub previous_hash: Option<String>,
}

/// Security agent
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Agent {
    pub id: Uuid,
    pub name: String,
    pub hostname: String,
    pub os: AgentOs,
    pub arch: AgentArch,
    pub version: String,
    pub status: AgentStatus,
    pub last_seen: Option<DateTime<Utc>>,
    pub enrolled_at: DateTime<Utc>,
    pub enrolled_by: Option<Uuid>,
    #[serde(skip_serializing)]
    pub token_hash: String,
    pub modules: serde_json::Value,
    pub config: serde_json::Value,
    pub tags: serde_json::Value,
    pub org_id: Option<Uuid>,
    pub ip_address: Option<IpAddr>,
    pub network_zone: Option<String>,
    pub cpu_cores: Option<i32>,
    pub memory_mb: Option<i32>,
    pub disk_gb: Option<i32>,
    pub updated_at: DateTime<Utc>,
}

/// Agent metrics data point
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AgentMetric {
    pub timestamp: DateTime<Utc>,
    pub agent_id: Uuid,
    pub metric_type: MetricType,
    pub metric_name: String,
    pub value: f64,
    pub labels: serde_json::Value,
}

/// Alert
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Alert {
    pub id: Uuid,
    pub rule_id: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub severity: AlertSeverity,
    pub status: AlertStatus,
    pub source: String,
    pub source_ref: Option<String>,
    pub agent_id: Option<Uuid>,
    pub org_id: Option<Uuid>,
    pub assigned_to: Option<Uuid>,
    pub acknowledged_by: Option<Uuid>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub resolved_by: Option<Uuid>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolution_note: Option<String>,
    pub context: serde_json::Value,
    pub tags: serde_json::Value,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub occurrence_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Alert rule
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AlertRule {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub severity: AlertSeverity,
    pub condition_type: String,
    pub condition_config: serde_json::Value,
    pub throttle_minutes: Option<i32>,
    pub notification_channels: serde_json::Value,
    pub tags: serde_json::Value,
    pub org_id: Option<Uuid>,
    pub status: AlertRuleStatus,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Compliance standard
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ComplianceStandard {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub authority: Option<String>,
    pub url: Option<String>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Compliance control
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ComplianceControl {
    pub id: Uuid,
    pub standard_id: Uuid,
    pub control_ref: String,
    pub title: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub parent_id: Option<Uuid>,
    pub severity: AlertSeverity,
    pub implementation_guidance: Option<String>,
    pub automated: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Compliance score
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ComplianceScore {
    pub id: Uuid,
    pub org_id: Uuid,
    pub standard_id: Uuid,
    pub overall_score: f64,
    pub status: ComplianceStatus,
    pub controls_total: i32,
    pub controls_passed: i32,
    pub controls_failed: i32,
    pub controls_na: i32,
    pub breakdown: serde_json::Value,
    pub assessed_at: DateTime<Utc>,
    pub next_assessment: Option<DateTime<Utc>>,
    pub assessed_by: Option<Uuid>,
}

/// Policy
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Policy {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub summary: Option<String>,
    pub category: PolicyCategory,
    pub status: PolicyStatus,
    pub standard_mapping: serde_json::Value,
    pub version: i32,
    pub previous_version: Option<Uuid>,
    pub org_id: Uuid,
    pub created_by: Uuid,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub published_at: Option<DateTime<Utc>>,
    pub review_cycle: String,
    pub next_review: Option<DateTime<Utc>>,
    pub last_reviewed: Option<DateTime<Utc>>,
    pub tags: serde_json::Value,
    pub effective_date: Option<NaiveDate>,
    pub expiry_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Document
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Document {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub doc_type: DocumentType,
    pub status: DocumentStatus,
    pub content: Option<String>,
    pub content_format: String,
    pub file_path: Option<String>,
    pub file_size: Option<i64>,
    pub mime_type: Option<String>,
    pub checksum: Option<String>,
    pub version: i32,
    pub org_id: Uuid,
    pub created_by: Uuid,
    pub updated_by: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub tags: serde_json::Value,
    pub metadata: serde_json::Value,
    pub access_level: String,
    pub retention_until: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Threat indicator
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ThreatIndicator {
    pub id: Uuid,
    pub indicator_type: IndicatorType,
    pub value: String,
    pub source: String,
    pub source_ref: Option<String>,
    pub severity: ThreatSeverity,
    pub confidence: i32,
    pub status: IndicatorStatus,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub tags: serde_json::Value,
    pub metadata: serde_json::Value,
    pub context: Option<String>,
    pub org_id: Option<Uuid>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// ML model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MlModel {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub model_type: MlModelType,
    pub status: MlModelStatus,
    pub description: Option<String>,
    pub metrics: serde_json::Value,
    pub hyperparameters: serde_json::Value,
    pub file_path: Option<String>,
    pub file_size: Option<i64>,
    pub checksum: Option<String>,
    pub training_data: serde_json::Value,
    pub feature_columns: serde_json::Value,
    pub target_column: Option<String>,
    pub framework: Option<String>,
    pub runtime_version: Option<String>,
    pub org_id: Option<Uuid>,
    pub created_by: Option<Uuid>,
    pub deployed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Monitored database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MonitoredDatabase {
    pub id: Uuid,
    pub name: String,
    pub db_type: MonitoredDbType,
    pub host: String,
    pub port: i32,
    pub database_name: Option<String>,
    #[serde(skip_serializing)]
    pub credentials_encrypted: String,
    pub ssl_enabled: bool,
    #[serde(skip_serializing)]
    pub ssl_ca_cert: Option<String>,
    pub monitoring_config: serde_json::Value,
    pub status: MonitorStatus,
    pub last_check: Option<DateTime<Utc>>,
    pub last_check_result: Option<serde_json::Value>,
    pub health_score: Option<f64>,
    pub alert_thresholds: serde_json::Value,
    pub org_id: Uuid,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

