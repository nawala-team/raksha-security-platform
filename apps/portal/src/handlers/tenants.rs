//! Tenant management handlers.

use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use raksha_auth::Claims;
use raksha_core::error::{AppError, AppResult};
use raksha_core::models::{Pagination, PaginatedResponse, PaginationMeta, UserRole};
use raksha_core::tenant::{TenantContext, TenantStatus};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_tenant).get(list_tenants))
        .route("/:id", get(get_tenant).put(update_tenant))
        .route("/:id/suspend", post(suspend_tenant))
        .route("/:id/stats", get(get_tenant_stats))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct TenantResponse {
    id: Uuid,
    name: String,
    slug: String,
    settings: serde_json::Value,
    status: TenantStatus,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
struct TenantStatsResponse {
    tenant_id: Uuid,
    agent_count: i64,
    alert_count: i64,
    user_count: i64,
}

#[derive(Debug, Deserialize, Validate)]
struct CreateTenantPayload {
    #[validate(length(min = 2, max = 100))]
    name: String,
    #[validate(length(min = 2, max = 50))]
    slug: String,
    settings: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Validate)]
struct UpdateTenantPayload {
    #[validate(length(min = 2, max = 100))]
    name: Option<String>,
    settings: Option<serde_json::Value>,
}

/// POST /api/v1/tenants - Create a new tenant (superadmin only)
async fn create_tenant(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(payload): Json<CreateTenantPayload>,
) -> AppResult<Json<TenantResponse>> {
    if !claims.role.has_permission(&UserRole::SuperAdmin) {
        return Err(AppError::Forbidden(
            "SuperAdmin access required to create tenants".to_string(),
        ));
    }

    payload
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    if !payload.slug.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        || payload.slug.starts_with('-')
        || payload.slug.ends_with('-')
    {
        return Err(AppError::Validation(
            "Slug must contain only lowercase letters, digits, and hyphens".to_string(),
        ));
    }

    let slug_exists = sqlx::query_scalar!(
        r#"SELECT EXISTS(SELECT 1 FROM tenants WHERE slug = $1) as "exists!""#,
        payload.slug
    )
    .fetch_one(&state.db)
    .await?;

    if slug_exists {
        return Err(AppError::Conflict(format!(
            "Tenant with slug '{}' already exists",
            payload.slug
        )));
    }

    let id = uuid::Uuid::now_v7();
    let settings = payload.settings.unwrap_or(serde_json::json!({}));

    let tenant = sqlx::query_as!(
        TenantResponse,
        r#"
        INSERT INTO tenants (id, name, slug, settings, status, created_at, updated_at)
        VALUES ($1, $2, $3, $4, 'active', NOW(), NOW())
        RETURNING id, name, slug, settings, status as "status: TenantStatus",
                  created_at, updated_at
        "#,
        id,
        payload.name,
        payload.slug,
        settings,
    )
    .fetch_one(&state.db)
    .await?;

    tracing::info!(
        tenant_id = %tenant.id,
        tenant_slug = %tenant.slug,
        created_by = %claims.sub,
        "Tenant created"
    );

    Ok(Json(tenant))
}

/// GET /api/v1/tenants - List all tenants (superadmin only)
async fn list_tenants(
    State(state): State<AppState>,
    Query(pagination): Query<Pagination>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> AppResult<Json<PaginatedResponse<TenantResponse>>> {
    if !claims.role.has_permission(&UserRole::SuperAdmin) {
        return Err(AppError::Forbidden(
            "SuperAdmin access required to list all tenants".to_string(),
        ));
    }

    let tenants = sqlx::query_as!(
        TenantResponse,
        r#"
        SELECT id, name, slug, settings, status as "status: TenantStatus",
               created_at, updated_at
        FROM tenants
        WHERE status != 'deleted'
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
        pagination.limit(),
        pagination.offset(),
    )
    .fetch_all(&state.db)
    .await?;

    let total = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM tenants WHERE status != 'deleted'"#
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(PaginatedResponse {
        data: tenants,
        meta: PaginationMeta {
            page: pagination.page,
            per_page: pagination.per_page,
            total,
            total_pages: ((total as f64) / (pagination.limit() as f64)).ceil() as u32,
        },
    }))
}

/// GET /api/v1/tenants/{id} - Get tenant details
async fn get_tenant(
    State(state): State<AppState>,
    Path(tenant_id): Path<Uuid>,
    axum::Extension(_claims): axum::Extension<Claims>,
    axum::Extension(tenant_ctx): axum::Extension<TenantContext>,
) -> AppResult<Json<TenantResponse>> {
    if !tenant_ctx.is_superadmin {
        let own_tenant = tenant_ctx.require_tenant_id()?;
        if own_tenant != tenant_id {
            return Err(AppError::Forbidden(
                "Cannot view other tenants".to_string(),
            ));
        }
    }

    let tenant = sqlx::query_as!(
        TenantResponse,
        r#"
        SELECT id, name, slug, settings, status as "status: TenantStatus",
               created_at, updated_at
        FROM tenants
        WHERE id = $1 AND status != 'deleted'
        "#,
        tenant_id,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound(format!("Tenant {} not found", tenant_id)))?;

    Ok(Json(tenant))
}

