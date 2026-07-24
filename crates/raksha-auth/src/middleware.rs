use axum::{
    body::Body,
    extract::Request,
    http::header::AUTHORIZATION,
    middleware::Next,
    response::Response,
};
use raksha_core::error::AppError;

use crate::{Claims, SessionManager, TokenService};

/// Auth middleware layer - validates JWT and attaches claims to request
#[derive(Clone)]
pub struct AuthLayer {
    pub token_service: TokenService,
    pub session_manager: SessionManager,
}

impl AuthLayer {
    pub fn new(token_service: TokenService, session_manager: SessionManager) -> Self {
        Self {
            token_service,
            session_manager,
        }
    }
}

/// Middleware function for authentication
pub async fn auth_middleware(
    token_service: TokenService,
    session_manager: SessionManager,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(AppError::Unauthorized)?;

    let claims = token_service.verify_token(token)?;

    // Verify token type is "access"
    if claims.token_type != "access" {
        return Err(AppError::Jwt("Invalid token type".to_string()));
    }

    // Verify session is still active
    let session = session_manager
        .get_session(&claims.sid)
        .await?
        .ok_or(AppError::Unauthorized)?;

    if !session.is_active {
        return Err(AppError::Unauthorized);
    }

    // Attach claims to request extensions
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}
