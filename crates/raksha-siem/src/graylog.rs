//! Graylog GELF integration.
//!
//! Supports sending events in GELF format via:
//! - HTTP POST to a GELF HTTP input
//! - UDP datagrams to a GELF UDP input (with chunking for large messages)

use std::net::UdpSocket;
use reqwest::Client;
use tracing::{debug, error, instrument, warn};

use crate::config::{GraylogTransport, SiemTarget};
use crate::formats::{to_gelf, SecurityEvent};

/// Maximum UDP datagram size before chunking (GELF spec: 8192 bytes).
const GELF_MAX_CHUNK_SIZE: usize = 8192;
/// GELF chunked message header size.
const GELF_CHUNK_HEADER_SIZE: usize = 12;
/// Maximum payload per chunk.
const GELF_CHUNK_DATA_SIZE: usize = GELF_MAX_CHUNK_SIZE - GELF_CHUNK_HEADER_SIZE;
/// GELF magic bytes for chunked messages.
const GELF_MAGIC: [u8; 2] = [0x1e, 0x0f];

/// Error type for Graylog operations.
#[derive(Debug, thiserror::Error)]
pub enum GraylogError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Graylog rejected the event: status={status}, body={body}")]
    Rejected { status: u16, body: String },

    #[error("UDP send failed: {0}")]
    Udp(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Message too large for UDP: {size} bytes ({chunks} chunks, max 128)")]
    MessageTooLarge { size: usize, chunks: usize },

    #[error("Invalid configuration: {0}")]
    Config(String),
}

/// Graylog GELF client.
#[derive(Debug, Clone)]
pub struct GraylogClient {
    http_client: Client,
    endpoint: String,
    transport: GraylogTransport,
}

impl GraylogClient {
    /// Create a new Graylog client from a [`SiemTarget::Graylog`] config.
    pub fn new(target: &SiemTarget) -> Result<Self, GraylogError> {
        let (endpoint, transport, verify_tls) = match target {
            SiemTarget::Graylog {
                endpoint,
                transport,
                verify_tls,
            } => (endpoint, transport, verify_tls),
            _ => return Err(GraylogError::Config("Expected Graylog target".into())),
        };

        let http_client = Client::builder()
            .danger_accept_invalid_certs(!verify_tls)
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        Ok(Self {
            http_client,
            endpoint: endpoint.trim_end_matches('/').to_string(),
            transport: transport.clone(),
        })
    }

    /// Send a single security event to Graylog.
    #[instrument(skip(self, event), fields(event_id = %event.id))]
    pub async fn send_event(&self, event: &SecurityEvent) -> Result<(), GraylogError> {
        let gelf_payload = to_gelf(event)?;

        match self.transport {
            GraylogTransport::Http => self.send_http(&gelf_payload).await,
            GraylogTransport::Udp => self.send_udp(&gelf_payload).await,
        }
    }

    /// Send a batch of events to Graylog.
    #[instrument(skip(self, events), fields(batch_size = events.len()))]
    pub async fn send_batch(&self, events: &[SecurityEvent]) -> Result<(), GraylogError> {
        let mut errors = Vec::new();

        for event in events {
            if let Err(e) = self.send_event(event).await {
                warn!(event_id = %event.id, error = %e, "Failed to send to Graylog");
                errors.push(e);
            }
        }

        if errors.is_empty() {
            debug!(count = events.len(), "Batch sent to Graylog");
            Ok(())
        } else if errors.len() == events.len() {
            Err(errors.into_iter().next().unwrap())
        } else {
            warn!(total = events.len(), failed = errors.len(), "Partial batch");
            Ok(())
        }
    }

    /// Check connectivity to the Graylog endpoint.
    #[instrument(skip(self))]
    pub async fn health_check(&self) -> Result<bool, GraylogError> {
        match self.transport {
            GraylogTransport::Http => {
                let response = self.http_client.head(&self.endpoint).send().await?;
                Ok(response.status().as_u16() != 502
                    && response.status().as_u16() != 503)
            }
            GraylogTransport::Udp => {
                let socket = UdpSocket::bind("0.0.0.0:0")?;
                drop(socket);
                Ok(true)
            }
        }
    }


    /// Send GELF payload via HTTP POST.
    async fn send_http(&self, payload: &str) -> Result<(), GraylogError> {
        let response = self
            .http_client
            .post(&self.endpoint)
            .header("Content-Type", "application/json")
            .body(payload.to_string())
            .send()
            .await?;

        let status = response.status();
        if status.is_success() || status.as_u16() == 202 {
            debug!("Event sent to Graylog via HTTP");
            Ok(())
        } else {
            let body = response.text().await.unwrap_or_default();
            error!(status = %status, "Graylog rejected event");
            Err(GraylogError::Rejected {
                status: status.as_u16(),
                body,
            })
        }
    }

    /// Send GELF payload via UDP (with chunking for large messages).
    async fn send_udp(&self, payload: &str) -> Result<(), GraylogError> {
        let data = payload.as_bytes();
        let socket = UdpSocket::bind("0.0.0.0:0")?;

        let udp_addr = self
            .endpoint
            .trim_start_matches("http://")
            .trim_start_matches("https://")
            .trim_start_matches("udp://");

        if data.len() <= GELF_MAX_CHUNK_SIZE {
            socket.send_to(data, udp_addr)?;
            debug!("GELF event sent via UDP (single datagram)");
        } else {
            let chunks: Vec<&[u8]> = data.chunks(GELF_CHUNK_DATA_SIZE).collect();
            let chunk_count = chunks.len();

            if chunk_count > 128 {
                return Err(GraylogError::MessageTooLarge {
                    size: data.len(),
                    chunks: chunk_count,
                });
            }

            let message_id: [u8; 8] = rand_bytes();

            for (seq, chunk) in chunks.iter().enumerate() {
                let mut packet = Vec::with_capacity(GELF_CHUNK_HEADER_SIZE + chunk.len());
                packet.extend_from_slice(&GELF_MAGIC);
                packet.extend_from_slice(&message_id);
                packet.push(seq as u8);
                packet.push(chunk_count as u8);
                packet.extend_from_slice(chunk);

                socket.send_to(&packet, udp_addr)?;
            }

            debug!(chunks = chunk_count, "GELF event sent via UDP (chunked)");
        }

        Ok(())
    }
}

/// Generate 8 random bytes for GELF message ID.
fn rand_bytes() -> [u8; 8] {
    let id = uuid::Uuid::now_v7();
    let bytes = id.as_bytes();
    let mut result = [0u8; 8];
    result.copy_from_slice(&bytes[..8]);
    result
}

