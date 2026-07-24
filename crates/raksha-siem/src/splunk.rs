//! Splunk HTTP Event Collector (HEC) integration.
//!
//! Sends security events to Splunk via the HEC REST API endpoint
//! (`/services/collector/event`). Supports batch sending and token-based auth.

use reqwest::Client;
use serde::Serialize;
use tracing::{debug, error, instrument, warn};

use crate::config::SiemTarget;
use crate::formats::{to_json, SecurityEvent};

/// Error type for Splunk HEC operations.
#[derive(Debug, thiserror::Error)]
pub enum SplunkError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Splunk HEC rejected the event: status={status}, body={body}")]
    Rejected { status: u16, body: String },

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid configuration: {0}")]
    Config(String),
}

/// Splunk HEC client for event forwarding.
#[derive(Debug, Clone)]
pub struct SplunkClient {
    client: Client,
    endpoint: String,
    token: String,
    index: Option<String>,
}

/// HEC event payload structure.
#[derive(Debug, Serialize)]
struct HecEvent {
    /// Unix timestamp.
    time: i64,
    /// Target index (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    index: Option<String>,
    /// Source type identifier.
    sourcetype: String,
    /// Source identifier.
    source: String,
    /// Host name.
    host: String,
    /// The event data (JSON object).
    event: serde_json::Value,
}

impl SplunkClient {
    /// Create a new Splunk HEC client from a [`SiemTarget::Splunk`] config.
    pub fn new(target: &SiemTarget) -> Result<Self, SplunkError> {
        let (endpoint, token, index, verify_tls) = match target {
            SiemTarget::Splunk {
                endpoint,
                token,
                index,
                verify_tls,
            } => (endpoint, token, index, verify_tls),
            _ => return Err(SplunkError::Config("Expected Splunk target".into())),
        };

        let client = Client::builder()
            .danger_accept_invalid_certs(!verify_tls)
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        Ok(Self {
            client,
            endpoint: endpoint.trim_end_matches('/').to_string(),
            token: token.clone(),
            index: index.clone(),
        })
    }

    /// Send a single security event to Splunk HEC.
    #[instrument(skip(self, event), fields(event_id = %event.id))]
    pub async fn send_event(&self, event: &SecurityEvent) -> Result<(), SplunkError> {
        let hec_event = self.build_hec_event(event)?;
        let body = serde_json::to_string(&hec_event)?;

        let response = self
            .client
            .post(format!("{}/services/collector/event", self.endpoint))
            .header("Authorization", format!("Splunk {}", self.token))
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            debug!("Event successfully sent to Splunk HEC");
            Ok(())
        } else {
            let body = response.text().await.unwrap_or_default();
            error!(status = %status, "Splunk HEC rejected event");
            Err(SplunkError::Rejected {
                status: status.as_u16(),
                body,
            })
        }
    }

    /// Send a batch of security events to Splunk HEC.
    ///
    /// Uses the HEC batch endpoint, sending newline-delimited JSON events.
    #[instrument(skip(self, events), fields(batch_size = events.len()))]
    pub async fn send_batch(&self, events: &[SecurityEvent]) -> Result<(), SplunkError> {
        if events.is_empty() {
            return Ok(());
        }

        let mut body = String::new();
        for event in events {
            let hec_event = self.build_hec_event(event)?;
            let line = serde_json::to_string(&hec_event)?;
            body.push_str(&line);
            body.push('\n');
        }

        let response = self
            .client
            .post(format!("{}/services/collector/event", self.endpoint))
            .header("Authorization", format!("Splunk {}", self.token))
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            debug!(count = events.len(), "Batch successfully sent to Splunk HEC");
            Ok(())
        } else {
            let body = response.text().await.unwrap_or_default();
            warn!(status = %status, "Splunk HEC rejected batch");
            Err(SplunkError::Rejected {
                status: status.as_u16(),
                body,
            })
        }
    }

    /// Check connectivity to the Splunk HEC endpoint.
    #[instrument(skip(self))]
    pub async fn health_check(&self) -> Result<bool, SplunkError> {
        let response = self
            .client
            .get(format!("{}/services/collector/health/1.0", self.endpoint))
            .header("Authorization", format!("Splunk {}", self.token))
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    /// Build a HEC event payload from a SecurityEvent.
    fn build_hec_event(&self, event: &SecurityEvent) -> Result<HecEvent, SplunkError> {
        let json_str = to_json(event)?;
        let event_value: serde_json::Value = serde_json::from_str(&json_str)?;

        Ok(HecEvent {
            time: event.timestamp.timestamp(),
            index: self.index.clone(),
            sourcetype: format!("raksha:{}", event.category),
            source: "raksha-security-platform".to_string(),
            host: event
                .source_host
                .clone()
                .unwrap_or_else(|| "raksha-agent".into()),
            event: event_value,
        })
    }
}
