use std::time::Instant;

use axum::{
    body::Body,
    extract::Request,
    middleware::Next,
    response::Response,
};
use tracing::info;

use crate::models::AuditAction;
use crate::store::AuditStore;

/// Audit trail middleware that logs every request
pub async fn audit_middleware(
    audit_store: AuditStore,
    request: Request<Body>,
    next: Next,
) -> Response {
    let start = Instant::now();

    // Extract request info before passing ownership
    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    let ip_address = request
        .headers()
        .get("x-forwarded-for")
        .or_else(|| request.headers().get("x-real-ip"))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // Extract user_id from extensions (set by auth middleware)
    let user_id = request
        .extensions()
        .get::<raksha_auth::Claims>()
        .map(|c| c.sub);

    let response = next.run(request).await;

    let duration = start.elapsed();
    let status = response.status().as_u16();

    // Determine action from method
    let action = match method.as_str() {
        "GET" => AuditAction::Read,
        "POST" => AuditAction::Create,
        "PUT" | "PATCH" => AuditAction::Update,
        "DELETE" => AuditAction::Delete,
        _ => AuditAction::Read,
    };

    // Record audit entry asynchronously (don't block response)
    let store = audit_store.clone();
    let method_clone = method.clone();
    let path_clone = path.clone();
    tokio::spawn(async move {
        if let Err(e) = store
            .create_entry(
                user_id,
                action,
                "http".to_string(),
                None,
                ip_address.clone(),
                user_agent.clone(),
                method_clone,
                path_clone,
                status,
                duration.as_millis() as u64,
                None,
            )
            .await
        {
            tracing::error!(error = %e, "Failed to record audit entry");
        }
    });

    info!(
        method = %method,
        path = %path,
        status = status,
        duration_ms = duration.as_millis() as u64,
        "Request completed"
    );

    response
}
