//! Backup job and run-history endpoints.

use axum::{
    extract::{Path, Query, State},
    routing::{get, patch},
    Json, Router,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use raksha_auth::Claims;
use raksha_core::error::{AppError, AppResult};
use raksha_core::models::{new_id, PaginatedResponse, Pagination, PaginationMeta, UserRole};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_jobs))
        .route("/jobs", get(list_jobs).post(create_job))
        .route("/jobs/:id", get(get_job).delete(remove_job))
        .route("/jobs/:id/status", patch(toggle_job))
        .route("/jobs/:id/runs", get(list_job_runs))
        .route("/runs", get(list_runs))
        .route("/summary", get(backup_summary))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
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
    let jobs = sqlx::query_as::<_, BackupJobResponse>(
        r#"
        SELECT id, name, description, backup_type, target_kind, source_ref,
               destination, destination_path, server_id, is_enabled,
               schedule_interval_mins, retention_days, encryption_enabled,
               encryption_algo, verify_after_backup, last_status, last_run_at,
               next_run_at, last_size_bytes, success_count, failure_count, created_at
        FROM backup_jobs
        ORDER BY name
        "#,
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
    let job = sqlx::query_as::<_, BackupJobResponse>(
        r#"
        SELECT id, name, description, backup_type, target_kind, source_ref,
               destination, destination_path, server_id, is_enabled,
               schedule_interval_mins, retention_days, encryption_enabled,
               encryption_algo, verify_after_backup, last_status, last_run_at,
               next_run_at, last_size_bytes, success_count, failure_count, created_at
        FROM backup_jobs WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Backup job not found".to_string()))?;

    Ok(Json(job))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct BackupRunResponse {
    id: Uuid,
    job_id: Option<Uuid>,
    trigger: Option<String>,
    status: Option<String>,
    size_bytes: Option<i64>,
    compressed_bytes: Option<i64>,
    file_count: Option<i32>,
    duration_secs: Option<i32>,
    checksum: Option<String>,
    error_message: Option<String>,
    started_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
}

async fn list_runs(
    State(state): State<AppState>,
    Query(pagination): Query<Pagination>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<PaginatedResponse<BackupRunResponse>>> {
    let runs = sqlx::query_as::<_, BackupRunResponse>(
        r#"
        SELECT id, job_id, trigger, status, size_bytes, compressed_bytes,
               file_count, duration_secs, checksum, error_message,
               started_at, completed_at
        FROM backup_runs
        ORDER BY started_at DESC NULLS LAST
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(&state.db)
    .await?;

    let total: i64 = sqlx::query_scalar(r#"SELECT COUNT(*) FROM backup_runs"#)
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

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
    Path(job_id): Path<Uuid>,
    Query(pagination): Query<Pagination>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<PaginatedResponse<BackupRunResponse>>> {
    let runs = sqlx::query_as::<_, BackupRunResponse>(
        r#"
        SELECT id, job_id, trigger, status, size_bytes, compressed_bytes,
               file_count, duration_secs, checksum, error_message,
               started_at, completed_at
        FROM backup_runs
        WHERE job_id = $1
        ORDER BY started_at DESC NULLS LAST
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(job_id)
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(&state.db)
    .await?;

    let total: i64 = sqlx::query_scalar(r#"SELECT COUNT(*) FROM backup_runs WHERE job_id = $1"#)
        .bind(job_id)
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

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

#[derive(Debug, Serialize)]
struct BackupSummary {
    total_jobs: i64,
    enabled_jobs: i64,
    disabled_jobs: i64,
    runs_24h: i64,
    successful_24h: i64,
    failed_24h: i64,
    total_bytes_24h: i64,
}

#[derive(Debug, sqlx::FromRow)]
struct JobSummaryRow {
    total: i64,
    enabled: i64,
    disabled: i64,
}

#[derive(Debug, sqlx::FromRow)]
struct RunSummaryRow {
    total: i64,
    success: i64,
    failed: i64,
    bytes: i64,
}

async fn backup_summary(
    State(state): State<AppState>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<BackupSummary>> {
    let since = Utc::now() - Duration::hours(24);

    let jobs: JobSummaryRow = sqlx::query_as(
        r#"
        SELECT
            COUNT(*)::bigint as total,
            COUNT(*) FILTER (WHERE is_enabled)::bigint as enabled,
            COUNT(*) FILTER (WHERE NOT is_enabled)::bigint as disabled
        FROM backup_jobs
        "#,
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(JobSummaryRow {
        total: 0,
        enabled: 0,
        disabled: 0,
    });

    let runs: RunSummaryRow = sqlx::query_as(
        r#"
        SELECT
            COUNT(*)::bigint as total,
            COUNT(*) FILTER (WHERE status = 'completed')::bigint as success,
            COUNT(*) FILTER (WHERE status = 'failed')::bigint as failed,
            COALESCE(SUM(size_bytes), 0)::bigint as bytes
        FROM backup_runs
        WHERE started_at >= $1
        "#,
    )
    .bind(since)
    .fetch_one(&state.db)
    .await
    .unwrap_or(RunSummaryRow {
        total: 0,
        success: 0,
        failed: 0,
        bytes: 0,
    });

    Ok(Json(BackupSummary {
        total_jobs: jobs.total,
        enabled_jobs: jobs.enabled,
        disabled_jobs: jobs.disabled,
        runs_24h: runs.total,
        successful_24h: runs.success,
        failed_24h: runs.failed,
        total_bytes_24h: runs.bytes,
    }))
}

#[derive(Debug, Deserialize)]
struct CreateJobRequest {
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default = "default_backup_type")]
    backup_type: String,
    #[serde(default = "default_target_kind")]
    target_kind: String,
    source_ref: String,
    destination: String,
    #[serde(default = "default_true")]
    is_enabled: bool,
    #[serde(default = "default_retention")]
    retention_days: i32,
    #[serde(default)]
    encryption_enabled: bool,
    #[serde(default)]
    verify_after_backup: bool,
}

fn default_backup_type() -> String {
    "full".to_string()
}
fn default_target_kind() -> String {
    "directory".to_string()
}
fn default_true() -> bool {
    true
}
fn default_retention() -> i32 {
    30
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
        return Err(AppError::Validation(
            "Backup job name is required".to_string(),
        ));
    }

    let id = new_id();
    let tenant_id = claims.tenant_id.unwrap_or_else(|| {
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap_or_else(|_| Uuid::nil())
    });
    let job = sqlx::query_as::<_, BackupJobResponse>(
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
    )
    .bind(id)
    .bind(tenant_id)
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(&payload.backup_type)
    .bind(&payload.target_kind)
    .bind(&payload.source_ref)
    .bind(&payload.destination)
    .bind(payload.is_enabled)
    .bind(payload.retention_days)
    .bind(payload.encryption_enabled)
    .bind(payload.verify_after_backup)
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
    let enabled = payload
        .get("is_enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let job = sqlx::query_as::<_, BackupJobResponse>(
        r#"
        UPDATE backup_jobs SET is_enabled = $2, updated_at = NOW()
        WHERE id = $1
        RETURNING id, name, description, backup_type, target_kind, source_ref,
                  destination, destination_path, server_id, is_enabled,
                  schedule_interval_mins, retention_days, encryption_enabled,
                  encryption_algo, verify_after_backup, last_status, last_run_at,
                  next_run_at, last_size_bytes, success_count, failure_count, created_at
        "#,
    )
    .bind(id)
    .bind(enabled)
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
    let result = sqlx::query("DELETE FROM backup_jobs WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Backup job not found".to_string()));
    }
    Ok(Json(serde_json::json!({ "deleted": true, "id": id })))
}
