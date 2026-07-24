//! # raksha-siem
//!
//! SIEM integration crate for the Raksha Security Platform.
//!
//! Provides log forwarding to external SIEM systems in industry-standard formats:
//! - **CEF** (Common Event Format) - ArcSight compatible
//! - **LEEF** (Log Event Extended Format) - QRadar compatible
//! - **Syslog RFC 5424** - Universal syslog transport
//! - **JSON** - Structured JSON for modern SIEM platforms
//! - **GELF** (Graylog Extended Log Format) - Graylog native
//!
//! Supported targets:
//! - Splunk (HTTP Event Collector)
//! - Elasticsearch / OpenSearch (Bulk API)
//! - Graylog (GELF over HTTP/UDP)
//! - Wazuh (Elasticsearch-compatible API)
//!
//! ## Example
//!
//! ```rust,no_run
//! use raksha_siem::{SiemForwarder, SiemConfig, SiemTarget, SecurityEvent, Severity};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = SiemConfig {
//!         targets: vec![SiemTarget::Splunk {
//!             endpoint: "https://splunk.example.com:8088".into(),
//!             token: "your-hec-token".into(),
//!             index: Some("security".into()),
//!             verify_tls: true,
//!         }],
//!         batch_size: 50,
//!         flush_interval_secs: 10,
//!         max_retries: 3,
//!         retry_delay_ms: 1000,
//!     };
//!
//!     let forwarder = SiemForwarder::new(config).await?;
//!     // Use forwarder.send() or forwarder.send_batch() to dispatch events
//!     Ok(())
//! }
//! ```

pub mod config;
pub mod elastic;
pub mod formats;
pub mod forwarder;
pub mod graylog;
pub mod splunk;

pub use config::{SiemConfig, SiemTarget};
pub use formats::{SecurityEvent, Severity};
pub use forwarder::SiemForwarder;
