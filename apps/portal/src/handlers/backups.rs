//! Backup job and run-history endpoints.

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
        .route("/jobs", get(list_jobs))
        .route("/jobs/:id", get(get_job))
        .route("/jobs/:id/runs", get(list_job_runs))
        .route("/runs", get(list_runs))
        .route("/summary", get(backup_summary))
}

#[derive(Debug, Serialize)]
struct BackupJobResponse {
    id: Uuid,
    name: String,
    description: Option<String>,
    backup_type: String,
    target_kind: String,
    source_ref: String,
    destination: String,
    destination_path: Option<String>,
    server_id: Option<Uuid>,
    is_enabled: bool,
    schedule_interval_mins: Option<i32>,
    retention_days: i32,
    encryption_enabled: bool,
    encryption_algo: Option<String>,
    verify_after_backup: bool,
    last_status: Option<String>,
    last_run_at: Option<DateTime<Utc>>,
    next_run_at: Option<DateTime<Utc>>,
    last_size_bytes: Option<i64>,
    success_count: i64,
    failure_count: i64,
    created_at: DateTime<Utc>,
}

async fn list_jobs(
    State(state): State<AppState>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<Vec<BackupJobResponse>>> {
    let jobs = sqlx::query_as!(
        BackupJobResponse,
        r#"
        SELECT id, name, description, backup_type, target_kind, source_ref,
               destination, destination_path, server_id, is_enabled,
               schedule_interval_mins, retention_days, encryption_enabled,
               encryption_algo, verify_after_backup, last_status, last_run_at,
               next_run_at, last_size_bytes, success_count, failure_count, created_at
        FROM backup_jobs
        ORDER BY name
        "#
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(jobs))
}

async fn get_job(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<BackupJobResponse>> {
    let job = sqlx::query_as!(
        BackupJobResponse,
        r#"
        SELECT id, name, description, backup_type, target_kind, source_ref,
               destination, destination_path, server_id, is_enabled,
               schedule_interval_mins, retention_days, encryption_enabled,
               encryption_algo, verify_after_backup, last_status, last_run_at,
               next_run_at, last_size_bytes, success_count, failure_count, created_at
        FROM backup_jobs WHERE id = $1
        "#,
        id,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Backup job not found".to_string()))?;

    Ok(Json(job))
}

#[derive(Debug, Serialize)]
struct BackupRunResponse {
    id: Uuid,
    job_id: Uuid,
    trigger: String,
    status: String,
    size_bytes: Option<i64>,
    compressed_bytes: Option<i64>,
    file_count: Option<i64>,
    duration_secs: Option<i32>,
    checksum: Option<String>,
    verified: bool,
    verified_at: Option<DateTime<Utc>>,
    restore_tested: bool,
    error_message: Option<String>,
    expires_at: Option<DateTime<Utc>>,
    started_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
}

async fn list_runs(
    State(state): State<AppState>,
    Query(pagination): Query<Pagination>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<PaginatedResponse<BackupRunResponse>>> {
    let runs = sqlx::query_as!(
        BackupRunResponse,
        r#"
        SELECT id, job_id, trigger, status, size_bytes, compressed_bytes,
               file_count, duration_secs, checksum, verified, verified_at,
               restore_tested, error_message, expires_at, started_at, completed_at
        FROM backup_runs
        ORDER BY started_at DESC
        LIMIT $1 OFFSET $2
        "#,
        pagination.limit(),
        pagination.offset(),
    )
    .fetch_all(&state.db)
    .await?;

    let total = sqlx::query_scalar!(r#"SELECT COUNT(*) as "count!" FROM backup_runs"#)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(PaginatedResponse {
        data: runs,
        meta: PaginationMeta {
            page: pagination.page,
            per_page: pagination.per_page,
            total,
            total_pages: ((total as f64) / (pagination.limit() as f64)).ceil() as u32,
        },
    }))
}

async fn list_job_runs(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<Vec<BackupRunResponse>>> {
    let runs = sqlx::query_as!(
        BackupRunResponse,
        r#"
        SELECT id, job_id, trigger, status, size_bytes, compressed_bytes,
               file_count, duration_secs, checksum, verified, verified_at,
               restore_tested, error_message, expires_at, started_at, completed_at
        FROM backup_runs
        WHERE job_id = $1
        ORDER BY started_at DESC
        LIMIT 50
        "#,
        id,
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(runs))
}

#[derive(Debug, Serialize)]
struct BackupSummary {
    total_jobs: i64,
    enabled_jobs: i64,
    failing_jobs: i64,
    never_run_jobs: i64,
    /// Jobs storing data without encryption: a posture gap worth surfacing.
    unencrypted_jobs: i64,
    total_backup_bytes: i64,
    unverified_runs: i64,
}

async fn backup_summary(
    State(state): State<AppState>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<BackupSummary>> {
    let jobs = sqlx::query!(
        r#"
        SELECT
            COUNT(*) as "total!",
            COUNT(*) FILTER (WHERE is_enabled) as "enabled!",
            COUNT(*) FILTER (WHERE last_status = 'failed') as "failing!",
            COUNT(*) FILTER (WHERE last_run_at IS NULL) as "never_run!",
            COUNT(*) FILTER (WHERE NOT encryption_enabled) as "unencrypted!"
        FROM backup_jobs
        "#
    )
    .fetch_one(&state.db)
    .await?;

    let runs = sqlx::query!(
        r#"
        SELECT
            COALESCE(SUM(size_bytes), 0)::bigint as "total_bytes!",
            COUNT(*) FILTER (WHERE status = 'success' AND NOT verified) as "unverified!"
        FROM backup_runs
        "#
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(BackupSummary {
        total_jobs: jobs.total,
        enabled_jobs: jobs.enabled,
        failing_jobs: jobs.failing,
        never_run_jobs: jobs.never_run,
        unencrypted_jobs: jobs.unencrypted,
        total_backup_bytes: runs.total_bytes,
        unverified_runs: runs.unverified,
    }))
}

