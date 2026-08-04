//! Monitored database instances (Database Guard module).
//!
//! Real CRUD backed by the `monitored_databases` table instead of the previous
//! stub handlers. Credentials are stored in `password_enc` and never returned.

use axum::{
    extract::{Path, State},
    routing,
    Extension, Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use raksha_auth::Claims;
use raksha_core::error::{AppError, AppResult};
use raksha_core::models::{new_id, UserRole};

use crate::state::AppState;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct DatabaseInstanceResponse {
    pub id: Uuid,
    pub name: String,
    pub db_type: String,
    pub host: String,
    pub port: i32,
    pub ssl_enabled: bool,
    pub status: String,
    pub connections: i32,
    pub max_connections: i32,
    pub query_rate: i64,
    pub replication_lag_ms: Option<i64>,
    pub size_bytes: i64,
    pub encrypted: bool,
    pub version: Option<String>,
    pub alerts: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterDatabaseRequest {
    pub name: String,
    pub db_type: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    #[serde(default = "default_true")]
    pub ssl_enabled: bool,
}

fn default_true() -> bool {
    true
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", routing::get(list_databases).post(register_database))
        .route(
            "/:db_id",
            routing::get(get_database).delete(remove_database),
        )
        .route("/:db_id/metrics", routing::get(get_metrics))
}

async fn list_databases(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
) -> AppResult<Json<Vec<DatabaseInstanceResponse>>> {
    let dbs = sqlx::query_as!(
        DatabaseInstanceResponse,
        r#"
        SELECT id, name, db_type, host, port, ssl_enabled, status,
               connections, max_connections, query_rate, replication_lag_ms,
               size_bytes, encrypted, version, alerts, created_at
        FROM monitored_databases
        ORDER BY name
        "#
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(dbs))
}

async fn register_database(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<RegisterDatabaseRequest>,
) -> AppResult<Json<DatabaseInstanceResponse>> {
    // Operator or higher may register a database.
    if !claims.role.has_permission(&UserRole::Operator) {
        return Err(AppError::Forbidden(
            "Operator access required to register databases".to_string(),
        ));
    }
    if payload.name.trim().is_empty() {
        return Err(AppError::Validation("Name is required".to_string()));
    }

    let id = new_id();
    // Store credentials in `password_enc`; never expose them in responses.
    let password_enc = payload.password;

    let db = sqlx::query_as!(
        DatabaseInstanceResponse,
        r#"
        INSERT INTO monitored_databases
            (id, name, db_type, host, port, username, password_enc, ssl_enabled,
             status, connections, max_connections, query_rate, encrypted, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'online', 0, 100, 0, $9, NOW(), NOW())
        RETURNING id, name, db_type, host, port, ssl_enabled, status,
                  connections, max_connections, query_rate, replication_lag_ms,
                  size_bytes, encrypted, version, alerts, created_at
        "#,
        id,
        payload.name,
        payload.db_type,
        payload.host,
        payload.port as i32,
        payload.username,
        password_enc,
        payload.ssl_enabled,
        payload.ssl_enabled,
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(db))
}

async fn get_database(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Path(db_id): Path<Uuid>,
) -> AppResult<Json<DatabaseInstanceResponse>> {
    let db = sqlx::query_as!(
        DatabaseInstanceResponse,
        r#"
        SELECT id, name, db_type, host, port, ssl_enabled, status,
               connections, max_connections, query_rate, replication_lag_ms,
               size_bytes, encrypted, version, alerts, created_at
        FROM monitored_databases WHERE id = $1
        "#,
        db_id,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Database not found".to_string()))?;

    Ok(Json(db))
}

async fn remove_database(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(db_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    // Admin or higher may remove a monitored database.
    if !claims.role.has_permission(&UserRole::Admin) {
        return Err(AppError::Forbidden(
            "Admin access required to remove databases".to_string(),
        ));
    }
    let result = sqlx::query!("DELETE FROM monitored_databases WHERE id = $1", db_id)
        .execute(&state.db)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Database not found".to_string()));
    }
    Ok(Json(serde_json::json!({ "deleted": true, "id": db_id })))
}

async fn get_metrics(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Path(db_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let db = sqlx::query!(
        r#"
        SELECT connections, query_rate, replication_lag_ms, size_bytes
        FROM monitored_databases WHERE id = $1
        "#,
        db_id,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Database not found".to_string()))?;

    Ok(Json(serde_json::json!({
        "db_id": db_id,
        "connections": db.connections,
        "query_rate": db.query_rate,
        "replication_lag_ms": db.replication_lag_ms,
        "size_bytes": db.size_bytes,
    })))
}

