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
        .route("/logout", post(logout))
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

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<Json<AuthResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    // Fetch user from database
    let user = sqlx::query!(
        r#"
        SELECT id, email, name, password_hash, role as "role: UserRole"
        FROM users WHERE email = $1 AND is_active = true
        "#,
        payload.email,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::Unauthorized)?;

    // Verify password
    let valid = PasswordService::verify_password(&payload.password, &user.password_hash)?;
    if !valid {
        return Err(AppError::Unauthorized);
    }

    // Create session
    let session = state
        .session_manager
        .create_session(user.id, "0.0.0.0".to_string(), "unknown".to_string())
        .await?;

    // Generate tokens
    let tokens = state
        .token_service
        .generate_token_pair(user.id, user.role.clone(), session.id)?;

    // Update last login
    sqlx::query!("UPDATE users SET last_login_at = NOW() WHERE id = $1", user.id)
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

    // Check for existing user
    let exists = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1) as \"exists!\"",
        payload.email,
    )
    .fetch_one(&state.db)
    .await?;

    if exists {
        return Err(AppError::Conflict("Email already registered".to_string()));
    }

    // Hash password
    let password_hash = PasswordService::hash_password(&payload.password)?;

    // Create user
    let user_id = raksha_core::models::new_id();
    let role = UserRole::Viewer; // Default role for self-registration

    sqlx::query!(
        r#"
        INSERT INTO users (id, email, name, password_hash, role, is_active, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, true, NOW(), NOW())
        "#,
        user_id,
        payload.email,
        payload.name,
        password_hash,
        role as UserRole,
    )
    .execute(&state.db)
    .await?;

    // Create session
    let session = state
        .session_manager
        .create_session(user_id, "0.0.0.0".to_string(), "unknown".to_string())
        .await?;

    // Generate tokens
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

    // Verify session is active
    let session = state
        .session_manager
        .get_session(&claims.sid)
        .await?
        .ok_or(AppError::Unauthorized)?;

    if !session.is_active {
        return Err(AppError::Unauthorized);
    }

    // Generate new token pair
    let tokens = state
        .token_service
        .generate_token_pair(claims.sub, claims.role, claims.sid)?;

    Ok(Json(tokens))
}

async fn logout(
    State(state): State<AppState>,
    claims: axum::Extension<Claims>,
) -> AppResult<Json<serde_json::Value>> {
    state.session_manager.invalidate_session(&claims.sid).await?;

    Ok(Json(serde_json::json!({ "message": "Logged out successfully" })))
}
