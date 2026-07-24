//! Log format converters for SIEM integration.
//!
//! Converts [`SecurityEvent`] into industry-standard formats:
//! - CEF (Common Event Format) for ArcSight
//! - LEEF (Log Event Extended Format) for QRadar
//! - Syslog RFC 5424
//! - Structured JSON
//! - GELF (Graylog Extended Log Format)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Severity level for security events, mapped to standard syslog severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    /// CEF severity (0-10 scale).
    pub fn to_cef_severity(self) -> u8 {
        match self {
            Self::Info => 1,
            Self::Low => 3,
            Self::Medium => 5,
            Self::High => 7,
            Self::Critical => 10,
        }
    }

    /// Syslog severity (RFC 5424 numeric).
    pub fn to_syslog_severity(self) -> u8 {
        match self {
            Self::Critical => 2,
            Self::High => 3,
            Self::Medium => 4,
            Self::Low => 5,
            Self::Info => 6,
        }
    }

    /// GELF level (syslog-compatible).
    pub fn to_gelf_level(self) -> u8 {
        self.to_syslog_severity()
    }
}

/// A normalized security event for SIEM forwarding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub severity: Severity,
    pub category: String,
    pub name: String,
    pub message: String,
    pub source_ip: Option<String>,
    pub destination_ip: Option<String>,
    pub source_host: Option<String>,
    pub user: Option<String>,
    pub process: Option<String>,
    pub device_id: Option<String>,
    #[serde(default)]
    pub extensions: HashMap<String, String>,
}

impl SecurityEvent {
    /// Create a new security event with required fields.
    pub fn new(
        severity: Severity,
        category: impl Into<String>,
        name: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            timestamp: Utc::now(),
            severity,
            category: category.into(),
            name: name.into(),
            message: message.into(),
            source_ip: None,
            destination_ip: None,
            source_host: None,
            user: None,
            process: None,
            device_id: None,
            extensions: HashMap::new(),
        }
    }

    pub fn with_source_ip(mut self, ip: impl Into<String>) -> Self {
        self.source_ip = Some(ip.into());
        self
    }

    pub fn with_destination_ip(mut self, ip: impl Into<String>) -> Self {
        self.destination_ip = Some(ip.into());
        self
    }

    pub fn with_source_host(mut self, host: impl Into<String>) -> Self {
        self.source_host = Some(host.into());
        self
    }

    pub fn with_user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    pub fn with_process(mut self, process: impl Into<String>) -> Self {
        self.process = Some(process.into());
        self
    }

    pub fn with_device_id(mut self, device_id: impl Into<String>) -> Self {
        self.device_id = Some(device_id.into());
        self
    }

    pub fn with_extension(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extensions.insert(key.into(), value.into());
        self
    }
}

// ─── Format Converters ───────────────────────────────────────────────────────

/// Convert a security event to CEF (Common Event Format).
///
/// Format: `CEF:0|Vendor|Product|Version|SignatureID|Name|Severity|Extensions`
pub fn to_cef(event: &SecurityEvent) -> String {
    let mut extensions = Vec::new();

    if let Some(ref ip) = event.source_ip {
        extensions.push(format!("src={ip}"));
    }
    if let Some(ref ip) = event.destination_ip {
        extensions.push(format!("dst={ip}"));
    }
    if let Some(ref host) = event.source_host {
        extensions.push(format!("shost={host}"));
    }
    if let Some(ref user) = event.user {
        extensions.push(format!("suser={user}"));
    }
    if let Some(ref proc) = event.process {
        extensions.push(format!("sproc={proc}"));
    }
    if let Some(ref device) = event.device_id {
        extensions.push(format!("deviceExternalId={device}"));
    }

    extensions.push(format!("msg={}", cef_escape(&event.message)));
    extensions.push(format!("cat={}", &event.category));
    extensions.push(format!("rt={}", event.timestamp.timestamp_millis()));
    extensions.push(format!("externalId={}", event.id));

    for (k, v) in &event.extensions {
        extensions.push(format!("{}={}", k, cef_escape(v)));
    }

    format!(
        "CEF:0|Raksha|SecurityPlatform|1.0|{}|{}|{}|{}",
        cef_escape(&event.category),
        cef_escape(&event.name),
        event.severity.to_cef_severity(),
        extensions.join(" ")
    )
}

/// Convert a security event to LEEF (Log Event Extended Format).
///
/// Format: `LEEF:2.0|Vendor|Product|Version|EventID|key=value\tkey=value`
pub fn to_leef(event: &SecurityEvent) -> String {
    let mut attrs = Vec::new();

    attrs.push(format!("cat={}", &event.category));
    attrs.push(format!("sev={}", event.severity.to_cef_severity()));
    attrs.push(format!("devTime={}", event.timestamp.to_rfc3339()));
    attrs.push(format!("msg={}", &event.message));

    if let Some(ref ip) = event.source_ip {
        attrs.push(format!("src={ip}"));
    }
    if let Some(ref ip) = event.destination_ip {
        attrs.push(format!("dst={ip}"));
    }
    if let Some(ref host) = event.source_host {
        attrs.push(format!("srcHostName={host}"));
    }
    if let Some(ref user) = event.user {
        attrs.push(format!("usrName={user}"));
    }

    for (k, v) in &event.extensions {
        attrs.push(format!("{k}={v}"));
    }

    format!(
        "LEEF:2.0|Raksha|SecurityPlatform|1.0|{}|{}",
        event.id,
        attrs.join("\t")
    )
}

