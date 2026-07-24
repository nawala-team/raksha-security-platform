//! Enrollment API handler - Agent registration endpoint
//!
//! POST /api/v1/agents/enroll - Register a new agent
//! POST /api/v1/agents/tokens - Generate enrollment token (admin)
//! GET  /api/v1/agents/tokens - List active tokens (admin)
//! DELETE /api/v1/agents/tokens/:id - Revoke token (admin)

use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct EnrollAgentRequest {
    pub token: String,
    pub fingerprint: AgentFingerprint,
    pub agent_name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AgentFingerprint {
    pub hostname: String,
    pub os: String,
    pub os_version: String,
    pub arch: String,
    pub machine_id: String,
    pub cpu_cores: u32,
    pub total_memory: u64,
    pub mac_hash: String,
}

#[derive(Debug, Serialize)]
pub struct EnrollAgentResponse {
    pub agent_id: Uuid,
    pub org_id: Uuid,
    pub status: String,
    pub portal_url: String,
    pub config: AgentConfig,
}

#[derive(Debug, Serialize)]
pub struct AgentConfig {
    pub report_interval_secs: u64,
    pub heartbeat_interval_secs: u64,
    pub modules: Vec<String>,
    pub log_level: String,
}

#[derive(Debug, Deserialize)]
pub struct GenerateTokenRequest {
    pub agent_name: Option<String>,
    pub labels: Vec<String>,
    pub expiry_hours: Option<i64>,
    pub max_uses: Option<u32>,
    pub allowed_modules: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct GenerateTokenResponse {
    pub token_id: Uuid,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub max_uses: u32,
    pub install_command_linux: String,
    pub install_command_windows: String,
}

/// POST /api/v1/agents/enroll
pub async fn enroll_agent(
    State(state): State<AppState>,
    Json(req): Json<EnrollAgentRequest>,
) -> impl IntoResponse {
    if !req.token.starts_with("rkat_") || req.token.len() < 40 {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "error": "invalid_token_format",
            "message": "Token format invalid"
        })));
    }
    if req.fingerprint.hostname.is_empty() || req.fingerprint.machine_id.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "error": "invalid_fingerprint",
            "message": "hostname and machine_id required"
        })));
    }

    let agent_id = Uuid::now_v7();
    let org_id = Uuid::now_v7();

    let response = EnrollAgentResponse {
        agent_id,
        org_id,
        status: "enrolled".to_string(),
        portal_url: state.config.portal_url.clone(),
        config: AgentConfig {
            report_interval_secs: 30,
            heartbeat_interval_secs: 10,
            modules: vec!["server".into(), "network".into()],
            log_level: "info".to_string(),
        },
    };
    (StatusCode::CREATED, Json(serde_json::json!(response)))
}

/// POST /api/v1/agents/tokens
pub async fn generate_token(
    State(state): State<AppState>,
    Json(req): Json<GenerateTokenRequest>,
) -> impl IntoResponse {
    let token_id = Uuid::now_v7();
    let max_uses = req.max_uses.unwrap_or(1);
    let expiry_hours = req.expiry_hours.unwrap_or(24);
    let expires_at = Utc::now() + chrono::Duration::hours(expiry_hours);
    let random_hex = format!("{:032x}", rand::random::<u128>());
    let token = format!("rkat_default0_{}", random_hex);
    let portal_url = &state.config.portal_url;

    let response = GenerateTokenResponse {
        token_id,
        token: token.clone(),
        expires_at,
        max_uses,
        install_command_linux: format!(
            "curl -fsSL {}/api/v1/agent/install | RAKSHA_TOKEN=\"{}\" bash",
            portal_url, token
        ),
        install_command_windows: format!(
            "$env:RAKSHA_TOKEN=\"{}\"; irm {}/api/v1/agent/install.ps1 | iex",
            token, portal_url
        ),
    };
    (StatusCode::CREATED, Json(serde_json::json!(response)))
}

/// GET /api/v1/agents/tokens
pub async fn list_tokens(
    State(_state): State<AppState>,
) -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({ "tokens": [], "total": 0 })))
}

/// DELETE /api/v1/agents/tokens/:token_id
pub async fn revoke_token(
    State(_state): State<AppState>,
    Path(token_id): Path<Uuid>,
) -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({
        "token_id": token_id,
        "status": "revoked"
    })))
}

/// POST /api/v1/agents/:agent_id/rotate-certificate
pub async fn rotate_certificate(
    State(_state): State<AppState>,
    Path(agent_id): Path<Uuid>,
) -> impl IntoResponse {
    let expires = Utc::now() + chrono::Duration::days(30);
    (StatusCode::OK, Json(serde_json::json!({
        "agent_id": agent_id,
        "expires_at": expires,
        "status": "rotated"
    })))
}

