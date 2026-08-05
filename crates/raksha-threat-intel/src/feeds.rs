use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use crate::ioc::{IOC, IOCType, ThreatSeverity};

/// Manages threat intelligence feeds and their sync schedules
#[derive(Clone)]
pub struct FeedManager {
    ioc_store: Arc<RwLock<Vec<IOC>>>,
    http_client: reqwest::Client,
}

impl FeedManager {
    pub fn new() -> Self {
        Self {
            ioc_store: Arc::new(RwLock::new(Vec::new())),
            http_client: reqwest::Client::builder()
                .user_agent("Raksha-Security-Platform/0.1")
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap(),
        }
    }

    /// Get current IOC count
    pub async fn ioc_count(&self) -> usize {
        self.ioc_store.read().await.len()
    }

    /// Get all active IOCs
    pub async fn get_active_iocs(&self) -> Vec<IOC> {
        self.ioc_store.read().await.iter().filter(|i| i.is_active()).cloned().collect()
    }

    /// Fetch CISA Known Exploited Vulnerabilities
    pub async fn sync_cisa_kev(&self) -> anyhow::Result<usize> {
        let url = "https://www.cisa.gov/sites/default/files/feeds/known_exploited_vulnerabilities.json";
        let resp = self.http_client.get(url).send().await?;
        let data: serde_json::Value = resp.json().await?;

        let mut count = 0;
        if let Some(vulns) = data.get("vulnerabilities").and_then(|v| v.as_array()) {
            let mut store = self.ioc_store.write().await;
            for vuln in vulns.iter().take(500) {
                let cve = vuln.get("cveID").and_then(|v| v.as_str()).unwrap_or("");
                let desc = vuln.get("shortDescription").and_then(|v| v.as_str()).unwrap_or("");
                let mut ioc = IOC::new(
                    IOCType::Sha256, // placeholder type for CVE
                    cve.to_string(),
                    "cisa_kev".to_string(),
                    ThreatSeverity::Critical,
                    1.0,
                );
                ioc.description = Some(desc.to_string());
                ioc.tags = vec!["kev".into(), "exploit".into()];
                store.push(ioc);
                count += 1;
            }
        }
        info!("CISA KEV: synced {count} indicators");
        Ok(count)
    }

    /// Fetch Abuse.ch Feodo Tracker blocklist
    pub async fn sync_feodo_tracker(&self) -> anyhow::Result<usize> {
        let url = "https://feodotracker.abuse.ch/downloads/ipblocklist.json";
        let resp = self.http_client.get(url).send().await?;
        let data: serde_json::Value = resp.json().await?;

        let mut count = 0;
        if let Some(entries) = data.as_array() {
            let mut store = self.ioc_store.write().await;
            for entry in entries.iter().take(1000) {
                let ip = entry.get("ip_address").and_then(|v| v.as_str()).unwrap_or("");
                if ip.is_empty() { continue; }
                let mut ioc = IOC::new(
                    IOCType::IPv4,
                    ip.to_string(),
                    "feodo_tracker".to_string(),
                    ThreatSeverity::Critical,
                    0.95,
                );
                ioc.tags = vec!["c2".into(), "botnet".into()];
                store.push(ioc);
                count += 1;
            }
        }
        info!("Feodo Tracker: synced {count} indicators");
        Ok(count)
    }

    /// Sync all configured feeds
    pub async fn sync_all(&self) -> HashMap<String, Result<usize, String>> {
        let mut results = HashMap::new();

        match self.sync_cisa_kev().await {
            Ok(n) => { results.insert("cisa_kev".into(), Ok(n)); }
            Err(e) => { results.insert("cisa_kev".into(), Err(e.to_string())); }
        }

        match self.sync_feodo_tracker().await {
            Ok(n) => { results.insert("feodo_tracker".into(), Ok(n)); }
            Err(e) => { results.insert("feodo_tracker".into(), Err(e.to_string())); }
        }

        results
    }
}
