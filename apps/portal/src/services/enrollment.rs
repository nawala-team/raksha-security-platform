//! Agent enrollment service - business logic layer

#![allow(dead_code)]

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Enrollment service handles the full agent registration lifecycle
pub struct EnrollmentService {
    portal_url: String,
    cert_validity_days: i64,
    default_token_expiry_hours: i64,
}

/// Agent status after enrollment
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentStatus {
    /// Just enrolled, waiting for first heartbeat
    Pending,
    /// Active and reporting
    Online,
    /// Missed heartbeats (>3x interval)
    Offline,
    /// Manually disabled by admin
    Disabled,
    /// Certificate expired, needs re-enrollment
    CertExpired,
    /// Removed from organization
    Deregistered,
}

/// Enrollment token status in database
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TokenStatus {
    Active,
    Used,
    Expired,
    Revoked,
}

/// Stored token record (in database - never stores raw token)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredToken {
    pub token_id: Uuid,
    /// SHA-256 hash of the actual token
    pub token_hash: String,
    /// First 12 chars of token for display: "rkat_xxxx..."
    pub token_prefix: String,
    pub org_id: Uuid,
    pub agent_name: Option<String>,
    pub labels: Vec<String>,
    pub created_by: Uuid,
    pub created_at: chrono::DateTime<Utc>,
    pub expires_at: chrono::DateTime<Utc>,
    pub max_uses: u32,
    pub use_count: u32,
    pub status: TokenStatus,
    pub allowed_modules: Vec<String>,
    /// IP that last used the token
    pub last_used_ip: Option<String>,
    pub last_used_at: Option<chrono::DateTime<Utc>>,
}

/// Registered agent record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredAgent {
    pub agent_id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub hostname: String,
    pub os: String,
    pub arch: String,
    pub version: String,
    pub status: AgentStatus,
    /// Identity hash from machine fingerprint
    pub identity_hash: String,
    pub enrolled_at: chrono::DateTime<Utc>,
    pub last_seen: Option<chrono::DateTime<Utc>>,
    pub certificate_serial: String,
    pub certificate_expires_at: chrono::DateTime<Utc>,
    pub labels: Vec<String>,
    pub modules: Vec<String>,
    pub config: AgentRuntimeConfig,
}

/// Runtime configuration pushed to agent after enrollment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRuntimeConfig {
    pub report_interval_secs: u64,
    pub heartbeat_interval_secs: u64,
    pub log_level: String,
    pub buffer_size: usize,
    pub retry_attempts: u32,
    pub tls_verify: bool,
}

impl Default for AgentRuntimeConfig {
    fn default() -> Self {
        Self {
            report_interval_secs: 30,
            heartbeat_interval_secs: 10,
            log_level: "info".to_string(),
            buffer_size: 1000,
            retry_attempts: 3,
            tls_verify: true,
        }
    }
}

impl EnrollmentService {
    pub fn new(portal_url: String) -> Self {
        Self {
            portal_url,
            cert_validity_days: 30,
            default_token_expiry_hours: 24,
        }
    }

    /// Validate that a token can be used for enrollment
    pub fn validate_token(&self, token: &StoredToken) -> Result<(), EnrollError> {
        let now = Utc::now();
        
        match token.status {
            TokenStatus::Revoked => return Err(EnrollError::TokenRevoked),
            TokenStatus::Used => return Err(EnrollError::TokenAlreadyUsed),
            TokenStatus::Expired => return Err(EnrollError::TokenExpired),
            TokenStatus::Active => {}
        }

        if now > token.expires_at {
            return Err(EnrollError::TokenExpired);
        }

        if token.use_count >= token.max_uses {
            return Err(EnrollError::TokenMaxUsesReached);
        }

        Ok(())
    }

    /// Check if an agent with the same identity hash already exists
    pub fn check_duplicate_agent(
        &self,
        identity_hash: &str,
        _existing_agents: &[RegisteredAgent],
    ) -> Result<(), EnrollError> {
        // In production: query DB for identity_hash
        // If found and status != Deregistered, reject
        let _ = identity_hash;
        Ok(())
    }

    /// Generate install command for a given token
    pub fn generate_install_command(&self, token: &str, os: &str) -> String {
        match os {
            "linux" | "darwin" => format!(
                "curl -fsSL {}/api/v1/agent/install | RAKSHA_TOKEN=\"{}\" RAKSHA_PORTAL=\"{}\" bash",
                self.portal_url, token, self.portal_url
            ),
            "windows" => format!(
                "$env:RAKSHA_TOKEN=\"{}\"; $env:RAKSHA_PORTAL=\"{}\"; irm {}/api/v1/agent/install.ps1 | iex",
                token, self.portal_url, self.portal_url
            ),
            _ => format!(
                "RAKSHA_TOKEN=\"{}\" RAKSHA_PORTAL=\"{}\" raksha-agent enroll",
                token, self.portal_url
            ),
        }
    }
}

/// Enrollment-related errors
#[derive(Debug, thiserror::Error)]
pub enum EnrollError {
    #[error("Enrollment token has been revoked")]
    TokenRevoked,
    #[error("Enrollment token has already been used")]
    TokenAlreadyUsed,
    #[error("Enrollment token has expired")]
    TokenExpired,
    #[error("Enrollment token max uses reached")]
    TokenMaxUsesReached,
    #[error("Agent with this identity already enrolled")]
    DuplicateAgent,
    #[error("Invalid machine fingerprint: {0}")]
    InvalidFingerprint(String),
    #[error("Certificate issuance failed: {0}")]
    CertificateFailed(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
}
