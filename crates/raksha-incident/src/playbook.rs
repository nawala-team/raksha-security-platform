//! Playbook engine for automated incident response.
//!
//! Supports loading playbooks from YAML, matching them to alerts based on
//! trigger conditions, and tracking step-by-step execution progress.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use uuid::Uuid;

use crate::models::{Id, IncidentSeverity};

// ============================================================
// Playbook Definitions
// ============================================================

/// Type of action a playbook step performs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    /// Requires human intervention
    Manual,
    /// Runs automatically via command/script
    Automated,
    /// Requires explicit approval before proceeding
    Approval,
}

/// A single step within a playbook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookStep {
    /// Execution order (1-based)
    pub order: u32,
    pub title: String,
    pub description: String,
    pub action_type: ActionType,
    /// Command or script to execute for automated steps
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    /// Timeout in seconds for this step (default: 300)
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    /// Whether failure of this step should halt the playbook
    #[serde(default)]
    pub fail_fast: bool,
}

fn default_timeout() -> u64 {
    300
}

/// Conditions that trigger a playbook suggestion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerCondition {
    /// Alert source types that match (e.g., "ransomware", "brute_force")
    #[serde(default)]
    pub alert_types: Vec<String>,
    /// Minimum severity to trigger
    #[serde(default)]
    pub min_severity: Option<IncidentSeverity>,
    /// Keyword patterns in alert title/description
    #[serde(default)]
    pub keywords: Vec<String>,
}

/// A complete response playbook definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playbook {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub trigger_conditions: Vec<TriggerCondition>,
    pub steps: Vec<PlaybookStep>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

// ============================================================
// Execution Tracking
// ============================================================

/// Status of a single playbook step execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
    AwaitingApproval,
}

/// Tracks execution state of a single step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepExecution {
    pub step_order: u32,
    pub status: StepStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub output: Option<String>,
    pub error: Option<String>,
    pub approved_by: Option<Id>,
}

/// Overall playbook execution state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    NotStarted,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

/// Tracks the full execution of a playbook against an incident.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookExecution {
    pub id: Id,
    pub incident_id: Id,
    pub playbook_id: String,
    pub status: ExecutionStatus,
    pub current_step: u32,
    pub steps: Vec<StepExecution>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub started_by: Id,
}

impl PlaybookExecution {
    /// Create a new execution tracker for a playbook.
    pub fn new(incident_id: Id, playbook: &Playbook, started_by: Id) -> Self {
        let steps = playbook
            .steps
            .iter()
            .map(|s| StepExecution {
                step_order: s.order,
                status: StepStatus::Pending,
                started_at: None,
                completed_at: None,
                output: None,
                error: None,
                approved_by: None,
            })
            .collect();

        Self {
            id: Uuid::now_v7(),
            incident_id,
            playbook_id: playbook.id.clone(),
            status: ExecutionStatus::NotStarted,
            current_step: 0,
            steps,
            started_at: Utc::now(),
            completed_at: None,
            started_by,
        }
    }

    /// Advance to the next step. Returns the step order or None if complete.
    pub fn advance(&mut self) -> Option<u32> {
        if self.status == ExecutionStatus::Cancelled || self.status == ExecutionStatus::Failed {
            return None;
        }

        let next = self.current_step + 1;
        if (next as usize) <= self.steps.len() {
            self.current_step = next;
            self.status = ExecutionStatus::InProgress;
            if let Some(step) = self.steps.iter_mut().find(|s| s.step_order == next) {
                step.status = StepStatus::Running;
                step.started_at = Some(Utc::now());
            }
            Some(next)
        } else {
            self.status = ExecutionStatus::Completed;
            self.completed_at = Some(Utc::now());
            None
        }
    }

    /// Mark the current step as completed.
    pub fn complete_current_step(&mut self, output: Option<String>) {
        if let Some(step) = self
            .steps
            .iter_mut()
            .find(|s| s.step_order == self.current_step)
        {
            step.status = StepStatus::Completed;
            step.completed_at = Some(Utc::now());
            step.output = output;
        }
    }

    /// Mark the current step as failed.
    pub fn fail_current_step(&mut self, error: String, fail_fast: bool) {
        if let Some(step) = self
            .steps
            .iter_mut()
            .find(|s| s.step_order == self.current_step)
        {
            step.status = StepStatus::Failed;
            step.completed_at = Some(Utc::now());
            step.error = Some(error);
        }
        if fail_fast {
            self.status = ExecutionStatus::Failed;
            self.completed_at = Some(Utc::now());
        }
    }
}

// ============================================================
// Playbook Engine
// ============================================================

