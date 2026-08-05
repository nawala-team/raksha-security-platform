//! GRC endpoints: risk register, policies and control framework mappings.

use axum::{
    extract::{Path, State},
    routing::get,
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
        .route("/policies", get(list_policies).post(create_policy))
        .route("/policies/:id", get(get_policy).delete(delete_policy))
        .route("/controls", get(list_controls))
        .route("/summary", get(grc_summary))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
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
    let risks = sqlx::query_as::<_, RiskResponse>(
        r#"
        SELECT id, title, description, category, likelihood, impact,
               risk_score, owner, status, mitigation_plan, review_date,
               created_at, updated_at
        FROM grc_risks
        ORDER BY risk_score DESC, review_date
        "#,
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
    let risk = sqlx::query_as::<_, RiskResponse>(
        r#"
        SELECT id, title, description, category, likelihood, impact,
               risk_score, owner, status, mitigation_plan, review_date,
               created_at, updated_at
        FROM grc_risks WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Risk not found".to_string()))?;

    Ok(Json(risk))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
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

async fn list_policies(
    State(state): State<AppState>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<Vec<PolicyResponse>>> {
    let policies = sqlx::query_as::<_, PolicyResponse>(
        r#"
        SELECT id, title, version, status, approved_by, effective_date,
               review_cycle_days, created_at, updated_at
        FROM grc_policies
        ORDER BY title
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(policies))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
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
    let controls = sqlx::query_as::<_, ControlResponse>(
        r#"
        SELECT id, title, description, framework, control_ref, status,
               evidence, last_assessed, created_at
        FROM grc_controls
        ORDER BY framework, control_ref
        "#,
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

#[derive(Debug, sqlx::FromRow)]
struct RiskSummaryRow {
    total: i64,
    high: i64,
    open: i64,
    due: i64,
}

#[derive(Debug, sqlx::FromRow)]
struct PolicySummaryRow {
    total: i64,
    published: i64,
}

#[derive(Debug, sqlx::FromRow)]
struct ControlSummaryRow {
    total: i64,
    implemented: i64,
}

async fn grc_summary(
    State(state): State<AppState>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<GrcSummary>> {
    let risks: RiskSummaryRow = sqlx::query_as(
        r#"
        SELECT
            COUNT(*)::bigint as total,
            COUNT(*) FILTER (WHERE risk_score >= 15)::bigint as high,
            COUNT(*) FILTER (WHERE status != 'closed')::bigint as open,
            COUNT(*) FILTER (WHERE review_date <= CURRENT_DATE)::bigint as due
        FROM grc_risks
        "#,
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(RiskSummaryRow {
        total: 0,
        high: 0,
        open: 0,
        due: 0,
    });

    let policies: PolicySummaryRow = sqlx::query_as(
        r#"
        SELECT
            COUNT(*)::bigint as total,
            COUNT(*) FILTER (WHERE status = 'published')::bigint as published
        FROM grc_policies
        "#,
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(PolicySummaryRow {
        total: 0,
        published: 0,
    });

    let controls: ControlSummaryRow = sqlx::query_as(
        r#"
        SELECT
            COUNT(*)::bigint as total,
            COUNT(*) FILTER (WHERE status = 'implemented')::bigint as implemented
        FROM grc_controls
        "#,
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(ControlSummaryRow {
        total: 0,
        implemented: 0,
    });

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
    let tenant_id = claims.tenant_id.unwrap_or_else(|| {
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap_or_else(|_| Uuid::nil())
    });
    let review_date = (chrono::Utc::now() + chrono::Duration::days(90)).date_naive();

    let risk = sqlx::query_as::<_, RiskResponse>(
        r#"
        INSERT INTO grc_risks
            (id, tenant_id, title, description, category, likelihood, impact,
             owner, status, mitigation_plan, review_date, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'identified', $9, $10, NOW(), NOW())
        RETURNING id, title, description, category, likelihood, impact, risk_score,
                  owner, status, mitigation_plan, review_date, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(tenant_id)
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(&payload.category)
    .bind(payload.likelihood)
    .bind(payload.impact)
    .bind(claims.sub)
    .bind(&payload.mitigation_plan)
    .bind(review_date)
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
    let result = sqlx::query("DELETE FROM grc_risks WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Risk not found".to_string()));
    }
    Ok(Json(serde_json::json!({ "deleted": true, "id": id })))
}

#[derive(Debug, Deserialize)]
struct CreatePolicyRequest {
    title: String,
    #[serde(default)]
    content: Option<String>,
    #[serde(default = "default_version")]
    version: String,
    #[serde(default)]
    effective_date: Option<NaiveDate>,
    #[serde(default = "default_review_cycle")]
    review_cycle_days: i32,
}

fn default_version() -> String {
    "1.0".to_string()
}
fn default_review_cycle() -> i32 {
    365
}

async fn create_policy(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(payload): Json<CreatePolicyRequest>,
) -> AppResult<Json<PolicyResponse>> {
    if !claims.role.has_permission(&UserRole::Admin) {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    if payload.title.trim().is_empty() {
        return Err(AppError::Validation("Title is required".to_string()));
    }

    // Use tenant from claims or default
    let tenant_id = claims.tenant_id.unwrap_or_else(|| {
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap_or_else(|_| Uuid::nil())
    });
    let content = payload.content.unwrap_or_default();

    let id = new_id();
    let policy = sqlx::query_as::<_, PolicyResponse>(
        r#"
        INSERT INTO grc_policies (id, tenant_id, title, version, content, status, approved_by, effective_date, review_cycle_days, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, 'draft', $6, $7, $8, NOW(), NOW())
        RETURNING id, title, version, status, approved_by, effective_date, review_cycle_days, created_at, updated_at
        "#
    )
    .bind(id)
    .bind(tenant_id)
    .bind(&payload.title)
    .bind(&payload.version)
    .bind(&content)
    .bind(claims.sub)
    .bind(payload.effective_date)
    .bind(payload.review_cycle_days)
    .fetch_one(&state.db)
    .await?;

    tracing::info!(policy_id = %id, title = %payload.title, "Policy created");
    Ok(Json(policy))
}

async fn get_policy(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<PolicyResponse>> {
    let policy = sqlx::query_as::<_, PolicyResponse>(
        r#"
        SELECT id, title, version, status, approved_by, effective_date, review_cycle_days, created_at, updated_at
        FROM grc_policies WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Policy not found".to_string()))?;

    Ok(Json(policy))
}

async fn delete_policy(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> AppResult<Json<serde_json::Value>> {
    if !claims.role.has_permission(&UserRole::Admin) {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    let result = sqlx::query("DELETE FROM grc_policies WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Policy not found".to_string()));
    }

    tracing::info!(policy_id = %id, deleted_by = %claims.sub, "Policy deleted");
    Ok(Json(serde_json::json!({"status": "deleted", "id": id})))
}
