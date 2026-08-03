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
use raksha_core::models::{Pagination, PaginatedResponse, PaginationMeta};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/monitors", get(list_monitors))
        .route("/findings", get(list_findings))
        .route("/findings/:id", get(get_finding))
        .route("/summary", get(darkweb_summary))
}

#[derive(Debug, Serialize)]
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
    let monitors = sqlx::query_as!(
        MonitorResponse,
        r#"
        SELECT id, name, monitor_type, keyword, is_enabled, severity_floor,
               finding_count, new_finding_count, last_scanned_at, next_scan_at,
               scan_interval_mins, created_at
        FROM darkweb_monitors
        ORDER BY name
        "#
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(monitors))
}

/// Finding as exposed to the UI. The stored excerpt is already redacted at
/// ingest time, and no raw source URL is returned.
#[derive(Debug, Serialize)]
struct FindingResponse {
    id: Uuid,
    monitor_id: Uuid,
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
    let findings = sqlx::query_as!(
        FindingResponse,
        r#"
        SELECT id, monitor_id, title, description, finding_type, severity,
               status, source_name, source_type, excerpt_redacted,
               record_count, confidence, alert_id, incident_id, discovered_at
        FROM darkweb_findings
        ORDER BY discovered_at DESC
        LIMIT $1 OFFSET $2
        "#,
        pagination.limit(),
        pagination.offset(),
    )
    .fetch_all(&state.db)
    .await?;

    let total = sqlx::query_scalar!(r#"SELECT COUNT(*) as "count!" FROM darkweb_findings"#)
        .fetch_one(&state.db)
        .await?;

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
    let finding = sqlx::query_as!(
        FindingResponse,
        r#"
        SELECT id, monitor_id, title, description, finding_type, severity,
               status, source_name, source_type, excerpt_redacted,
               record_count, confidence, alert_id, incident_id, discovered_at
        FROM darkweb_findings WHERE id = $1
        "#,
        id,
    )
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

async fn darkweb_summary(
    State(state): State<AppState>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<DarkwebSummary>> {
    let active_monitors = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM darkweb_monitors WHERE is_enabled = true"#
    )
    .fetch_one(&state.db)
    .await?;

    let row = sqlx::query!(
        r#"
        SELECT
            COUNT(*) as "total!",
            COUNT(*) FILTER (WHERE status = 'new') as "new!",
            COUNT(*) FILTER (WHERE severity = 'critical') as "critical!",
            COUNT(*) FILTER (WHERE finding_type = 'credential_leak') as "creds!",
            COALESCE(SUM(record_count), 0)::bigint as "records!"
        FROM darkweb_findings
        "#
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(DarkwebSummary {
        active_monitors,
        total_findings: row.total,
        new_findings: row.new,
        critical_findings: row.critical,
        credential_leaks: row.creds,
        exposed_records: row.records,
    }))
}
