//! Axum HTTP handlers for the Incident Response API.
//!
//! Provides REST endpoints for incident CRUD, status transitions,
//! assignment, timeline management, and playbook execution.

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::lifecycle::IncidentStateMachine;
use crate::models::*;
use crate::playbook::{PlaybookEngine, PlaybookExecution};

// ============================================================
// Application State
// ============================================================

/// Shared application state for the incident module.
/// In production this would use a database; here we use in-memory storage
/// to keep the crate self-contained without requiring a running PostgreSQL.
#[derive(Debug, Clone)]
pub struct IncidentState {
    pub incidents: Arc<RwLock<Vec<Incident>>>,
    pub executions: Arc<RwLock<Vec<PlaybookExecution>>>,
    pub playbook_engine: Arc<RwLock<PlaybookEngine>>,
}

impl IncidentState {
    pub fn new(engine: PlaybookEngine) -> Self {
        Self {
            incidents: Arc::new(RwLock::new(Vec::new())),
            executions: Arc::new(RwLock::new(Vec::new())),
            playbook_engine: Arc::new(RwLock::new(engine)),
        }
    }
}

// ============================================================
// Handlers
// ============================================================

/// POST /incidents - Create a new incident.
pub async fn create_incident(
    State(state): State<IncidentState>,
    Json(req): Json<CreateIncidentRequest>,
) -> impl IntoResponse {
    // Use a placeholder tenant/user ID; in production these come from auth middleware
    let created_by = Uuid::nil();
    let tenant_id = Uuid::nil();

    let mut incident = Incident::new(req.title, req.description, req.severity, created_by, tenant_id);

    if let Some(alert_ids) = req.alert_ids {
        for aid in alert_ids {
            incident.link_alert(aid, Some(created_by));
        }
    }

    if let Some(tags) = req.tags {
        incident.tags = tags;
    }

    if let Some(assignee) = req.assigned_to {
        incident.assigned_to = Some(assignee);
        incident.timeline.push(IncidentTimelineEvent::new(
            incident.id,
            TimelineEventType::Assignment,
            Some(created_by),
            format!("Assigned to {assignee}"),
        ));
    }

    incident.playbook_id = req.playbook_id;

    let id = incident.id;
    state.incidents.write().await.push(incident);

    tracing::info!(incident_id = %id, "Incident created");
    (StatusCode::CREATED, Json(serde_json::json!({ "id": id })))
}

/// GET /incidents/:id - Get incident details.
pub async fn get_incident(
    State(state): State<IncidentState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let incidents = state.incidents.read().await;
    match incidents.iter().find(|i| i.id == id) {
        Some(incident) => Ok(Json(incident.clone())),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": format!("Incident {id} not found") })),
        )),
    }
}

/// GET /incidents - List incidents with optional filters.
pub async fn list_incidents(
    State(state): State<IncidentState>,
    Query(filter): Query<IncidentFilter>,
) -> impl IntoResponse {
    let incidents = state.incidents.read().await;

    let filtered: Vec<IncidentSummary> = incidents
        .iter()
        .filter(|i| filter.severity.map_or(true, |s| i.severity == s))
        .filter(|i| filter.status.map_or(true, |s| i.status == s))
        .filter(|i| filter.assigned_to.map_or(true, |a| i.assigned_to == Some(a)))
        .filter(|i| filter.tenant_id.map_or(true, |t| i.tenant_id == t))
        .filter(|i| filter.from_date.map_or(true, |d| i.created_at >= d))
        .filter(|i| filter.to_date.map_or(true, |d| i.created_at <= d))
        .map(IncidentSummary::from)
        .collect();

    Json(serde_json::json!({
        "data": filtered,
        "total": filtered.len(),
    }))
}

/// PUT /incidents/:id/status - Update incident status.
pub async fn update_status(
    State(state): State<IncidentState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateStatusRequest>,
) -> impl IntoResponse {
    let actor = Uuid::nil(); // from auth middleware in production
    let mut incidents = state.incidents.write().await;

    let incident = match incidents.iter_mut().find(|i| i.id == id) {
        Some(i) => i,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": format!("Incident {id} not found") })),
            ));
        }
    };

    match IncidentStateMachine::transition(incident, req.status, actor, req.reason.as_deref()) {
        Ok(()) => Ok(Json(serde_json::json!({
            "id": id,
            "status": incident.status,
        }))),
        Err(e) => Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

/// PUT /incidents/:id/assign - Assign incident to a user.
pub async fn assign_incident(
    State(state): State<IncidentState>,
    Path(id): Path<Uuid>,
    Json(req): Json<AssignIncidentRequest>,
) -> impl IntoResponse {
    let actor = Uuid::nil();
    let mut incidents = state.incidents.write().await;

    let incident = match incidents.iter_mut().find(|i| i.id == id) {
        Some(i) => i,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": format!("Incident {id} not found") })),
            ));
        }
    };

    incident.assigned_to = Some(req.assigned_to);
    incident.updated_at = chrono::Utc::now();
    incident.timeline.push(IncidentTimelineEvent::new(
        incident.id,
        TimelineEventType::Assignment,
        Some(actor),
        format!("Assigned to {}", req.assigned_to),
    ));

    tracing::info!(incident_id = %id, assigned_to = %req.assigned_to, "Incident assigned");
    Ok(Json(serde_json::json!({
        "id": id,
        "assigned_to": req.assigned_to,
    })))
}

