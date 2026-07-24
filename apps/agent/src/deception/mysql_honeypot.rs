//! MySQL Honeypot - Fake MySQL protocol responder.
//!
//! Sends a valid MySQL handshake packet, accepts authentication, logs
//! credentials, and returns an error on any query attempt. This traps
//! attackers scanning for exposed databases.

use super::manager::{dispatch_alert, HoneypotConfig, HoneypotEvent, SeverityLevel};
use crate::reporter::Reporter;
use chrono::Utc;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tracing::{info, warn};

const DEFAULT_MYSQL_VERSION: &str = "5.7.42-0ubuntu0.18.04.1";

/// Run the MySQL honeypot listener.
pub async fn run(
    cfg: HoneypotConfig,
    reporter: Arc<Reporter>,
    agent_id: String,
    hostname: String,
    mut shutdown_rx: broadcast::Receiver<()>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let bind_addr = format!("{}:{}", cfg.bind_addr, cfg.port);
    let listener = TcpListener::bind(&bind_addr).await?;
    let version = if cfg.banner.is_empty() {
        DEFAULT_MYSQL_VERSION.to_string()
    } else {
        cfg.banner.clone()
    };

    info!("MySQL honeypot '{}' listening on {}", cfg.name, bind_addr);

    loop {
        tokio::select! {
            _ = shutdown_rx.recv() => {
                info!("MySQL honeypot '{}' shutting down", cfg.name);
                break;
            }
            accept_result = listener.accept() => {
                let (stream, peer_addr) = match accept_result {
                    Ok(v) => v,
                    Err(e) => {
                        warn!("MySQL honeypot accept error: {e}");
                        continue;
                    }
                };

                let reporter = reporter.clone();
                let agent_id = agent_id.clone();
                let hostname = hostname.clone();
                let name = cfg.name.clone();
                let severity = cfg.alert_severity.clone();
                let version = version.clone();

                tokio::spawn(async move {
                    handle_mysql_connection(
                        stream, peer_addr, &reporter, &agent_id,
                        &hostname, &name, &severity, &version,
                    ).await;
                });
            }
        }
    }
    Ok(())
}

async fn handle_mysql_connection(
    mut stream: tokio::net::TcpStream,
    peer_addr: std::net::SocketAddr,
    reporter: &Reporter,
    agent_id: &str,
    hostname: &str,
    honeypot_name: &str,
    severity: &SeverityLevel,
    version: &str,
) {
    info!("MySQL honeypot connection from {}", peer_addr);

    // Send MySQL handshake packet (protocol v10)
    let handshake = build_handshake_packet(version);
    if stream.write_all(&handshake).await.is_err() {
        return;
    }

    // Dispatch connection alert
    let event = HoneypotEvent {
        honeypot_name: honeypot_name.to_string(),
        honeypot_type: "mysql".to_string(),
        source_addr: peer_addr,
        timestamp: Utc::now(),
        event_type: "connection".to_string(),
        details: serde_json::json!({"version": version}),
    };
    dispatch_alert(reporter, agent_id, hostname, &event, severity).await;

    // Read client auth response
    let mut auth_buf = [0u8; 4096];
    let auth_read = tokio::time::timeout(
        std::time::Duration::from_secs(15),
        stream.read(&mut auth_buf),
    ).await;

    let username = match auth_read {
        Ok(Ok(n)) if n > 36 => extract_null_string(&auth_buf[36..n]),
        _ => String::from("(unknown)"),
    };

    // Log auth attempt
    let event = HoneypotEvent {
        honeypot_name: honeypot_name.to_string(),
        honeypot_type: "mysql".to_string(),
        source_addr: peer_addr,
        timestamp: Utc::now(),
        event_type: "auth_attempt".to_string(),
        details: serde_json::json!({
            "username": username,
        }),
    };
    dispatch_alert(reporter, agent_id, hostname, &event, severity).await;

    // Send OK packet to simulate successful auth
    let ok_packet = build_ok_packet(2);
    if stream.write_all(&ok_packet).await.is_err() {
        return;
    }

    // Wait for query and respond with error
    let mut query_buf = [0u8; 4096];
    let query_read = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        stream.read(&mut query_buf),
    ).await;

    if let Ok(Ok(n)) = query_read {
        if n > 5 {
            let cmd_byte = query_buf[4];
            let query_text = if cmd_byte == 0x03 {
                String::from_utf8_lossy(&query_buf[5..n]).to_string()
            } else {
                format!("(command: 0x{:02x})", cmd_byte)
            };

            let event = HoneypotEvent {
                honeypot_name: honeypot_name.to_string(),
                honeypot_type: "mysql".to_string(),
                source_addr: peer_addr,
                timestamp: Utc::now(),
                event_type: "query_attempt".to_string(),
                details: serde_json::json!({
                    "username": username,
                    "query": query_text,
                }),
            };
            dispatch_alert(reporter, agent_id, hostname, &event, severity).await;
        }
    }

    // Send error response
    let err = build_error_packet(
        3, 1045, "28000",
        &format!("Access denied for user '{}'@'{}'", username, peer_addr.ip()),
    );
    let _ = stream.write_all(&err).await;
}

