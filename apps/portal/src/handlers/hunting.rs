//! Threat hunting endpoints: saved RQL queries, run history and validation.

use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::warn;
use uuid::Uuid;

use raksha_auth::Claims;
use raksha_core::error::{AppError, AppResult};
use raksha_core::hunting::Parser;
use raksha_core::models::{new_id, PaginatedResponse, Pagination, PaginationMeta, UserRole};

use crate::state::AppState;

/// Default tenant ID - in production this should come from claims
fn default_tenant_id() -> Uuid {
    Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap_or_else(|_| Uuid::nil())
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_queries))
        .route("/queries", get(list_queries).post(create_query))
        .route("/queries/:id", get(get_query).delete(remove_query))
        .route("/queries/:id/runs", get(list_query_runs))
        .route("/runs", get(list_runs))
        .route("/validate", post(validate_rql))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
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
    let queries = sqlx::query_as::<_, HuntingQueryResponse>(
        r#"
        SELECT id, name, description, rql, query_source, is_scheduled,
               schedule_interval_mins, alert_on_hits, alert_threshold,
               alert_severity, last_run_at, next_run_at, last_hit_count,
               run_count, created_at
        FROM hunting_queries
        ORDER BY name
        "#,
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
    let query = sqlx::query_as::<_, HuntingQueryResponse>(
        r#"
        SELECT id, name, description, rql, query_source, is_scheduled,
               schedule_interval_mins, alert_on_hits, alert_threshold,
               alert_severity, last_run_at, next_run_at, last_hit_count,
               run_count, created_at
        FROM hunting_queries WHERE id = \$1
        "#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Hunting query not found".to_string()))?;

    Ok(Json(query))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct HuntingRunResponse {
    id: Uuid,
    query_id: Option<Uuid>,
    status: Option<String>,
    results_count: Option<i32>,
    error_message: Option<String>,
    started_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
}

async fn list_runs(
    State(state): State<AppState>,
    Query(pagination): Query<Pagination>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<PaginatedResponse<HuntingRunResponse>>> {
    let runs = sqlx::query_as::<_, HuntingRunResponse>(
        r#"
        SELECT id, query_id, status,
               COALESCE(results_count, total_hits::integer, 0) as results_count,
               error_message, started_at, completed_at
        FROM hunting_runs
        ORDER BY started_at DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(&state.db)
    .await?;

    let total: i64 = sqlx::query_scalar(r#"SELECT COUNT(*) FROM hunting_runs"#)
        .fetch_one(&state.db)
        .await
        .unwrap_or_else(|e| {
            warn!("Failed to count hunting_runs: {}", e);
            0
        });

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
    Path(query_id): Path<Uuid>,
    Query(pagination): Query<Pagination>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<PaginatedResponse<HuntingRunResponse>>> {
    let runs = sqlx::query_as::<_, HuntingRunResponse>(
        r#"
        SELECT id, query_id, status,
               COALESCE(results_count, total_hits::integer, 0) as results_count,
               error_message, started_at, completed_at
        FROM hunting_runs
        WHERE query_id = $1
        ORDER BY started_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(query_id)
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(&state.db)
    .await?;

    let total: i64 = sqlx::query_scalar(r#"SELECT COUNT(*) FROM hunting_runs WHERE query_id = $1"#)
        .bind(query_id)
        .fetch_one(&state.db)
        .await
        .unwrap_or_else(|e| {
            warn!("Failed to count runs for query {}: {}", query_id, e);
            0
        });

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
    limit: Option<usize>,
}

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
            limit: ast.limit.map(|l| l as usize),
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

#[derive(Debug, Deserialize)]
struct CreateQueryRequest {
    name: String,
    #[serde(default)]
    description: Option<String>,
    rql: String,
    #[serde(default = "default_source")]
    query_source: String,
}

fn default_source() -> String {
    "events".to_string()
}

/// Valid query sources
const VALID_SOURCES: &[&str] = &["events", "alerts", "network", "fim", "logs"];

async fn create_query(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(payload): Json<CreateQueryRequest>,
) -> AppResult<Json<HuntingQueryResponse>> {
    if !claims.role.has_permission(&UserRole::Operator) {
        return Err(AppError::Forbidden(
            "Operator access required to create hunting queries".to_string(),
        ));
    }
    if payload.name.trim().is_empty() {
        return Err(AppError::Validation("Query name is required".to_string()));
    }
    if payload.rql.trim().is_empty() {
        return Err(AppError::Validation("RQL query is required".to_string()));
    }

    // Validate RQL syntax before saving
    if let Err(e) = Parser::parse_query(&payload.rql) {
        return Err(AppError::Validation(format!("Invalid RQL syntax: {}", e)));
    }

    // Validate query_source
    let query_source = payload.query_source.to_lowercase();
    if !VALID_SOURCES.contains(&query_source.as_str()) {
        return Err(AppError::Validation(format!(
            "Invalid query_source '{}'. Must be one of: {:?}",
            payload.query_source, VALID_SOURCES
        )));
    }

    let id = new_id();
    // Get tenant_id from claims or use default
    let tenant_id = claims.tenant_id.unwrap_or_else(default_tenant_id);

    let query = sqlx::query_as::<_, HuntingQueryResponse>(
        r#"
        INSERT INTO hunting_queries
            (id, tenant_id, name, description, rql, query_source, is_scheduled,
             alert_on_hits, alert_threshold, alert_severity, run_count, created_by, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, false, false, 1, 'medium', 0, $7, NOW(), NOW())
        RETURNING id, name, description, rql, query_source, is_scheduled,
                  schedule_interval_mins, alert_on_hits, alert_threshold, alert_severity,
                  last_run_at, next_run_at, last_hit_count, run_count, created_at
        "#
    )
    .bind(id)
    .bind(tenant_id)
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(&payload.rql)
    .bind(&query_source)
    .bind(claims.sub)
    .fetch_one(&state.db)
    .await?;

    tracing::info!(query_id = %id, name = %payload.name, "Hunting query created");
    Ok(Json(query))
}

async fn remove_query(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> AppResult<Json<serde_json::Value>> {
    if !claims.role.has_permission(&UserRole::Admin) {
        return Err(AppError::Forbidden(
            "Admin access required to delete hunting queries".to_string(),
        ));
    }
    let result = sqlx::query("DELETE FROM hunting_queries WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Hunting query not found".to_string()));
    }
    tracing::info!(query_id = %id, deleted_by = %claims.sub, "Hunting query deleted");
    Ok(Json(serde_json::json!({ "deleted": true, "id": id })))
}
