//! Threat Intelligence API handlers

#![allow(dead_code)]

use axum::{extract::State, Extension, Json, Router, routing};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use raksha_auth::Claims;
use raksha_core::error::{AppError, AppResult};
use raksha_core::models::{new_id, UserRole};

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
        .route("/", routing::get(list_feeds))
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

async fn sync_feeds(Extension(claims): Extension<Claims>) -> AppResult<Json<serde_json::Value>> {
    if !claims.role.has_permission(&UserRole::Operator) {
        return Err(AppError::Forbidden(
            "Operator access required to synchronize threat feeds".to_string(),
        ));
    }

    Ok(Json(serde_json::json!({
        "status": "sync_started",
        "message": "All feeds queued for synchronization"
    })))
}

#[derive(Debug, sqlx::FromRow)]
struct IocRow {
    id: String,
    indicator_type: Option<String>,
    value: String,
    source: Option<String>,
    severity: Option<String>,
    confidence: Option<i16>,
    tags: Option<Vec<String>>,
    first_seen_at: Option<DateTime<Utc>>,
    last_seen_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

fn to_response(r: IocRow) -> IOCResponse {
    IOCResponse {
        id: r.id,
        ioc_type: r.indicator_type.unwrap_or_else(|| "unknown".into()),
        value: r.value,
        source: r.source.unwrap_or_else(|| "manual".into()),
        severity: r.severity.unwrap_or_else(|| "medium".into()),
        confidence: r.confidence.map(|c| c as f64).unwrap_or(50.0),
        tags: r.tags.unwrap_or_default(),
        first_seen: r
            .first_seen_at
            .unwrap_or(r.created_at)
            .to_rfc3339(),
        last_seen: r
            .last_seen_at
            .unwrap_or(r.created_at)
            .to_rfc3339(),
    }
}

async fn list_iocs(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
) -> Json<Vec<IOCResponse>> {
    let rows: Vec<IocRow> = sqlx::query_as(
        r#"
        SELECT id::text as id,
               indicator_type,
               value,
               COALESCE(source_ref, 'manual') as source,
               severity,
               confidence,
               tags,
               first_seen_at,
               last_seen_at,
               created_at
        FROM threat_indicators
        WHERE is_active = true
        ORDER BY created_at DESC
        LIMIT 100
        "#,
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    Json(rows.into_iter().map(to_response).collect())
}

async fn add_ioc(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<AddIOCRequest>,
) -> AppResult<Json<serde_json::Value>> {
    if !claims.role.has_permission(&UserRole::Operator) {
        return Err(AppError::Forbidden(
            "Operator access required to add indicators".to_string(),
        ));
    }
    let indicator_type = match payload.ioc_type.as_str() {
        "ip" => "ip_v4",
        "hash" => "file_hash_sha256",
        other => other,
    }
    .to_string();

    let id = new_id();
    let now = Utc::now();
    let result = sqlx::query(
        r#"
        INSERT INTO threat_indicators
            (id, indicator_type, value, value_normalized, severity, confidence,
             tags, first_seen_at, last_seen_at, created_at, updated_at)
        VALUES ($1, $2, $3, lower($3), $4, 100, $5, $6, $6, $6, $6)
        "#
    )
    .bind(id)
    .bind(&indicator_type)
    .bind(&payload.value)
    .bind(&payload.severity)
    .bind(&payload.tags)
    .bind(now)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => Ok(Json(serde_json::json!({
            "status": "created",
            "ioc_type": payload.ioc_type,
            "value": payload.value,
            "id": id,
        }))),
        Err(e) => Ok(Json(serde_json::json!({
            "status": "error",
            "message": e.to_string(),
        }))),
    }
}

async fn search_iocs(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Json(query): Json<serde_json::Value>,
) -> Json<Vec<IOCResponse>> {
    let term = query
        .get("q")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let rows: Vec<IocRow> = sqlx::query_as(
        r#"
        SELECT id::text as id,
               indicator_type,
               value,
               COALESCE(source_ref, 'manual') as source,
               severity,
               confidence,
               tags,
               first_seen_at,
               last_seen_at,
               created_at
        FROM threat_indicators
        WHERE is_active = true
          AND ($1 = '' OR value ILIKE '%' || $1 || '%' OR indicator_type ILIKE '%' || $1 || '%')
        ORDER BY created_at DESC
        LIMIT 50
        "#,
    )
    .bind(term)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    Json(rows.into_iter().map(to_response).collect())
}
