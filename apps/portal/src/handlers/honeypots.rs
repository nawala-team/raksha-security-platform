//! Honeypot deployment and captured attacker interaction endpoints.

use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Duration, Utc};
use serde::Serialize;
use uuid::Uuid;

use raksha_auth::Claims;
use raksha_core::error::AppResult;
use raksha_core::models::{Pagination, PaginatedResponse, PaginationMeta};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_honeypots))
        .route("/summary", get(honeypot_summary))
        .route("/interactions", get(list_interactions))
        .route("/top-attackers", get(top_attackers))
}

#[derive(Debug, Serialize)]
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
    let honeypots = sqlx::query_as!(
        HoneypotResponse,
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

async fn honeypot_summary(
    State(state): State<AppState>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<HoneypotSummary>> {
    let since = Utc::now() - Duration::hours(24);

    let pots = sqlx::query!(
        r#"
        SELECT
            COUNT(*) as "total!",
            COUNT(*) FILTER (WHERE status = 'running') as "running!",
            COUNT(*) FILTER (WHERE status = 'stopped') as "stopped!"
        FROM honeypots
        "#
    )
    .fetch_one(&state.db)
    .await?;

    let acts = sqlx::query!(
        r#"
        SELECT
            COUNT(*) as "total!",
            COUNT(DISTINCT source_ip) as "attackers!",
            COUNT(*) FILTER (WHERE interaction_type = 'exploit_attempt') as "exploits!",
            COUNT(*) FILTER (WHERE severity = 'critical') as "critical!"
        FROM honeypot_interactions
        WHERE occurred_at >= $1
        "#,
        since
    )
    .fetch_one(&state.db)
    .await?;

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

/// Captured interaction. `password_tried` is intentionally omitted from the
/// response: it is attacker-supplied data against a decoy and there is no
/// operational reason to surface it through the API.
#[derive(Debug, Serialize)]
struct InteractionResponse {
    id: Uuid,
    honeypot_id: Uuid,
    source_ip: String,
    source_port: Option<i32>,
    country_code: Option<String>,
    asn: Option<String>,
    interaction_type: String,
    username_tried: Option<String>,
    severity: String,
    occurred_at: DateTime<Utc>,
}

async fn list_interactions(
    State(state): State<AppState>,
    Query(pagination): Query<Pagination>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<PaginatedResponse<InteractionResponse>>> {
    let interactions = sqlx::query_as!(
        InteractionResponse,
        r#"
        SELECT id, honeypot_id, source_ip::text as "source_ip!", source_port,
               country_code, asn, interaction_type, username_tried,
               severity, occurred_at
        FROM honeypot_interactions
        ORDER BY occurred_at DESC
        LIMIT $1 OFFSET $2
        "#,
        pagination.limit(),
        pagination.offset(),
    )
    .fetch_all(&state.db)
    .await?;

    let total = sqlx::query_scalar!(r#"SELECT COUNT(*) as "count!" FROM honeypot_interactions"#)
        .fetch_one(&state.db)
        .await?;

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

#[derive(Debug, Serialize)]
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
    let rows = sqlx::query_as!(
        TopAttacker,
        r#"
        SELECT
            source_ip::text as "source_ip!",
            MAX(country_code) as "country_code",
            COUNT(*) as "interaction_count!",
            COUNT(*) FILTER (WHERE interaction_type = 'exploit_attempt') as "exploit_attempts!",
            MAX(occurred_at) as "last_seen"
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
