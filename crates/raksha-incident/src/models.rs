//! Incident Response domain models.
//!
//! Defines the core data structures for incidents, timeline events,
//! severity levels, and status tracking.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier type (matches raksha-core convention)
pub type Id = Uuid;

// ============================================================
// Severity
// ============================================================

/// Incident severity levels ordered by impact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IncidentSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for IncidentSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

// ============================================================
// Status
// ============================================================

/// Incident lifecycle status with controlled transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IncidentStatus {
    Open,
    Investigating,
    Contained,
    Resolved,
    Closed,
}

impl std::fmt::Display for IncidentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open => write!(f, "open"),
            Self::Investigating => write!(f, "investigating"),
            Self::Contained => write!(f, "contained"),
            Self::Resolved => write!(f, "resolved"),
            Self::Closed => write!(f, "closed"),
        }
    }
}

// ============================================================
// Timeline Event Types
// ============================================================

/// Categories of timeline events that can occur during an incident.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimelineEventType {
    StatusChange,
    Comment,
    Action,
    Escalation,
    PlaybookStep,
    Assignment,
    AlertLinked,
}

impl std::fmt::Display for TimelineEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StatusChange => write!(f, "status_change"),
            Self::Comment => write!(f, "comment"),
            Self::Action => write!(f, "action"),
            Self::Escalation => write!(f, "escalation"),
            Self::PlaybookStep => write!(f, "playbook_step"),
            Self::Assignment => write!(f, "assignment"),
            Self::AlertLinked => write!(f, "alert_linked"),
        }
    }
}

// ============================================================
// Timeline Event
// ============================================================

/// A single event in an incident's timeline, providing full audit trail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentTimelineEvent {
    pub id: Id,
    pub incident_id: Id,
    pub event_type: TimelineEventType,
    pub actor: Option<Id>,
    pub description: String,
    /// Optional structured metadata (e.g., old/new status, playbook step details)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}

impl IncidentTimelineEvent {
    pub fn new(
        incident_id: Id,
        event_type: TimelineEventType,
        actor: Option<Id>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            incident_id,
            event_type,
            actor,
            description: description.into(),
            metadata: None,
            timestamp: Utc::now(),
        }
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

// ============================================================
// Incident
// ============================================================

/// Core incident record representing a security event under investigation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Incident {
    pub id: Id,
    pub title: String,
    pub description: String,
    pub severity: IncidentSeverity,
    pub status: IncidentStatus,
    pub assigned_to: Option<Id>,
    pub created_by: Id,
    /// Alert IDs that triggered or are linked to this incident.
    pub alert_ids: Vec<Id>,
    /// Timeline of all events during the incident lifecycle.
    pub timeline: Vec<IncidentTimelineEvent>,
    /// Active playbook being executed, if any.
    pub playbook_id: Option<String>,
    /// Tenant/organization scope for multi-tenancy.
    pub tenant_id: Id,
    /// Tags for categorization and filtering.
    #[serde(default)]
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
}

impl Incident {
    /// Create a new incident with Open status.
    pub fn new(
        title: impl Into<String>,
        description: impl Into<String>,
        severity: IncidentSeverity,
        created_by: Id,
        tenant_id: Id,
    ) -> Self {
        let now = Utc::now();
        let id = Uuid::now_v7();

        let mut incident = Self {
            id,
            title: title.into(),
            description: description.into(),
            severity,
            status: IncidentStatus::Open,
            assigned_to: None,
            created_by,
            alert_ids: Vec::new(),
            timeline: Vec::new(),
            playbook_id: None,
            tenant_id,
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
            resolved_at: None,
            closed_at: None,
        };

        // Record creation event
        incident.timeline.push(IncidentTimelineEvent::new(
            id,
            TimelineEventType::StatusChange,
            Some(created_by),
            format!("Incident created with severity: {severity}"),
        ));

        incident
    }

    /// Link an alert ID to this incident.
    pub fn link_alert(&mut self, alert_id: Id, actor: Option<Id>) {
        if !self.alert_ids.contains(&alert_id) {
            self.alert_ids.push(alert_id);
            self.updated_at = Utc::now();
            self.timeline.push(IncidentTimelineEvent::new(
                self.id,
                TimelineEventType::AlertLinked,
                actor,
                format!("Alert {alert_id} linked to incident"),
            ));
        }
    }

    /// Add a comment to the incident timeline.
    pub fn add_comment(&mut self, actor: Id, comment: impl Into<String>) {
        self.updated_at = Utc::now();
        self.timeline.push(IncidentTimelineEvent::new(
            self.id,
            TimelineEventType::Comment,
            Some(actor),
            comment,
        ));
    }
}

// ============================================================
// API Request/Response Types
// ============================================================

/// Request to create a new incident.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateIncidentRequest {
    pub title: String,
    pub description: String,
    pub severity: IncidentSeverity,
    pub alert_ids: Option<Vec<Id>>,
    pub playbook_id: Option<String>,
    pub tags: Option<Vec<String>>,
    pub assigned_to: Option<Id>,
}

/// Request to update an incident's status.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateStatusRequest {
    pub status: IncidentStatus,
    pub reason: Option<String>,
}

/// Request to assign an incident to a user.
#[derive(Debug, Clone, Deserialize)]
pub struct AssignIncidentRequest {
    pub assigned_to: Id,
}

/// Request to add a timeline event.
#[derive(Debug, Clone, Deserialize)]
pub struct AddTimelineEventRequest {
    pub event_type: TimelineEventType,
    pub description: String,
    pub metadata: Option<serde_json::Value>,
}

/// Filter for listing incidents.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct IncidentFilter {
    pub severity: Option<IncidentSeverity>,
    pub status: Option<IncidentStatus>,
    pub assigned_to: Option<Id>,
    pub tenant_id: Option<Id>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
}

/// Summary view of an incident for list responses.
#[derive(Debug, Clone, Serialize)]
pub struct IncidentSummary {
    pub id: Id,
    pub title: String,
    pub severity: IncidentSeverity,
    pub status: IncidentStatus,
    pub assigned_to: Option<Id>,
    pub alert_count: usize,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&Incident> for IncidentSummary {
    fn from(incident: &Incident) -> Self {
        Self {
            id: incident.id,
            title: incident.title.clone(),
            severity: incident.severity,
            status: incident.status,
            assigned_to: incident.assigned_to,
            alert_count: incident.alert_ids.len(),
            created_at: incident.created_at,
            updated_at: incident.updated_at,
        }
    }
}
