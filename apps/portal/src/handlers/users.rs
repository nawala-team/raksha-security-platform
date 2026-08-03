use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use raksha_auth::{Claims, PasswordService};
use raksha_core::error::{AppError, AppResult};
use raksha_core::models::{new_id, Pagination, PaginatedResponse, PaginationMeta, UserRole};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_users).post(create_user))
        .route("/me", get(get_current_user))
        .route("/:id", get(get_user).put(update_user).delete(delete_user))
        .route("/:id/role", put(update_role))
}

/// The default tenant seeded by `migrations/20260724010000_create_tenants.sql`.
const DEFAULT_TENANT_ID: Uuid = Uuid::from_u128(0x00000000_0000_0000_0000_000000000001);

/// Map a `UserRole` to the matching row name in the `roles` table.
/// The role names are seeded by `20260724020001_create_user_roles.sql` and do
/// not always match the enum's snake_case form (`Admin` -> `tenant_admin`).
fn role_name_for(role: &UserRole) -> &'static str {
    match role {
        UserRole::SuperAdmin => "super_admin",
        UserRole::Admin => "tenant_admin",
        UserRole::Analyst => "analyst",
        UserRole::Operator => "operator",
        UserRole::Viewer => "viewer",
    }
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

#[derive(Debug, Deserialize)]
struct CreateUserRequest {
    email: String,
    name: String,
    password: String,
    #[serde(default)]
    role: Option<UserRole>,
}

async fn create_user(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(payload): Json<CreateUserRequest>,
) -> AppResult<Json<UserResponse>> {
    // Only admins and above can create users.
    if !claims.role.has_permission(&UserRole::Admin) {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    let email = payload.email.trim().to_lowercase();
    if payload.password.len() < 8 {
        return Err(AppError::Validation(
            "Password must be at least 8 characters".to_string(),
        ));
    }

    // Reject duplicate emails.
    let exists = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1) as \"exists!\"",
        email,
    )
    .fetch_one(&state.db)
    .await?;
    if exists {
        return Err(AppError::Conflict("Email already registered".to_string()));
    }

    // Hash the password (argon2) and pick the target role.
    let password_hash = PasswordService::hash_password(&payload.password)?;
    let role = payload.role.unwrap_or(UserRole::Viewer);
    let user_id = new_id();

    sqlx::query!(
        r#"
        INSERT INTO users (id, email, name, password_hash, role, is_active, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, true, NOW(), NOW())
        "#,
        user_id,
        email,
        payload.name,
        password_hash,
        role.clone() as UserRole,
    )
    .execute(&state.db)
    .await?;

    // Grant membership in the default tenant with the matching named role so
    // `tenant_context_layer` can resolve a tenant for the new user.
    let role_name = role_name_for(&role);
    let role_id = sqlx::query_scalar!("SELECT id FROM roles WHERE name = $1", role_name)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| {
            AppError::Internal(format!("Built-in role '{role_name}' is missing from the database"))
        })?;

    sqlx::query!(
        r#"
        INSERT INTO user_roles (user_id, role_id, org_id, is_active)
        VALUES ($1, $2, $3, true)
        ON CONFLICT (user_id, role_id, org_id) DO NOTHING
        "#,
        user_id,
        role_id,
        DEFAULT_TENANT_ID,
    )
    .execute(&state.db)
    .await?;

    let user = sqlx::query_as!(
        UserResponse,
        r#"
        SELECT id, email, name, role as "role: UserRole", is_active, last_login_at, created_at
        FROM users WHERE id = $1
        "#,
        user_id,
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(user))
}

async fn delete_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> AppResult<Json<serde_json::Value>> {
    // Only admins and above can delete users.
    if !claims.role.has_permission(&UserRole::Admin) {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }
    if claims.sub == user_id {
        return Err(AppError::Validation(
            "You cannot delete your own account".to_string(),
        ));
    }

    let result = sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
        .execute(&state.db)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("User not found".to_string()));
    }

    Ok(Json(serde_json::json!({ "deleted": true, "id": user_id })))
}

