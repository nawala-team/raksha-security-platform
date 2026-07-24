use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Enrollment token used by agents to register with the platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrollmentToken {
    pub id: Uuid,
    pub token: String,
    pub label: String,
    pub tenant_id: Uuid,
    pub created_by: Uuid,
    pub expires_at: DateTime<Utc>,
    pub max_uses: Option<u32>,
    pub use_count: u32,
    pub is_revoked: bool,
    pub created_at: DateTime<Utc>,
}

/// Generate a cryptographically random enrollment token string
pub fn generate_token_string() -> String {
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.gen();
    format!("rk_{}", hex::encode(bytes))
}

/// Enrollment request from an agent
#[derive(Debug, Deserialize)]
pub struct EnrollmentRequest {
    pub token: String,
    pub hostname: String,
    pub os: String,
    pub arch: String,
    pub agent_version: String,
    pub labels: Option<Vec<String>>,
}

/// Enrollment response sent back to the agent
#[derive(Debug, Serialize)]
pub struct EnrollmentResponse {
    pub agent_id: Uuid,
    pub tenant_id: Uuid,
    pub certificate: String,
    pub ca_certificate: String,
    pub server_endpoints: Vec<String>,
    pub heartbeat_interval_secs: u64,
    pub config: AgentConfig,
}

/// Agent configuration sent during enrollment
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentConfig {
    pub collect_interval_secs: u64,
    pub fim_enabled: bool,
    pub fim_paths: Vec<String>,
    pub process_monitoring: bool,
    pub network_monitoring: bool,
    pub log_level: String,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            collect_interval_secs: 30,
            fim_enabled: true,
            fim_paths: vec!["/etc".into(), "/usr/bin".into()],
            process_monitoring: true,
            network_monitoring: true,
            log_level: "info".into(),
        }
    }
}
