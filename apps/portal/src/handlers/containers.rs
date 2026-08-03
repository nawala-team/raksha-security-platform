//! Container inventory and image scan endpoints.

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
        .route("/", get(list_containers))
        .route("/summary", get(container_summary))
        .route("/scans", get(list_scans))
        .route("/:id", get(get_container))
}

#[derive(Debug, Serialize)]
struct ContainerResponse {
    id: Uuid,
    agent_id: Option<Uuid>,
    server_id: Option<Uuid>,
    container_id: String,
    name: String,
    image: String,
    image_tag: Option<String>,
    runtime: String,
    orchestrator: Option<String>,
    namespace: Option<String>,
    pod_name: Option<String>,
    status: String,
    privileged: bool,
    root_user: bool,
    host_network: bool,
    cpu_usage_pct: Option<f64>,
    memory_mb: Option<i32>,
    critical_vulns: i32,
    high_vulns: i32,
    medium_vulns: i32,
    low_vulns: i32,
    compliance_score: Option<f64>,
    started_at: Option<DateTime<Utc>>,
    last_scanned_at: Option<DateTime<Utc>>,
}

async fn list_containers(
    State(state): State<AppState>,
    Query(pagination): Query<Pagination>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<PaginatedResponse<ContainerResponse>>> {
    let containers = sqlx::query_as!(
        ContainerResponse,
        r#"
        SELECT id, agent_id, server_id, container_id, name, image, image_tag,
               runtime, orchestrator, namespace, pod_name, status,
               privileged, root_user, host_network, cpu_usage_pct, memory_mb,
               critical_vulns, high_vulns, medium_vulns, low_vulns,
               compliance_score, started_at, last_scanned_at
        FROM containers
        ORDER BY critical_vulns DESC, high_vulns DESC, name
        LIMIT $1 OFFSET $2
        "#,
        pagination.limit(),
        pagination.offset(),
    )
    .fetch_all(&state.db)
    .await?;

    let total = sqlx::query_scalar!(r#"SELECT COUNT(*) as "count!" FROM containers"#)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(PaginatedResponse {
        data: containers,
        meta: PaginationMeta {
            page: pagination.page,
            per_page: pagination.per_page,
            total,
            total_pages: ((total as f64) / (pagination.limit() as f64)).ceil() as u32,
        },
    }))
}

async fn get_container(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<ContainerResponse>> {
    let container = sqlx::query_as!(
        ContainerResponse,
        r#"
        SELECT id, agent_id, server_id, container_id, name, image, image_tag,
               runtime, orchestrator, namespace, pod_name, status,
               privileged, root_user, host_network, cpu_usage_pct, memory_mb,
               critical_vulns, high_vulns, medium_vulns, low_vulns,
               compliance_score, started_at, last_scanned_at
        FROM containers WHERE id = $1
        "#,
        id,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Container not found".to_string()))?;

    Ok(Json(container))
}

#[derive(Debug, Serialize)]
struct ContainerSummary {
    total: i64,
    running: i64,
    stopped: i64,
    privileged: i64,
    running_as_root: i64,
    host_network: i64,
    critical_vulns: i64,
    high_vulns: i64,
}

async fn container_summary(
    State(state): State<AppState>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<ContainerSummary>> {
    let row = sqlx::query!(
        r#"
        SELECT
            COUNT(*) as "total!",
            COUNT(*) FILTER (WHERE status = 'running') as "running!",
            COUNT(*) FILTER (WHERE status = 'stopped') as "stopped!",
            COUNT(*) FILTER (WHERE privileged) as "privileged!",
            COUNT(*) FILTER (WHERE root_user) as "as_root!",
            COUNT(*) FILTER (WHERE host_network) as "host_net!",
            COALESCE(SUM(critical_vulns), 0) as "critical!",
            COALESCE(SUM(high_vulns), 0) as "high!"
        FROM containers
        "#
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(ContainerSummary {
        total: row.total,
        running: row.running,
        stopped: row.stopped,
        privileged: row.privileged,
        running_as_root: row.as_root,
        host_network: row.host_net,
        critical_vulns: row.critical,
        high_vulns: row.high,
    }))
}

#[derive(Debug, Serialize)]
struct ImageScanResponse {
    id: Uuid,
    image: String,
    image_digest: Option<String>,
    scanner: String,
    status: String,
    critical_count: i32,
    high_count: i32,
    medium_count: i32,
    low_count: i32,
    fixable_count: i32,
    secrets_found: i32,
    misconfigs: i32,
    duration_secs: Option<i32>,
    error_message: Option<String>,
    started_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
}

async fn list_scans(
    State(state): State<AppState>,
    Query(pagination): Query<Pagination>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<PaginatedResponse<ImageScanResponse>>> {
    let scans = sqlx::query_as!(
        ImageScanResponse,
        r#"
        SELECT id, image, image_digest, scanner, status,
               critical_count, high_count, medium_count, low_count,
               fixable_count, secrets_found, misconfigs,
               duration_secs, error_message, started_at, completed_at
        FROM container_image_scans
        ORDER BY started_at DESC
        LIMIT $1 OFFSET $2
        "#,
        pagination.limit(),
        pagination.offset(),
    )
    .fetch_all(&state.db)
    .await?;

    let total = sqlx::query_scalar!(r#"SELECT COUNT(*) as "count!" FROM container_image_scans"#)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(PaginatedResponse {
        data: scans,
        meta: PaginationMeta {
            page: pagination.page,
            per_page: pagination.per_page,
            total,
            total_pages: ((total as f64) / (pagination.limit() as f64)).ceil() as u32,
        },
    }))
}

