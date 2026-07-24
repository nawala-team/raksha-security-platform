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
