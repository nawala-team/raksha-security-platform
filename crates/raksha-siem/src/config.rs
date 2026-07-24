//! SIEM configuration types.
//!
//! Defines connection parameters and target selection for log forwarding.

use serde::{Deserialize, Serialize};

/// Top-level SIEM forwarding configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiemConfig {
    /// List of SIEM targets to forward events to.
    pub targets: Vec<SiemTarget>,

    /// Number of events to accumulate before flushing a batch.
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,

    /// Interval (seconds) to flush buffered events even if batch_size is not reached.
    #[serde(default = "default_flush_interval")]
    pub flush_interval_secs: u64,

    /// Maximum retry attempts per delivery.
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Delay between retries in milliseconds (exponential backoff base).
    #[serde(default = "default_retry_delay")]
    pub retry_delay_ms: u64,
}

/// Supported SIEM target backends.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SiemTarget {
    /// Splunk HTTP Event Collector.
    Splunk {
        /// HEC endpoint URL (e.g., `https://splunk:8088`).
        endpoint: String,
        /// HEC authentication token.
        token: String,
        /// Optional target index.
        index: Option<String>,
        /// Whether to verify TLS certificates.
        #[serde(default = "default_true")]
        verify_tls: bool,
    },

    /// Elasticsearch or OpenSearch cluster.
    Elasticsearch {
        /// Cluster endpoint URLs (supports multiple nodes for failover).
        endpoints: Vec<String>,
        /// Index name pattern (e.g., `raksha-events-{date}`).
        index_pattern: String,
        /// Optional username for basic auth.
        username: Option<String>,
        /// Optional password for basic auth.
        password: Option<String>,
        /// Optional API key for authentication.
        api_key: Option<String>,
        /// Whether to verify TLS certificates.
        #[serde(default = "default_true")]
        verify_tls: bool,
    },

    /// Wazuh (uses Elasticsearch-compatible API).
    Wazuh {
        /// Wazuh indexer endpoint.
        endpoint: String,
        /// Index name for security events.
        index: String,
        /// Username for basic auth.
        username: String,
        /// Password for basic auth.
        password: String,
        /// Whether to verify TLS certificates.
        #[serde(default = "default_true")]
        verify_tls: bool,
    },

    /// Graylog via GELF.
    Graylog {
        /// Graylog GELF HTTP input URL (e.g., `http://graylog:12201/gelf`).
        endpoint: String,
        /// Transport protocol.
        #[serde(default)]
        transport: GraylogTransport,
        /// Whether to verify TLS certificates (HTTP transport only).
        #[serde(default = "default_true")]
        verify_tls: bool,
    },
}

/// Graylog transport protocol.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GraylogTransport {
    /// HTTP/HTTPS POST to GELF input.
    #[default]
    Http,
    /// UDP datagrams (max 8192 bytes per chunk).
    Udp,
}

fn default_batch_size() -> usize {
    50
}

fn default_flush_interval() -> u64 {
    10
}

fn default_max_retries() -> u32 {
    3
}

fn default_retry_delay() -> u64 {
    1000
}

fn default_true() -> bool {
    true
}

impl Default for SiemConfig {
    fn default() -> Self {
        Self {
            targets: Vec::new(),
            batch_size: default_batch_size(),
            flush_interval_secs: default_flush_interval(),
            max_retries: default_max_retries(),
            retry_delay_ms: default_retry_delay(),
        }
    }
}
