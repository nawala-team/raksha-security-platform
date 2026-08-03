//! Threat hunting endpoints: saved RQL queries, run history and validation.
//!
//! Query *execution* targets the OpenSearch-backed SIEM via
//! `raksha_core::hunting::QueryExecutor`; these endpoints manage the saved
//! query definitions and their run history, plus syntax validation using the
//! real RQL parser so the UI can check a query before saving it.

use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use raksha_auth::Claims;
use raksha_core::error::{AppError, AppResult};
use raksha_core::hunting::Parser;
use raksha_core::models::{Pagination, PaginatedResponse, PaginationMeta};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/queries", get(list_queries))
        .route("/queries/:id", get(get_query))
        .route("/queries/:id/runs", get(list_query_runs))
        .route("/runs", get(list_runs))
        .route("/validate", post(validate_rql))
}

#[derive(Debug, Serialize)]
struct HuntingQueryResponse {
    id: Uuid,
    name: String,
    description: Option<String>,
    rql: String,
    query_source: String,
    is_scheduled: bool,
    schedule_interval_mins: Option<i32>,
    alert_on_hits: bool,
    alert_threshold: i32,
    alert_severity: String,
    last_run_at: Option<DateTime<Utc>>,
    next_run_at: Option<DateTime<Utc>>,
    last_hit_count: Option<i64>,
    run_count: i64,
    created_at: DateTime<Utc>,
}

async fn list_queries(
    State(state): State<AppState>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<Vec<HuntingQueryResponse>>> {
    let queries = sqlx::query_as!(
        HuntingQueryResponse,
        r#"
        SELECT id, name, description, rql, query_source, is_scheduled,
               schedule_interval_mins, alert_on_hits, alert_threshold,
               alert_severity, last_run_at, next_run_at, last_hit_count,
               run_count, created_at
        FROM hunting_queries
        ORDER BY name
        "#
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(queries))
}

async fn get_query(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<HuntingQueryResponse>> {
    let query = sqlx::query_as!(
        HuntingQueryResponse,
        r#"
        SELECT id, name, description, rql, query_source, is_scheduled,
               schedule_interval_mins, alert_on_hits, alert_threshold,
               alert_severity, last_run_at, next_run_at, last_hit_count,
               run_count, created_at
        FROM hunting_queries WHERE id = $1
        "#,
        id,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Hunting query not found".to_string()))?;

    Ok(Json(query))
}

#[derive(Debug, Serialize)]
struct HuntingRunResponse {
    id: Uuid,
    query_id: Uuid,
    trigger: String,
    status: String,
    total_hits: Option<i64>,
    duration_ms: Option<i64>,
    alert_triggered: bool,
    alert_id: Option<Uuid>,
    error_message: Option<String>,
    started_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
}

async fn list_runs(
    State(state): State<AppState>,
    Query(pagination): Query<Pagination>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<PaginatedResponse<HuntingRunResponse>>> {
    let runs = sqlx::query_as!(
        HuntingRunResponse,
        r#"
        SELECT id, query_id, trigger, status, total_hits, duration_ms,
               alert_triggered, alert_id, error_message, started_at, completed_at
        FROM hunting_runs
        ORDER BY started_at DESC
        LIMIT $1 OFFSET $2
        "#,
        pagination.limit(),
        pagination.offset(),
    )
    .fetch_all(&state.db)
    .await?;

    let total = sqlx::query_scalar!(r#"SELECT COUNT(*) as "count!" FROM hunting_runs"#)
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

async fn list_query_runs(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<Vec<HuntingRunResponse>>> {
    let runs = sqlx::query_as!(
        HuntingRunResponse,
        r#"
        SELECT id, query_id, trigger, status, total_hits, duration_ms,
               alert_triggered, alert_id, error_message, started_at, completed_at
        FROM hunting_runs
        WHERE query_id = $1
        ORDER BY started_at DESC
        LIMIT 50
        "#,
        id,
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(runs))
}

#[derive(Debug, Deserialize)]
struct ValidateRequest {
    rql: String,
}

#[derive(Debug, Serialize)]
struct ValidateResponse {
    valid: bool,
    error: Option<String>,
    source: Option<String>,
    has_filter: bool,
    aggregation_count: usize,
    limit: Option<u64>,
}

/// Parse an RQL string with the real parser and report what it resolved to.
async fn validate_rql(
    _claims: axum::Extension<Claims>,
    Json(req): Json<ValidateRequest>,
) -> AppResult<Json<ValidateResponse>> {
    match Parser::parse_query(&req.rql) {
        Ok(ast) => Ok(Json(ValidateResponse {
            valid: true,
            error: None,
            source: serde_json::to_value(&ast.source)
                .ok()
                .and_then(|v| v.as_str().map(str::to_string)),
            has_filter: ast.filter.is_some(),
            aggregation_count: ast.aggregations.len(),
            limit: ast.limit,
        })),
        Err(e) => Ok(Json(ValidateResponse {
            valid: false,
            error: Some(e.to_string()),
            source: None,
            has_filter: false,
            aggregation_count: 0,
            limit: None,
        })),
    }
}
