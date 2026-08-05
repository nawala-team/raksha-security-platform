//! Vulnerability scan endpoints.

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
        .route("/", get(list_scans))
        .route("/scans", get(list_scans))
        .route("/scans/:id", get(get_scan))
        .route("/summary", get(vuln_summary))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct ScanResponse {
    id: Uuid,
    agent_id: Uuid,
    scan_type: String,
    scanner: String,
    status: String,
    total_packages: Option<i32>,
    total_vulns: Option<i32>,
    critical_count: Option<i32>,
    high_count: Option<i32>,
    medium_count: Option<i32>,
    low_count: Option<i32>,
    info_count: Option<i32>,
    fixable_count: Option<i32>,
    duration_secs: Option<i32>,
    error_message: Option<String>,
    started_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
}

async fn list_scans(
    State(state): State<AppState>,
    Query(pagination): Query<Pagination>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<PaginatedResponse<ScanResponse>>> {
    let scans = sqlx::query_as::<_, ScanResponse>(
        r#"
        SELECT id, agent_id, scan_type, scanner, status, total_packages,
               total_vulns, critical_count, high_count, medium_count,
               low_count, info_count, fixable_count, duration_secs,
               error_message, started_at, completed_at
        FROM vulnerability_scans
        ORDER BY started_at DESC
        LIMIT $1 OFFSET $2
        "#
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(&state.db)
    .await?;

    let total: i64 = sqlx::query_scalar(r#"SELECT COUNT(*) FROM vulnerability_scans"#)
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

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

async fn get_scan(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<ScanResponse>> {
    let scan = sqlx::query_as::<_, ScanResponse>(
        r#"
        SELECT id, agent_id, scan_type, scanner, status, total_packages,
               total_vulns, critical_count, high_count, medium_count,
               low_count, info_count, fixable_count, duration_secs,
               error_message, started_at, completed_at
        FROM vulnerability_scans WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Scan not found".to_string()))?;

    Ok(Json(scan))
}

#[derive(Debug, Serialize)]
struct VulnSummary {
    total_scans: i64,
    completed_scans: i64,
    failed_scans: i64,
    running_scans: i64,
    critical_vulns: i64,
    high_vulns: i64,
    medium_vulns: i64,
    low_vulns: i64,
    fixable_vulns: i64,
    agents_scanned: i64,
}

#[derive(Debug, sqlx::FromRow)]
struct ScanCountsRow {
    total: i64,
    completed: i64,
    failed: i64,
    running: i64,
}

#[derive(Debug, sqlx::FromRow)]
struct LatestVulnsRow {
    critical: i64,
    high: i64,
    medium: i64,
    low: i64,
    fixable: i64,
    agents: i64,
}

async fn vuln_summary(
    State(state): State<AppState>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<VulnSummary>> {
    let counts: ScanCountsRow = sqlx::query_as(
        r#"
        SELECT
            COUNT(*)::bigint as total,
            COUNT(*) FILTER (WHERE status = 'completed')::bigint as completed,
            COUNT(*) FILTER (WHERE status = 'failed')::bigint as failed,
            COUNT(*) FILTER (WHERE status = 'running')::bigint as running
        FROM vulnerability_scans
        "#
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(ScanCountsRow { total: 0, completed: 0, failed: 0, running: 0 });

    let latest: LatestVulnsRow = sqlx::query_as(
        r#"
        WITH latest AS (
            SELECT DISTINCT ON (agent_id)
                agent_id, critical_count, high_count, medium_count,
                low_count, fixable_count
            FROM vulnerability_scans
            WHERE status = 'completed'
            ORDER BY agent_id, started_at DESC
        )
        SELECT
            COALESCE(SUM(critical_count), 0)::bigint as critical,
            COALESCE(SUM(high_count), 0)::bigint as high,
            COALESCE(SUM(medium_count), 0)::bigint as medium,
            COALESCE(SUM(low_count), 0)::bigint as low,
            COALESCE(SUM(fixable_count), 0)::bigint as fixable,
            COUNT(*)::bigint as agents
        FROM latest
        "#
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(LatestVulnsRow { critical: 0, high: 0, medium: 0, low: 0, fixable: 0, agents: 0 });

    Ok(Json(VulnSummary {
        total_scans: counts.total,
        completed_scans: counts.completed,
        failed_scans: counts.failed,
        running_scans: counts.running,
        critical_vulns: latest.critical,
        high_vulns: latest.high,
        medium_vulns: latest.medium,
        low_vulns: latest.low,
        fixable_vulns: latest.fixable,
        agents_scanned: latest.agents,
    }))
}
