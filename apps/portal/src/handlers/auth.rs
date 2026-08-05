use axum::{
    extract::State,
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use raksha_auth::{Claims, PasswordService, TokenPair};
use raksha_core::error::{AppError, AppResult};
use raksha_core::models::UserRole;

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
        .route("/refresh", post(refresh_token))
}

#[derive(Debug, Deserialize, Validate)]
struct LoginRequest {
    #[validate(email)]
    email: String,
    #[validate(length(min = 8, max = 128))]
    password: String,
}

#[derive(Debug, Serialize)]
struct AuthResponse {
    user: UserInfo,
    tokens: TokenPair,
}

#[derive(Debug, Serialize)]
struct UserInfo {
    id: Uuid,
    email: String,
    name: String,
    role: UserRole,
}

#[derive(Debug, sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    email: String,
    name: String,
    password_hash: String,
    role: UserRole,
}

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<Json<AuthResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    let user: UserRow = sqlx::query_as(
        r#"
        SELECT id, email, name, password_hash, role
        FROM users WHERE email = $1 AND is_active = true
        "#
    )
    .bind(&payload.email)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::Unauthorized)?;

    let valid = PasswordService::verify_password(&payload.password, &user.password_hash)?;
    if !valid {
        return Err(AppError::Unauthorized);
    }

    let session = state
        .session_manager
        .create_session(user.id, "0.0.0.0".to_string(), "unknown".to_string())
        .await?;

    let tokens = state
        .token_service
        .generate_token_pair(user.id, user.role.clone(), session.id)?;

    sqlx::query("UPDATE users SET last_login_at = NOW() WHERE id = $1")
        .bind(user.id)
        .execute(&state.db)
        .await?;

    Ok(Json(AuthResponse {
        user: UserInfo {
            id: user.id,
            email: user.email,
            name: user.name,
            role: user.role,
        },
        tokens,
    }))
}

#[derive(Debug, Deserialize, Validate)]
struct RegisterRequest {
    #[validate(email)]
    email: String,
    #[validate(length(min = 8, max = 128))]
    password: String,
    #[validate(length(min = 1, max = 100))]
    name: String,
}

async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> AppResult<Json<AuthResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)"
    )
    .bind(&payload.email)
    .fetch_one(&state.db)
    .await
    .unwrap_or(false);

    if exists {
        return Err(AppError::Conflict("Email already registered".to_string()));
    }

    let password_hash = PasswordService::hash_password(&payload.password)?;
    let user_id = raksha_core::models::new_id();
    let role = UserRole::Viewer;

    sqlx::query(
        r#"
        INSERT INTO users (id, email, name, password_hash, role, is_active, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, true, NOW(), NOW())
        "#
    )
    .bind(user_id)
    .bind(&payload.email)
    .bind(&payload.name)
    .bind(&password_hash)
    .bind(&role)
    .execute(&state.db)
    .await?;

    assign_default_tenant_role(&state, user_id, &role).await?;

    let session = state
        .session_manager
        .create_session(user_id, "0.0.0.0".to_string(), "unknown".to_string())
        .await?;

    let tokens = state
        .token_service
        .generate_token_pair(user_id, role.clone(), session.id)?;

    Ok(Json(AuthResponse {
        user: UserInfo {
            id: user_id,
            email: payload.email,
            name: payload.name,
            role,
        },
        tokens,
    }))
}

const DEFAULT_TENANT_ID: Uuid = Uuid::from_u128(0x00000000_0000_0000_0000_000000000001);

fn role_name_for(role: &UserRole) -> &'static str {
    match role {
        UserRole::SuperAdmin => "super_admin",
        UserRole::Admin => "tenant_admin",
        UserRole::Analyst => "analyst",
        UserRole::Operator => "operator",
        UserRole::Viewer => "viewer",
    }
}

async fn assign_default_tenant_role(
    state: &AppState,
    user_id: Uuid,
    role: &UserRole,
) -> AppResult<()> {
    let role_name = role_name_for(role);

    let role_id: Option<Uuid> = sqlx::query_scalar("SELECT id FROM roles WHERE name = $1")
        .bind(role_name)
        .fetch_optional(&state.db)
        .await?;

    let role_id = role_id.ok_or_else(|| {
        AppError::Internal(format!("Built-in role '{}' is missing from the database", role_name))
    })?;

    sqlx::query(
        r#"
        INSERT INTO user_roles (user_id, role_id, org_id, is_active)
        VALUES ($1, $2, $3, true)
        ON CONFLICT (user_id, role_id, org_id) DO NOTHING
        "#
    )
    .bind(user_id)
    .bind(role_id)
    .bind(DEFAULT_TENANT_ID)
    .execute(&state.db)
    .await?;

    Ok(())
}

#[derive(Debug, Deserialize)]
struct RefreshRequest {
    refresh_token: String,
}

async fn refresh_token(
    State(state): State<AppState>,
    Json(payload): Json<RefreshRequest>,
) -> AppResult<Json<TokenPair>> {
    let claims = state.token_service.verify_token(&payload.refresh_token)?;

    if claims.token_type != "refresh" {
        return Err(AppError::Jwt("Invalid token type".to_string()));
    }

    let session = state
        .session_manager
        .get_session(&claims.sid)
        .await?
        .ok_or(AppError::Unauthorized)?;

    if !session.is_active {
        return Err(AppError::Unauthorized);
    }

    let tokens = state
        .token_service
        .generate_token_pair(claims.sub, claims.role, claims.sid)?;

    Ok(Json(tokens))
}

pub async fn logout(
    State(state): State<AppState>,
    claims: axum::Extension<Claims>,
) -> AppResult<Json<serde_json::Value>> {
    state.session_manager.invalidate_session(&claims.sid).await?;
    Ok(Json(serde_json::json!({ "message": "Logged out successfully" })))
}
