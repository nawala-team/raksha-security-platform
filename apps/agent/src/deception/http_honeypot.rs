//! HTTP Honeypot - Fake web server mimicking common admin panels.
//!
//! Serves fake login pages that capture credentials and detect scanner
//! patterns (path brute-forcing). All requests are logged with full
//! headers, body, and source IP for forensic analysis.

use super::manager::{dispatch_alert, HoneypotConfig, HoneypotEvent, SeverityLevel};
use crate::reporter::Reporter;
use chrono::Utc;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::{broadcast, Mutex};
use tracing::{info, warn};

const MAX_REQUEST_SIZE: usize = 8192;
const SCANNER_THRESHOLD: u32 = 10;
const SCANNER_WINDOW_SECS: i64 = 60;

/// Tracks request counts per IP for scanner detection.
struct ScannerDetector {
    counts: HashMap<std::net::IpAddr, (u32, chrono::DateTime<Utc>)>,
}

impl ScannerDetector {
    fn new() -> Self {
        Self { counts: HashMap::new() }
    }

    /// Returns true if this IP has exceeded the scanner threshold.
    fn record_request(&mut self, ip: std::net::IpAddr) -> bool {
        let now = Utc::now();
        let entry = self.counts.entry(ip).or_insert((0, now));

        // Reset window if expired
        if (now - entry.1).num_seconds() > SCANNER_WINDOW_SECS {
            *entry = (1, now);
            return false;
        }

        entry.0 += 1;
        entry.0 >= SCANNER_THRESHOLD
    }
}

/// Run the HTTP honeypot listener.
pub async fn run(
    cfg: HoneypotConfig,
    reporter: Arc<Reporter>,
    agent_id: String,
    hostname: String,
    mut shutdown_rx: broadcast::Receiver<()>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let bind_addr = format!("{}:{}", cfg.bind_addr, cfg.port);
    let listener = TcpListener::bind(&bind_addr).await?;
    let scanner_detector = Arc::new(Mutex::new(ScannerDetector::new()));

    info!("HTTP honeypot '{}' listening on {}", cfg.name, bind_addr);

    loop {
        tokio::select! {
            _ = shutdown_rx.recv() => {
                info!("HTTP honeypot '{}' shutting down", cfg.name);
                break;
            }
            accept_result = listener.accept() => {
                let (stream, peer_addr) = match accept_result {
                    Ok(v) => v,
                    Err(e) => {
                        warn!("HTTP honeypot accept error: {e}");
                        continue;
                    }
                };

                let reporter = reporter.clone();
                let agent_id = agent_id.clone();
                let hostname = hostname.clone();
                let name = cfg.name.clone();
                let severity = cfg.alert_severity.clone();
                let detector = scanner_detector.clone();

                tokio::spawn(async move {
                    handle_http_connection(
                        stream, peer_addr, &reporter, &agent_id,
                        &hostname, &name, &severity, detector,
                    ).await;
                });
            }
        }
    }
    Ok(())
}

async fn handle_http_connection(
    mut stream: tokio::net::TcpStream,
    peer_addr: SocketAddr,
    reporter: &Reporter,
    agent_id: &str,
    hostname: &str,
    honeypot_name: &str,
    severity: &SeverityLevel,
    scanner_detector: Arc<Mutex<ScannerDetector>>,
) {
    let (reader, mut writer) = stream.split();
    let mut buf_reader = BufReader::new(reader);
    let mut request_line = String::new();

    // Read request line
    let read_result = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        buf_reader.read_line(&mut request_line),
    ).await;

    if read_result.is_err() || matches!(read_result, Ok(Ok(0))) {
        return;
    }

    let request_line = request_line.trim().to_string();

    // Parse method and path
    let parts: Vec<&str> = request_line.splitn(3, ' ').collect();
    let (method, path) = if parts.len() >= 2 {
        (parts[0].to_string(), parts[1].to_string())
    } else {
        ("UNKNOWN".to_string(), "/".to_string())
    };

    // Read headers
    let mut headers: Vec<(String, String)> = Vec::new();
    let mut content_length: usize = 0;
    loop {
        let mut header_line = String::new();
        let r = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            buf_reader.read_line(&mut header_line),
        ).await;

        match r {
            Ok(Ok(0)) | Err(_) => break,
            Ok(Ok(_)) => {
                let trimmed = header_line.trim().to_string();
                if trimmed.is_empty() { break; }
                if let Some((key, value)) = trimmed.split_once(':') {
                    let k = key.trim().to_lowercase();
                    let v = value.trim().to_string();
                    if k == "content-length" {
                        content_length = v.parse().unwrap_or(0);
                    }
                    headers.push((k, v));
                }
            }
            _ => break,
        }
    }

    // Read body if present (capped to prevent abuse)
    let body = if content_length > 0 {
        let read_len = content_length.min(MAX_REQUEST_SIZE);
        let mut body_buf = vec![0u8; read_len];
        let _ = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            buf_reader.read_exact(&mut body_buf),
        ).await;
        String::from_utf8_lossy(&body_buf).to_string()
    } else {
        String::new()
    };

    // Check for scanner pattern
    let is_scanner = {
        let mut detector = scanner_detector.lock().await;
        detector.record_request(peer_addr.ip())
    };

    // Log the request and dispatch alert
    let event = HoneypotEvent {
        honeypot_name: honeypot_name.to_string(),
        honeypot_type: "http".to_string(),
        source_addr: peer_addr,
        timestamp: Utc::now(),
        event_type: if is_scanner {
            "scanner_detected".to_string()
        } else {
            "http_request".to_string()
        },
        details: serde_json::json!({
            "method": method,
            "path": path,
            "headers": headers,
            "body": body,
            "is_scanner": is_scanner,
        }),
    };
    dispatch_alert(reporter, agent_id, hostname, &event, severity).await;

    // Serve fake login page
    let response_body = fake_login_page(&path);
    let response = format!(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: text/html\r\n\
         Content-Length: {}\r\n\
         Server: Apache/2.4.52\r\n\
         Connection: close\r\n\r\n{}",
        response_body.len(),
        response_body
    );

    let _ = writer.write_all(response.as_bytes()).await;
}

fn fake_login_page(path: &str) -> String {
    let title = match path {
        p if p.contains("wp-admin") || p.contains("wp-login") => "WordPress &mdash; Login",
        p if p.contains("admin") => "Admin Panel &mdash; Login",
        p if p.contains("phpmyadmin") => "phpMyAdmin",
        p if p.contains("cpanel") => "cPanel Login",
        _ => "Login",
    };

    format!(
        r#"<!DOCTYPE html>
<html><head><title>{title}</title></head>
<body style="font-family:sans-serif;display:flex;justify-content:center;align-items:center;height:100vh;margin:0;background:#f0f0f0">
<div style="background:white;padding:2rem;border-radius:8px;box-shadow:0 2px 10px rgba(0,0,0,.1);width:320px">
<h2 style="text-align:center;margin-bottom:1.5rem">{title}</h2>
<form method="POST" action="{path}">
<label for="username">Username</label>
<input id="username" type="text" name="username" placeholder="Username" style="width:100%;padding:8px;margin:8px 0;box-sizing:border-box" required>
<label for="password">Password</label>
<input id="password" type="password" name="password" placeholder="Password" style="width:100%;padding:8px;margin:8px 0;box-sizing:border-box" required>
<button type="submit" style="width:100%;padding:10px;background:#0073aa;color:white;border:none;border-radius:4px;cursor:pointer;margin-top:12px">Log In</button>
</form></div></body></html>"#
    )
}
