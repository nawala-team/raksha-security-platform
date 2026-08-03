//! HTTP handlers for the GRC module.
//!
//! Provides REST API endpoints for risk management, policy lifecycle,
//! control mapping, dashboard statistics, and framework coverage.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::control_mapping::ControlMapper;
use crate::models::*;
use crate::policy_manager::PolicyManager;
use crate::risk_engine::RiskEngine;

/// Shared application state for GRC handlers.
#[derive(Clone)]
pub struct GrcState {
    pub pool: PgPool,
}

/// Query parameters for tenant-scoped requests.
#[derive(Debug, Deserialize)]
pub struct TenantQuery {
    pub tenant_id: Uuid,
}

/// Query parameters for risk trending.
#[derive(Debug, Deserialize)]
pub struct TrendQuery {
    pub tenant_id: Uuid,
    pub days: Option<i32>,
}

/// Query parameters for risk acceptance.
#[derive(Debug, Deserialize)]
pub struct AcceptRiskRequest {
    pub accepted_by: Uuid,
    pub justification: String,
}

/// Build the GRC router with all endpoints.
pub fn router(state: GrcState) -> Router {
    Router::new()
        // Risk endpoints
        .route("/risks", get(list_risks).post(create_risk))
        .route("/risks/:id", get(get_risk).put(update_risk))
        .route("/risks/:id/accept", post(accept_risk))
        // Policy endpoints
        .route("/policies", get(list_policies).post(create_policy))
        .route("/policies/:id", get(get_policy))
        .route("/policies/:id/activate", post(activate_policy))
        .route("/policies/:id/archive", post(archive_policy))
        .route("/policies/:id/acknowledge", post(acknowledge_policy))
        // Control endpoints
        .route("/controls", get(list_controls).post(create_control))
        .route("/controls/:id", get(get_control).put(update_control))
        .route("/controls/mappings", post(add_control_mapping))
        // Dashboard & analytics
        .route("/dashboard", get(get_dashboard))
        .route("/heatmap", get(get_heatmap))
        .route("/coverage/:framework", get(get_coverage))
        .route("/gaps/:framework", get(get_gap_analysis))
        .with_state(state)
}

// ============================================================
// Risk Handlers
// ============================================================

async fn create_risk(
    State(state): State<GrcState>,
    Query(tenant): Query<TenantQuery>,
    Json(req): Json<CreateRiskRequest>,
) -> impl IntoResponse {
    let id = Uuid::now_v7();
    let now = chrono::Utc::now();
    let risk_score = RiskItem::calculate_score(req.likelihood, req.impact);

    let result = sqlx::query(
        r#"
        INSERT INTO grc_risks (id, tenant_id, title, description, category, likelihood,
            impact, owner, status, mitigation_plan, review_date, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'identified', $9, $10, $11, $11)
        "#,
    )
    .bind(id)
    .bind(tenant.tenant_id)
    .bind(&req.title)
    .bind(&req.description)
    .bind(req.category.to_string())
    .bind(req.likelihood as i16)
    .bind(req.impact as i16)
    .bind(req.owner)
    .bind(&req.mitigation_plan)
    .bind(req.review_date)
    .bind(now)
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => {
            let risk = RiskItem {
                id,
                tenant_id: tenant.tenant_id,
                title: req.title,
                description: req.description,
                category: req.category,
                likelihood: req.likelihood,
                impact: req.impact,
                risk_score,
                owner: req.owner,
                status: RiskStatus::Identified,
                mitigation_plan: req.mitigation_plan,
                review_date: req.review_date,
                created_at: now,
                updated_at: now,
            };
            (StatusCode::CREATED, Json(serde_json::to_value(risk).unwrap())).into_response()
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response()
        }
    }
}

