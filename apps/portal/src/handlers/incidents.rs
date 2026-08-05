//! Incident response endpoints: incident records, timeline and tasks.

use axum::{
    extract::{Path, Query, State},
    routing::{get, patch},
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

#[derive(Debug, Serialize, sqlx::FromRow)]
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
    let incidents = sqlx::query_as::<_, IncidentResponse>(
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
        "#
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(&state.db)
    .await?;

    let total: i64 = sqlx::query_scalar(r#"SELECT COUNT(*) FROM incidents"#)
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

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
    let incident = sqlx::query_as::<_, IncidentResponse>(
        r#"
        SELECT id, incident_number, title, description, severity, status,
               priority, category, classification, commander_id, assigned_team,
               affected_systems, affected_users_count, impact_scope,
               mitre_tactics, mitre_techniques, attack_vector, root_cause,
               sla_breached, first_detected_at, first_response_at,
               contained_at, closed_at, created_at, updated_at
        FROM incidents WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Incident not found".to_string()))?;

    Ok(Json(incident))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct TimelineEvent {
    id: Uuid,
    incident_id: Uuid,
    event_type: String,
    title: String,
    description: Option<String>,
    actor_id: Option<Uuid>,
    occurred_at: DateTime<Utc>,
}

async fn get_timeline(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<Vec<TimelineEvent>>> {
    let events = sqlx::query_as::<_, TimelineEvent>(
        r#"
        SELECT id, incident_id, event_type, title, description, actor_id, occurred_at
        FROM incident_timeline
        WHERE incident_id = $1
        ORDER BY occurred_at DESC
        "#
    )
    .bind(id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(events))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct IncidentTask {
    id: Uuid,
    incident_id: Uuid,
    title: String,
    description: Option<String>,
    status: String,
    priority: String,
    assignee_id: Option<Uuid>,
    due_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

async fn get_tasks(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<Vec<IncidentTask>>> {
    let tasks = sqlx::query_as::<_, IncidentTask>(
        r#"
        SELECT id, incident_id, title, description, status, priority,
               assignee_id, due_at, completed_at, created_at
        FROM incident_tasks
        WHERE incident_id = $1
        ORDER BY priority, created_at
        "#
    )
    .bind(id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(tasks))
}

#[derive(Debug, Serialize)]
struct IncidentSummary {
    total: i64,
    open: i64,
    in_progress: i64,
    contained: i64,
    closed: i64,
    critical: i64,
    high: i64,
    sla_breached: i64,
}

#[derive(Debug, sqlx::FromRow)]
struct IncidentSummaryRow {
    total: i64,
    open: i64,
    in_progress: i64,
    contained: i64,
    closed: i64,
    critical: i64,
    high: i64,
    sla_breached: i64,
}

async fn incident_summary(
    State(state): State<AppState>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<IncidentSummary>> {
    let row: IncidentSummaryRow = sqlx::query_as(
        r#"
        SELECT
            COUNT(*)::bigint as total,
            COUNT(*) FILTER (WHERE status = 'open')::bigint as open,
            COUNT(*) FILTER (WHERE status = 'in_progress')::bigint as in_progress,
            COUNT(*) FILTER (WHERE status = 'contained')::bigint as contained,
            COUNT(*) FILTER (WHERE status = 'closed')::bigint as closed,
            COUNT(*) FILTER (WHERE severity = 'critical')::bigint as critical,
            COUNT(*) FILTER (WHERE severity = 'high')::bigint as high,
            COUNT(*) FILTER (WHERE sla_breached)::bigint as sla_breached
        FROM incidents
        "#
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(IncidentSummaryRow {
        total: 0, open: 0, in_progress: 0, contained: 0, closed: 0,
        critical: 0, high: 0, sla_breached: 0,
    });

    Ok(Json(IncidentSummary {
        total: row.total,
        open: row.open,
        in_progress: row.in_progress,
        contained: row.contained,
        closed: row.closed,
        critical: row.critical,
        high: row.high,
        sla_breached: row.sla_breached,
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

fn default_severity() -> String { "medium".to_string() }
fn default_priority() -> String { "medium".to_string() }

async fn create_incident(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(payload): Json<CreateIncidentRequest>,
) -> AppResult<Json<IncidentResponse>> {
    if !claims.role.has_permission(&UserRole::Operator) {
        return Err(AppError::Forbidden(
            "Operator access required to create incidents".to_string(),
        ));
    }
    if payload.title.trim().is_empty() {
        return Err(AppError::Validation("Incident title is required".to_string()));
    }

    let id = new_id();
    let incident_number = format!("INC-{}", &id.to_string()[..8].to_uppercase());

    let incident = sqlx::query_as::<_, IncidentResponse>(
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
        "#
    )
    .bind(id)
    .bind(&incident_number)
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(&payload.severity)
    .bind(&payload.priority)
    .bind(&payload.category)
    .bind(&payload.impact_scope)
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
        sqlx::query_as::<_, IncidentResponse>(
            r#"
            UPDATE incidents SET status = $2, closed_at = NOW(), updated_at = NOW()
            WHERE id = $1
            RETURNING id, incident_number, title, description, severity, status,
                      priority, category, classification, commander_id, assigned_team,
                      affected_systems, affected_users_count, impact_scope,
                      mitre_tactics, mitre_techniques, attack_vector, root_cause,
                      sla_breached, first_detected_at, first_response_at,
                      contained_at, closed_at, created_at, updated_at
            "#
        )
        .bind(id)
        .bind(&status)
        .fetch_optional(&state.db)
        .await?
    } else {
        sqlx::query_as::<_, IncidentResponse>(
            r#"
            UPDATE incidents SET status = $2, updated_at = NOW()
            WHERE id = $1
            RETURNING id, incident_number, title, description, severity, status,
                      priority, category, classification, commander_id, assigned_team,
                      affected_systems, affected_users_count, impact_scope,
                      mitre_tactics, mitre_techniques, attack_vector, root_cause,
                      sla_breached, first_detected_at, first_response_at,
                      contained_at, closed_at, created_at, updated_at
            "#
        )
        .bind(id)
        .bind(&status)
        .fetch_optional(&state.db)
        .await?
    }
    .ok_or(AppError::NotFound("Incident not found".to_string()))?;

    Ok(Json(incident))
}
