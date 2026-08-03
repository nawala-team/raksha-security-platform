use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use raksha_auth::Claims;
use raksha_core::error::{AppError, AppResult};
use raksha_core::models::{AgentOs, AgentStatus, Pagination, PaginatedResponse, PaginationMeta};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_agents))
        .route("/:id", get(get_agent))
        .route("/:id/metrics", get(get_agent_metrics))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct AgentResponse {
    id: Uuid,
    name: String,
    hostname: String,
    os: AgentOs,
    version: String,
    status: AgentStatus,
    last_seen: Option<DateTime<Utc>>,
    enrolled_at: DateTime<Utc>,
    ip_address: Option<String>,
    network_zone: Option<String>,
}

async fn list_agents(
    State(state): State<AppState>,
    Query(pagination): Query<Pagination>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<PaginatedResponse<AgentResponse>>> {
    let agents = sqlx::query_as!(
        AgentResponse,
        r#"
        SELECT id, name, hostname, os as "os: AgentOs", version,
               status as "status: AgentStatus", last_seen, enrolled_at,
               ip_address::text, network_zone
        FROM agents
        ORDER BY last_seen DESC NULLS LAST
        LIMIT $1 OFFSET $2
        "#,
        pagination.limit(),
        pagination.offset(),
    )
    .fetch_all(&state.db)
    .await?;

    let total = sqlx::query_scalar!(r#"SELECT COUNT(*) as "count!" FROM agents"#)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(PaginatedResponse {
        data: agents,
        meta: PaginationMeta {
            page: pagination.page,
            per_page: pagination.per_page,
            total,
            total_pages: ((total as f64) / (pagination.limit() as f64)).ceil() as u32,
        },
    }))
}

async fn get_agent(
    State(state): State<AppState>,
    Path(agent_id): Path<Uuid>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<AgentResponse>> {
    let agent = sqlx::query_as!(
        AgentResponse,
        r#"
        SELECT id, name, hostname, os as "os: AgentOs", version,
               status as "status: AgentStatus", last_seen, enrolled_at,
               ip_address::text, network_zone
        FROM agents WHERE id = $1
        "#,
        agent_id,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Agent not found".to_string()))?;

    Ok(Json(agent))
}

#[derive(Debug, Serialize)]
struct MetricsResponse {
    agent_id: Uuid,
    metrics: Vec<MetricPoint>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct MetricPoint {
    metric_name: String,
    value: f64,
    timestamp: DateTime<Utc>,
}

async fn get_agent_metrics(
    State(state): State<AppState>,
    Path(agent_id): Path<Uuid>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<MetricsResponse>> {
    let metrics = sqlx::query_as!(
        MetricPoint,
        r#"
        SELECT metric_name, value, timestamp
        FROM agent_metrics
        WHERE agent_id = $1
        ORDER BY timestamp DESC
        LIMIT 100
        "#,
        agent_id,
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(MetricsResponse { agent_id, metrics }))
}
