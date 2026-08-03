//! File Integrity Monitoring (FIM) endpoints: change events and baselines.

use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Duration, Utc};
use serde::Serialize;
use uuid::Uuid;

use raksha_auth::Claims;
use raksha_core::error::{AppError, AppResult};
use raksha_core::models::{Pagination, PaginatedResponse, PaginationMeta};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/events", get(list_events))
        .route("/events/:id", get(get_event))
        .route("/summary", get(fim_summary))
        .route("/top-paths", get(top_changed_paths))
}

/// FIM event as exposed to the UI. `diff_content` is deliberately excluded from
/// the list response: file diffs can contain secrets, so they stay behind the
/// single-event detail endpoint.
#[derive(Debug, Serialize)]
struct FimEventResponse {
    id: Uuid,
    agent_id: Uuid,
    hostname: String,
    event_type: String,
    severity: String,
    file_path: String,
    file_name: String,
    directory: String,
    file_type: Option<String>,
    file_size: Option<i64>,
    hash_algorithm: String,
    hash_before: Option<String>,
    hash_after: Option<String>,
    content_changed: Option<bool>,
    diff_available: bool,
    permissions_before: Option<String>,
    permissions_after: Option<String>,
    owner_before: Option<String>,
    owner_after: Option<String>,
    process_name: Option<String>,
    process_user: Option<String>,
    rule_name: Option<String>,
    is_baseline_drift: bool,
    is_whitelisted: bool,
    alert_id: Option<Uuid>,
    detected_at: DateTime<Utc>,
}

async fn list_events(
    State(state): State<AppState>,
    Query(pagination): Query<Pagination>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<PaginatedResponse<FimEventResponse>>> {
    let events = sqlx::query_as!(
        FimEventResponse,
        r#"
        SELECT id, agent_id, hostname, event_type, severity, file_path,
               file_name, directory, file_type, file_size, hash_algorithm,
               hash_before, hash_after, content_changed, diff_available,
               permissions_before, permissions_after, owner_before, owner_after,
               process_name, process_user, rule_name, is_baseline_drift,
               is_whitelisted, alert_id, detected_at
        FROM fim_events
        ORDER BY detected_at DESC
        LIMIT $1 OFFSET $2
        "#,
        pagination.limit(),
        pagination.offset(),
    )
    .fetch_all(&state.db)
    .await?;

    let total = sqlx::query_scalar!(r#"SELECT COUNT(*) as "count!" FROM fim_events"#)
        .fetch_one(&state.db)
        .await?;

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

async fn get_event(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<FimEventResponse>> {
    let event = sqlx::query_as!(
        FimEventResponse,
        r#"
        SELECT id, agent_id, hostname, event_type, severity, file_path,
               file_name, directory, file_type, file_size, hash_algorithm,
               hash_before, hash_after, content_changed, diff_available,
               permissions_before, permissions_after, owner_before, owner_after,
               process_name, process_user, rule_name, is_baseline_drift,
               is_whitelisted, alert_id, detected_at
        FROM fim_events WHERE id = $1
        "#,
        id,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("FIM event not found".to_string()))?;

    Ok(Json(event))
}

#[derive(Debug, Serialize)]
struct FimSummary {
    events_24h: i64,
    critical_24h: i64,
    created_24h: i64,
    modified_24h: i64,
    deleted_24h: i64,
    permission_changes_24h: i64,
    baseline_drift_24h: i64,
    monitored_hosts: i64,
    baselines: i64,
}

async fn fim_summary(
    State(state): State<AppState>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<FimSummary>> {
    let since = Utc::now() - Duration::hours(24);

    let row = sqlx::query!(
        r#"
        SELECT
            COUNT(*) as "total!",
            COUNT(*) FILTER (WHERE severity = 'critical') as "critical!",
            COUNT(*) FILTER (WHERE event_type = 'created') as "created!",
            COUNT(*) FILTER (WHERE event_type = 'modified') as "modified!",
            COUNT(*) FILTER (WHERE event_type = 'deleted') as "deleted!",
            COUNT(*) FILTER (
                WHERE permissions_before IS DISTINCT FROM permissions_after
                  AND permissions_after IS NOT NULL
            ) as "perms!",
            COUNT(*) FILTER (WHERE is_baseline_drift) as "drift!",
            COUNT(DISTINCT agent_id) as "hosts!"
        FROM fim_events
        WHERE detected_at >= $1
        "#,
        since,
    )
    .fetch_one(&state.db)
    .await?;

    let baselines = sqlx::query_scalar!(r#"SELECT COUNT(*) as "count!" FROM fim_baselines"#)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(FimSummary {
        events_24h: row.total,
        critical_24h: row.critical,
        created_24h: row.created,
        modified_24h: row.modified,
        deleted_24h: row.deleted,
        permission_changes_24h: row.perms,
        baseline_drift_24h: row.drift,
        monitored_hosts: row.hosts,
        baselines,
    }))
}

#[derive(Debug, Serialize)]
struct ChangedPath {
    file_path: String,
    change_count: i64,
    last_change: Option<DateTime<Utc>>,
    max_severity: Option<String>,
}

/// Most frequently changed paths over the last 7 days: churn here usually means
/// either a noisy rule or something genuinely worth investigating.
async fn top_changed_paths(
    State(state): State<AppState>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<Vec<ChangedPath>>> {
    let since = Utc::now() - Duration::days(7);

    let rows = sqlx::query_as!(
        ChangedPath,
        r#"
        SELECT
            file_path as "file_path!",
            COUNT(*) as "change_count!",
            MAX(detected_at) as "last_change",
            MAX(severity) as "max_severity"
        FROM fim_events
        WHERE detected_at >= $1
        GROUP BY file_path
        ORDER BY COUNT(*) DESC
        LIMIT 15
        "#,
        since,
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(rows))
}

