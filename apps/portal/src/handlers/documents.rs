//! Document and evidence management endpoints.
//!
//! Metadata only: binary content lives in object storage under `storage_key`.

use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use raksha_auth::Claims;
use raksha_core::error::{AppError, AppResult};
use raksha_core::models::{new_id, PaginatedResponse, Pagination, PaginationMeta, UserRole};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_documents).post(create_document))
        .route("/summary", get(document_summary))
        .route("/expiring", get(list_expiring))
        .route("/:id", get(get_document).delete(remove_document))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct DocumentResponse {
    id: Uuid,
    title: String,
    description: Option<String>,
    doc_type: String,
    category: Option<String>,
    status: String,
    classification: Option<String>,
    version: i32,
    file_path: Option<String>,
    mime_type: Option<String>,
    file_size: Option<i64>,
    checksum: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

async fn list_documents(
    State(state): State<AppState>,
    Query(pagination): Query<Pagination>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<PaginatedResponse<DocumentResponse>>> {
    let docs = sqlx::query_as::<_, DocumentResponse>(
        r#"
        SELECT id, title, description, doc_type::text, category, status::text,
               classification, version, file_path, mime_type, file_size,
               checksum, created_at, updated_at
        FROM documents
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(&state.db)
    .await?;

    let total: i64 = sqlx::query_scalar(r#"SELECT COUNT(*) FROM documents"#)
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

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
    let doc = sqlx::query_as::<_, DocumentResponse>(
        r#"
        SELECT id, title, description, doc_type::text, category, status::text,
               classification, version, file_path, mime_type, file_size,
               checksum, created_at, updated_at
        FROM documents WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Document not found".to_string()))?;

    Ok(Json(doc))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct ExpiringDoc {
    id: Uuid,
    title: String,
    doc_type: String,
    retention_until: Option<NaiveDate>,
}

async fn list_expiring(
    State(state): State<AppState>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<Vec<ExpiringDoc>>> {
    let soon = (Utc::now() + Duration::days(30)).date_naive();
    let docs = sqlx::query_as::<_, ExpiringDoc>(
        r#"
        SELECT id, title, doc_type::text, retention_until
        FROM documents
        WHERE retention_until IS NOT NULL AND retention_until <= $1 AND status != 'archived'
        ORDER BY retention_until
        "#,
    )
    .bind(soon)
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

#[derive(Debug, sqlx::FromRow)]
struct DocSummaryRow {
    total: i64,
    published: i64,
    draft: i64,
    in_review: i64,
    expired: i64,
    expiring: i64,
    size: i64,
}

async fn document_summary(
    State(state): State<AppState>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<DocumentSummary>> {
    let now = Utc::now().date_naive();
    let soon = (Utc::now() + Duration::days(30)).date_naive();

    let row: DocSummaryRow = sqlx::query_as(
        r#"
        SELECT
            COUNT(*)::bigint as total,
            COUNT(*) FILTER (WHERE status = 'published')::bigint as published,
            COUNT(*) FILTER (WHERE status = 'draft')::bigint as draft,
            COUNT(*) FILTER (WHERE status = 'in_review')::bigint as in_review,
            COUNT(*) FILTER (WHERE status = 'archived')::bigint as expired,
            COUNT(*) FILTER (
                WHERE retention_until IS NOT NULL AND retention_until >= $1 AND retention_until <= $2
            )::bigint as expiring,
            COALESCE(SUM(file_size), 0)::bigint as size
        FROM documents
        "#
    )
    .bind(now)
    .bind(soon)
    .fetch_one(&state.db)
    .await
    .unwrap_or(DocSummaryRow {
        total: 0, published: 0, draft: 0, in_review: 0, expired: 0, expiring: 0, size: 0,
    });

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

#[derive(Debug, Deserialize)]
struct CreateDocumentRequest {
    title: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default = "default_doc_type")]
    doc_type: String,
    #[serde(default)]
    category: Option<String>,
    #[serde(default = "default_classification")]
    classification: String,
    #[serde(default = "default_version")]
    #[allow(dead_code)]
    version: String,
}

fn default_doc_type() -> String {
    "policy".to_string()
}
fn default_classification() -> String {
    "internal".to_string()
}
fn default_version() -> String {
    "1.0".to_string()
}

async fn create_document(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(payload): Json<CreateDocumentRequest>,
) -> AppResult<Json<DocumentResponse>> {
    if !claims.role.has_permission(&UserRole::Operator) {
        return Err(AppError::Forbidden(
            "Operator access required to create documents".to_string(),
        ));
    }
    if payload.title.trim().is_empty() {
        return Err(AppError::Validation(
            "Document title is required".to_string(),
        ));
    }

    let id = new_id();
    let org_id = claims.tenant_id.unwrap_or_else(|| {
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap_or_else(|_| Uuid::nil())
    });
    let slug = payload.title.to_lowercase().replace(' ', "-");

    let doc = sqlx::query_as::<_, DocumentResponse>(
        r#"
        INSERT INTO documents
            (id, org_id, title, slug, description, doc_type, category, status, classification, version,
             created_by, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6::document_type, $7, 'draft', $8, 1, $9, NOW(), NOW())
        RETURNING id, title, description, doc_type::text, category, status::text, classification, version,
                  file_path, mime_type, file_size, checksum, created_at, updated_at
        "#
    )
    .bind(id)
    .bind(org_id)
    .bind(&payload.title)
    .bind(&slug)
    .bind(&payload.description)
    .bind(&payload.doc_type)
    .bind(&payload.category)
    .bind(&payload.classification)
    .bind(claims.sub)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(doc))
}

async fn remove_document(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> AppResult<Json<serde_json::Value>> {
    if !claims.role.has_permission(&UserRole::Admin) {
        return Err(AppError::Forbidden(
            "Admin access required to delete documents".to_string(),
        ));
    }
    let result = sqlx::query("DELETE FROM documents WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Document not found".to_string()));
    }
    Ok(Json(serde_json::json!({ "deleted": true, "id": id })))
}
