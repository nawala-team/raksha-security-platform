use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use raksha_core::error::AppError;
use raksha_core::models::UserRole;

use crate::Claims;

/// Middleware that requires a minimum role level
#[derive(Clone)]
pub struct RequireRole {
    pub minimum_role: UserRole,
}

impl RequireRole {
    pub fn new(minimum_role: UserRole) -> Self {
        Self { minimum_role }
    }
}

/// Axum middleware function to enforce role-based access
pub async fn require_role(
    minimum_role: UserRole,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let claims = request
        .extensions()
        .get::<Claims>()
        .ok_or(AppError::Unauthorized)?;

    if !claims.role.has_permission(&minimum_role) {
        return Err(AppError::Forbidden(format!(
            "Requires {:?} role or higher",
            minimum_role
        )));
    }

    Ok(next.run(request).await)
}
