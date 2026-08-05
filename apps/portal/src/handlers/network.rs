//! Network monitoring endpoints: traffic events and firewall rules.

use axum::{
    extract::{Path, Query, State},
    routing::{delete, get},
    Json, Router,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use raksha_auth::Claims;
use raksha_core::error::{AppError, AppResult};
use raksha_core::models::{new_id, Pagination, PaginatedResponse, PaginationMeta, UserRole};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_events))
        .route("/events", get(list_events))
        .route("/rules", get(list_rules).post(create_rule))
        .route("/rules/:id", delete(remove_rule))
        .route("/summary", get(network_summary))
        .route("/top-talkers", get(top_talkers))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct NetworkEventResponse {
    id: Uuid,
    agent_id: Option<Uuid>,
    protocol: Option<String>,
    source_ip: Option<String>,
    source_port: Option<i32>,
    dest_ip: Option<String>,
    dest_port: Option<i32>,
    bytes_sent: Option<i64>,
    bytes_received: Option<i64>,
    timestamp: Option<DateTime<Utc>>,
}

async fn list_events(
    State(state): State<AppState>,
    Query(pagination): Query<Pagination>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<PaginatedResponse<NetworkEventResponse>>> {
    let events = sqlx::query_as::<_, NetworkEventResponse>(
        r#"
        SELECT id, agent_id, protocol,
               source_ip::text, source_port, dest_ip::text, dest_port,
               bytes_sent, bytes_received, timestamp
        FROM network_events
        ORDER BY timestamp DESC
        LIMIT $1 OFFSET $2
        "#
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(&state.db)
    .await?;

    let total: i64 = sqlx::query_scalar(r#"SELECT COUNT(*) FROM network_events"#)
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    Ok(Json(PaginatedResponse {
        data: events,
        meta: PaginationMeta {
            page: pagination.page,
            per_page: pagination.per_page,
            total,
            total_pages: ((total as f64) / (pagination.limit() as f64)).ceil() as u32,
        },
    }))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct NetworkRuleResponse {
    id: Uuid,
    name: String,
    description: Option<String>,
    is_enabled: bool,
    priority: i32,
    direction: String,
    action: String,
    protocol: Option<String>,
    source_cidr: Option<String>,
    dest_cidr: Option<String>,
    port_range: Option<String>,
    hit_count: i64,
    last_hit_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

async fn list_rules(
    State(state): State<AppState>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<Vec<NetworkRuleResponse>>> {
    let rules = sqlx::query_as::<_, NetworkRuleResponse>(
        r#"
        SELECT id, name, description, is_enabled, priority, direction,
               action, protocol, source_cidr, dest_cidr, port_range,
               hit_count, last_hit_at, created_at
        FROM network_rules
        ORDER BY priority, name
        "#
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(rules))
}

#[derive(Debug, Serialize)]
struct NetworkSummary {
    total_events_24h: i64,
    blocked_24h: i64,
    allowed_24h: i64,
    threats_24h: i64,
    bytes_in_24h: i64,
    bytes_out_24h: i64,
    active_rules: i64,
}

#[derive(Debug, sqlx::FromRow)]
struct EventSummaryRow {
    total: i64,
    blocked: i64,
    allowed: i64,
    threats: i64,
    bytes_in: i64,
    bytes_out: i64,
}

async fn network_summary(
    State(state): State<AppState>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<NetworkSummary>> {
    let since = Utc::now() - Duration::hours(24);

    let events: EventSummaryRow = sqlx::query_as(
        r#"
        SELECT
            COUNT(*)::bigint as total,
            COUNT(*) FILTER (WHERE action = 'block')::bigint as blocked,
            COUNT(*) FILTER (WHERE action = 'allow')::bigint as allowed,
            COUNT(*) FILTER (WHERE is_threat)::bigint as threats,
            COALESCE(SUM(bytes_received), 0)::bigint as bytes_in,
            COALESCE(SUM(bytes_sent), 0)::bigint as bytes_out
        FROM network_events
        WHERE occurred_at >= $1
        "#
    )
    .bind(since)
    .fetch_one(&state.db)
    .await
    .unwrap_or(EventSummaryRow {
        total: 0, blocked: 0, allowed: 0, threats: 0, bytes_in: 0, bytes_out: 0,
    });

    let active_rules: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(*) FROM network_rules WHERE is_enabled = true"#
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    Ok(Json(NetworkSummary {
        total_events_24h: events.total,
        blocked_24h: events.blocked,
        allowed_24h: events.allowed,
        threats_24h: events.threats,
        bytes_in_24h: events.bytes_in,
        bytes_out_24h: events.bytes_out,
        active_rules,
    }))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct TopTalker {
    source_ip: String,
    event_count: i64,
    total_bytes: i64,
    threat_count: i64,
}

async fn top_talkers(
    State(state): State<AppState>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<Vec<TopTalker>>> {
    let since = Utc::now() - Duration::hours(24);
    let rows = sqlx::query_as::<_, TopTalker>(
        r#"
        SELECT
            source_ip::text as source_ip,
            COUNT(*)::bigint as event_count,
            COALESCE(SUM(COALESCE(bytes_sent, 0) + COALESCE(bytes_received, 0)), 0)::bigint as total_bytes,
            COUNT(*) FILTER (WHERE is_threat)::bigint as threat_count
        FROM network_events
        WHERE occurred_at >= $1 AND source_ip IS NOT NULL
        GROUP BY source_ip
        ORDER BY COUNT(*) DESC
        LIMIT 10
        "#
    )
    .bind(since)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(rows))
}

#[derive(Debug, Deserialize)]
struct CreateRuleRequest {
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default = "default_direction")]
    direction: String,
    #[serde(default = "default_action")]
    action: String,
    #[serde(default)]
    protocol: Option<String>,
    #[serde(default)]
    source_cidr: Option<String>,
    #[serde(default)]
    dest_cidr: Option<String>,
    #[serde(default)]
    port_range: Option<String>,
    #[serde(default = "default_priority")]
    priority: i32,
}

fn default_direction() -> String { "inbound".to_string() }
fn default_action() -> String { "block".to_string() }
fn default_priority() -> i32 { 100 }

async fn create_rule(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(payload): Json<CreateRuleRequest>,
) -> AppResult<Json<NetworkRuleResponse>> {
    if !claims.role.has_permission(&UserRole::Operator) {
        return Err(AppError::Forbidden(
            "Operator access required to create network rules".to_string(),
        ));
    }
    if payload.name.trim().is_empty() {
        return Err(AppError::Validation("Rule name is required".to_string()));
    }

    let id = new_id();
    let tenant_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();

    let rule = sqlx::query_as::<_, NetworkRuleResponse>(
        r#"
        INSERT INTO network_rules
            (id, tenant_id, name, description, is_enabled, priority, direction, action,
             protocol, source_cidr, dest_cidr, port_range, hit_count, created_at, updated_at)
        VALUES ($1, $2, $3, $4, true, $5, $6, $7, $8, $9, $10, $11, 0, NOW(), NOW())
        RETURNING id, name, description, is_enabled, priority, direction, action,
                  protocol, source_cidr, dest_cidr, port_range, hit_count, last_hit_at, created_at
        "#
    )
    .bind(id)
    .bind(tenant_id)
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(payload.priority)
    .bind(&payload.direction)
    .bind(&payload.action)
    .bind(&payload.protocol)
    .bind(&payload.source_cidr)
    .bind(&payload.dest_cidr)
    .bind(&payload.port_range)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(rule))
}

async fn remove_rule(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> AppResult<Json<serde_json::Value>> {
    if !claims.role.has_permission(&UserRole::Admin) {
        return Err(AppError::Forbidden(
            "Admin access required to delete network rules".to_string(),
        ));
    }
    let result = sqlx::query("DELETE FROM network_rules WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Network rule not found".to_string()));
    }
    Ok(Json(serde_json::json!({ "deleted": true, "id": id })))
}
