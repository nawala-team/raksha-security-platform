use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    pub portal_url: String,
    pub token: String,
    #[serde(default)]
    pub default_output: String,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            portal_url: "https://localhost:3000/api".to_string(),
            token: String::new(),
            default_output: "table".to_string(),
        }
    }
}

impl CliConfig {
    pub fn config_path() -> PathBuf {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("raksha");
        config_dir.join("cli.toml")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|c| toml::from_str(&c).ok())
                .unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }
}

pub fn show() {
    let config = CliConfig::load();
    let path = CliConfig::config_path();
    println!("{} ({})", "CLI Configuration".bold(), path.display());
    println!("{}", "-".repeat(40));
    println!("  Portal URL: {}", config.portal_url);
    println!("  Token:      {}", if config.token.is_empty() {
        "<not set>".red().to_string()
    } else {
        format!("{}...", &config.token[..8.min(config.token.len())])
    });
    println!("  Output:     {}", config.default_output);
}

pub fn set(key: &str, value: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = CliConfig::load();
    match key {
        "portal_url" | "url" => config.portal_url = value.to_string(),
        "token" => config.token = value.to_string(),
        "output" => config.default_output = value.to_string(),
        _ => {
            return Err(format!("Unknown config key: {key}").into());
        }
    }
    config.save()?;
    println!("{} Set {} = {}", "OK".green(), key, value);
    Ok(())
}

pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    let config = CliConfig::default();
    config.save()?;
    println!("{} Config initialized at {}", "OK".green(), CliConfig::config_path().display());
    Ok(())
}
