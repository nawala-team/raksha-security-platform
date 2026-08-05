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
use raksha_core::models::{ComplianceStatus, Pagination, PaginatedResponse, PaginationMeta};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_compliance_scores))
        .route("/scores", get(list_compliance_scores))
        .route("/scores/:id", get(get_compliance_score))
        .route("/standards", get(list_standards))
        .route("/controls", get(list_controls))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct ComplianceScoreResponse {
    id: Uuid,
    org_id: Uuid,
    standard_id: Uuid,
    overall_score: f64,
    status: ComplianceStatus,
    controls_total: i32,
    controls_passed: i32,
    controls_failed: i32,
    controls_na: i32,
    assessed_at: DateTime<Utc>,
}

async fn list_compliance_scores(
    State(state): State<AppState>,
    Query(pagination): Query<Pagination>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<PaginatedResponse<ComplianceScoreResponse>>> {
    let scores = sqlx::query_as::<_, ComplianceScoreResponse>(
        r#"
        SELECT id, org_id, standard_id, overall_score,
               status,
               controls_total, controls_passed, controls_failed, controls_na,
               assessed_at
        FROM compliance_scores
        ORDER BY assessed_at DESC
        LIMIT $1 OFFSET $2
        "#
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(&state.db)
    .await?;

    let total: i64 = sqlx::query_scalar(r#"SELECT COUNT(*) FROM compliance_scores"#)
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    Ok(Json(PaginatedResponse {
        data: scores,
        meta: PaginationMeta {
            page: pagination.page,
            per_page: pagination.per_page,
            total,
            total_pages: ((total as f64) / (pagination.limit() as f64)).ceil() as u32,
        },
    }))
}

async fn get_compliance_score(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<ComplianceScoreResponse>> {
    let score = sqlx::query_as::<_, ComplianceScoreResponse>(
        r#"
        SELECT id, org_id, standard_id, overall_score,
               status,
               controls_total, controls_passed, controls_failed, controls_na,
               assessed_at
        FROM compliance_scores WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Compliance score not found".to_string()))?;

    Ok(Json(score))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct StandardResponse {
    id: Uuid,
    name: String,
    version: String,
    description: Option<String>,
    authority: Option<String>,
    is_active: bool,
    created_at: DateTime<Utc>,
}

async fn list_standards(
    State(state): State<AppState>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<Vec<StandardResponse>>> {
    let standards = sqlx::query_as::<_, StandardResponse>(
        r#"
        SELECT id, name, version, description, authority, is_active, created_at
        FROM compliance_standards
        WHERE is_active = true
        ORDER BY name
        "#
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(standards))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct ControlResponse {
    id: Uuid,
    standard_id: Uuid,
    control_ref: String,
    title: String,
    description: Option<String>,
    category: Option<String>,
    automated: bool,
}

async fn list_controls(
    State(state): State<AppState>,
    Query(pagination): Query<Pagination>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<PaginatedResponse<ControlResponse>>> {
    let controls = sqlx::query_as::<_, ControlResponse>(
        r#"
        SELECT id, standard_id, control_ref, title, description, category, automated
        FROM compliance_controls
        ORDER BY control_ref
        LIMIT $1 OFFSET $2
        "#
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(&state.db)
    .await?;

    let total: i64 = sqlx::query_scalar(r#"SELECT COUNT(*) FROM compliance_controls"#)
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    Ok(Json(PaginatedResponse {
        data: controls,
        meta: PaginationMeta {
            page: pagination.page,
            per_page: pagination.per_page,
            total,
            total_pages: ((total as f64) / (pagination.limit() as f64)).ceil() as u32,
        },
    }))
}
