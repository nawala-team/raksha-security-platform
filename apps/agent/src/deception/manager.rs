//! Honeypot Manager - orchestrates lifecycle of all deception services.
//!
//! Loads configuration from YAML, spawns honeypot listeners, and dispatches
//! alerts through the Reporter when connections are detected.

use crate::collector::{Alert, AlertSeverity, MetricCategory};
use crate::reporter::Reporter;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

/// Top-level deception configuration loaded from YAML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeceptionConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub honeypots: Vec<HoneypotConfig>,
}

/// Configuration for a single honeypot instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoneypotConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub honeypot_type: HoneypotType,
    pub bind_addr: String,
    pub port: u16,
    #[serde(default = "default_severity")]
    pub alert_severity: SeverityLevel,
    #[serde(default)]
    pub banner: String,
    #[serde(default)]
    pub options: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HoneypotType {
    Ssh,
    Http,
    Smtp,
    Mysql,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SeverityLevel {
    Low,
    Medium,
    High,
    Critical,
}

fn default_severity() -> SeverityLevel {
    SeverityLevel::Critical
}

impl From<&SeverityLevel> for AlertSeverity {
    fn from(level: &SeverityLevel) -> Self {
        match level {
            SeverityLevel::Low => AlertSeverity::Low,
            SeverityLevel::Medium => AlertSeverity::Medium,
            SeverityLevel::High => AlertSeverity::High,
            SeverityLevel::Critical => AlertSeverity::Critical,
        }
    }
}

/// Event emitted by honeypot services when an interaction occurs.
#[derive(Debug, Clone, Serialize)]
pub struct HoneypotEvent {
    pub honeypot_name: String,
    pub honeypot_type: String,
    pub source_addr: SocketAddr,
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub details: serde_json::Value,
}

/// Manages the lifecycle of all honeypot services.
pub struct HoneypotManager {
    config: DeceptionConfig,
    reporter: Arc<Reporter>,
    agent_id: String,
    hostname: String,
    handles: Vec<JoinHandle<()>>,
    shutdown_tx: broadcast::Sender<()>,
}

impl HoneypotManager {
    /// Create a new HoneypotManager from configuration.
    pub fn new(
        config: DeceptionConfig,
        reporter: Arc<Reporter>,
        agent_id: String,
        hostname: String,
    ) -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        Self {
            config,
            reporter,
            agent_id,
            hostname,
            handles: Vec::new(),
            shutdown_tx,
        }
    }

    /// Load deception configuration from a YAML file.
    pub fn load_config(path: &Path) -> Result<DeceptionConfig, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: DeceptionConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    /// Load configuration from default platform path.
    pub fn load_default_config() -> Result<DeceptionConfig, Box<dyn std::error::Error>> {
        let path = Self::default_config_path();
        if path.exists() {
            Self::load_config(&path)
        } else {
            Ok(DeceptionConfig {
                enabled: false,
                honeypots: Vec::new(),
            })
        }
    }

    /// Default configuration file path per platform.
    pub fn default_config_path() -> std::path::PathBuf {
        if cfg!(windows) {
            std::path::PathBuf::from(r"C:\ProgramData\Raksha\honeypots.yml")
        } else if cfg!(target_os = "macos") {
            std::path::PathBuf::from("/Library/Application Support/Raksha/honeypots.yml")
        } else {
            std::path::PathBuf::from("/etc/raksha/honeypots.yml")
        }
    }

    /// Start all configured honeypot services.
    pub async fn start(&mut self) {
        if !self.config.enabled {
            info!("Deception module disabled in configuration");
            return;
        }

        info!(
            "Starting deception module with {} honeypot(s)",
            self.config.honeypots.len()
        );

        for honeypot_cfg in self.config.honeypots.clone() {
            let handle = self.spawn_honeypot(honeypot_cfg).await;
            if let Some(h) = handle {
                self.handles.push(h);
            }
        }

        info!("All honeypots started successfully");
    }

    /// Stop all running honeypot services.
    pub async fn stop(&mut self) {
        info!("Stopping deception module...");
        let _ = self.shutdown_tx.send(());
        for handle in self.handles.drain(..) {
            handle.abort();
            let _ = handle.await;
        }
        info!("Deception module stopped");
    }

    async fn spawn_honeypot(&self, cfg: HoneypotConfig) -> Option<JoinHandle<()>> {
        let reporter = self.reporter.clone();
        let agent_id = self.agent_id.clone();
        let hostname = self.hostname.clone();
        let shutdown_rx = self.shutdown_tx.subscribe();

        info!("Starting {:?} honeypot '{}' on {}:{}",
            cfg.honeypot_type, cfg.name, cfg.bind_addr, cfg.port);

        match cfg.honeypot_type {
            HoneypotType::Ssh => Some(tokio::spawn(async move {
                if let Err(e) = super::ssh_honeypot::run(
                    cfg, reporter, agent_id, hostname, shutdown_rx,
                ).await { error!("SSH honeypot error: {e}"); }
            })),
            HoneypotType::Http => Some(tokio::spawn(async move {
                if let Err(e) = super::http_honeypot::run(
                    cfg, reporter, agent_id, hostname, shutdown_rx,
                ).await { error!("HTTP honeypot error: {e}"); }
            })),
            HoneypotType::Smtp => Some(tokio::spawn(async move {
                if let Err(e) = super::smtp_honeypot::run(
                    cfg, reporter, agent_id, hostname, shutdown_rx,
                ).await { error!("SMTP honeypot error: {e}"); }
            })),
            HoneypotType::Mysql => Some(tokio::spawn(async move {
                if let Err(e) = super::mysql_honeypot::run(
                    cfg, reporter, agent_id, hostname, shutdown_rx,
                ).await { error!("MySQL honeypot error: {e}"); }
            })),
        }
    }
}

/// Helper to dispatch an alert from any honeypot service.
pub async fn dispatch_alert(
    reporter: &Reporter,
    agent_id: &str,
    hostname: &str,
    event: &HoneypotEvent,
    severity: &SeverityLevel,
) {
    let alert = Alert {
        agent_id: agent_id.to_string(),
        hostname: hostname.to_string(),
        timestamp: event.timestamp,
        severity: AlertSeverity::from(severity),
        category: MetricCategory::Network,
        title: format!(
            "Honeypot [{}] connection from {}",
            event.honeypot_name, event.source_addr
        ),
        description: format!(
            "Deception service '{}' ({}) detected: {}",
            event.honeypot_name, event.honeypot_type, event.event_type
        ),
        metadata: serde_json::json!({
            "honeypot_name": event.honeypot_name,
            "honeypot_type": event.honeypot_type,
            "source_ip": event.source_addr.ip().to_string(),
            "source_port": event.source_addr.port(),
            "event_type": event.event_type,
            "details": event.details,
        }),
    };

    if !reporter.send_alert(&alert).await {
        warn!("Failed to dispatch honeypot alert for {}", event.source_addr);
    }
}