/// PUT /api/v1/tenants/{id} - Update tenant details
async fn update_tenant(
    State(state): State<AppState>,
    Path(tenant_id): Path<Uuid>,
    axum::Extension(claims): axum::Extension<Claims>,
    axum::Extension(tenant_ctx): axum::Extension<TenantContext>,
    Json(payload): Json<UpdateTenantPayload>,
) -> AppResult<Json<TenantResponse>> {
    if !tenant_ctx.is_superadmin {
        let own_tenant = tenant_ctx.require_tenant_id()?;
        if own_tenant != tenant_id {
            return Err(AppError::Forbidden(
                "Cannot update other tenants".to_string(),
            ));
        }
        if !claims.role.has_permission(&UserRole::Admin) {
            return Err(AppError::Forbidden(
                "Admin access required to update tenant".to_string(),
            ));
        }
    }

    payload
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let tenant = sqlx::query_as!(
        TenantResponse,
        r#"
        UPDATE tenants
        SET name = COALESCE($2, name),
            settings = COALESCE($3, settings),
            updated_at = NOW()
        WHERE id = $1 AND status != 'deleted'
        RETURNING id, name, slug, settings, status as "status: TenantStatus",
                  created_at, updated_at
        "#,
        tenant_id,
        payload.name,
        payload.settings,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound(format!("Tenant {} not found", tenant_id)))?;

    tracing::info!(
        tenant_id = %tenant.id,
        updated_by = %claims.sub,
        "Tenant updated"
    );

    Ok(Json(tenant))
}

/// POST /api/v1/tenants/{id}/suspend - Suspend a tenant (superadmin only)
async fn suspend_tenant(
    State(state): State<AppState>,
    Path(tenant_id): Path<Uuid>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> AppResult<Json<TenantResponse>> {
    if !claims.role.has_permission(&UserRole::SuperAdmin) {
        return Err(AppError::Forbidden(
            "SuperAdmin access required to suspend tenants".to_string(),
        ));
    }

    let tenant = sqlx::query_as!(
        TenantResponse,
        r#"
        UPDATE tenants
        SET status = 'suspended', updated_at = NOW()
        WHERE id = $1 AND status = 'active'
        RETURNING id, name, slug, settings, status as "status: TenantStatus",
                  created_at, updated_at
        "#,
        tenant_id,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound(format!(
        "Active tenant {} not found",
        tenant_id
    )))?;

    tracing::warn!(
        tenant_id = %tenant.id,
        tenant_slug = %tenant.slug,
        suspended_by = %claims.sub,
        "Tenant suspended"
    );

    Ok(Json(tenant))
}

/// GET /api/v1/tenants/{id}/stats - Get tenant statistics
async fn get_tenant_stats(
    State(state): State<AppState>,
    Path(tenant_id): Path<Uuid>,
    axum::Extension(_claims): axum::Extension<Claims>,
    axum::Extension(tenant_ctx): axum::Extension<TenantContext>,
) -> AppResult<Json<TenantStatsResponse>> {
    if !tenant_ctx.is_superadmin {
        let own_tenant = tenant_ctx.require_tenant_id()?;
        if own_tenant != tenant_id {
            return Err(AppError::Forbidden(
                "Cannot view stats for other tenants".to_string(),
            ));
        }
    }

    let tenant_exists = sqlx::query_scalar!(
        r#"SELECT EXISTS(SELECT 1 FROM tenants WHERE id = $1 AND status != 'deleted') as "exists!""#,
        tenant_id
    )
    .fetch_one(&state.db)
    .await?;

    if !tenant_exists {
        return Err(AppError::NotFound(format!("Tenant {} not found", tenant_id)));
    }

    let agent_count = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM agents WHERE org_id = $1"#,
        tenant_id
    )
    .fetch_one(&state.db)
    .await?;

    let alert_count = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM alerts a
        INNER JOIN agents ag ON a.agent_id = ag.id
        WHERE ag.org_id = $1"#,
        tenant_id
    )
    .fetch_one(&state.db)
    .await?;

    let user_count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(DISTINCT ura.user_id) as "count!"
        FROM user_roles ura
        WHERE ura.org_id = $1 AND ura.is_active = true
        "#,
        tenant_id
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(TenantStatsResponse {
        tenant_id,
        agent_count,
        alert_count,
        user_count,
    }))
}
