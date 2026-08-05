//! Dark web monitoring endpoints: standing monitors and their findings.

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
use raksha_core::models::{PaginatedResponse, Pagination, PaginationMeta};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_monitors))
        .route("/monitors", get(list_monitors))
        .route("/findings", get(list_findings))
        .route("/findings/:id", get(get_finding))
        .route("/summary", get(darkweb_summary))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct MonitorResponse {
    id: Uuid,
    name: String,
    monitor_type: String,
    keyword: String,
    is_enabled: bool,
    severity_floor: String,
    finding_count: i64,
    new_finding_count: i64,
    last_scanned_at: Option<DateTime<Utc>>,
    next_scan_at: Option<DateTime<Utc>>,
    scan_interval_mins: i32,
    created_at: DateTime<Utc>,
}

async fn list_monitors(
    State(state): State<AppState>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<Vec<MonitorResponse>>> {
    let monitors = sqlx::query_as::<_, MonitorResponse>(
        r#"
        SELECT id, name, monitor_type, keyword, is_enabled, severity_floor,
               finding_count, new_finding_count, last_scanned_at, next_scan_at,
               scan_interval_mins, created_at
        FROM darkweb_monitors
        ORDER BY name
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(monitors))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct FindingResponse {
    id: Uuid,
    monitor_id: Option<Uuid>,
    title: String,
    description: Option<String>,
    finding_type: String,
    severity: String,
    status: String,
    source_name: Option<String>,
    source_type: Option<String>,
    excerpt_redacted: Option<String>,
    record_count: Option<i32>,
    confidence: Option<i16>,
    alert_id: Option<Uuid>,
    incident_id: Option<Uuid>,
    discovered_at: DateTime<Utc>,
}

async fn list_findings(
    State(state): State<AppState>,
    Query(pagination): Query<Pagination>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<PaginatedResponse<FindingResponse>>> {
    let findings = sqlx::query_as::<_, FindingResponse>(
        r#"
        SELECT id, monitor_id, title, description, finding_type, severity,
               status, source_name, source_type, excerpt_redacted,
               record_count, confidence, alert_id, incident_id, discovered_at
        FROM darkweb_findings
        ORDER BY discovered_at DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(&state.db)
    .await?;

    let total: i64 = sqlx::query_scalar(r#"SELECT COUNT(*) FROM darkweb_findings"#)
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    Ok(Json(PaginatedResponse {
        data: findings,
        meta: PaginationMeta {
            page: pagination.page,
            per_page: pagination.per_page,
            total,
            total_pages: ((total as f64) / (pagination.limit() as f64)).ceil() as u32,
        },
    }))
}

async fn get_finding(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<FindingResponse>> {
    let finding = sqlx::query_as::<_, FindingResponse>(
        r#"
        SELECT id, monitor_id, title, description, finding_type, severity,
               status, source_name, source_type, excerpt_redacted,
               record_count, confidence, alert_id, incident_id, discovered_at
        FROM darkweb_findings WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Finding not found".to_string()))?;

    Ok(Json(finding))
}

#[derive(Debug, Serialize)]
struct DarkwebSummary {
    active_monitors: i64,
    total_findings: i64,
    new_findings: i64,
    critical_findings: i64,
    credential_leaks: i64,
    exposed_records: i64,
}

#[derive(Debug, sqlx::FromRow)]
struct FindingSummaryRow {
    total: i64,
    new: i64,
    critical: i64,
    creds: i64,
}

async fn darkweb_summary(
    State(state): State<AppState>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<DarkwebSummary>> {
    let active_monitors: i64 =
        sqlx::query_scalar(r#"SELECT COUNT(*) FROM darkweb_monitors WHERE is_enabled = true"#)
            .fetch_one(&state.db)
            .await
            .unwrap_or(0);

    let row: FindingSummaryRow = sqlx::query_as(
        r#"
        SELECT
            COUNT(*)::bigint as total,
            COUNT(*) FILTER (WHERE status = 'new')::bigint as new,
            COUNT(*) FILTER (WHERE severity = 'critical')::bigint as critical,
            COUNT(*) FILTER (WHERE finding_type = 'credential_leak')::bigint as creds
        FROM darkweb_findings
        "#,
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(FindingSummaryRow {
        total: 0,
        new: 0,
        critical: 0,
        creds: 0,
    });

    Ok(Json(DarkwebSummary {
        active_monitors,
        total_findings: row.total,
        new_findings: row.new,
        critical_findings: row.critical,
        credential_leaks: row.creds,
        exposed_records: 0, // Not tracked in current schema
    }))
}
