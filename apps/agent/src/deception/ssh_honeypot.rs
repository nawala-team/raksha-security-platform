//! SSH Honeypot - Fake SSH server that captures credential attempts.
//!
//! Presents a configurable SSH banner and accepts connections but never
//! grants shell access. All authentication attempts are logged and trigger
//! CRITICAL alerts since any interaction with this decoy is malicious.

use super::manager::{dispatch_alert, HoneypotConfig, HoneypotEvent, SeverityLevel};
use crate::reporter::Reporter;
use chrono::Utc;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tracing::{info, warn};

const DEFAULT_SSH_BANNER: &str = "SSH-2.0-OpenSSH_8.9p1 Ubuntu-3ubuntu0.6";
const MAX_LINE_LENGTH: usize = 1024;

/// Run the SSH honeypot listener.
pub async fn run(
    cfg: HoneypotConfig,
    reporter: Arc<Reporter>,
    agent_id: String,
    hostname: String,
    mut shutdown_rx: broadcast::Receiver<()>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let bind_addr = format!("{}:{}", cfg.bind_addr, cfg.port);
    let listener = TcpListener::bind(&bind_addr).await?;
    let banner = if cfg.banner.is_empty() {
        DEFAULT_SSH_BANNER.to_string()
    } else {
        cfg.banner.clone()
    };

    info!("SSH honeypot '{}' listening on {}", cfg.name, bind_addr);

    loop {
        tokio::select! {
            _ = shutdown_rx.recv() => {
                info!("SSH honeypot '{}' shutting down", cfg.name);
                break;
            }
            accept_result = listener.accept() => {
                let (stream, peer_addr) = match accept_result {
                    Ok(v) => v,
                    Err(e) => {
                        warn!("SSH honeypot accept error: {e}");
                        continue;
                    }
                };

                let reporter = reporter.clone();
                let agent_id = agent_id.clone();
                let hostname = hostname.clone();
                let name = cfg.name.clone();
                let severity = cfg.alert_severity.clone();
                let banner = banner.clone();

                tokio::spawn(async move {
                    handle_ssh_connection(
                        stream, peer_addr, &reporter, &agent_id,
                        &hostname, &name, &severity, &banner,
                    ).await;
                });
            }
        }
    }
    Ok(())
}

async fn handle_ssh_connection(
    mut stream: tokio::net::TcpStream,
    peer_addr: std::net::SocketAddr,
    reporter: &Reporter,
    agent_id: &str,
    hostname: &str,
    honeypot_name: &str,
    severity: &SeverityLevel,
    banner: &str,
) {
    info!("SSH honeypot connection from {}", peer_addr);

    // Send SSH banner
    let banner_line = format!("{}\r\n", banner);
    if stream.write_all(banner_line.as_bytes()).await.is_err() {
        return;
    }

    // Dispatch immediate alert on connection
    let event = HoneypotEvent {
        honeypot_name: honeypot_name.to_string(),
        honeypot_type: "ssh".to_string(),
        source_addr: peer_addr,
        timestamp: Utc::now(),
        event_type: "connection".to_string(),
        details: serde_json::json!({
            "banner_sent": banner,
        }),
    };
    dispatch_alert(reporter, agent_id, hostname, &event, severity).await;

    // Read client banner and any auth attempts
    let (reader, _writer) = stream.split();
    let mut buf_reader = BufReader::new(reader);
    let mut attempts: Vec<serde_json::Value> = Vec::new();
    let mut line_buf = String::new();

    for _ in 0..20 {
        line_buf.clear();
        let read_result = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            buf_reader.read_line(&mut line_buf),
        ).await;

        match read_result {
            Ok(Ok(0)) => break, // Connection closed
            Ok(Ok(_)) => {
                let line = line_buf.trim().to_string();
                if line.len() > MAX_LINE_LENGTH {
                    break; // Prevent memory abuse
                }
                attempts.push(serde_json::json!({
                    "timestamp": Utc::now(),
                    "data": line,
                }));
            }
            _ => break, // Timeout or error
        }
    }

    // Log all captured data
    if !attempts.is_empty() {
        let event = HoneypotEvent {
            honeypot_name: honeypot_name.to_string(),
            honeypot_type: "ssh".to_string(),
            source_addr: peer_addr,
            timestamp: Utc::now(),
            event_type: "auth_attempt".to_string(),
            details: serde_json::json!({
                "attempts": attempts,
                "total_lines": attempts.len(),
            }),
        };
        dispatch_alert(reporter, agent_id, hostname, &event, severity).await;
    }
}
