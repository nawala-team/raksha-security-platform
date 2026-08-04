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

/// Permission levels for fine-grained access control.
/// Higher numeric values represent more privilege.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PermissionLevel {
    /// Can read resources
    Read = 1,
    /// Can create resources
    Create = 2,
    /// Can modify existing resources
    Update = 3,
    /// Can remove resources
    Delete = 4,
    /// Full administrative access
    Admin = 5,
}

/// Maps roles to their maximum permission level.
/// Hierarchy: Viewer < Operator/Analyst < Admin < SuperAdmin
pub fn role_permission_level(role: &UserRole) -> PermissionLevel {
    match role {
        UserRole::Viewer => PermissionLevel::Read,
        UserRole::Operator => PermissionLevel::Create,
        UserRole::Analyst => PermissionLevel::Create,
        UserRole::Admin => PermissionLevel::Admin,
        UserRole::SuperAdmin => PermissionLevel::Admin,
    }
}

/// Axum middleware function to enforce role-based access.
/// Checks that the authenticated user's role meets the minimum required level.
/// Returns 401 if no claims are present, 403 if insufficient permissions.
pub async fn require_role(
    minimum_role: UserRole,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let claims = request
        .extensions()
        .get::<Claims>()
        .ok_or(AppError::Unauthorized)?;

    // Verify the token hasn't been tampered with by checking token_type
    if claims.token_type != "access" {
        return Err(AppError::Unauthorized);
    }

    if !claims.role.has_permission(&minimum_role) {
        tracing::warn!(
            user_id = %claims.sub,
            session_id = %claims.sid,
            user_role = ?claims.role,
            required_role = ?minimum_role,
            "RBAC denial: insufficient role"
        );
        return Err(AppError::Forbidden(format!(
            "Requires {:?} role or higher",
            minimum_role
        )));
    }

    Ok(next.run(request).await)
}

/// Axum middleware function to enforce permission-level access.
/// More granular than role-based — checks specific operation permissions.
pub async fn require_permission(
    required_level: PermissionLevel,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let claims = request
        .extensions()
        .get::<Claims>()
        .ok_or(AppError::Unauthorized)?;

    if claims.token_type != "access" {
        return Err(AppError::Unauthorized);
    }

    let user_level = role_permission_level(&claims.role);

    if user_level < required_level {
        tracing::warn!(
            user_id = %claims.sub,
            session_id = %claims.sid,
            user_level = ?user_level,
            required_level = ?required_level,
            "Permission denial: insufficient permission level"
        );
        return Err(AppError::Forbidden(format!(
            "Requires {:?} permission or higher",
            required_level
        )));
    }

    Ok(next.run(request).await)
}
