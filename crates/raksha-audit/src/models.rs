use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub user_id: Option<Uuid>,
    pub action: AuditAction,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_method: String,
    pub request_path: String,
    pub response_status: u16,
    pub duration_ms: u64,
    pub metadata: Option<serde_json::Value>,
    pub hash: String,
    pub previous_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    // Auth
    Login,
    Logout,
    TokenRefresh,
    PasswordChange,
    MfaEnable,
    MfaDisable,
    // CRUD
    Create,
    Read,
    Update,
    Delete,
    // Security
    AccessDenied,
    RateLimited,
    SuspiciousActivity,
    // System
    ConfigChange,
    SystemStart,
    SystemStop,
}

impl std::fmt::Display for AuditAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_value(self)
            .ok()
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| format!("{:?}", self));
        write!(f, "{}", s)
    }
}

impl AuditAction {
    /// Maps the action onto the `audit_action_type` Postgres enum.
    ///
    /// The database enum is intentionally narrower than this Rust enum, so
    /// several application-level actions collapse onto the same storage value.
    /// The original action is always preserved verbatim in `metadata.action`.
    pub fn db_action_type(&self) -> &'static str {
        match self {
            Self::Login | Self::TokenRefresh => "login",
            Self::Logout => "logout",
            Self::PasswordChange | Self::MfaEnable | Self::MfaDisable => "update",
            Self::Create => "create",
            Self::Read => "read",
            Self::Update => "update",
            Self::Delete => "delete",
            Self::AccessDenied | Self::RateLimited => "rejection",
            Self::SuspiciousActivity => "escalation",
            Self::ConfigChange | Self::SystemStart | Self::SystemStop => "config_change",
        }
    }

    /// Maps the action onto the `audit_action_category` Postgres enum.
    pub fn db_action_category(&self) -> &'static str {
        match self {
            Self::Login | Self::Logout | Self::TokenRefresh => "authentication",
            Self::AccessDenied => "authorization",
            Self::PasswordChange | Self::MfaEnable | Self::MfaDisable => "user_management",
            Self::Read => "data_access",
            Self::Create | Self::Update | Self::Delete => "data_modification",
            Self::ConfigChange | Self::SystemStart | Self::SystemStop => "system_config",
            Self::RateLimited | Self::SuspiciousActivity => "security_event",
        }
    }

    /// Maps the action onto the `audit_risk_level` Postgres enum.
    pub fn db_risk_level(&self) -> &'static str {
        match self {
            Self::AccessDenied | Self::SuspiciousActivity => "high",
            Self::RateLimited
            | Self::PasswordChange
            | Self::MfaEnable
            | Self::MfaDisable
            | Self::ConfigChange
            | Self::Delete => "medium",
            _ => "low",
        }
    }
}
