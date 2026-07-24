use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type of indicator
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum IOCType {
    IPv4,
    IPv6,
    Domain,
    Url,
    Sha256,
    Md5,
    Sha1,
    Email,
    Ja3,
}

/// Threat severity for an IOC
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ThreatSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// An Indicator of Compromise
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IOC {
    pub id: Uuid,
    pub ioc_type: IOCType,
    pub value: String,
    pub source_feed: String,
    pub severity: ThreatSeverity,
    pub confidence: f64,
    pub tags: Vec<String>,
    pub description: Option<String>,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
}

impl IOC {
    pub fn new(
        ioc_type: IOCType,
        value: String,
        source_feed: String,
        severity: ThreatSeverity,
        confidence: f64,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            ioc_type,
            value,
            source_feed,
            severity,
            confidence,
            tags: Vec::new(),
            description: None,
            first_seen: now,
            last_seen: now,
            expires_at: None,
            metadata: serde_json::Value::Null,
        }
    }

    /// Check if this IOC is still valid (not expired)
    pub fn is_active(&self) -> bool {
        match self.expires_at {
            Some(exp) => Utc::now() < exp,
            None => true,
        }
    }
}
