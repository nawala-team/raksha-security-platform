//! Server / infrastructure inventory endpoints.

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
use raksha_core::models::{Pagination, PaginatedResponse, PaginationMeta};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_servers))
        .route("/summary", get(server_summary))
        .route("/:id", get(get_server))
}

#[derive(Debug, Serialize)]
struct ServerResponse {
    id: Uuid,
    agent_id: Option<Uuid>,
    hostname: String,
    display_name: Option<String>,
    environment: String,
    role: Option<String>,
    provider: Option<String>,
    region: Option<String>,
    ip_address: Option<String>,
    os_family: Option<String>,
    os_version: Option<String>,
    cpu_cores: Option<i32>,
    memory_mb: Option<i32>,
    disk_gb: Option<i32>,
    status: String,
    cpu_usage_pct: Option<f64>,
    memory_usage_pct: Option<f64>,
    disk_usage_pct: Option<f64>,
    uptime_secs: Option<i64>,
    last_seen_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

async fn list_servers(
    State(state): State<AppState>,
    Query(pagination): Query<Pagination>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<PaginatedResponse<ServerResponse>>> {
    let servers = sqlx::query_as!(
        ServerResponse,
        r#"
        SELECT id, agent_id, hostname, display_name, environment, role,
               provider, region, ip_address::text, os_family, os_version,
               cpu_cores, memory_mb, disk_gb, status,
               cpu_usage_pct, memory_usage_pct, disk_usage_pct,
               uptime_secs, last_seen_at, created_at
        FROM servers
        ORDER BY hostname
        LIMIT $1 OFFSET $2
        "#,
        pagination.limit(),
        pagination.offset(),
    )
    .fetch_all(&state.db)
    .await?;

    let total = sqlx::query_scalar!(r#"SELECT COUNT(*) as "count!" FROM servers"#)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(PaginatedResponse {
        data: servers,
        meta: PaginationMeta {
            page: pagination.page,
            per_page: pagination.per_page,
            total,
            total_pages: ((total as f64) / (pagination.limit() as f64)).ceil() as u32,
        },
    }))
}

async fn get_server(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<ServerResponse>> {
    let server = sqlx::query_as!(
        ServerResponse,
        r#"
        SELECT id, agent_id, hostname, display_name, environment, role,
               provider, region, ip_address::text, os_family, os_version,
               cpu_cores, memory_mb, disk_gb, status,
               cpu_usage_pct, memory_usage_pct, disk_usage_pct,
               uptime_secs, last_seen_at, created_at
        FROM servers WHERE id = $1
        "#,
        id,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Server not found".to_string()))?;

    Ok(Json(server))
}

#[derive(Debug, Serialize)]
struct ServerSummary {
    total: i64,
    online: i64,
    offline: i64,
    degraded: i64,
    maintenance: i64,
    avg_cpu_usage: Option<f64>,
    avg_memory_usage: Option<f64>,
}

async fn server_summary(
    State(state): State<AppState>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<ServerSummary>> {
    let row = sqlx::query!(
        r#"
        SELECT
            COUNT(*) as "total!",
            COUNT(*) FILTER (WHERE status = 'online') as "online!",
            COUNT(*) FILTER (WHERE status = 'offline') as "offline!",
            COUNT(*) FILTER (WHERE status = 'degraded') as "degraded!",
            COUNT(*) FILTER (WHERE status = 'maintenance') as "maintenance!",
            AVG(cpu_usage_pct) as "avg_cpu",
            AVG(memory_usage_pct) as "avg_mem"
        FROM servers
        "#
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(ServerSummary {
        total: row.total,
        online: row.online,
        offline: row.offline,
        degraded: row.degraded,
        maintenance: row.maintenance,
        avg_cpu_usage: row.avg_cpu,
        avg_memory_usage: row.avg_mem,
    }))
}
