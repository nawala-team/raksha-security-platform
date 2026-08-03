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
        .route("/scans", get(list_scans))
        .route("/scans/:id", get(get_scan))
        .route("/summary", get(vuln_summary))
}

#[derive(Debug, Serialize)]
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
    let scans = sqlx::query_as!(
        ScanResponse,
        r#"
        SELECT id, agent_id, scan_type, scanner, status, total_packages,
               total_vulns, critical_count, high_count, medium_count,
               low_count, info_count, fixable_count, duration_secs,
               error_message, started_at, completed_at
        FROM vulnerability_scans
        ORDER BY started_at DESC
        LIMIT $1 OFFSET $2
        "#,
        pagination.limit(),
        pagination.offset(),
    )
    .fetch_all(&state.db)
    .await?;

    let total = sqlx::query_scalar!(r#"SELECT COUNT(*) as "count!" FROM vulnerability_scans"#)
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

async fn get_scan(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<ScanResponse>> {
    let scan = sqlx::query_as!(
        ScanResponse,
        r#"
        SELECT id, agent_id, scan_type, scanner, status, total_packages,
               total_vulns, critical_count, high_count, medium_count,
               low_count, info_count, fixable_count, duration_secs,
               error_message, started_at, completed_at
        FROM vulnerability_scans WHERE id = $1
        "#,
        id,
    )
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
    /// Severity totals from the most recent completed scan per agent, so hosts
    /// scanned repeatedly are not counted many times over.
    critical_vulns: i64,
    high_vulns: i64,
    medium_vulns: i64,
    low_vulns: i64,
    fixable_vulns: i64,
    agents_scanned: i64,
}

async fn vuln_summary(
    State(state): State<AppState>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<VulnSummary>> {
    let counts = sqlx::query!(
        r#"
        SELECT
            COUNT(*) as "total!",
            COUNT(*) FILTER (WHERE status = 'completed') as "completed!",
            COUNT(*) FILTER (WHERE status = 'failed') as "failed!",
            COUNT(*) FILTER (WHERE status = 'running') as "running!"
        FROM vulnerability_scans
        "#
    )
    .fetch_one(&state.db)
    .await?;

    let latest = sqlx::query!(
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
            COALESCE(SUM(critical_count), 0)::bigint as "critical!",
            COALESCE(SUM(high_count), 0)::bigint as "high!",
            COALESCE(SUM(medium_count), 0)::bigint as "medium!",
            COALESCE(SUM(low_count), 0)::bigint as "low!",
            COALESCE(SUM(fixable_count), 0)::bigint as "fixable!",
            COUNT(*) as "agents!"
        FROM latest
        "#
    )
    .fetch_one(&state.db)
    .await?;

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
