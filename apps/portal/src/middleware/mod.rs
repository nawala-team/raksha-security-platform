pub mod tenant;

use axum::{
    body::Body,
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use raksha_auth::middleware::auth_middleware;
use raksha_core::error::AppError;

use crate::state::AppState;

pub use tenant::tenant_context_layer;

/// Authentication middleware layer
pub async fn auth_layer(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    auth_middleware(state.token_service, state.session_manager, request, next).await
}

/// Rate limiting middleware using Redis sliding window
pub async fn rate_limit_layer(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    use redis::AsyncCommands;

    let ip = request
        .headers()
        .get("x-forwarded-for")
        .or_else(|| request.headers().get("x-real-ip"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    let key = format!("raksha:ratelimit:{}", ip);
    let window_secs: u64 = 60;
    let max_requests: i64 = 1000;  // Increased for production

    let mut conn = state
        .redis
        .get()
        .await
        .map_err(|e| AppError::Redis(e.to_string()))?;

    let count: i64 = conn
        .incr(&key, 1i64)
        .await
        .map_err(|e| AppError::Redis(e.to_string()))?;

    if count == 1 {
        conn.expire::<_, ()>(&key, window_secs as i64)
            .await
            .map_err(|e| AppError::Redis(e.to_string()))?;
    }

    if count > max_requests {
        return Err(AppError::RateLimited);
    }

    Ok(next.run(request).await)
}
