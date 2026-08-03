//! Document and evidence management endpoints.
//!
//! Metadata only: binary content lives in object storage under `storage_key`.

use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use uuid::Uuid;

use raksha_auth::Claims;
use raksha_core::error::{AppError, AppResult};
use raksha_core::models::{Pagination, PaginatedResponse, PaginationMeta};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_documents))
        .route("/summary", get(document_summary))
        .route("/expiring", get(list_expiring))
        .route("/:id", get(get_document))
}

#[derive(Debug, Serialize)]
struct DocumentResponse {
    id: Uuid,
    title: String,
    description: Option<String>,
    doc_type: String,
    category: Option<String>,
    status: String,
    classification: String,
    version: String,
    file_name: Option<String>,
    mime_type: Option<String>,
    size_bytes: Option<i64>,
    content_sha256: Option<String>,
    grc_policy_id: Option<Uuid>,
    grc_control_id: Option<Uuid>,
    incident_id: Option<Uuid>,
    owner_id: Option<Uuid>,
    approved_by: Option<Uuid>,
    approved_at: Option<DateTime<Utc>>,
    effective_date: Option<NaiveDate>,
    expires_at: Option<DateTime<Utc>>,
    download_count: i64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

async fn list_documents(
    State(state): State<AppState>,
    Query(pagination): Query<Pagination>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<PaginatedResponse<DocumentResponse>>> {
    let docs = sqlx::query_as!(
        DocumentResponse,
        r#"
        SELECT id, title, description, doc_type, category, status,
               classification, version, file_name, mime_type, size_bytes,
               content_sha256, grc_policy_id, grc_control_id, incident_id,
               owner_id, approved_by, approved_at, effective_date, expires_at,
               download_count, created_at, updated_at
        FROM documents
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
        pagination.limit(),
        pagination.offset(),
    )
    .fetch_all(&state.db)
    .await?;

    let total = sqlx::query_scalar!(r#"SELECT COUNT(*) as "count!" FROM documents"#)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(PaginatedResponse {
        data: docs,
        meta: PaginationMeta {
            page: pagination.page,
            per_page: pagination.per_page,
            total,
            total_pages: ((total as f64) / (pagination.limit() as f64)).ceil() as u32,
        },
    }))
}

async fn get_document(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<DocumentResponse>> {
    let doc = sqlx::query_as!(
        DocumentResponse,
        r#"
        SELECT id, title, description, doc_type, category, status,
               classification, version, file_name, mime_type, size_bytes,
               content_sha256, grc_policy_id, grc_control_id, incident_id,
               owner_id, approved_by, approved_at, effective_date, expires_at,
               download_count, created_at, updated_at
        FROM documents WHERE id = $1
        "#,
        id,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Document not found".to_string()))?;

    Ok(Json(doc))
}

/// Published documents past or nearing their expiry date, so compliance owners
/// can renew evidence before it goes stale.
async fn list_expiring(
    State(state): State<AppState>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<Vec<DocumentResponse>>> {
    let cutoff = Utc::now() + chrono::Duration::days(30);

    let docs = sqlx::query_as!(
        DocumentResponse,
        r#"
        SELECT id, title, description, doc_type, category, status,
               classification, version, file_name, mime_type, size_bytes,
               content_sha256, grc_policy_id, grc_control_id, incident_id,
               owner_id, approved_by, approved_at, effective_date, expires_at,
               download_count, created_at, updated_at
        FROM documents
        WHERE expires_at IS NOT NULL AND expires_at <= $1
        ORDER BY expires_at
        LIMIT 100
        "#,
        cutoff,
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(docs))
}

#[derive(Debug, Serialize)]
struct DocumentSummary {
    total: i64,
    published: i64,
    draft: i64,
    in_review: i64,
    expired: i64,
    expiring_soon: i64,
    total_size_bytes: i64,
}

async fn document_summary(
    State(state): State<AppState>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<DocumentSummary>> {
    let soon = Utc::now() + chrono::Duration::days(30);
    let now = Utc::now();

    let row = sqlx::query!(
        r#"
        SELECT
            COUNT(*) as "total!",
            COUNT(*) FILTER (WHERE status = 'published') as "published!",
            COUNT(*) FILTER (WHERE status = 'draft') as "draft!",
            COUNT(*) FILTER (WHERE status = 'in_review') as "in_review!",
            COUNT(*) FILTER (WHERE expires_at IS NOT NULL AND expires_at < $1) as "expired!",
            COUNT(*) FILTER (
                WHERE expires_at IS NOT NULL AND expires_at >= $1 AND expires_at <= $2
            ) as "expiring!",
            COALESCE(SUM(size_bytes), 0)::bigint as "size!"
        FROM documents
        "#,
        now,
        soon,
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(DocumentSummary {
        total: row.total,
        published: row.published,
        draft: row.draft,
        in_review: row.in_review,
        expired: row.expired,
        expiring_soon: row.expiring,
        total_size_bytes: row.size,
    }))
}
