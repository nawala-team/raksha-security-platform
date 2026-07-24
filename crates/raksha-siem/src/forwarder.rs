//! SIEM event forwarder with retry logic, batching, and health checks.
//!
//! The [`SiemForwarder`] is the primary entry point for dispatching security events
//! to one or more configured SIEM targets.

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::{error, info, instrument, warn};

use crate::config::{SiemConfig, SiemTarget};
use crate::elastic::ElasticClient;
use crate::formats::SecurityEvent;
use crate::graylog::GraylogClient;
use crate::splunk::SplunkClient;

/// Error type for forwarder operations.
#[derive(Debug, thiserror::Error)]
pub enum ForwarderError {
    #[error("Splunk error: {0}")]
    Splunk(#[from] crate::splunk::SplunkError),

    #[error("Elasticsearch error: {0}")]
    Elastic(#[from] crate::elastic::ElasticError),

    #[error("Graylog error: {0}")]
    Graylog(#[from] crate::graylog::GraylogError),

    #[error("All retries exhausted for target: {target}")]
    RetriesExhausted { target: String },

    #[error("No targets configured")]
    NoTargets,

    #[error("Partial delivery: {succeeded}/{total} targets received events")]
    PartialDelivery { succeeded: usize, total: usize },
}

/// Holds initialized client instances for each target type.
#[derive(Debug)]
enum TargetClient {
    Splunk(SplunkClient),
    Elastic(ElasticClient),
    Graylog(GraylogClient),
}

/// The primary SIEM forwarder that dispatches events to configured targets.
#[derive(Debug)]
pub struct SiemForwarder {
    config: SiemConfig,
    clients: Vec<TargetClient>,
    buffer: Arc<Mutex<Vec<SecurityEvent>>>,
}

impl SiemForwarder {
    /// Create a new forwarder with the given configuration.
    pub async fn new(config: SiemConfig) -> anyhow::Result<Self> {
        let mut clients = Vec::with_capacity(config.targets.len());

        for target in &config.targets {
            let client = match target {
                SiemTarget::Splunk { .. } => {
                    TargetClient::Splunk(SplunkClient::new(target).map_err(|e| anyhow::anyhow!(e))?)
                }
                SiemTarget::Elasticsearch { .. } | SiemTarget::Wazuh { .. } => {
                    TargetClient::Elastic(ElasticClient::new(target).map_err(|e| anyhow::anyhow!(e))?)
                }
                SiemTarget::Graylog { .. } => {
                    TargetClient::Graylog(GraylogClient::new(target).map_err(|e| anyhow::anyhow!(e))?)
                }
            };
            clients.push(client);
        }

        info!(targets = clients.len(), batch_size = config.batch_size, "SIEM forwarder initialized");

        Ok(Self {
            config,
            clients,
            buffer: Arc::new(Mutex::new(Vec::new())),
        })
    }

    /// Send a single event immediately to all configured targets.
    #[instrument(skip(self, event), fields(event_id = %event.id))]
    pub async fn send(&self, event: &SecurityEvent) -> Result<(), ForwarderError> {
        if self.clients.is_empty() {
            return Err(ForwarderError::NoTargets);
        }

        let mut succeeded = 0;
        let total = self.clients.len();

        for (idx, client) in self.clients.iter().enumerate() {
            match self.send_with_retry(client, &[event.clone()]).await {
                Ok(()) => succeeded += 1,
                Err(e) => {
                    error!(target_index = idx, error = %e, "Delivery failed after retries");
                }
            }
        }

        if succeeded == 0 {
            Err(ForwarderError::PartialDelivery { succeeded, total })
        } else if succeeded < total {
            warn!(succeeded, total, "Partial delivery");
            Ok(())
        } else {
            Ok(())
        }
    }


    /// Buffer an event for batch sending. Returns `true` if a flush was triggered.
    #[instrument(skip(self, event), fields(event_id = %event.id))]
    pub async fn buffer(&self, event: SecurityEvent) -> Result<bool, ForwarderError> {
        let should_flush = {
            let mut buf = self.buffer.lock().await;
            buf.push(event);
            buf.len() >= self.config.batch_size
        };

        if should_flush {
            self.flush().await?;
            return Ok(true);
        }

        Ok(false)
    }

    /// Flush all buffered events to configured targets.
    #[instrument(skip(self))]
    pub async fn flush(&self) -> Result<(), ForwarderError> {
        let events = {
            let mut buf = self.buffer.lock().await;
            std::mem::take(&mut *buf)
        };

        if events.is_empty() {
            return Ok(());
        }

        self.send_batch(&events).await
    }

    /// Send a batch of events to all configured targets.
    #[instrument(skip(self, events), fields(batch_size = events.len()))]
    pub async fn send_batch(&self, events: &[SecurityEvent]) -> Result<(), ForwarderError> {
        if self.clients.is_empty() {
            return Err(ForwarderError::NoTargets);
        }
        if events.is_empty() {
            return Ok(());
        }

        let mut succeeded = 0;
        let total = self.clients.len();

        for (idx, client) in self.clients.iter().enumerate() {
            match self.send_with_retry(client, events).await {
                Ok(()) => succeeded += 1,
                Err(e) => {
                    error!(target_index = idx, error = %e, "Batch delivery failed");
                }
            }
        }

        if succeeded == 0 {
            Err(ForwarderError::PartialDelivery { succeeded, total })
        } else if succeeded < total {
            warn!(succeeded, total, "Partial batch delivery");
            Ok(())
        } else {
            Ok(())
        }
    }

    /// Perform a health check against all configured targets.
    #[instrument(skip(self))]
    pub async fn health_check(&self) -> Vec<(usize, bool)> {
        let mut results = Vec::with_capacity(self.clients.len());

        for (idx, client) in self.clients.iter().enumerate() {
            let healthy = match client {
                TargetClient::Splunk(c) => c.health_check().await.unwrap_or(false),
                TargetClient::Elastic(c) => c.health_check().await.unwrap_or(false),
                TargetClient::Graylog(c) => c.health_check().await.unwrap_or(false),
            };
            results.push((idx, healthy));
        }

        results
    }

    /// Returns the number of currently buffered events.
    pub async fn buffered_count(&self) -> usize {
        self.buffer.lock().await.len()
    }

    /// Send events to a single target with exponential backoff retry.
    async fn send_with_retry(
        &self,
        client: &TargetClient,
        events: &[SecurityEvent],
    ) -> Result<(), ForwarderError> {
        let mut attempt = 0;
        let mut delay = Duration::from_millis(self.config.retry_delay_ms);

        loop {
            let result = match client {
                TargetClient::Splunk(c) => {
                    c.send_batch(events).await.map_err(ForwarderError::Splunk)
                }
                TargetClient::Elastic(c) => {
                    c.send_batch(events).await.map_err(ForwarderError::Elastic)
                }
                TargetClient::Graylog(c) => {
                    c.send_batch(events).await.map_err(ForwarderError::Graylog)
                }
            };

            match result {
                Ok(()) => return Ok(()),
                Err(e) if attempt < self.config.max_retries => {
                    attempt += 1;
                    warn!(
                        attempt,
                        max_retries = self.config.max_retries,
                        delay_ms = delay.as_millis() as u64,
                        error = %e,
                        "Retrying SIEM delivery"
                    );
                    sleep(delay).await;
                    delay = std::cmp::min(delay * 2, Duration::from_secs(30));
                }
                Err(e) => {
                    error!(error = %e, "All retries exhausted");
                    return Err(e);
                }
            }
        }
    }
}

