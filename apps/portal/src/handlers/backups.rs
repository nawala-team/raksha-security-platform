//! Backup job and run-history endpoints.

use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, patch, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use raksha_auth::Claims;
use raksha_core::error::{AppError, AppResult};
use raksha_core::models::{new_id, Pagination, PaginatedResponse, PaginationMeta, UserRole};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/jobs", get(list_jobs).post(create_job))
        .route("/jobs/:id", get(get_job).delete(remove_job))
        .route("/jobs/:id/status", patch(toggle_job))
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

#[derive(Debug, Deserialize)]
struct CreateJobRequest {
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default = "default_type")]
    backup_type: String,
    #[serde(default = "default_target_kind")]
    target_kind: String,
    #[serde(default = "default_source_ref")]
    source_ref: String,
    #[serde(default = "default_dest")]
    destination: String,
    #[serde(default = "default_retention")]
    retention_days: i32,
    #[serde(default = "default_true")]
    encryption_enabled: bool,
    #[serde(default = "default_true")]
    verify_after_backup: bool,
    #[serde(default)]
    is_enabled: bool,
}

fn default_type() -> String {
    "full".to_string()
}
fn default_target_kind() -> String {
    "database".to_string()
}
fn default_source_ref() -> String {
    "/data".to_string()
}
fn default_dest() -> String {
    "local".to_string()
}
fn default_retention() -> i32 {
    30
}
fn default_true() -> bool {
    true
}

async fn create_job(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(payload): Json<CreateJobRequest>,
) -> AppResult<Json<BackupJobResponse>> {
    if !claims.role.has_permission(&UserRole::Operator) {
        return Err(AppError::Forbidden(
            "Operator access required to create backup jobs".to_string(),
        ));
    }
    if payload.name.trim().is_empty() {
        return Err(AppError::Validation("Backup job name is required".to_string()));
    }

    let id = new_id();
    // Use the default tenant for now; multi-tenancy can derive this from claims later.
    let tenant_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    let job = sqlx::query_as!(
        BackupJobResponse,
        r#"
        INSERT INTO backup_jobs
            (id, tenant_id, name, description, backup_type, target_kind, source_ref, destination,
             is_enabled, retention_days, encryption_enabled, verify_after_backup,
             success_count, failure_count, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, 0, 0, NOW(), NOW())
        RETURNING id, name, description, backup_type, target_kind, source_ref,
                  destination, destination_path, server_id, is_enabled,
                  schedule_interval_mins, retention_days, encryption_enabled,
                  encryption_algo, verify_after_backup, last_status, last_run_at,
                  next_run_at, last_size_bytes, success_count, failure_count, created_at
        "#,
        id,
        tenant_id,
        payload.name,
        payload.description,
        payload.backup_type,
        payload.target_kind,
        payload.source_ref,
        payload.destination,
        payload.is_enabled,
        payload.retention_days,
        payload.encryption_enabled,
        payload.verify_after_backup,
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(job))
}

async fn toggle_job(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(payload): Json<serde_json::Value>,
) -> AppResult<Json<BackupJobResponse>> {
    if !claims.role.has_permission(&UserRole::Operator) {
        return Err(AppError::Forbidden(
            "Operator access required to update backup jobs".to_string(),
        ));
    }
    let enabled = payload.get("is_enabled").and_then(|v| v.as_bool()).unwrap_or(true);
    let job = sqlx::query_as!(
        BackupJobResponse,
        r#"
        UPDATE backup_jobs SET is_enabled = $2, updated_at = NOW()
        WHERE id = $1
        RETURNING id, name, description, backup_type, target_kind, source_ref,
                  destination, destination_path, server_id, is_enabled,
                  schedule_interval_mins, retention_days, encryption_enabled,
                  encryption_algo, verify_after_backup, last_status, last_run_at,
                  next_run_at, last_size_bytes, success_count, failure_count, created_at
        "#,
        id,
        enabled,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Backup job not found".to_string()))?;

    Ok(Json(job))
}

async fn remove_job(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> AppResult<Json<serde_json::Value>> {
    if !claims.role.has_permission(&UserRole::Admin) {
        return Err(AppError::Forbidden(
            "Admin access required to delete backup jobs".to_string(),
        ));
    }
    let result = sqlx::query!("DELETE FROM backup_jobs WHERE id = $1", id)
        .execute(&state.db)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Backup job not found".to_string()));
    }
    Ok(Json(serde_json::json!({ "deleted": true, "id": id })))
}