/// POST /incidents/:id/timeline - Add a timeline event.
pub async fn add_timeline_event(
    State(state): State<IncidentState>,
    Path(id): Path<Uuid>,
    Json(req): Json<AddTimelineEventRequest>,
) -> impl IntoResponse {
    let actor = Uuid::nil();
    let mut incidents = state.incidents.write().await;

    let incident = match incidents.iter_mut().find(|i| i.id == id) {
        Some(i) => i,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": format!("Incident {id} not found") })),
            ));
        }
    };

    let mut event = IncidentTimelineEvent::new(incident.id, req.event_type, Some(actor), req.description);
    if let Some(meta) = req.metadata {
        event = event.with_metadata(meta);
    }

    let event_id = event.id;
    incident.timeline.push(event);
    incident.updated_at = chrono::Utc::now();

    Ok(Json(serde_json::json!({ "event_id": event_id })))
}

/// POST /incidents/:id/playbook/:playbook_id - Execute a playbook against an incident.
pub async fn execute_playbook(
    State(state): State<IncidentState>,
    Path((id, playbook_id)): Path<(Uuid, String)>,
) -> impl IntoResponse {
    let actor = Uuid::nil();

    // Verify incident exists
    {
        let incidents = state.incidents.read().await;
        if !incidents.iter().any(|i| i.id == id) {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": format!("Incident {id} not found") })),
            ));
        }
    }

    // Start playbook execution
    let engine = state.playbook_engine.read().await;
    let execution = match engine.start_execution(&playbook_id, id, actor) {
        Ok(exec) => exec,
        Err(e) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": e.to_string() })),
            ));
        }
    };

    let exec_id = execution.id;

    // Record in incident timeline
    {
        let mut incidents = state.incidents.write().await;
        if let Some(incident) = incidents.iter_mut().find(|i| i.id == id) {
            incident.playbook_id = Some(playbook_id.clone());
            incident.updated_at = chrono::Utc::now();
            incident.timeline.push(IncidentTimelineEvent::new(
                id,
                TimelineEventType::PlaybookStep,
                Some(actor),
                format!("Playbook '{playbook_id}' execution started"),
            ));
        }
    }

    state.executions.write().await.push(execution);

    Ok(Json(serde_json::json!({
        "execution_id": exec_id,
        "playbook_id": playbook_id,
        "incident_id": id,
        "status": "not_started",
    })))
}

/// Request for playbook suggestions.
#[derive(Debug, serde::Deserialize)]
pub struct SuggestPlaybookQuery {
    pub alert_type: String,
    pub severity: crate::models::IncidentSeverity,
    #[serde(default)]
    pub description: String,
}

/// GET /playbooks/suggest - Get playbook suggestions for an alert.
pub async fn get_playbook_suggestions(
    State(state): State<IncidentState>,
    Query(query): Query<SuggestPlaybookQuery>,
) -> impl IntoResponse {
    let engine = state.playbook_engine.read().await;
    let suggestions = engine.suggest(&query.alert_type, query.severity, &query.description);

    let results: Vec<serde_json::Value> = suggestions
        .iter()
        .map(|pb| {
            serde_json::json!({
                "id": pb.id,
                "name": pb.name,
                "description": pb.description,
                "steps_count": pb.steps.len(),
                "tags": pb.tags,
            })
        })
        .collect();

    Json(serde_json::json!({
        "suggestions": results,
        "total": results.len(),
    }))
}

// ============================================================
// Router
// ============================================================

/// Build the Axum router for incident endpoints.
pub fn router(state: IncidentState) -> axum::Router {
    use axum::routing::{get, post, put};

    axum::Router::new()
        .route("/incidents", post(create_incident).get(list_incidents))
        .route("/incidents/:id", get(get_incident))
        .route("/incidents/:id/status", put(update_status))
        .route("/incidents/:id/assign", put(assign_incident))
        .route("/incidents/:id/timeline", post(add_timeline_event))
        .route("/incidents/:id/playbook/:playbook_id", post(execute_playbook))
        .route("/playbooks/suggest", get(get_playbook_suggestions))
        .with_state(state)
}
