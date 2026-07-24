//! Elasticsearch / OpenSearch integration.
//!
//! Sends security events via the Bulk API for high-throughput indexing.
//! Supports basic auth, API key auth, and multi-node failover.

use chrono::Utc;
use reqwest::Client;
use serde_json::json;
use tracing::{debug, error, instrument, warn};

use crate::config::SiemTarget;
use crate::formats::{to_json, SecurityEvent};

/// Error type for Elasticsearch operations.
#[derive(Debug, thiserror::Error)]
pub enum ElasticError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Elasticsearch rejected the request: status={status}, body={body}")]
    Rejected { status: u16, body: String },

    #[error("Bulk indexing had errors: {failures} of {total} failed")]
    BulkPartialFailure { total: usize, failures: usize },

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid configuration: {0}")]
    Config(String),

    #[error("No healthy nodes available")]
    NoHealthyNodes,
}

/// Authentication method for Elasticsearch.
#[derive(Debug, Clone)]
enum ElasticAuth {
    None,
    Basic { username: String, password: String },
    ApiKey(String),
}

/// Elasticsearch/OpenSearch bulk API client.
#[derive(Debug, Clone)]
pub struct ElasticClient {
    client: Client,
    endpoints: Vec<String>,
    index_pattern: String,
    auth: ElasticAuth,
}

impl ElasticClient {
    /// Create a new Elasticsearch client from a [`SiemTarget::Elasticsearch`] or [`SiemTarget::Wazuh`] config.
    pub fn new(target: &SiemTarget) -> Result<Self, ElasticError> {
        match target {
            SiemTarget::Elasticsearch {
                endpoints,
                index_pattern,
                username,
                password,
                api_key,
                verify_tls,
            } => {
                let client = Client::builder()
                    .danger_accept_invalid_certs(!verify_tls)
                    .timeout(std::time::Duration::from_secs(30))
                    .build()?;

                let auth = if let Some(key) = api_key {
                    ElasticAuth::ApiKey(key.clone())
                } else if let (Some(u), Some(p)) = (username, password) {
                    ElasticAuth::Basic {
                        username: u.clone(),
                        password: p.clone(),
                    }
                } else {
                    ElasticAuth::None
                };

                Ok(Self {
                    client,
                    endpoints: endpoints.iter().map(|e| e.trim_end_matches('/').to_string()).collect(),
                    index_pattern: index_pattern.clone(),
                    auth,
                })
            }
            SiemTarget::Wazuh {
                endpoint,
                index,
                username,
                password,
                verify_tls,
            } => {
                let client = Client::builder()
                    .danger_accept_invalid_certs(!verify_tls)
                    .timeout(std::time::Duration::from_secs(30))
                    .build()?;

                Ok(Self {
                    client,
                    endpoints: vec![endpoint.trim_end_matches('/').to_string()],
                    index_pattern: index.clone(),
                    auth: ElasticAuth::Basic {
                        username: username.clone(),
                        password: password.clone(),
                    },
                })
            }
            _ => Err(ElasticError::Config("Expected Elasticsearch or Wazuh target".into())),
        }
    }

    /// Send a single event to Elasticsearch.
    #[instrument(skip(self, event), fields(event_id = %event.id))]
    pub async fn send_event(&self, event: &SecurityEvent) -> Result<(), ElasticError> {
        self.send_batch(&[event.clone()]).await
    }


    /// Send a batch of events using the Bulk API.
    #[instrument(skip(self, events), fields(batch_size = events.len()))]
    pub async fn send_batch(&self, events: &[SecurityEvent]) -> Result<(), ElasticError> {
        if events.is_empty() {
            return Ok(());
        }

        let index_name = self.resolve_index_name();
        let mut ndjson_body = String::new();

        for event in events {
            let action = json!({
                "index": {
                    "_index": index_name,
                    "_id": event.id.to_string()
                }
            });
            ndjson_body.push_str(&serde_json::to_string(&action)?);
            ndjson_body.push('\n');

            let doc = to_json(event)?;
            ndjson_body.push_str(&doc);
            ndjson_body.push('\n');
        }

        // Try each endpoint (simple failover)
        let mut last_err = None;
        for endpoint in &self.endpoints {
            match self.post_bulk(endpoint, &ndjson_body).await {
                Ok(()) => {
                    debug!(count = events.len(), endpoint = %endpoint, "Batch indexed");
                    return Ok(());
                }
                Err(e) => {
                    warn!(endpoint = %endpoint, error = %e, "Node failed, trying next");
                    last_err = Some(e);
                }
            }
        }

        Err(last_err.unwrap_or(ElasticError::NoHealthyNodes))
    }

    /// Check cluster health.
    #[instrument(skip(self))]
    pub async fn health_check(&self) -> Result<bool, ElasticError> {
        for endpoint in &self.endpoints {
            let mut request = self.client.get(format!("{endpoint}/_cluster/health"));
            request = self.apply_auth(request);

            match request.send().await {
                Ok(resp) if resp.status().is_success() => return Ok(true),
                _ => continue,
            }
        }
        Ok(false)
    }

    /// Perform the bulk POST to a specific endpoint.
    async fn post_bulk(&self, endpoint: &str, body: &str) -> Result<(), ElasticError> {
        let mut request = self
            .client
            .post(format!("{endpoint}/_bulk"))
            .header("Content-Type", "application/x-ndjson")
            .body(body.to_string());

        request = self.apply_auth(request);

        let response = request.send().await?;
        let status = response.status();

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(ElasticError::Rejected {
                status: status.as_u16(),
                body,
            });
        }

        let resp_body: serde_json::Value = response.json().await?;
        if resp_body["errors"].as_bool().unwrap_or(false) {
            let items = resp_body["items"].as_array();
            let total = items.map(|i| i.len()).unwrap_or(0);
            let failures = items
                .map(|items| {
                    items.iter().filter(|item| {
                        item["index"]["status"].as_u64().unwrap_or(0) >= 400
                    }).count()
                })
                .unwrap_or(0);

            if failures > 0 {
                error!(total, failures, "Partial bulk indexing failure");
                return Err(ElasticError::BulkPartialFailure { total, failures });
            }
        }

        Ok(())
    }

    /// Apply authentication to a request builder.
    fn apply_auth(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match &self.auth {
            ElasticAuth::None => request,
            ElasticAuth::Basic { username, password } => {
                request.basic_auth(username, Some(password))
            }
            ElasticAuth::ApiKey(key) => {
                request.header("Authorization", format!("ApiKey {key}"))
            }
        }
    }

    /// Resolve the index pattern to a concrete index name.
    fn resolve_index_name(&self) -> String {
        let today = Utc::now().format("%Y.%m.%d").to_string();
        self.index_pattern.replace("{date}", &today)
    }
}