/// Convert a security event to Syslog RFC 5424 format.
pub fn to_syslog_rfc5424(event: &SecurityEvent) -> String {
    let priority = 32 + event.severity.to_syslog_severity();
    let hostname = event.source_host.as_deref().unwrap_or("raksha-platform");
    let app_name = event.process.as_deref().unwrap_or("raksha");
    let proc_id = event.device_id.as_deref().unwrap_or("-");
    let msg_id = &event.category;

    let mut sd_params = vec![
        format!("id=\"{}\"", event.id),
        format!("severity=\"{:?}\"", event.severity),
    ];
    if let Some(ref ip) = event.source_ip {
        sd_params.push(format!("src=\"{ip}\""));
    }
    if let Some(ref ip) = event.destination_ip {
        sd_params.push(format!("dst=\"{ip}\""));
    }
    if let Some(ref user) = event.user {
        sd_params.push(format!("user=\"{user}\""));
    }

    let structured_data = format!("[raksha@49152 {}]", sd_params.join(" "));

    format!(
        "<{}>1 {} {} {} {} {} {} {}: {}",
        priority,
        event.timestamp.to_rfc3339(),
        hostname,
        app_name,
        proc_id,
        msg_id,
        structured_data,
        event.name,
        event.message,
    )
}

/// Convert a security event to structured JSON.
pub fn to_json(event: &SecurityEvent) -> Result<String, serde_json::Error> {
    serde_json::to_string(event)
}

/// Convert a security event to GELF (Graylog Extended Log Format).
pub fn to_gelf(event: &SecurityEvent) -> Result<String, serde_json::Error> {
    let mut gelf = serde_json::Map::new();

    gelf.insert("version".into(), serde_json::Value::String("1.1".into()));
    gelf.insert(
        "host".into(),
        serde_json::Value::String(
            event.source_host.clone().unwrap_or_else(|| "raksha-platform".into()),
        ),
    );
    gelf.insert("short_message".into(), serde_json::Value::String(event.name.clone()));
    gelf.insert("full_message".into(), serde_json::Value::String(event.message.clone()));
    gelf.insert(
        "timestamp".into(),
        serde_json::Value::Number(serde_json::Number::from(event.timestamp.timestamp())),
    );
    gelf.insert(
        "level".into(),
        serde_json::Value::Number(serde_json::Number::from(event.severity.to_gelf_level())),
    );

    // Additional fields (prefixed with underscore per GELF spec)
    gelf.insert("_event_id".into(), serde_json::Value::String(event.id.to_string()));
    gelf.insert("_category".into(), serde_json::Value::String(event.category.clone()));

    if let Some(ref ip) = event.source_ip {
        gelf.insert("_source_ip".into(), serde_json::Value::String(ip.clone()));
    }
    if let Some(ref ip) = event.destination_ip {
        gelf.insert("_destination_ip".into(), serde_json::Value::String(ip.clone()));
    }
    if let Some(ref user) = event.user {
        gelf.insert("_user".into(), serde_json::Value::String(user.clone()));
    }
    if let Some(ref proc) = event.process {
        gelf.insert("_process".into(), serde_json::Value::String(proc.clone()));
    }
    if let Some(ref device) = event.device_id {
        gelf.insert("_device_id".into(), serde_json::Value::String(device.clone()));
    }

    for (k, v) in &event.extensions {
        gelf.insert(format!("_{k}"), serde_json::Value::String(v.clone()));
    }

    serde_json::to_string(&gelf)
}

/// Escape special characters for CEF format (pipe and backslash).
fn cef_escape(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('|', "\\|")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_event() -> SecurityEvent {
        SecurityEvent::new(
            Severity::High,
            "authentication",
            "Failed Login Attempt",
            "Multiple failed login attempts detected from suspicious IP",
        )
        .with_source_ip("192.168.1.100")
        .with_destination_ip("10.0.0.5")
        .with_user("admin")
        .with_source_host("workstation-42")
        .with_process("sshd")
    }

    #[test]
    fn test_to_cef() {
        let event = sample_event();
        let cef = to_cef(&event);
        assert!(cef.starts_with("CEF:0|Raksha|SecurityPlatform|1.0|"));
        assert!(cef.contains("src=192.168.1.100"));
        assert!(cef.contains("dst=10.0.0.5"));
        assert!(cef.contains("suser=admin"));
        assert!(cef.contains("|7|"));
    }

    #[test]
    fn test_to_leef() {
        let event = sample_event();
        let leef = to_leef(&event);
        assert!(leef.starts_with("LEEF:2.0|Raksha|SecurityPlatform|1.0|"));
        assert!(leef.contains("src=192.168.1.100"));
        assert!(leef.contains("usrName=admin"));
    }

    #[test]
    fn test_to_syslog_rfc5424() {
        let event = sample_event();
        let syslog = to_syslog_rfc5424(&event);
        assert!(syslog.starts_with("<35>1 "));
        assert!(syslog.contains("workstation-42"));
        assert!(syslog.contains("[raksha@49152"));
    }

    #[test]
    fn test_to_json() {
        let event = sample_event();
        let json = to_json(&event).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["severity"], "high");
        assert_eq!(parsed["category"], "authentication");
    }

    #[test]
    fn test_to_gelf() {
        let event = sample_event();
        let gelf = to_gelf(&event).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&gelf).unwrap();
        assert_eq!(parsed["version"], "1.1");
        assert_eq!(parsed["host"], "workstation-42");
        assert_eq!(parsed["level"], 3);
        assert_eq!(parsed["_category"], "authentication");
    }

    #[test]
    fn test_cef_escape() {
        assert_eq!(cef_escape("hello|world"), "hello\\|world");
        assert_eq!(cef_escape("back\\slash"), "back\\\\slash");
    }
}

