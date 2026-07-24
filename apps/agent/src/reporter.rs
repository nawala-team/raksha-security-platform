use crate::collector::{Alert, MetricPayload};
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

/// Offline buffer for metrics that failed to send.
#[derive(Debug)]
pub struct OfflineBuffer {
    items: VecDeque<serde_json::Value>,
    max_size: usize,
    persist_path: Option<PathBuf>,
}

impl OfflineBuffer {
    pub fn new(max_size: usize, persist_path: Option<PathBuf>) -> Self {
        let mut buffer = Self {
            items: VecDeque::new(),
            max_size,
            persist_path: persist_path.clone(),
        };
        if let Some(ref path) = persist_path {
            buffer.load_from_disk(path);
        }
        buffer
    }

    pub fn push(&mut self, item: serde_json::Value) {
        if self.items.len() >= self.max_size {
            self.items.pop_front();
        }
        self.items.push_back(item);
    }

    pub fn drain_all(&mut self) -> Vec<serde_json::Value> {
        self.items.drain(..).collect()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn persist_to_disk(&self) {
        if let Some(ref path) = self.persist_path {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let data = serde_json::to_string(&self.items).unwrap_or_default();
            if let Err(e) = std::fs::write(path, data) {
                error!("Failed to persist buffer: {e}");
            }
        }
    }

    fn load_from_disk(&mut self, path: &Path) {
        if path.exists() {
            if let Ok(data) = std::fs::read_to_string(path) {
                if let Ok(items) = serde_json::from_str::<VecDeque<serde_json::Value>>(&data) {
                    self.items = items;
                    info!("Loaded {} buffered items from disk", self.items.len());
                }
            }
        }
    }
}

/// Reporter handles sending metrics and alerts to the portal API.
pub struct Reporter {
    client: reqwest::Client,
    portal_url: String,
    auth_token: String,
    buffer: Arc<Mutex<OfflineBuffer>>,
}

impl Reporter {
    pub fn new(
        portal_url: String,
        auth_token: String,
        max_buffer: usize,
        buffer_path: Option<PathBuf>,
    ) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            portal_url,
            auth_token,
            buffer: Arc::new(Mutex::new(OfflineBuffer::new(max_buffer, buffer_path))),
        }
    }

    /// Send a metric payload to the portal.
    pub async fn send_metrics(&self, payload: &MetricPayload) -> bool {
        let url = format!("{}/agents/metrics", self.portal_url);
        let value = match serde_json::to_value(payload) {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to serialize metrics: {e}");
                return false;
            }
        };

        if self.post_json(&url, &value).await {
            self.flush_buffer().await;
            true
        } else {
            let mut buf = self.buffer.lock().await;
            buf.push(value);
            buf.persist_to_disk();
            false
        }
    }

    /// Send an alert to the portal.
    pub async fn send_alert(&self, alert: &Alert) -> bool {
        let url = format!("{}/agents/alerts", self.portal_url);
        let value = match serde_json::to_value(alert) {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to serialize alert: {e}");
                return false;
            }
        };

        if self.post_json(&url, &value).await {
            true
        } else {
            let mut buf = self.buffer.lock().await;
            buf.push(value);
            buf.persist_to_disk();
            false
        }
    }

    /// Flush buffered items to the portal.
    pub async fn flush_buffer(&self) {
        let mut buf = self.buffer.lock().await;
        if buf.len() == 0 {
            return;
        }

        let url = format!("{}/agents/metrics/batch", self.portal_url);
        let items = buf.drain_all();
        let batch = serde_json::json!({ "items": items });

        if self.post_json(&url, &batch).await {
            info!("Flushed {} buffered items", items.len());
            buf.persist_to_disk();
        } else {
            for item in items {
                buf.push(item);
            }
            warn!("Failed to flush buffer, {} items remain", buf.len());
        }
    }

    /// Send a heartbeat to the portal.
    pub async fn heartbeat(&self, agent_id: &str, hostname: &str) -> bool {
        let url = format!("{}/agents/heartbeat", self.portal_url);
        let body = serde_json::json!({
            "agent_id": agent_id,
            "hostname": hostname,
            "timestamp": chrono::Utc::now(),
            "version": env!("CARGO_PKG_VERSION"),
        });
        self.post_json(&url, &body).await
    }

    async fn post_json(&self, url: &str, body: &serde_json::Value) -> bool {
        let result = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await;

        match result {
            Ok(resp) if resp.status().is_success() => true,
            Ok(resp) => {
                warn!("Portal returned {}: {}", resp.status(), url);
                false
            }
            Err(e) => {
                warn!("Failed to reach portal: {e}");
                false
            }
        }
    }

    pub async fn buffer_size(&self) -> usize {
        self.buffer.lock().await.len()
    }
}

