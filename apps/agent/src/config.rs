use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Agent configuration loaded from TOML file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    #[serde(default = "default_agent_id")]
    pub agent_id: String,
    pub portal_url: String,
    #[serde(default)]
    pub auth_token: String,
    #[serde(default = "default_hostname")]
    pub hostname: String,
    #[serde(default)]
    pub modules: ModulesConfig,
    #[serde(default)]
    pub intervals: IntervalsConfig,
    #[serde(default)]
    pub updater: UpdaterConfig,
    #[serde(default)]
    pub buffer: BufferConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModulesConfig {
    #[serde(default = "default_true")]
    pub server_metrics: bool,
    #[serde(default = "default_true")]
    pub network_metrics: bool,
    #[serde(default = "default_true")]
    pub process_monitor: bool,
    #[serde(default = "default_true")]
    pub file_integrity: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntervalsConfig {
    #[serde(default = "default_metric_interval")]
    pub metrics_secs: u64,
    #[serde(default = "default_process_interval")]
    pub process_secs: u64,
    #[serde(default = "default_fim_interval")]
    pub file_integrity_secs: u64,
    #[serde(default = "default_report_interval")]
    pub report_secs: u64,
    #[serde(default = "default_update_interval")]
    pub update_check_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdaterConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub update_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferConfig {
    #[serde(default = "default_buffer_size")]
    pub max_items: usize,
    #[serde(default)]
    pub file_path: String,
}

fn default_agent_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn default_hostname() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "unknown".to_string())
}

fn default_true() -> bool { true }
fn default_metric_interval() -> u64 { 30 }
fn default_process_interval() -> u64 { 60 }
fn default_fim_interval() -> u64 { 300 }
fn default_report_interval() -> u64 { 15 }
fn default_update_interval() -> u64 { 3600 }
fn default_buffer_size() -> usize { 10_000 }

impl Default for ModulesConfig {
    fn default() -> Self {
        Self {
            server_metrics: true,
            network_metrics: true,
            process_monitor: true,
            file_integrity: true,
        }
    }
}

impl Default for IntervalsConfig {
    fn default() -> Self {
        Self {
            metrics_secs: 30,
            process_secs: 60,
            file_integrity_secs: 300,
            report_secs: 15,
            update_check_secs: 3600,
        }
    }
}

impl Default for UpdaterConfig {
    fn default() -> Self {
        Self { enabled: true, update_url: String::new() }
    }
}

impl Default for BufferConfig {
    fn default() -> Self {
        Self { max_items: 10_000, file_path: String::new() }
    }
}

impl AgentConfig {
    pub fn load(path: Option<&Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = path
            .map(|p| p.to_path_buf())
            .unwrap_or_else(Self::default_config_path);

        if !config_path.exists() {
            return Err(format!("Config not found: {}", config_path.display()).into());
        }

        let content = std::fs::read_to_string(&config_path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn write_default(path: Option<&Path>) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let config_path = path
            .map(|p| p.to_path_buf())
            .unwrap_or_else(Self::default_config_path);

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let default_config = Self {
            agent_id: default_agent_id(),
            portal_url: "https://localhost:3000/api".to_string(),
            auth_token: String::new(),
            hostname: default_hostname(),
            modules: ModulesConfig::default(),
            intervals: IntervalsConfig::default(),
            updater: UpdaterConfig::default(),
            buffer: BufferConfig::default(),
        };

        let content = toml::to_string_pretty(&default_config)?;
        std::fs::write(&config_path, content)?;
        Ok(config_path)
    }

    pub fn default_config_path() -> PathBuf {
        // Check environment variable first
        if let Ok(path) = std::env::var("RAKSHA_CONFIG") {
            return PathBuf::from(path);
        }
        
        if cfg!(windows) {
            PathBuf::from(r"C:\ProgramData\Raksha\agent.toml")
        } else if cfg!(target_os = "macos") {
            PathBuf::from("/Library/Application Support/Raksha/agent.toml")
        } else if cfg!(target_os = "android") {
            // Android/Termux: use home directory
            dirs::home_dir()
                .map(|h| h.join(".config/raksha/agent.toml"))
                .unwrap_or_else(|| PathBuf::from("/data/data/com.termux/files/home/.config/raksha/agent.toml"))
        } else {
            PathBuf::from("/etc/raksha/agent.toml")
        }
    }
}
