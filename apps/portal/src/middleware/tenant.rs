//! Tenant extraction middleware for multi-tenancy support.
//!
//! Extracts `tenant_id` from JWT claims and injects a `TenantContext`
//! into request extensions. Superadmins can bypass tenant filtering.

use axum::{
    body::Body,
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use raksha_auth::Claims;
use raksha_core::error::AppError;
use raksha_core::models::UserRole;
use raksha_core::tenant::TenantContext;

use crate::state::AppState;

/// Middleware that extracts tenant context from authenticated requests.
///
/// Expected flow:
/// 1. Auth middleware runs first and inserts `Claims` into extensions.
/// 2. This middleware reads the claims, determines tenant scope.
/// 3. Injects `TenantContext` into extensions for downstream handlers.
///
/// Superadmins get an unscoped context allowing cross-tenant access.
/// All other users must have a `tenant_id` derived from their org membership.
pub async fn tenant_context_layer(
    State(state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    // Claims are inserted by the auth middleware that runs before us
    let claims = request
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    let tenant_ctx = if claims.role == UserRole::SuperAdmin {
        // Superadmins can optionally scope to a tenant via header
        let header_tenant = request
            .headers()
            .get("X-Tenant-Id")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| uuid::Uuid::parse_str(s).ok());

        match header_tenant {
            Some(tid) => {
                // Verify the tenant exists and is active
                let exists = sqlx::query_scalar!(
                    r#"SELECT EXISTS(SELECT 1 FROM tenants WHERE id = $1 AND status = 'active') as "exists!""#,
                    tid
                )
                .fetch_one(&state.db)
                .await?;

                if !exists {
                    return Err(AppError::NotFound(format!(
                        "Tenant {} not found or inactive",
                        tid
                    )));
                }

                TenantContext {
                    tenant_id: Some(tid),
                    is_superadmin: true,
                }
            }
            None => TenantContext::superadmin(),
        }
    } else {
        // Regular users: resolve tenant from their org membership
        let tenant_id = resolve_tenant_for_user(&state, &claims).await?;
        TenantContext::scoped(tenant_id)
    };

    tracing::debug!(
        tenant_id = ?tenant_ctx.tenant_id,
        is_superadmin = tenant_ctx.is_superadmin,
        user_id = %claims.sub,
        "Tenant context resolved"
    );

    request.extensions_mut().insert(tenant_ctx);
    Ok(next.run(request).await)
}

/// Resolve the tenant ID for a non-superadmin user.
/// Looks up the user's organization membership to determine their tenant.
async fn resolve_tenant_for_user(
    state: &AppState,
    claims: &Claims,
) -> Result<uuid::Uuid, AppError> {
    let tenant_id = sqlx::query_scalar!(
        r#"
        SELECT t.id as "id!"
        FROM tenants t
        INNER JOIN user_roles ura ON ura.org_id = t.id
        WHERE ura.user_id = $1 AND ura.is_active = true AND t.status = 'active'
        ORDER BY ura.granted_at ASC
        LIMIT 1
        "#,
        claims.sub
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| {
        AppError::Forbidden("User is not assigned to any active tenant".to_string())
    })?;

    Ok(tenant_id)
}