async fn list_risks(
    State(state): State<GrcState>,
    Query(tenant): Query<TenantQuery>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, (Uuid, Uuid, String, String, String, i16, i16, i16, Uuid, String, Option<String>, chrono::NaiveDate, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
        r#"
        SELECT id, tenant_id, title, description, category, likelihood, impact,
            risk_score, owner, status, mitigation_plan, review_date, created_at, updated_at
        FROM grc_risks
        WHERE tenant_id = $1
        ORDER BY risk_score DESC, created_at DESC
        "#,
    )
    .bind(tenant.tenant_id)
    .fetch_all(&state.pool)
    .await;

    match result {
        Ok(rows) => {
            let risks: Vec<serde_json::Value> = rows
                .into_iter()
                .map(|(id, tenant_id, title, desc, cat, likelihood, impact, score, owner, status, plan, review, created, updated)| {
                    serde_json::json!({
                        "id": id,
                        "tenant_id": tenant_id,
                        "title": title,
                        "description": desc,
                        "category": cat,
                        "likelihood": likelihood,
                        "impact": impact,
                        "risk_score": score,
                        "owner": owner,
                        "status": status,
                        "mitigation_plan": plan,
                        "review_date": review,
                        "created_at": created,
                        "updated_at": updated,
                    })
                })
                .collect();
            (StatusCode::OK, Json(serde_json::json!({"risks": risks}))).into_response()
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response()
        }
    }
}

async fn get_risk(
    State(state): State<GrcState>,
    Path(id): Path<Uuid>,
    Query(tenant): Query<TenantQuery>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, (Uuid, String, String, String, i16, i16, i16, Uuid, String, Option<String>, chrono::NaiveDate)>(
        r#"
        SELECT id, title, description, category, likelihood, impact,
            risk_score, owner, status, mitigation_plan, review_date
        FROM grc_risks
        WHERE id = $1 AND tenant_id = $2
        "#,
    )
    .bind(id)
    .bind(tenant.tenant_id)
    .fetch_optional(&state.pool)
    .await;

    match result {
        Ok(Some(row)) => {
            let (id, title, desc, cat, likelihood, impact, score, owner, status, plan, review) = row;
            (StatusCode::OK, Json(serde_json::json!({
                "id": id, "title": title, "description": desc, "category": cat,
                "likelihood": likelihood, "impact": impact, "risk_score": score,
                "owner": owner, "status": status, "mitigation_plan": plan, "review_date": review,
            }))).into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "risk not found"}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

async fn update_risk(
    State(state): State<GrcState>,
    Path(id): Path<Uuid>,
    Query(tenant): Query<TenantQuery>,
    Json(req): Json<CreateRiskRequest>,
) -> impl IntoResponse {
    let result = sqlx::query(
        r#"
        UPDATE grc_risks
        SET title = $3, description = $4, category = $5, likelihood = $6,
            impact = $7, owner = $8, mitigation_plan = $9,
            review_date = $10, updated_at = NOW()
        WHERE id = $1 AND tenant_id = $2
        "#,
    )
    .bind(id)
    .bind(tenant.tenant_id)
    .bind(&req.title)
    .bind(&req.description)
    .bind(req.category.to_string())
    .bind(req.likelihood as i16)
    .bind(req.impact as i16)
    .bind(req.owner)
    .bind(&req.mitigation_plan)
    .bind(req.review_date)
    .execute(&state.pool)
    .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            (StatusCode::OK, Json(serde_json::json!({"status": "updated"}))).into_response()
        }
        Ok(_) => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "risk not found"}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

async fn accept_risk(
    State(state): State<GrcState>,
    Path(id): Path<Uuid>,
    Query(tenant): Query<TenantQuery>,
    Json(req): Json<AcceptRiskRequest>,
) -> impl IntoResponse {
    let engine = RiskEngine::new(state.pool);
    match engine.accept_risk(tenant.tenant_id, id, req.accepted_by, &req.justification).await {
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({"status": "accepted"}))).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}
// ============================================================
// Policy Handlers
// ============================================================

