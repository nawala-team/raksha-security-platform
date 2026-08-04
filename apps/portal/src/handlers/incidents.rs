//! Incident response endpoints: incident records, timeline and tasks.

use axum::{
    extract::{Path, Query, State},
    routing::{get, patch, post},
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
        .route("/", get(list_incidents).post(create_incident))
        .route("/summary", get(incident_summary))
        .route("/:id", get(get_incident))
        .route("/:id/status", patch(update_incident_status))
        .route("/:id/timeline", get(get_timeline))
        .route("/:id/tasks", get(get_tasks))
}

#[derive(Debug, Serialize)]
struct IncidentResponse {
    id: Uuid,
    incident_number: String,
    title: String,
    description: Option<String>,
    severity: String,
    status: String,
    priority: String,
    category: Option<String>,
    classification: Option<String>,
    commander_id: Option<Uuid>,
    assigned_team: Option<String>,
    affected_systems: Option<serde_json::Value>,
    affected_users_count: Option<i32>,
    impact_scope: Option<String>,
    mitre_tactics: Option<serde_json::Value>,
    mitre_techniques: Option<serde_json::Value>,
    attack_vector: Option<String>,
    root_cause: Option<String>,
    sla_breached: bool,
    first_detected_at: Option<DateTime<Utc>>,
    first_response_at: Option<DateTime<Utc>>,
    contained_at: Option<DateTime<Utc>>,
    closed_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

async fn list_incidents(
    State(state): State<AppState>,
    Query(pagination): Query<Pagination>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<PaginatedResponse<IncidentResponse>>> {
    let incidents = sqlx::query_as!(
        IncidentResponse,
        r#"
        SELECT id, incident_number, title, description, severity, status,
               priority, category, classification, commander_id, assigned_team,
               affected_systems, affected_users_count, impact_scope,
               mitre_tactics, mitre_techniques, attack_vector, root_cause,
               sla_breached, first_detected_at, first_response_at,
               contained_at, closed_at, created_at, updated_at
        FROM incidents
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
        pagination.limit(),
        pagination.offset(),
    )
    .fetch_all(&state.db)
    .await?;

    let total = sqlx::query_scalar!(r#"SELECT COUNT(*) as "count!" FROM incidents"#)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(PaginatedResponse {
        data: incidents,
        meta: PaginationMeta {
            page: pagination.page,
            per_page: pagination.per_page,
            total,
            total_pages: ((total as f64) / (pagination.limit() as f64)).ceil() as u32,
        },
    }))
}

