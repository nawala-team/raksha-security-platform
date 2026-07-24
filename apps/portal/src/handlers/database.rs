use axum::{extract::State, routing, Json, Router};
use serde::{Deserialize, Serialize};

use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct DatabaseInstanceResponse {
    pub id: String,
    pub name: String,
    pub db_type: String,
    pub host: String,
    pub port: u16,
    pub status: String,
    pub connections: u32,
    pub max_connections: u32,
    pub query_rate: u64,
    pub replication_lag_ms: Option<u64>,
    pub size_bytes: u64,
    pub encrypted: bool,
    pub version: String,
    pub alerts: u32,
}

#[derive(Debug, Deserialize)]
pub struct RegisterDatabaseRequest {
    pub name: String,
    pub db_type: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub ssl_enabled: bool,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", routing::get(list_databases))
        .route("/", routing::post(register_database))
        .route("/{db_id}", routing::get(get_database))
        .route("/{db_id}", routing::delete(remove_database))
        .route("/{db_id}/metrics", routing::get(get_metrics))
        .route("/{db_id}/queries", routing::get(get_slow_queries))
        .route("/{db_id}/permissions", routing::get(check_permissions))
}

async fn list_databases(State(_state): State<AppState>) -> Json<Vec<DatabaseInstanceResponse>> {
    Json(vec![])
}

async fn register_database(
    State(_state): State<AppState>,
    Json(payload): Json<RegisterDatabaseRequest>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "registered",
        "name": payload.name,
        "type": payload.db_type,
    }))
}

async fn get_database(State(_state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"error": "not_found"}))
}

async fn remove_database(State(_state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "removed"}))
}

async fn get_metrics(State(_state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"metrics": []}))
}

async fn get_slow_queries(State(_state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"slow_queries": []}))
}

async fn check_permissions(State(_state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"permissions_check": "passed", "issues": []}))
}