async fn create_policy(
    State(state): State<GrcState>,
    Query(tenant): Query<TenantQuery>,
    Json(req): Json<CreatePolicyRequest>,
) -> impl IntoResponse {
    let mgr = PolicyManager::new(state.pool);
    let review_days = req.review_cycle_days.unwrap_or(365);

    match mgr.create_policy(
        tenant.tenant_id, &req.title, &req.version, &req.content,
        req.effective_date, review_days,
    ).await {
        Ok(policy) => (StatusCode::CREATED, Json(serde_json::to_value(policy).unwrap())).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

async fn list_policies(
    State(state): State<GrcState>,
    Query(tenant): Query<TenantQuery>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, (Uuid, String, String, String, Option<Uuid>, Option<chrono::NaiveDate>, i32, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
        r#"
        SELECT id, title, version, status, approved_by, effective_date,
            review_cycle_days, created_at, updated_at
        FROM grc_policies
        WHERE tenant_id = $1
        ORDER BY updated_at DESC
        "#,
    )
    .bind(tenant.tenant_id)
    .fetch_all(&state.pool)
    .await;

    match result {
        Ok(rows) => {
            let policies: Vec<serde_json::Value> = rows
                .into_iter()
                .map(|(id, title, version, status, approved, effective, cycle, created, updated)| {
                    serde_json::json!({
                        "id": id, "title": title, "version": version,
                        "status": status, "approved_by": approved,
                        "effective_date": effective, "review_cycle_days": cycle,
                        "created_at": created, "updated_at": updated,
                    })
                })
                .collect();
            (StatusCode::OK, Json(serde_json::json!({"policies": policies}))).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

async fn get_policy(
    State(state): State<GrcState>,
    Path(id): Path<Uuid>,
    Query(tenant): Query<TenantQuery>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, (Uuid, String, String, String, String, Option<Uuid>, Option<chrono::NaiveDate>, i32)>(
        r#"
        SELECT id, title, version, content, status, approved_by, effective_date, review_cycle_days
        FROM grc_policies
        WHERE id = $1 AND tenant_id = $2
        "#,
    )
    .bind(id)
    .bind(tenant.tenant_id)
    .fetch_optional(&state.pool)
    .await;

    match result {
        Ok(Some(row)) => {
            let (id, title, version, content, status, approved, effective, cycle) = row;
            (StatusCode::OK, Json(serde_json::json!({
                "id": id, "title": title, "version": version, "content": content,
                "status": status, "approved_by": approved, "effective_date": effective,
                "review_cycle_days": cycle,
            }))).into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "policy not found"}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

async fn activate_policy(
    State(state): State<GrcState>,
    Path(id): Path<Uuid>,
    Query(tenant): Query<TenantQuery>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let approved_by = body.get("approved_by")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());

    let Some(approved_by) = approved_by else {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "approved_by required"}))).into_response();
    };

    let mgr = PolicyManager::new(state.pool);
    match mgr.activate_policy(tenant.tenant_id, id, approved_by).await {
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({"status": "activated"}))).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

async fn archive_policy(
    State(state): State<GrcState>,
    Path(id): Path<Uuid>,
    Query(tenant): Query<TenantQuery>,
) -> impl IntoResponse {
    let mgr = PolicyManager::new(state.pool);
    match mgr.archive_policy(tenant.tenant_id, id).await {
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({"status": "archived"}))).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

async fn acknowledge_policy(
    State(state): State<GrcState>,
    Path(id): Path<Uuid>,
    Query(tenant): Query<TenantQuery>,
    Json(req): Json<AcknowledgePolicyRequest>,
) -> impl IntoResponse {
    let mgr = PolicyManager::new(state.pool);
    match mgr.acknowledge_policy(tenant.tenant_id, id, req.user_id).await {
        Ok(ack) => (StatusCode::OK, Json(serde_json::to_value(ack).unwrap())).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

// ============================================================
// Control Handlers
// ============================================================

async fn create_control(
    State(state): State<GrcState>,
    Query(tenant): Query<TenantQuery>,
    Json(req): Json<CreateControlRequest>,
) -> impl IntoResponse {
    let id = Uuid::now_v7();
    let now = chrono::Utc::now();

    let result = sqlx::query(
        r#"
        INSERT INTO grc_controls (id, tenant_id, title, description, framework,
            control_ref, status, evidence, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, 'not_implemented', $7, $8, $8)
        "#,
    )
    .bind(id)
    .bind(tenant.tenant_id)
    .bind(&req.title)
    .bind(&req.description)
    .bind(req.framework.to_string())
    .bind(&req.control_ref)
    .bind(&req.evidence)
    .bind(now)
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => (StatusCode::CREATED, Json(serde_json::json!({
            "id": id, "title": req.title, "framework": req.framework,
            "control_ref": req.control_ref, "status": "not_implemented",
        }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

async fn list_controls(
    State(state): State<GrcState>,
    Query(tenant): Query<TenantQuery>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, (Uuid, String, String, String, String, String, Option<String>, Option<chrono::DateTime<chrono::Utc>>)>(
        r#"
        SELECT id, title, description, framework, control_ref, status, evidence, last_assessed
        FROM grc_controls
        WHERE tenant_id = $1
        ORDER BY framework, control_ref
        "#,
    )
    .bind(tenant.tenant_id)
    .fetch_all(&state.pool)
    .await;

    match result {
        Ok(rows) => {
            let controls: Vec<serde_json::Value> = rows
                .into_iter()
                .map(|(id, title, desc, fw, ctrl_ref, status, evidence, assessed)| {
                    serde_json::json!({
                        "id": id, "title": title, "description": desc,
                        "framework": fw, "control_ref": ctrl_ref, "status": status,
                        "evidence": evidence, "last_assessed": assessed,
                    })
                })
                .collect();
            (StatusCode::OK, Json(serde_json::json!({"controls": controls}))).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

async fn get_control(
    State(state): State<GrcState>,
    Path(id): Path<Uuid>,
    Query(tenant): Query<TenantQuery>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, (Uuid, String, String, String, String, String, Option<String>, Option<chrono::DateTime<chrono::Utc>>)>(
        r#"
        SELECT id, title, description, framework, control_ref, status, evidence, last_assessed
        FROM grc_controls
        WHERE id = $1 AND tenant_id = $2
        "#,
    )
    .bind(id)
    .bind(tenant.tenant_id)
    .fetch_optional(&state.pool)
    .await;

    match result {
        Ok(Some((id, title, desc, fw, ctrl_ref, status, evidence, assessed))) => {
            (StatusCode::OK, Json(serde_json::json!({
                "id": id, "title": title, "description": desc,
                "framework": fw, "control_ref": ctrl_ref, "status": status,
                "evidence": evidence, "last_assessed": assessed,
            }))).into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "control not found"}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

async fn update_control(
    State(state): State<GrcState>,
    Path(id): Path<Uuid>,
    Query(tenant): Query<TenantQuery>,
    Json(req): Json<CreateControlRequest>,
) -> impl IntoResponse {
    let result = sqlx::query(
        r#"
        UPDATE grc_controls
        SET title = $3, description = $4, framework = $5, control_ref = $6,
            evidence = $7, updated_at = NOW()
        WHERE id = $1 AND tenant_id = $2
        "#,
    )
    .bind(id)
    .bind(tenant.tenant_id)
    .bind(&req.title)
    .bind(&req.description)
    .bind(req.framework.to_string())
    .bind(&req.control_ref)
    .bind(&req.evidence)
    .execute(&state.pool)
    .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            (StatusCode::OK, Json(serde_json::json!({"status": "updated"}))).into_response()
        }
        Ok(_) => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "control not found"}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

async fn add_control_mapping(
    State(state): State<GrcState>,
    Json(req): Json<CreateControlMappingRequest>,
) -> impl IntoResponse {
    let mapper = ControlMapper::new(state.pool);
    match mapper.add_mapping(&req).await {
        Ok(mapping) => (StatusCode::CREATED, Json(serde_json::to_value(mapping).unwrap())).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

// ============================================================
// Dashboard & Analytics Handlers
// ============================================================

async fn get_dashboard(
    State(state): State<GrcState>,
    Query(tenant): Query<TenantQuery>,
) -> impl IntoResponse {
    let today = chrono::Utc::now().date_naive();

    let stats = sqlx::query_as::<_, (i64, i64, i64, i64, i64, i64, i64, i64, i64)>(
        r#"
        SELECT
            (SELECT COUNT(*) FROM grc_risks WHERE tenant_id = $1 AND status != 'closed')::bigint,
            (SELECT COUNT(*) FROM grc_risks WHERE tenant_id = $1 AND risk_score >= 16 AND status != 'closed')::bigint,
            (SELECT COUNT(*) FROM grc_risks WHERE tenant_id = $1 AND risk_score BETWEEN 10 AND 15 AND status != 'closed')::bigint,
            (SELECT COUNT(*) FROM grc_risks WHERE tenant_id = $1 AND review_date < $2 AND status NOT IN ('closed', 'accepted'))::bigint,
            (SELECT COUNT(*) FROM grc_policies WHERE tenant_id = $1 AND status = 'active')::bigint,
            (SELECT COUNT(*) FROM grc_controls WHERE tenant_id = $1)::bigint,
            (SELECT COUNT(*) FROM grc_controls WHERE tenant_id = $1 AND status = 'implemented')::bigint,
            (SELECT COUNT(*) FROM grc_controls WHERE tenant_id = $1 AND status = 'partial')::bigint,
            (SELECT COUNT(*) FROM grc_controls WHERE tenant_id = $1 AND status = 'not_implemented')::bigint
        "#,
    )
    .bind(tenant.tenant_id)
    .bind(today)
    .fetch_one(&state.pool)
    .await;

    match stats {
        Ok((total_risks, critical, high, overdue, active_policies, total_controls, implemented, partial, not_impl)) => {
            let dashboard = GrcDashboard {
                total_risks: total_risks as u64,
                critical_risks: critical as u64,
                high_risks: high as u64,
                overdue_reviews: overdue as u64,
                active_policies: active_policies as u64,
                pending_acknowledgments: 0, // computed separately if needed
                total_controls: total_controls as u64,
                implemented_controls: implemented as u64,
                partial_controls: partial as u64,
                not_implemented_controls: not_impl as u64,
            };
            (StatusCode::OK, Json(serde_json::to_value(dashboard).unwrap())).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

async fn get_heatmap(
    State(state): State<GrcState>,
    Query(tenant): Query<TenantQuery>,
) -> impl IntoResponse {
    let engine = RiskEngine::new(state.pool);
    match engine.generate_heatmap(tenant.tenant_id).await {
        Ok(heatmap) => (StatusCode::OK, Json(serde_json::to_value(heatmap).unwrap())).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

async fn get_coverage(
    State(state): State<GrcState>,
    Path(framework): Path<String>,
    Query(tenant): Query<TenantQuery>,
) -> impl IntoResponse {
    let fw = match framework.to_uppercase().as_str() {
        "CIS" => Framework::Cis,
        "NIST" => Framework::Nist,
        "PCI-DSS" | "PCIDSS" => Framework::PciDss,
        "ISO-27001" | "ISO27001" => Framework::Iso27001,
        "SOC2" => Framework::Soc2,
        "HIPAA" => Framework::Hipaa,
        _ => {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "error": "unsupported framework",
                "supported": ["CIS", "NIST", "PCI-DSS", "ISO-27001", "SOC2", "HIPAA"]
            }))).into_response();
        }
    };

    let mapper = ControlMapper::new(state.pool);
    match mapper.get_coverage(tenant.tenant_id, fw).await {
        Ok(coverage) => (StatusCode::OK, Json(serde_json::to_value(coverage).unwrap())).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

async fn get_gap_analysis(
    State(state): State<GrcState>,
    Path(framework): Path<String>,
    Query(tenant): Query<TenantQuery>,
) -> impl IntoResponse {
    let fw = match framework.to_uppercase().as_str() {
        "CIS" => Framework::Cis,
        "NIST" => Framework::Nist,
        "PCI-DSS" | "PCIDSS" => Framework::PciDss,
        "ISO-27001" | "ISO27001" => Framework::Iso27001,
        "SOC2" => Framework::Soc2,
        "HIPAA" => Framework::Hipaa,
        _ => {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "error": "unsupported framework"
            }))).into_response();
        }
    };

    let mapper = ControlMapper::new(state.pool);
    match mapper.gap_analysis(tenant.tenant_id, fw).await {
        Ok(gaps) => (StatusCode::OK, Json(serde_json::to_value(gaps).unwrap())).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}



