//! Attack surface asset inventory (subdomains, services, ports, cloud).
//!
//! Real CRUD backed by the `attack_surface_assets` table.

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
struct AssetResponse {
    id: Uuid,
    domain: String,
    asset_type: String,
    status: String,
    risk: String,
    details: Option<String>,
    last_scan_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct AddAssetRequest {
    domain: String,
    asset_type: String,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    risk: Option<String>,
    #[serde(default)]
    details: Option<String>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", routing::get(list_assets).post(add_asset))
        .route("/:asset_id", routing::delete(remove_asset))
        .route("/summary", routing::get(asset_summary))
}

async fn list_assets(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
) -> AppResult<Json<Vec<AssetResponse>>> {
    let assets = sqlx::query_as!(
        AssetResponse,
        r#"
        SELECT id, domain, asset_type, status, risk, details, last_scan_at, created_at
        FROM attack_surface_assets
        ORDER BY created_at DESC
        LIMIT 200
        "#
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(assets))
}

async fn add_asset(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<AddAssetRequest>,
) -> AppResult<Json<AssetResponse>> {
    // Operator or higher may add assets.
    if !claims.role.has_permission(&UserRole::Operator) {
        return Err(AppError::Forbidden(
            "Operator access required to add assets".to_string(),
        ));
    }
    if payload.domain.trim().is_empty() {
        return Err(AppError::Validation("Asset domain is required".to_string()));
    }

    let id = new_id();
    let status = payload.status.unwrap_or_else(|| "exposed".to_string());
    let risk = payload.risk.unwrap_or_else(|| "low".to_string());

    let asset = sqlx::query_as!(
        AssetResponse,
        r#"
        INSERT INTO attack_surface_assets
            (id, domain, asset_type, status, risk, details, last_scan_at, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW(), NOW())
        RETURNING id, domain, asset_type, status, risk, details, last_scan_at, created_at
        "#,
        id,
        payload.domain,
        payload.asset_type,
        status,
        risk,
        payload.details,
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(asset))
}

async fn remove_asset(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(asset_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    // Admin or higher may remove assets.
    if !claims.role.has_permission(&UserRole::Admin) {
        return Err(AppError::Forbidden(
            "Admin access required to remove assets".to_string(),
        ));
    }
    let result = sqlx::query!("DELETE FROM attack_surface_assets WHERE id = $1", asset_id)
        .execute(&state.db)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Asset not found".to_string()));
    }
    Ok(Json(serde_json::json!({ "deleted": true, "id": asset_id })))
}

async fn asset_summary(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
) -> AppResult<Json<serde_json::Value>> {
    let total = sqlx::query_scalar!("SELECT COUNT(*) as \"count!\" FROM attack_surface_assets")
        .fetch_one(&state.db)
        .await?;
    let critical = sqlx::query_scalar!(
        "SELECT COUNT(*) as \"count!\" FROM attack_surface_assets WHERE risk = 'critical'"
    )
    .fetch_one(&state.db)
    .await?;
    let exposed = sqlx::query_scalar!(
        "SELECT COUNT(*) as \"count!\" FROM attack_surface_assets WHERE status = 'exposed'"
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(serde_json::json!({
        "total": total,
        "critical": critical,
        "exposed": exposed,
    })))
}
