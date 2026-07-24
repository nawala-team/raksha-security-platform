use axum::{
    body::Body,
    extract::Request,
    http::header::AUTHORIZATION,
    middleware::Next,
    response::Response,
};
use raksha_core::error::AppError;

use crate::{Claims, SessionManager, TokenService};

/// Maximum Authorization header length (prevents memory abuse)
const MAX_AUTH_HEADER_LEN: usize = 4096;

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

/// Middleware function for authentication.
/// Validates JWT signature, expiry, audience, issuer, and session liveness.
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

    // Reject excessively long headers (potential DoS vector)
    if auth_header.len() > MAX_AUTH_HEADER_LEN {
        return Err(AppError::Unauthorized);
    }

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(AppError::Unauthorized)?;

    // Reject empty tokens
    if token.is_empty() {
        return Err(AppError::Unauthorized);
    }

    // verify_access_token checks signature, expiry, issuer, audience, nbf, AND token_type
    let claims = token_service.verify_access_token(token)?;

    // Verify session is still active in Redis (enables server-side revocation)
    let session = session_manager
        .get_session(&claims.sid)
        .await?
        .ok_or_else(|| {
            tracing::warn!(
                user_id = %claims.sub,
                session_id = %claims.sid,
                "Auth rejected: session not found (revoked or expired)"
            );
            AppError::Unauthorized
        })?;

    if !session.is_active {
        tracing::warn!(
            user_id = %claims.sub,
            session_id = %claims.sid,
            "Auth rejected: session marked inactive"
        );
        return Err(AppError::Unauthorized);
    }

    // Verify the session belongs to the claimed user (prevents session fixation)
    if session.user_id != claims.sub {
        tracing::error!(
            token_user = %claims.sub,
            session_user = %session.user_id,
            session_id = %claims.sid,
            "CRITICAL: Token user does not match session user — possible token theft"
        );
        // Invalidate the suspicious session immediately
        let _ = session_manager.invalidate_session(&claims.sid).await;
        return Err(AppError::Unauthorized);
    }

    // Attach claims to request extensions for downstream handlers
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}