async fn get_incident(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<IncidentResponse>> {
    let incident = sqlx::query_as!(
        IncidentResponse,
        r#"
        SELECT id, incident_number, title, description, severity, status,
               priority, category, classification, commander_id, assigned_team,
               affected_systems, affected_users_count, impact_scope,
               mitre_tactics, mitre_techniques, attack_vector, root_cause,
               sla_breached, first_detected_at, first_response_at,
               contained_at, closed_at, created_at, updated_at
        FROM incidents WHERE id = $1
        "#,
        id,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Incident not found".to_string()))?;

    Ok(Json(incident))
}

#[derive(Debug, Serialize)]
struct TimelineEntry {
    id: Uuid,
    incident_id: Uuid,
    actor_id: Option<Uuid>,
    event_type: String,
    title: String,
    content: Option<String>,
    is_automated: bool,
    occurred_at: DateTime<Utc>,
}

async fn get_timeline(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<Vec<TimelineEntry>>> {
    let entries = sqlx::query_as!(
        TimelineEntry,
        r#"
        SELECT id, incident_id, actor_id, event_type, title, content,
               is_automated, occurred_at
        FROM incident_timeline
        WHERE incident_id = $1
        ORDER BY occurred_at DESC
        "#,
        id,
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(entries))
}

#[derive(Debug, Serialize)]
struct IncidentTask {
    id: Uuid,
    incident_id: Uuid,
    title: String,
    description: Option<String>,
    status: String,
    priority: String,
    assigned_to: Option<Uuid>,
    due_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
    completed_by: Option<Uuid>,
    created_at: DateTime<Utc>,
}

async fn get_tasks(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<Vec<IncidentTask>>> {
    let tasks = sqlx::query_as!(
        IncidentTask,
        r#"
        SELECT id, incident_id, title, description, status, priority,
               assigned_to, due_at, completed_at, completed_by, created_at
        FROM incident_tasks
        WHERE incident_id = $1
        ORDER BY created_at
        "#,
        id,
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(tasks))
}

#[derive(Debug, Serialize)]
struct IncidentSummary {
    total: i64,
    open: i64,
    critical: i64,
    sla_breached: i64,
    unassigned: i64,
    closed_last_30d: i64,
    /// Mean time to first response, in minutes, over responded incidents.
    mttr_minutes: Option<f64>,
}

async fn incident_summary(
    State(state): State<AppState>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<IncidentSummary>> {
    let cutoff = Utc::now() - chrono::Duration::days(30);

    let row = sqlx::query!(
        r#"
        SELECT
            COUNT(*) as "total!",
            COUNT(*) FILTER (WHERE status NOT IN ('closed', 'resolved')) as "open!",
            COUNT(*) FILTER (WHERE severity = 'critical') as "critical!",
            COUNT(*) FILTER (WHERE sla_breached) as "breached!",
            COUNT(*) FILTER (WHERE commander_id IS NULL) as "unassigned!",
            COUNT(*) FILTER (WHERE closed_at IS NOT NULL AND closed_at >= $1) as "closed_30d!",
            AVG(
                EXTRACT(EPOCH FROM (first_response_at - first_detected_at)) / 60.0
            ) FILTER (
                WHERE first_response_at IS NOT NULL AND first_detected_at IS NOT NULL
            )::float8 as "mttr"
        FROM incidents
        "#,
        cutoff,
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(IncidentSummary {
        total: row.total,
        open: row.open,
        critical: row.critical,
        sla_breached: row.breached,
        unassigned: row.unassigned,
        closed_last_30d: row.closed_30d,
        mttr_minutes: row.mttr.map(|v| v.round()),
    }))
}

#[derive(Debug, Deserialize)]
struct CreateIncidentRequest {
    title: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default = "default_severity")]
    severity: String,
    #[serde(default = "default_priority")]
    priority: String,
    #[serde(default)]
    category: Option<String>,
    #[serde(default)]
    impact_scope: Option<String>,
}

fn default_severity() -> String {
    "medium".to_string()
}
fn default_priority() -> String {
    "medium".to_string()
}

async fn create_incident(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(payload): Json<CreateIncidentRequest>,
) -> AppResult<Json<IncidentResponse>> {
    // Operator or higher may create incidents.
    if !claims.role.has_permission(&UserRole::Operator) {
        return Err(AppError::Forbidden(
            "Operator access required to create incidents".to_string(),
        ));
    }
    if payload.title.trim().is_empty() {
        return Err(AppError::Validation("Incident title is required".to_string()));
    }

    let id = new_id();
    // Keep within the incident_number VARCHAR(20) limit.
    let incident_number = format!("INC-{}", &id.to_string()[..8].to_uppercase());

    let incident = sqlx::query_as!(
        IncidentResponse,
        r#"
        INSERT INTO incidents (id, incident_number, title, description, severity, status, priority,
                               category, impact_scope, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, 'open', $6, $7, $8, NOW(), NOW())
        RETURNING id, incident_number, title, description, severity, status,
                  priority, category, classification, commander_id, assigned_team,
                  affected_systems, affected_users_count, impact_scope,
                  mitre_tactics, mitre_techniques, attack_vector, root_cause,
                  sla_breached, first_detected_at, first_response_at,
                  contained_at, closed_at, created_at, updated_at
        "#,
        id,
        incident_number,
        payload.title,
        payload.description,
        payload.severity,
        payload.priority,
        payload.category,
        payload.impact_scope,
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(incident))
}

#[derive(Debug, Deserialize)]
struct UpdateIncidentStatusRequest {
    status: String,
}

async fn update_incident_status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(payload): Json<UpdateIncidentStatusRequest>,
) -> AppResult<Json<IncidentResponse>> {
    if !claims.role.has_permission(&UserRole::Operator) {
        return Err(AppError::Forbidden(
            "Operator access required to update incidents".to_string(),
        ));
    }

    let status = payload.status;
    let incident = if status == "closed" {
        sqlx::query_as!(
            IncidentResponse,
            r#"
            UPDATE incidents SET status = $2, closed_at = NOW(), updated_at = NOW()
            WHERE id = $1
            RETURNING id, incident_number, title, description, severity, status,
                      priority, category, classification, commander_id, assigned_team,
                      affected_systems, affected_users_count, impact_scope,
                      mitre_tactics, mitre_techniques, attack_vector, root_cause,
                      sla_breached, first_detected_at, first_response_at,
                      contained_at, closed_at, created_at, updated_at
            "#,
            id,
            status,
        )
        .fetch_optional(&state.db)
        .await?
    } else {
        sqlx::query_as!(
            IncidentResponse,
            r#"
            UPDATE incidents SET status = $2, updated_at = NOW()
            WHERE id = $1
            RETURNING id, incident_number, title, description, severity, status,
                      priority, category, classification, commander_id, assigned_team,
                      affected_systems, affected_users_count, impact_scope,
                      mitre_tactics, mitre_techniques, attack_vector, root_cause,
                      sla_breached, first_detected_at, first_response_at,
                      contained_at, closed_at, created_at, updated_at
            "#,
            id,
            status,
        )
        .fetch_optional(&state.db)
        .await?
    }
    .ok_or(AppError::NotFound("Incident not found".to_string()))?;

    Ok(Json(incident))
}
