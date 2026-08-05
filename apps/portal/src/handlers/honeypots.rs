//! Honeypot deployment and captured attacker interaction endpoints.

use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use raksha_auth::Claims;
use raksha_core::error::{AppError, AppResult};
use raksha_core::models::{Pagination, PaginatedResponse, PaginationMeta, UserRole};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_honeypots).post(create_honeypot))
        .route("/summary", get(honeypot_summary))
        .route("/interactions", get(list_interactions))
        .route("/top-attackers", get(top_attackers))
        .route("/:id", get(get_honeypot).delete(delete_honeypot))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct HoneypotResponse {
    id: Uuid,
    name: String,
    description: Option<String>,
    honeypot_type: String,
    status: String,
    listen_ip: Option<String>,
    listen_port: i32,
    server_id: Option<Uuid>,
    emulated_banner: Option<String>,
    interaction_count: i64,
    unique_attackers: i64,
    last_interaction_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

async fn list_honeypots(
    State(state): State<AppState>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<Vec<HoneypotResponse>>> {
    let honeypots = sqlx::query_as::<_, HoneypotResponse>(
        r#"
        SELECT id, name, description, honeypot_type, status,
               listen_ip::text, listen_port, server_id, emulated_banner,
               interaction_count, unique_attackers, last_interaction_at, created_at
        FROM honeypots
        ORDER BY name
        "#
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(honeypots))
}

#[derive(Debug, Serialize)]
struct HoneypotSummary {
    total: i64,
    running: i64,
    stopped: i64,
    interactions_24h: i64,
    unique_attackers_24h: i64,
    exploit_attempts_24h: i64,
    critical_interactions_24h: i64,
}

#[derive(Debug, sqlx::FromRow)]
struct PotSummaryRow {
    total: i64,
    running: i64,
    stopped: i64,
}

#[derive(Debug, sqlx::FromRow)]
struct InteractionSummaryRow {
    total: i64,
    attackers: i64,
    exploits: i64,
    critical: i64,
}

async fn honeypot_summary(
    State(state): State<AppState>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<HoneypotSummary>> {
    let since = Utc::now() - Duration::hours(24);

    let pots: PotSummaryRow = sqlx::query_as(
        r#"
        SELECT
            COUNT(*)::bigint as total,
            COUNT(*) FILTER (WHERE status = 'running')::bigint as running,
            COUNT(*) FILTER (WHERE status = 'stopped')::bigint as stopped
        FROM honeypots
        "#
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(PotSummaryRow { total: 0, running: 0, stopped: 0 });

    let acts: InteractionSummaryRow = sqlx::query_as(
        r#"
        SELECT
            COUNT(*)::bigint as total,
            COUNT(DISTINCT source_ip)::bigint as attackers,
            COUNT(*) FILTER (WHERE interaction_type = 'exploit_attempt')::bigint as exploits,
            COUNT(*) FILTER (WHERE is_threat = true)::bigint as critical
        FROM honeypot_interactions
        WHERE occurred_at >= $1
        "#
    )
    .bind(since)
    .fetch_one(&state.db)
    .await
    .unwrap_or(InteractionSummaryRow { total: 0, attackers: 0, exploits: 0, critical: 0 });

    Ok(Json(HoneypotSummary {
        total: pots.total,
        running: pots.running,
        stopped: pots.stopped,
        interactions_24h: acts.total,
        unique_attackers_24h: acts.attackers,
        exploit_attempts_24h: acts.exploits,
        critical_interactions_24h: acts.critical,
    }))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct InteractionResponse {
    id: Uuid,
    honeypot_id: Option<Uuid>,
    source_ip: Option<String>,
    source_port: Option<i32>,
    country_code: Option<String>,
    asn: Option<String>,
    interaction_type: Option<String>,
    username_tried: Option<String>,
    status: Option<String>,
    occurred_at: Option<DateTime<Utc>>,
}

async fn list_interactions(
    State(state): State<AppState>,
    Query(pagination): Query<Pagination>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<PaginatedResponse<InteractionResponse>>> {
    let interactions = sqlx::query_as::<_, InteractionResponse>(
        r#"
        SELECT id, honeypot_id, source_ip::text as source_ip, source_port,
               country_code, asn, interaction_type, username_tried,
               status, occurred_at
        FROM honeypot_interactions
        ORDER BY occurred_at DESC
        LIMIT $1 OFFSET $2
        "#
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(&state.db)
    .await?;

    let total: i64 = sqlx::query_scalar(r#"SELECT COUNT(*) FROM honeypot_interactions"#)
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    Ok(Json(PaginatedResponse {
        data: interactions,
        meta: PaginationMeta {
            page: pagination.page,
            per_page: pagination.per_page,
            total,
            total_pages: ((total as f64) / (pagination.limit() as f64)).ceil() as u32,
        },
    }))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct TopAttacker {
    source_ip: String,
    country_code: Option<String>,
    interaction_count: i64,
    exploit_attempts: i64,
    last_seen: Option<DateTime<Utc>>,
}

async fn top_attackers(
    State(state): State<AppState>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<Vec<TopAttacker>>> {
    let rows = sqlx::query_as::<_, TopAttacker>(
        r#"
        SELECT
            source_ip::text as source_ip,
            MAX(country_code) as country_code,
            COUNT(*)::bigint as interaction_count,
            COUNT(*) FILTER (WHERE interaction_type = 'exploit_attempt')::bigint as exploit_attempts,
            MAX(occurred_at) as last_seen
        FROM honeypot_interactions
        GROUP BY source_ip
        ORDER BY COUNT(*) DESC
        LIMIT 10
        "#
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(rows))
}

#[derive(Debug, Deserialize)]
struct CreateHoneypotRequest {
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(rename = "type")]
    honeypot_type: String,
    #[serde(default)]
    ip_address: Option<String>,
    #[serde(default = "default_port")]
    port: i32,
    #[serde(default)]
    emulated_banner: Option<String>,
}

fn default_port() -> i32 { 22 }

async fn create_honeypot(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(payload): Json<CreateHoneypotRequest>,
) -> AppResult<Json<HoneypotResponse>> {
    if !claims.role.has_permission(&UserRole::Admin) {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    if payload.name.trim().is_empty() {
        return Err(AppError::Validation("Name is required".to_string()));
    }

    let id = Uuid::now_v7();
    let honeypot = sqlx::query_as::<_, HoneypotResponse>(
        r#"
        INSERT INTO honeypots (id, name, description, honeypot_type, status, listen_ip, listen_port, 
                               emulated_banner, interaction_count, unique_attackers, created_at)
        VALUES ($1, $2, $3, $4, 'active', $5::inet, $6, $7, 0, 0, NOW())
        RETURNING id, name, description, honeypot_type, status, listen_ip::text, listen_port, 
                  server_id, emulated_banner, interaction_count, unique_attackers, last_interaction_at, created_at
        "#
    )
    .bind(id)
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(&payload.honeypot_type)
    .bind(&payload.ip_address)
    .bind(payload.port)
    .bind(&payload.emulated_banner)
    .fetch_one(&state.db)
    .await?;

    tracing::info!(honeypot_id = %id, name = %payload.name, "Honeypot created");
    Ok(Json(honeypot))
}

async fn get_honeypot(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<HoneypotResponse>> {
    let honeypot = sqlx::query_as::<_, HoneypotResponse>(
        r#"
        SELECT id, name, description, honeypot_type, status,
               listen_ip::text, listen_port, server_id, emulated_banner,
               interaction_count, unique_attackers, last_interaction_at, created_at
        FROM honeypots WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Honeypot not found".to_string()))?;

    Ok(Json(honeypot))
}

async fn delete_honeypot(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> AppResult<Json<serde_json::Value>> {
    if !claims.role.has_permission(&UserRole::Admin) {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    let result = sqlx::query("DELETE FROM honeypots WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Honeypot not found".to_string()));
    }

    tracing::info!(honeypot_id = %id, deleted_by = %claims.sub, "Honeypot deleted");
    Ok(Json(serde_json::json!({"status": "deleted", "id": id})))
}
