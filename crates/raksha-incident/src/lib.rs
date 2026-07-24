//! Raksha Incident Response Playbook Engine
//!
//! Provides incident lifecycle management, automated response playbooks,
//! escalation rules, and timeline tracking for security incidents.

pub mod handlers;
pub mod lifecycle;
pub mod models;
pub mod playbook;

pub use lifecycle::IncidentStateMachine;
pub use models::{Incident, IncidentSeverity, IncidentStatus, IncidentTimelineEvent};
pub use playbook::{Playbook, PlaybookEngine, PlaybookStep};
