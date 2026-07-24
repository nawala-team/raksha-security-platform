//! SMTP Honeypot - Fake mail server that captures relay attempts.
//!
//! Implements minimal SMTP protocol (EHLO, MAIL FROM, RCPT TO, DATA)
//! but never actually delivers mail. All relay attempts are logged with
//! full envelope information for threat intelligence.

use super::manager::{dispatch_alert, HoneypotConfig, HoneypotEvent, SeverityLevel};
use crate::reporter::Reporter;
use chrono::Utc;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tracing::{info, warn};

const DEFAULT_SMTP_BANNER: &str = "220 mail.example.com ESMTP Postfix (Ubuntu)";
const MAX_LINE_LENGTH: usize = 1024;
const MAX_DATA_SIZE: usize = 65536;

/// Run the SMTP honeypot listener.
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
        DEFAULT_SMTP_BANNER.to_string()
    } else {
        cfg.banner.clone()
    };

    info!("SMTP honeypot '{}' listening on {}", cfg.name, bind_addr);

    loop {
        tokio::select! {
            _ = shutdown_rx.recv() => {
                info!("SMTP honeypot '{}' shutting down", cfg.name);
                break;
            }
            accept_result = listener.accept() => {
                let (stream, peer_addr) = match accept_result {
                    Ok(v) => v,
                    Err(e) => {
                        warn!("SMTP honeypot accept error: {e}");
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
                    handle_smtp_connection(
                        stream, peer_addr, &reporter, &agent_id,
                        &hostname, &name, &severity, &banner,
                    ).await;
                });
            }
        }
    }
    Ok(())
}

async fn handle_smtp_connection(
    mut stream: tokio::net::TcpStream,
    peer_addr: std::net::SocketAddr,
    reporter: &Reporter,
    agent_id: &str,
    hostname: &str,
    honeypot_name: &str,
    severity: &SeverityLevel,
    banner: &str,
) {
    info!("SMTP honeypot connection from {}", peer_addr);

    // Send greeting banner
    let greeting = format!("{}\r\n", banner);
    if stream.write_all(greeting.as_bytes()).await.is_err() {
        return;
    }

    // Dispatch immediate connection alert
    let event = HoneypotEvent {
        honeypot_name: honeypot_name.to_string(),
        honeypot_type: "smtp".to_string(),
        source_addr: peer_addr,
        timestamp: Utc::now(),
        event_type: "connection".to_string(),
        details: serde_json::json!({"banner_sent": banner}),
    };
    dispatch_alert(reporter, agent_id, hostname, &event, severity).await;

    let (reader, mut writer) = stream.split();
    let mut buf_reader = BufReader::new(reader);

    let mut mail_from: Option<String> = None;
    let mut rcpt_to: Vec<String> = Vec::new();
    let mut data_content = String::new();
    let mut ehlo_domain = String::new();

    // SMTP conversation loop
    loop {
        let mut line = String::new();
        let read_result = tokio::time::timeout(
            std::time::Duration::from_secs(60),
            buf_reader.read_line(&mut line),
        ).await;

        match read_result {
            Ok(Ok(0)) => break,
            Ok(Ok(_)) => {}
            _ => break,
        }

        let line = line.trim().to_string();
        if line.len() > MAX_LINE_LENGTH {
            let _ = writer.write_all(b"500 Line too long\r\n").await;
            break;
        }

        let cmd_upper = line.to_uppercase();

        if cmd_upper.starts_with("EHLO") || cmd_upper.starts_with("HELO") {
            ehlo_domain = line.splitn(2, ' ').nth(1).unwrap_or("").to_string();
            let resp = format!(
                "250-mail.example.com Hello {}\r\n\
                 250-SIZE 10485760\r\n\
                 250-8BITMIME\r\n\
                 250 OK\r\n",
                ehlo_domain
            );
            let _ = writer.write_all(resp.as_bytes()).await;
        } else if cmd_upper.starts_with("MAIL FROM:") {
            mail_from = Some(extract_address(&line));
            let _ = writer.write_all(b"250 OK\r\n").await;
        } else if cmd_upper.starts_with("RCPT TO:") {
            rcpt_to.push(extract_address(&line));
            let _ = writer.write_all(b"250 OK\r\n").await;
        } else if cmd_upper.starts_with("DATA") {
            let _ = writer.write_all(
                b"354 End data with <CR><LF>.<CR><LF>\r\n"
            ).await;

            // Read message body until lone dot
            loop {
                let mut data_line = String::new();
                let r = tokio::time::timeout(
                    std::time::Duration::from_secs(30),
                    buf_reader.read_line(&mut data_line),
                ).await;

                match r {
                    Ok(Ok(0)) => break,
                    Ok(Ok(_)) => {
                        if data_line.trim() == "." {
                            break;
                        }
                        if data_content.len() < MAX_DATA_SIZE {
                            data_content.push_str(&data_line);
                        }
                    }
                    _ => break,
                }
            }

            let _ = writer.write_all(b"250 OK: queued\r\n").await;

            // Log the relay attempt
            let event = HoneypotEvent {
                honeypot_name: honeypot_name.to_string(),
                honeypot_type: "smtp".to_string(),
                source_addr: peer_addr,
                timestamp: Utc::now(),
                event_type: "relay_attempt".to_string(),
                details: serde_json::json!({
                    "ehlo_domain": ehlo_domain,
                    "mail_from": mail_from,
                    "rcpt_to": rcpt_to,
                    "data_size": data_content.len(),
                    "data_preview": &data_content[..data_content.len().min(2048)],
                }),
            };
            dispatch_alert(reporter, agent_id, hostname, &event, severity).await;

            // Reset state for next message
            mail_from = None;
            rcpt_to.clear();
            data_content.clear();
        } else if cmd_upper.starts_with("QUIT") {
            let _ = writer.write_all(b"221 Bye\r\n").await;
            break;
        } else if cmd_upper.starts_with("RSET") {
            mail_from = None;
            rcpt_to.clear();
            data_content.clear();
            let _ = writer.write_all(b"250 OK\r\n").await;
        } else if cmd_upper.starts_with("NOOP") {
            let _ = writer.write_all(b"250 OK\r\n").await;
        } else {
            let _ = writer.write_all(b"502 Command not implemented\r\n").await;
        }
    }
}

/// Extract email address from MAIL FROM: or RCPT TO: line.
fn extract_address(line: &str) -> String {
    if let Some(start) = line.find('<') {
        if let Some(end) = line.find('>') {
            return line[start + 1..end].to_string();
        }
    }
    // Fallback: take everything after the colon
    line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string()
}
