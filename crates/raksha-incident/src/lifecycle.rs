//! Incident lifecycle state machine.
//!
//! Enforces valid status transitions and provides auto-escalation rules
//! for incidents that are not acknowledged within configured timeframes.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::models::{
    Id, Incident, IncidentSeverity, IncidentStatus, IncidentTimelineEvent, TimelineEventType,
};

// ============================================================
// Transition Validation
// ============================================================

/// State machine that governs allowed incident status transitions.
#[derive(Debug, Clone)]
pub struct IncidentStateMachine;

/// Error when an invalid transition is attempted.
#[derive(Debug, thiserror::Error)]
pub enum LifecycleError {
    #[error("Invalid transition from {from} to {to}")]
    InvalidTransition {
        from: IncidentStatus,
        to: IncidentStatus,
    },

    #[error("Incident is already closed")]
    AlreadyClosed,

    #[error("Escalation error: {0}")]
    EscalationError(String),
}

impl IncidentStateMachine {
    /// Returns the set of valid next statuses from a given status.
    pub fn allowed_transitions(from: IncidentStatus) -> &'static [IncidentStatus] {
        match from {
            IncidentStatus::Open => &[
                IncidentStatus::Investigating,
                IncidentStatus::Closed, // can close without investigating (false positive)
            ],
            IncidentStatus::Investigating => &[
                IncidentStatus::Contained,
                IncidentStatus::Resolved,
                IncidentStatus::Open, // revert if misclassified
            ],
            IncidentStatus::Contained => &[
                IncidentStatus::Resolved,
                IncidentStatus::Investigating, // revert if containment failed
            ],
            IncidentStatus::Resolved => &[
                IncidentStatus::Closed,
                IncidentStatus::Investigating, // reopen if issue recurs
            ],
            IncidentStatus::Closed => &[], // terminal state
        }
    }

    /// Check if a transition is valid.
    pub fn can_transition(from: IncidentStatus, to: IncidentStatus) -> bool {
        Self::allowed_transitions(from).contains(&to)
    }

    /// Attempt to transition an incident to a new status.
    pub fn transition(
        incident: &mut Incident,
        new_status: IncidentStatus,
        actor: Id,
        reason: Option<&str>,
    ) -> Result<(), LifecycleError> {
        if incident.status == IncidentStatus::Closed {
            return Err(LifecycleError::AlreadyClosed);
        }

        if !Self::can_transition(incident.status, new_status) {
            return Err(LifecycleError::InvalidTransition {
                from: incident.status,
                to: new_status,
            });
        }

        let old_status = incident.status;
        incident.status = new_status;
        incident.updated_at = Utc::now();

        // Set resolution/closure timestamps
        match new_status {
            IncidentStatus::Resolved => {
                incident.resolved_at = Some(Utc::now());
            }
            IncidentStatus::Closed => {
                incident.closed_at = Some(Utc::now());
            }
            _ => {}
        }

        // Record timeline event
        let desc = match reason {
            Some(r) => format!("Status changed: {old_status} → {new_status}. Reason: {r}"),
            None => format!("Status changed: {old_status} → {new_status}"),
        };

        let event = IncidentTimelineEvent::new(
            incident.id,
            TimelineEventType::StatusChange,
            Some(actor),
            desc,
        )
        .with_metadata(serde_json::json!({
            "old_status": old_status.to_string(),
            "new_status": new_status.to_string(),
        }));

        incident.timeline.push(event);

        tracing::info!(
            incident_id = %incident.id,
            old_status = %old_status,
            new_status = %new_status,
            actor = %actor,
            "Incident status transitioned"
        );

        Ok(())
    }
}

// ============================================================
// Auto-Escalation Rules
// ============================================================

/// Configuration for auto-escalation based on severity and time thresholds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationRule {
    /// Severity level this rule applies to.
    pub severity: IncidentSeverity,
    /// Minutes before escalation triggers if incident is still in Open status.
    pub escalate_after_minutes: i64,
    /// Target severity to escalate to (if escalating severity).
    pub escalate_to_severity: Option<IncidentSeverity>,
    /// Notify these user/group IDs on escalation.
    #[serde(default)]
    pub notify_ids: Vec<Id>,
    /// Description of the escalation action.
    pub description: String,
}

/// Default escalation rules per severity level.
pub fn default_escalation_rules() -> Vec<EscalationRule> {
    vec![
        EscalationRule {
            severity: IncidentSeverity::Critical,
            escalate_after_minutes: 5,
            escalate_to_severity: None,
            notify_ids: Vec::new(),
            description: "Critical incident not acknowledged within 5 minutes".into(),
        },
        EscalationRule {
            severity: IncidentSeverity::High,
            escalate_after_minutes: 15,
            escalate_to_severity: Some(IncidentSeverity::Critical),
            notify_ids: Vec::new(),
            description: "High severity incident escalated to Critical after 15 minutes".into(),
        },
        EscalationRule {
            severity: IncidentSeverity::Medium,
            escalate_after_minutes: 60,
            escalate_to_severity: Some(IncidentSeverity::High),
            notify_ids: Vec::new(),
            description: "Medium severity incident escalated to High after 1 hour".into(),
        },
        EscalationRule {
            severity: IncidentSeverity::Low,
            escalate_after_minutes: 240,
            escalate_to_severity: Some(IncidentSeverity::Medium),
            notify_ids: Vec::new(),
            description: "Low severity incident escalated to Medium after 4 hours".into(),
        },
    ]
}

/// Result of checking escalation rules against an incident.
#[derive(Debug, Clone, Serialize)]
pub struct EscalationAction {
    pub incident_id: Id,
    pub rule: String,
    pub new_severity: Option<IncidentSeverity>,
    pub notify_ids: Vec<Id>,
}

/// Check if an incident should be escalated based on configured rules.
pub fn check_escalation(
    incident: &Incident,
    rules: &[EscalationRule],
    now: DateTime<Utc>,
) -> Option<EscalationAction> {
    // Only escalate incidents in Open status (not yet acknowledged)
    if incident.status != IncidentStatus::Open {
        return None;
    }

    let rule = rules.iter().find(|r| r.severity == incident.severity)?;
    let threshold = Duration::minutes(rule.escalate_after_minutes);
    let elapsed = now - incident.created_at;

    if elapsed >= threshold {
        tracing::warn!(
            incident_id = %incident.id,
            severity = %incident.severity,
            elapsed_minutes = elapsed.num_minutes(),
            "Incident escalation triggered"
        );

        Some(EscalationAction {
            incident_id: incident.id,
            rule: rule.description.clone(),
            new_severity: rule.escalate_to_severity,
            notify_ids: rule.notify_ids.clone(),
        })
    } else {
        None
    }
}

/// Apply an escalation action to an incident.
pub fn apply_escalation(incident: &mut Incident, action: &EscalationAction) {
    if let Some(new_severity) = action.new_severity {
        incident.severity = new_severity;
    }
    incident.updated_at = Utc::now();

    incident.timeline.push(IncidentTimelineEvent::new(
        incident.id,
        TimelineEventType::Escalation,
        None, // system-initiated
        format!("Auto-escalation: {}", action.rule),
    ).with_metadata(serde_json::json!({
        "new_severity": action.new_severity.map(|s| s.to_string()),
        "notify_ids": action.notify_ids,
    })));
}
