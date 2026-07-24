use axum::{
    extract::{Path, Query, State},
    routing::{get, put},
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
        .route("/", get(list_users))
        .route("/me", get(get_current_user))
        .route("/{id}", get(get_user).put(update_user))
        .route("/{id}/role", put(update_role))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct UserResponse {
    id: Uuid,
    email: String,
    name: String,
    role: UserRole,
    is_active: bool,
    last_login_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

async fn list_users(
    State(state): State<AppState>,
    Query(pagination): Query<Pagination>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> AppResult<Json<PaginatedResponse<UserResponse>>> {
    if !claims.role.has_permission(&UserRole::Admin) {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    let users = sqlx::query_as!(
        UserResponse,
        r#"
        SELECT id, email, name, role as "role: UserRole", is_active, last_login_at, created_at
        FROM users
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
        pagination.limit(),
        pagination.offset(),
    )
    .fetch_all(&state.db)
    .await?;

    let total = sqlx::query_scalar!(r#"SELECT COUNT(*) as "count!" FROM users"#)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(PaginatedResponse {
        data: users,
        meta: PaginationMeta {
            page: pagination.page,
            per_page: pagination.per_page,
            total,
            total_pages: ((total as f64) / (pagination.limit() as f64)).ceil() as u32,
        },
    }))
}

async fn get_current_user(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> AppResult<Json<UserResponse>> {
    let user = sqlx::query_as!(
        UserResponse,
        r#"
        SELECT id, email, name, role as "role: UserRole", is_active, last_login_at, created_at
        FROM users WHERE id = $1
        "#,
        claims.sub,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("User not found".to_string()))?;

    Ok(Json(user))
}

async fn get_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> AppResult<Json<UserResponse>> {
    // Users can view themselves; admins can view anyone
    if claims.sub != user_id && !claims.role.has_permission(&UserRole::Admin) {
        return Err(AppError::Forbidden("Cannot view other users".to_string()));
    }

    let user = sqlx::query_as!(
        UserResponse,
        r#"
        SELECT id, email, name, role as "role: UserRole", is_active, last_login_at, created_at
        FROM users WHERE id = $1
        "#,
        user_id,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("User not found".to_string()))?;

    Ok(Json(user))
}

#[derive(Debug, Deserialize)]
struct UpdateUserRequest {
    name: Option<String>,
    email: Option<String>,
}

async fn update_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(payload): Json<UpdateUserRequest>,
) -> AppResult<Json<UserResponse>> {
    if claims.sub != user_id && !claims.role.has_permission(&UserRole::Admin) {
        return Err(AppError::Forbidden("Cannot update other users".to_string()));
    }

    let user = sqlx::query_as!(
        UserResponse,
        r#"
        UPDATE users
        SET name = COALESCE($2, name),
            email = COALESCE($3, email),
            updated_at = NOW()
        WHERE id = $1
        RETURNING id, email, name, role as "role: UserRole", is_active, last_login_at, created_at
        "#,
        user_id,
        payload.name,
        payload.email,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("User not found".to_string()))?;

    Ok(Json(user))
}

#[derive(Debug, Deserialize)]
struct UpdateRoleRequest {
    role: UserRole,
}

async fn update_role(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(payload): Json<UpdateRoleRequest>,
) -> AppResult<Json<UserResponse>> {
    // Only super admins can change roles
    if !claims.role.has_permission(&UserRole::SuperAdmin) {
        return Err(AppError::Forbidden("SuperAdmin access required".to_string()));
    }

    let user = sqlx::query_as!(
        UserResponse,
        r#"
        UPDATE users SET role = $2, updated_at = NOW()
        WHERE id = $1
        RETURNING id, email, name, role as "role: UserRole", is_active, last_login_at, created_at
        "#,
        user_id,
        payload.role as UserRole,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("User not found".to_string()))?;

    Ok(Json(user))
}
