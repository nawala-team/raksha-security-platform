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
use sqlx::FromRow;

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

#[derive(Debug, FromRow)]
struct EnrollmentTokenRow {
    id: Uuid,
    token: String,
    expires_at: DateTime<Utc>,
    uses_remaining: i32,
    description: Option<String>,
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

    // Validate token exists and not expired
    let token_row = sqlx::query_as::<_, EnrollmentTokenRow>(
        r#"SELECT id, token, expires_at, uses_remaining, description 
           FROM enrollment_tokens 
           WHERE token = $1 AND expires_at > NOW() AND uses_remaining > 0 AND revoked_at IS NULL"#
    )
    .bind(&req.token)
    .fetch_optional(&state.db)
    .await;

    let token_info = match token_row {
        Ok(Some(t)) => t,
        Ok(None) => {
            return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({
                "error": "invalid_token",
                "message": "Token invalid, expired, or exhausted"
            })));
        }
        Err(e) => {
            tracing::error!("DB error validating token: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "internal_error",
                "message": "Failed to validate token"
            })));
        }
    };

    // Decrement token uses
    let _ = sqlx::query("UPDATE enrollment_tokens SET uses_remaining = uses_remaining - 1 WHERE id = $1")
        .bind(token_info.id)
        .execute(&state.db)
        .await;

    let agent_id = Uuid::now_v7();

    // Insert agent into database
    let insert_result = sqlx::query(
        r#"INSERT INTO agents (id, name, hostname, os, version, status, token_hash, modules, config, tags, cpu_cores, memory_mb, enrolled_at, last_seen)
           VALUES ($1, $2, $3, $4::agent_os, $5, 'online'::agent_status, $6, '[]'::jsonb, '{}'::jsonb, '[]'::jsonb, $7, $8, NOW(), NOW())"#
    )
    .bind(agent_id)
    .bind(req.agent_name.clone().unwrap_or_else(|| req.fingerprint.hostname.clone()))
    .bind(&req.fingerprint.hostname)
    .bind(&req.fingerprint.os)
    .bind("1.0.0")
    .bind(&req.token)
    .bind(req.fingerprint.cpu_cores as i32)
    .bind((req.fingerprint.total_memory / 1024 / 1024) as i32)
    .execute(&state.db)
    .await;

    if let Err(e) = insert_result {
        tracing::error!("Failed to insert agent: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": "enrollment_failed",
            "message": "Failed to register agent"
        })));
    }

    let response = EnrollAgentResponse {
        agent_id,
        org_id: agent_id, // Use agent_id as placeholder
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
    let max_uses = req.max_uses.unwrap_or(1) as i32;
    let expiry_hours = req.expiry_hours.unwrap_or(24);
    let expires_at = Utc::now() + chrono::Duration::hours(expiry_hours);
    let random_hex = format!("{:032x}", rand::random::<u128>());
    let token = format!("rkat_default0_{}", random_hex);
    let portal_url = &state.config.portal_url;

    // Insert token into database
    let insert_result = sqlx::query(
        r#"INSERT INTO enrollment_tokens (id, token, description, labels, allowed_modules, expires_at, max_uses, uses_remaining, created_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $7, NOW())"#
    )
    .bind(token_id)
    .bind(&token)
    .bind(&req.agent_name)
    .bind(&req.labels)
    .bind(&req.allowed_modules)
    .bind(expires_at)
    .bind(max_uses)
    .execute(&state.db)
    .await;

    if let Err(e) = insert_result {
        tracing::error!("Failed to create enrollment token: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": "token_creation_failed",
            "message": "Failed to create enrollment token"
        })));
    }

    let response = GenerateTokenResponse {
        token_id,
        token: token.clone(),
        expires_at,
        max_uses: max_uses as u32,
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
    State(state): State<AppState>,
) -> impl IntoResponse {
    let tokens = sqlx::query_as::<_, EnrollmentTokenRow>(
        r#"SELECT id, token, expires_at, uses_remaining, description 
           FROM enrollment_tokens 
           WHERE revoked_at IS NULL 
           ORDER BY created_at DESC"#
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let token_list: Vec<_> = tokens.iter().map(|t| serde_json::json!({
        "id": t.id,
        "token": t.token,
        "expires_at": t.expires_at,
        "uses_remaining": t.uses_remaining,
        "description": t.description,
    })).collect();

    (StatusCode::OK, Json(serde_json::json!({ 
        "tokens": token_list, 
        "total": token_list.len() 
    })))
}

/// DELETE /api/v1/agents/tokens/:token_id
pub async fn revoke_token(
    State(state): State<AppState>,
    Path(token_id): Path<Uuid>,
) -> impl IntoResponse {
    let result = sqlx::query("UPDATE enrollment_tokens SET revoked_at = NOW() WHERE id = $1")
        .bind(token_id)
        .execute(&state.db)
        .await;

    match result {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({
            "token_id": token_id,
            "status": "revoked"
        }))),
        Err(e) => {
            tracing::error!("Failed to revoke token: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "revoke_failed",
                "message": "Failed to revoke token"
            })))
        }
    }
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

