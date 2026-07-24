//! RQL Query Scheduler - Scheduled execution of saved threat hunting queries.
//!
//! Supports cron-like scheduling, alerting on result conditions,
//! and storing execution history for audit trails.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::executor::{OpenSearchConfig, QueryExecutor};
use super::models::*;

/// Alert condition for scheduled queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertCondition {
    /// Minimum number of results to trigger alert
    pub min_results: u64,
    /// Severity to assign to generated alerts
    pub alert_severity: String,
    /// Notification channels (e.g., "slack", "email", "webhook")
    pub notify_channels: Vec<String>,
}

/// Record of a scheduled query execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    pub id: Uuid,
    pub query_id: Uuid,
    pub executed_at: DateTime<Utc>,
    pub execution_time_ms: u64,
    pub total_hits: u64,
    pub alert_triggered: bool,
    pub error: Option<String>,
}

/// State of a scheduled query in the scheduler.
#[derive(Debug, Clone)]
struct ScheduledEntry {
    query: SavedQuery,
    alert_condition: Option<AlertCondition>,
    last_execution: Option<ExecutionRecord>,
    is_running: bool,
}

/// Query scheduler that manages periodic execution of saved queries.
pub struct QueryScheduler {
    executor: Arc<QueryExecutor>,
    entries: Arc<RwLock<HashMap<Uuid, ScheduledEntry>>>,
    history: Arc<RwLock<Vec<ExecutionRecord>>>,
    max_history: usize,
}

impl QueryScheduler {
    pub fn new(config: OpenSearchConfig) -> Self {
        Self {
            executor: Arc::new(QueryExecutor::new(config)),
            entries: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(Vec::new())),
            max_history: 10_000,
        }
    }

    /// Register a saved query for scheduled execution.
    pub async fn register_query(
        &self,
        query: SavedQuery,
        alert_condition: Option<AlertCondition>,
    ) -> Result<(), QueryValidationError> {
        // Validate the query text parses correctly
        super::parser::Parser::parse_query(&query.query_text)?;

        let entry = ScheduledEntry {
            query: query.clone(),
            alert_condition,
            last_execution: None,
            is_running: false,
        };

        let mut entries = self.entries.write().await;
        entries.insert(query.id, entry);
        Ok(())
    }

    /// Unregister a query from the scheduler.
    pub async fn unregister_query(&self, query_id: &Uuid) -> bool {
        let mut entries = self.entries.write().await;
        entries.remove(query_id).is_some()
    }

    /// Check if a query is due for execution based on its schedule.
    pub async fn is_due(&self, query_id: &Uuid) -> bool {
        let entries = self.entries.read().await;
        if let Some(entry) = entries.get(query_id) {
            if !entry.query.enabled {
                return false;
            }
            if let Some(ref schedule) = entry.query.schedule {
                if let Some(ref next_run) = schedule.next_run_at {
                    return Utc::now() >= *next_run;
                }
                // If no next_run_at calculated, it's due
                return true;
            }
        }
        false

    /// Execute a scheduled query and record the result.
    pub async fn execute_query(
        &self,
        query_id: &Uuid,
    ) -> Result<ExecutionRecord, QueryValidationError> {
        {
            let mut entries = self.entries.write().await;
            if let Some(entry) = entries.get_mut(query_id) {
                if entry.is_running {
                    return Err(QueryValidationError {
                        message: "Query is already running".to_string(),
                        position: 0, line: 0, column: 0,
                        kind: ValidationErrorKind::SyntaxError,
                    });
                }
                entry.is_running = true;
            } else {
                return Err(QueryValidationError {
                    message: "Query not found in scheduler".to_string(),
                    position: 0, line: 0, column: 0,
                    kind: ValidationErrorKind::SyntaxError,
                });
            }
        }

        let query_text = {
            let entries = self.entries.read().await;
            entries.get(query_id).unwrap().query.query_text.clone()
        };

        let start = std::time::Instant::now();
        let result = self.executor.execute(&query_text, 0, 100).await;
        let execution_time_ms = start.elapsed().as_millis() as u64;

        let record = match result {
            Ok(hunt_result) => {
                let alert_triggered = self
                    .check_alert_condition(query_id, hunt_result.total_hits)
                    .await;
                ExecutionRecord {
                    id: Uuid::now_v7(),
                    query_id: *query_id,
                    executed_at: Utc::now(),
                    execution_time_ms,
                    total_hits: hunt_result.total_hits,
                    alert_triggered,
                    error: None,
                }
            }
            Err(e) => ExecutionRecord {
                id: Uuid::now_v7(),
                query_id: *query_id,
                executed_at: Utc::now(),
                execution_time_ms,
                total_hits: 0,
                alert_triggered: false,
                error: Some(e.message.clone()),
            },
        };

        // Store and update state
        {
            let mut history = self.history.write().await;
            history.push(record.clone());
            if history.len() > self.max_history {
                history.drain(0..history.len() - self.max_history);
            }
        }
        {
            let mut entries = self.entries.write().await;
            if let Some(entry) = entries.get_mut(query_id) {
                entry.is_running = false;
                entry.last_execution = Some(record.clone());
            }
        }

        Ok(record)
    }


    /// Check if alert condition is met.
    async fn check_alert_condition(&self, query_id: &Uuid, total_hits: u64) -> bool {
        let entries = self.entries.read().await;
        if let Some(entry) = entries.get(query_id) {
            if let Some(ref condition) = entry.alert_condition {
                if total_hits >= condition.min_results {
                    tracing::warn!(
                        query_id = %query_id,
                        total_hits = total_hits,
                        severity = %condition.alert_severity,
                        "Scheduled hunt query triggered alert"
                    );
                    return true;
                }
            }
        }
        false
    }

    /// Get execution history for a specific query.
    pub async fn get_history(&self, query_id: &Uuid) -> Vec<ExecutionRecord> {
        let history = self.history.read().await;
        history.iter().filter(|r| r.query_id == *query_id).cloned().collect()
    }

    /// Get all execution history.
    pub async fn get_all_history(&self) -> Vec<ExecutionRecord> {
        let history = self.history.read().await;
        history.clone()
    }

    /// List all registered queries.
    pub async fn list_queries(&self) -> Vec<SavedQuery> {
        let entries = self.entries.read().await;
        entries.values().map(|e| e.query.clone()).collect()
    }

    /// Run the scheduler tick - executes all due queries.
    pub async fn tick(&self) -> Vec<ExecutionRecord> {
        let due_queries: Vec<Uuid> = {
            let entries = self.entries.read().await;
            entries
                .iter()
                .filter(|(_, e)| e.query.enabled && !e.is_running)
                .filter_map(|(id, e)| {
                    if let Some(ref sched) = e.query.schedule {
                        if let Some(ref next) = sched.next_run_at {
                            if Utc::now() >= *next {
                                return Some(*id);
                            }
                        }
                    }
                    None
                })
                .collect()
        };

        let mut results = Vec::new();
        for query_id in due_queries {
            match self.execute_query(&query_id).await {
                Ok(record) => results.push(record),
                Err(e) => {
                    tracing::error!(
                        query_id = %query_id,
                        error = %e.message,
                        "Scheduled query execution failed"
                    );
                }
            }
        }
        results
    }

    /// Start the scheduler background loop.
    pub fn start(self: Arc<Self>, interval_secs: u64) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                std::time::Duration::from_secs(interval_secs),
            );
            loop {
                interval.tick().await;
                let results = self.tick().await;
                if !results.is_empty() {
                    tracing::info!(
                        count = results.len(),
                        "Scheduler tick completed"
                    );
                }
            }
        })
    }
}
