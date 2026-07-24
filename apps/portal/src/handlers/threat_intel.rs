use axum::{extract::State, routing, Json, Router};
use serde::{Deserialize, Serialize};

use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct FeedStatus {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub last_sync: Option<String>,
    pub indicator_count: u64,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct IOCResponse {
    pub id: String,
    pub ioc_type: String,
    pub value: String,
    pub source: String,
    pub severity: String,
    pub confidence: f64,
    pub tags: Vec<String>,
    pub first_seen: String,
    pub last_seen: String,
}

#[derive(Debug, Deserialize)]
pub struct AddIOCRequest {
    pub ioc_type: String,
    pub value: String,
    pub severity: String,
    pub tags: Vec<String>,
    pub description: Option<String>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/feeds", routing::get(list_feeds))
        .route("/feeds/sync", routing::post(sync_feeds))
        .route("/iocs", routing::get(list_iocs))
        .route("/iocs", routing::post(add_ioc))
        .route("/iocs/search", routing::post(search_iocs))
}

async fn list_feeds(State(_state): State<AppState>) -> Json<Vec<FeedStatus>> {
    let feeds = vec![
        FeedStatus { id: "nvd_cve".into(), name: "NIST NVD".into(), enabled: true, last_sync: Some("2026-07-24T10:00:00Z".into()), indicator_count: 224500, status: "active".into() },
        FeedStatus { id: "cisa_kev".into(), name: "CISA KEV".into(), enabled: true, last_sync: Some("2026-07-24T09:00:00Z".into()), indicator_count: 1124, status: "active".into() },
        FeedStatus { id: "abuse_ch_urlhaus".into(), name: "URLhaus".into(), enabled: true, last_sync: Some("2026-07-24T10:30:00Z".into()), indicator_count: 18920, status: "active".into() },
        FeedStatus { id: "alienvault_otx".into(), name: "AlienVault OTX".into(), enabled: true, last_sync: Some("2026-07-24T10:15:00Z".into()), indicator_count: 45230, status: "active".into() },
        FeedStatus { id: "mitre_attack".into(), name: "MITRE ATT&CK".into(), enabled: true, last_sync: Some("2026-07-24T06:00:00Z".into()), indicator_count: 890, status: "active".into() },
    ];
    Json(feeds)
}

async fn sync_feeds(State(_state): State<AppState>) -> Json<serde_json::Value> {
    // In production this triggers async feed sync jobs
    Json(serde_json::json!({
        "status": "sync_started",
        "message": "All feeds queued for synchronization"
    }))
}

async fn list_iocs(State(_state): State<AppState>) -> Json<Vec<IOCResponse>> {
    // Placeholder — in production reads from Redis/PostgreSQL
    Json(vec![])
}

async fn add_ioc(
    State(_state): State<AppState>,
    Json(payload): Json<AddIOCRequest>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "created",
        "ioc_type": payload.ioc_type,
        "value": payload.value,
    }))
}

async fn search_iocs(State(_state): State<AppState>) -> Json<Vec<IOCResponse>> {
    Json(vec![])
}
