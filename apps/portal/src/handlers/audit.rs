use axum::{
    extract::{State, Query},
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use raksha_auth::Claims;
use raksha_core::error::{AppError, AppResult};
use raksha_core::models::{Pagination, PaginatedResponse, PaginationMeta, UserRole};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_audit_entries))
        .route("/integrity", get(verify_integrity))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct AuditEntryResponse {
    id: Uuid,
    timestamp: DateTime<Utc>,
    actor_id: Option<Uuid>,
    actor_email: Option<String>,
    action_type: String,
    action_category: String,
    resource_type: String,
    resource_id: Option<String>,
    risk_level: String,
    integrity_hash: String,
}

#[derive(Debug, Deserialize)]
struct AuditFilter {
    actor_id: Option<Uuid>,
    action_type: Option<String>,
    resource_type: Option<String>,
    from_date: Option<DateTime<Utc>>,
    to_date: Option<DateTime<Utc>>,
}

async fn list_audit_entries(
    State(state): State<AppState>,
    Query(pagination): Query<Pagination>,
    Query(filter): Query<AuditFilter>,
    claims: axum::Extension<Claims>,
) -> AppResult<Json<PaginatedResponse<AuditEntryResponse>>> {
    if !claims.role.has_permission(&UserRole::Admin) {
        return Err(AppError::Forbidden(
            "Admin access required to view audit trail".to_string(),
        ));
    }

    let entries = sqlx::query_as::<_, AuditEntryResponse>(
        r#"
        SELECT id, timestamp, actor_id, actor_email,
               action_type::text as action_type,
               action_category::text as action_category,
               resource_type, resource_id,
               risk_level::text as risk_level,
               integrity_hash
        FROM audit_trail
        WHERE ($1::uuid IS NULL OR actor_id = $1)
          AND ($2::text IS NULL OR action_type::text = $2)
          AND ($3::text IS NULL OR resource_type = $3)
          AND ($4::timestamptz IS NULL OR timestamp >= $4)
          AND ($5::timestamptz IS NULL OR timestamp <= $5)
        ORDER BY timestamp DESC
        LIMIT $6 OFFSET $7
        "#
    )
    .bind(filter.actor_id)
    .bind(&filter.action_type)
    .bind(&filter.resource_type)
    .bind(filter.from_date)
    .bind(filter.to_date)
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(&state.db)
    .await?;

    let total: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM audit_trail
        WHERE ($1::uuid IS NULL OR actor_id = $1)
          AND ($2::text IS NULL OR action_type::text = $2)
          AND ($3::text IS NULL OR resource_type = $3)
          AND ($4::timestamptz IS NULL OR timestamp >= $4)
          AND ($5::timestamptz IS NULL OR timestamp <= $5)
        "#
    )
    .bind(filter.actor_id)
    .bind(&filter.action_type)
    .bind(&filter.resource_type)
    .bind(filter.from_date)
    .bind(filter.to_date)
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    Ok(Json(PaginatedResponse {
        data: entries,
        meta: PaginationMeta {
            page: pagination.page,
            per_page: pagination.per_page,
            total,
            total_pages: ((total as f64) / (pagination.limit() as f64)).ceil() as u32,
        },
    }))
}

#[derive(Debug, Serialize)]
struct IntegrityCheckResponse {
    verified: bool,
    total_entries: i64,
    checked_entries: i64,
    broken_at: Option<Uuid>,
    message: String,
}

async fn verify_integrity(
    State(state): State<AppState>,
    claims: axum::Extension<Claims>,
) -> AppResult<Json<IntegrityCheckResponse>> {
    if !claims.role.has_permission(&UserRole::SuperAdmin) {
        return Err(AppError::Forbidden(
            "SuperAdmin access required".to_string(),
        ));
    }

    let total: i64 = sqlx::query_scalar(r#"SELECT COUNT(*) FROM audit_trail"#)
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    let broken: Option<Uuid> = sqlx::query_scalar(
        r#"
        SELECT a.id
        FROM audit_trail a
        LEFT JOIN audit_trail b ON a.previous_hash = b.integrity_hash
        WHERE a.previous_hash IS NOT NULL AND b.id IS NULL
        LIMIT 1
        "#
    )
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let (verified, message, broken_at) = match broken {
        None => (true, "Audit trail integrity verified".to_string(), None),
        Some(id) => (
            false,
            format!("Chain broken at entry {}", id),
            Some(id),
        ),
    };

    Ok(Json(IntegrityCheckResponse {
        verified,
        total_entries: total,
        checked_entries: total,
        broken_at,
        message,
    }))
}
