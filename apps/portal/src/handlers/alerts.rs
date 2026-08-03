use axum::{
    extract::{Path, Query, State},
    routing::{get, patch},
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

use raksha_alert::{AlertFilter, CreateAlert};
use raksha_auth::Claims;
use raksha_core::error::{AppError, AppResult};
use raksha_core::models::{AlertStatus, Pagination, PaginatedResponse, PaginationMeta};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_alerts).post(create_alert))
        .route("/:id", get(get_alert))
        .route("/:id/status", patch(update_alert_status))
}

async fn list_alerts(
    State(state): State<AppState>,
    Query(pagination): Query<Pagination>,
    Query(filter): Query<AlertFilter>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<PaginatedResponse<raksha_alert::Alert>>> {
    let (alerts, total) = state
        .alert_engine
        .list_alerts(&filter, pagination.limit(), pagination.offset())
        .await?;

    Ok(Json(PaginatedResponse {
        data: alerts,
        meta: PaginationMeta {
            page: pagination.page,
            per_page: pagination.per_page,
            total,
            total_pages: ((total as f64) / (pagination.limit() as f64)).ceil() as u32,
        },
    }))
}

async fn create_alert(
    State(state): State<AppState>,
    _claims: axum::Extension<Claims>,
    Json(payload): Json<CreateAlert>,
) -> AppResult<Json<raksha_alert::Alert>> {
    let alert = state.alert_engine.create_alert(payload).await?;
    Ok(Json(alert))
}

async fn get_alert(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<raksha_alert::Alert>> {
    let alert = state
        .alert_engine
        .get_alert(&id)
        .await?
        .ok_or(AppError::NotFound("Alert not found".to_string()))?;

    Ok(Json(alert))
}

#[derive(Debug, Deserialize)]
struct UpdateStatusRequest {
    status: AlertStatus,
}

async fn update_alert_status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _claims: axum::Extension<Claims>,
    Json(payload): Json<UpdateStatusRequest>,
) -> AppResult<Json<raksha_alert::Alert>> {
    let alert = state
        .alert_engine
        .update_status(&id, payload.status)
        .await?
        .ok_or(AppError::NotFound("Alert not found".to_string()))?;

    Ok(Json(alert))
}
