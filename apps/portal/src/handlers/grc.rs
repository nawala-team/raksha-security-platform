//! GRC endpoints: risk register, policies and control framework mappings.

use axum::{
    extract::{Path, State},
    routing::{delete, get, post},
    Json, Router,
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use raksha_auth::Claims;
use raksha_core::error::{AppError, AppResult};
use raksha_core::models::{new_id, UserRole};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/risks", get(list_risks).post(create_risk))
        .route("/risks/:id", get(get_risk).delete(remove_risk))
        .route("/policies", get(list_policies))
        .route("/controls", get(list_controls))
        .route("/summary", get(grc_summary))
}

#[derive(Debug, Serialize)]
struct RiskResponse {
    id: Uuid,
    title: String,
    description: String,
    category: String,
    likelihood: i16,
    impact: i16,
    risk_score: i16,
    owner: Uuid,
    status: String,
    mitigation_plan: Option<String>,
    review_date: NaiveDate,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

async fn list_risks(
    State(state): State<AppState>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<Vec<RiskResponse>>> {
    let risks = sqlx::query_as!(
        RiskResponse,
        r#"
        SELECT id, title, description, category, likelihood, impact,
               risk_score, owner, status, mitigation_plan, review_date,
               created_at, updated_at
        FROM grc_risks
        ORDER BY risk_score DESC, review_date
        "#
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(risks))
}

async fn get_risk(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<RiskResponse>> {
    let risk = sqlx::query_as!(
        RiskResponse,
        r#"
        SELECT id, title, description, category, likelihood, impact,
               risk_score, owner, status, mitigation_plan, review_date,
               created_at, updated_at
        FROM grc_risks WHERE id = $1
        "#,
        id,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Risk not found".to_string()))?;

    Ok(Json(risk))
}

#[derive(Debug, Serialize)]
struct PolicyResponse {
    id: Uuid,
    title: String,
    version: String,
    status: String,
    approved_by: Option<Uuid>,
    effective_date: Option<NaiveDate>,
    review_cycle_days: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

/// Policy list omits the full `content` body to keep the payload small; the
/// detail view can fetch it when needed.
async fn list_policies(
    State(state): State<AppState>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<Vec<PolicyResponse>>> {
    let policies = sqlx::query_as!(
        PolicyResponse,
        r#"
        SELECT id, title, version, status, approved_by, effective_date,
               review_cycle_days, created_at, updated_at
        FROM grc_policies
        ORDER BY title
        "#
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(policies))
}

#[derive(Debug, Serialize)]
struct ControlResponse {
    id: Uuid,
    title: String,
    description: String,
    framework: String,
    control_ref: String,
    status: String,
    evidence: Option<String>,
    last_assessed: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

async fn list_controls(
    State(state): State<AppState>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<Vec<ControlResponse>>> {
    let controls = sqlx::query_as!(
        ControlResponse,
        r#"
        SELECT id, title, description, framework, control_ref, status,
               evidence, last_assessed, created_at
        FROM grc_controls
        ORDER BY framework, control_ref
        "#
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(controls))
}

#[derive(Debug, Serialize)]
struct GrcSummary {
    total_risks: i64,
    high_risks: i64,
    open_risks: i64,
    risks_due_review: i64,
    total_policies: i64,
    published_policies: i64,
    total_controls: i64,
    implemented_controls: i64,
}

async fn grc_summary(
    State(state): State<AppState>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<GrcSummary>> {
    let risks = sqlx::query!(
        r#"
        SELECT
            COUNT(*) as "total!",
            COUNT(*) FILTER (WHERE risk_score >= 15) as "high!",
            COUNT(*) FILTER (WHERE status NOT IN ('closed', 'accepted')) as "open!",
            COUNT(*) FILTER (
                WHERE review_date <= CURRENT_DATE
                  AND status NOT IN ('closed', 'accepted')
            ) as "due!"
        FROM grc_risks
        "#
    )
    .fetch_one(&state.db)
    .await?;

    let policies = sqlx::query!(
        r#"
        SELECT
            COUNT(*) as "total!",
            COUNT(*) FILTER (WHERE status = 'published') as "published!"
        FROM grc_policies
        "#
    )
    .fetch_one(&state.db)
    .await?;

    let controls = sqlx::query!(
        r#"
        SELECT
            COUNT(*) as "total!",
            COUNT(*) FILTER (WHERE status = 'implemented') as "implemented!"
        FROM grc_controls
        "#
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(GrcSummary {
        total_risks: risks.total,
        high_risks: risks.high,
        open_risks: risks.open,
        risks_due_review: risks.due,
        total_policies: policies.total,
        published_policies: policies.published,
        total_controls: controls.total,
        implemented_controls: controls.implemented,
    }))
}

#[derive(Debug, Deserialize)]
struct CreateRiskRequest {
    title: String,
    description: String,
    #[serde(default = "default_category")]
    category: String,
    #[serde(default = "default_likelihood")]
    likelihood: i16,
    #[serde(default = "default_impact")]
    impact: i16,
    #[serde(default)]
    mitigation_plan: Option<String>,
}

fn default_category() -> String {
    "operational".to_string()
}
fn default_likelihood() -> i16 {
    3
}
fn default_impact() -> i16 {
    3
}

async fn create_risk(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(payload): Json<CreateRiskRequest>,
) -> AppResult<Json<RiskResponse>> {
    if !claims.role.has_permission(&UserRole::Operator) {
        return Err(AppError::Forbidden(
            "Operator access required to create risks".to_string(),
        ));
    }
    if payload.title.trim().is_empty() {
        return Err(AppError::Validation("Risk title is required".to_string()));
    }

    let id = new_id();
    let tenant_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    let review_date = (chrono::Utc::now() + chrono::Duration::days(90)).date_naive();

    let risk = sqlx::query_as!(
        RiskResponse,
        r#"
        INSERT INTO grc_risks
            (id, tenant_id, title, description, category, likelihood, impact,
             owner, status, mitigation_plan, review_date, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'identified', $9, $10, NOW(), NOW())
        RETURNING id, title, description, category, likelihood, impact, risk_score,
                  owner, status, mitigation_plan, review_date, created_at, updated_at
        "#,
        id,
        tenant_id,
        payload.title,
        payload.description,
        payload.category,
        payload.likelihood,
        payload.impact,
        claims.sub,
        payload.mitigation_plan,
        review_date,
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(risk))
}

async fn remove_risk(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> AppResult<Json<serde_json::Value>> {
    if !claims.role.has_permission(&UserRole::Admin) {
        return Err(AppError::Forbidden(
            "Admin access required to delete risks".to_string(),
        ));
    }
    let result = sqlx::query!("DELETE FROM grc_risks WHERE id = $1", id)
        .execute(&state.db)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Risk not found".to_string()));
    }
    Ok(Json(serde_json::json!({ "deleted": true, "id": id })))
}