/// Engine responsible for loading, matching, and executing playbooks.
#[derive(Debug, Clone)]
pub struct PlaybookEngine {
    /// All loaded playbooks indexed by ID.
    playbooks: HashMap<String, Playbook>,
}

/// Errors specific to playbook operations.
#[derive(Debug, thiserror::Error)]
pub enum PlaybookError {
    #[error("Playbook not found: {0}")]
    NotFound(String),

    #[error("Failed to parse playbook YAML: {0}")]
    ParseError(String),

    #[error("Failed to read playbook file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Playbook validation failed: {0}")]
    ValidationError(String),
}

impl PlaybookEngine {
    /// Create an empty playbook engine.
    pub fn new() -> Self {
        Self {
            playbooks: HashMap::new(),
        }
    }

    /// Load a single playbook from a YAML string.
    pub fn load_from_yaml(&mut self, yaml: &str) -> Result<String, PlaybookError> {
        let playbook: Playbook = serde_yaml::from_str(yaml)
            .map_err(|e| PlaybookError::ParseError(e.to_string()))?;
        self.validate_playbook(&playbook)?;
        let id = playbook.id.clone();
        tracing::info!(playbook_id = %id, name = %playbook.name, "Loaded playbook");
        self.playbooks.insert(id.clone(), playbook);
        Ok(id)
    }

    /// Load all YAML playbook files from a directory.
    pub fn load_from_directory(&mut self, dir: &Path) -> Result<Vec<String>, PlaybookError> {
        let mut loaded = Vec::new();
        let entries = std::fs::read_dir(dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str());

            if ext == Some("yml") || ext == Some("yaml") {
                let content = std::fs::read_to_string(&path)?;
                match self.load_from_yaml(&content) {
                    Ok(id) => loaded.push(id),
                    Err(e) => {
                        tracing::warn!(path = %path.display(), error = %e, "Skipping invalid playbook");
                    }
                }
            }
        }

        tracing::info!(count = loaded.len(), "Playbooks loaded from directory");
        Ok(loaded)
    }

    /// Get a playbook by ID.
    pub fn get(&self, id: &str) -> Option<&Playbook> {
        self.playbooks.get(id)
    }

    /// List all loaded playbooks.
    pub fn list(&self) -> Vec<&Playbook> {
        self.playbooks.values().collect()
    }

    /// Suggest playbooks based on alert type, severity, and description keywords.
    pub fn suggest(
        &self,
        alert_type: &str,
        severity: IncidentSeverity,
        description: &str,
    ) -> Vec<&Playbook> {
        let description_lower = description.to_lowercase();

        self.playbooks
            .values()
            .filter(|pb| {
                pb.trigger_conditions.iter().any(|tc| {
                    let type_match = tc.alert_types.is_empty()
                        || tc.alert_types.iter().any(|t| t.eq_ignore_ascii_case(alert_type));

                    let severity_match = tc.min_severity.map_or(true, |min| severity >= min);

                    let keyword_match = tc.keywords.is_empty()
                        || tc.keywords.iter().any(|kw| {
                            description_lower.contains(&kw.to_lowercase())
                        });

                    type_match && severity_match && keyword_match
                })
            })
            .collect()
    }

    /// Start execution of a playbook, returning an execution tracker.
    pub fn start_execution(
        &self,
        playbook_id: &str,
        incident_id: Id,
        started_by: Id,
    ) -> Result<PlaybookExecution, PlaybookError> {
        let playbook = self
            .playbooks
            .get(playbook_id)
            .ok_or_else(|| PlaybookError::NotFound(playbook_id.to_string()))?;

        let execution = PlaybookExecution::new(incident_id, playbook, started_by);
        tracing::info!(
            execution_id = %execution.id,
            playbook_id = %playbook_id,
            incident_id = %incident_id,
            "Playbook execution started"
        );
        Ok(execution)
    }

    /// Validate playbook structure.
    fn validate_playbook(&self, playbook: &Playbook) -> Result<(), PlaybookError> {
        if playbook.id.is_empty() {
            return Err(PlaybookError::ValidationError("Playbook ID cannot be empty".into()));
        }
        if playbook.name.is_empty() {
            return Err(PlaybookError::ValidationError("Playbook name cannot be empty".into()));
        }
        if playbook.steps.is_empty() {
            return Err(PlaybookError::ValidationError("Playbook must have at least one step".into()));
        }

        // Validate step ordering - no duplicates
        let mut orders: Vec<u32> = playbook.steps.iter().map(|s| s.order).collect();
        orders.sort_unstable();
        orders.dedup();
        if orders.len() != playbook.steps.len() {
            return Err(PlaybookError::ValidationError("Duplicate step order values".into()));
        }

        Ok(())
    }
}

impl Default for PlaybookEngine {
    fn default() -> Self {
        Self::new()
    }
}