/// Build a MySQL protocol handshake packet (protocol version 10).
fn build_handshake_packet(version: &str) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.push(10u8); // Protocol version
    payload.extend_from_slice(version.as_bytes());
    payload.push(0); // Null-terminated version
    payload.extend_from_slice(&1u32.to_le_bytes()); // Connection ID
    payload.extend_from_slice(b"AbCdEfGh"); // Auth data part 1 (8 bytes)
    payload.push(0); // Filler
    payload.extend_from_slice(&0xF7FFu16.to_le_bytes()); // Capability flags lower
    payload.push(33); // Character set (utf8)
    payload.extend_from_slice(&0x0002u16.to_le_bytes()); // Status flags
    payload.extend_from_slice(&0x81FFu16.to_le_bytes()); // Capability flags upper
    payload.push(21); // Auth plugin data length
    payload.extend_from_slice(&[0u8; 10]); // Reserved
    payload.extend_from_slice(b"IjKlMnOpQrSt"); // Auth data part 2
    payload.push(0);
    payload.extend_from_slice(b"mysql_native_password");
    payload.push(0);

    // Wrap in MySQL packet header
    let mut packet = Vec::new();
    let len = payload.len() as u32;
    packet.extend_from_slice(&len.to_le_bytes()[..3]);
    packet.push(0); // Sequence ID
    packet.extend(payload);
    packet
}

/// Build a MySQL OK packet.
fn build_ok_packet(seq_id: u8) -> Vec<u8> {
    let payload: Vec<u8> = vec![0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00];
    let mut packet = Vec::new();
    let len = payload.len() as u32;
    packet.extend_from_slice(&len.to_le_bytes()[..3]);
    packet.push(seq_id);
    packet.extend(payload);
    packet
}

/// Build a MySQL ERR packet.
fn build_error_packet(seq_id: u8, code: u16, state: &str, msg: &str) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.push(0xFF); // ERR marker
    payload.extend_from_slice(&code.to_le_bytes());
    payload.push(b'#');
    let state_bytes = state.as_bytes();
    let state_len = state_bytes.len().min(5);
    payload.extend_from_slice(&state_bytes[..state_len]);
    payload.extend_from_slice(msg.as_bytes());

    let mut packet = Vec::new();
    let len = payload.len() as u32;
    packet.extend_from_slice(&len.to_le_bytes()[..3]);
    packet.push(seq_id);
    packet.extend(payload);
    packet
}

/// Extract a null-terminated string from a byte slice.
fn extract_null_string(data: &[u8]) -> String {
    let end = data.iter().position(|&b| b == 0).unwrap_or(data.len());
    String::from_utf8_lossy(&data[..end]).to_string()
}
