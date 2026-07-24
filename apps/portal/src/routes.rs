use axum::{
    middleware as axum_middleware,
    response::IntoResponse,
    http::{header, StatusCode},
    Router,
};
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};

use crate::handlers;
use crate::middleware::{auth_layer, rate_limit_layer};
use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Public routes (no auth required)
    let public_routes = Router::new()
        .nest("/auth", handlers::auth::routes())
        .nest("/health", handlers::health::routes())
        .route("/agents/enroll", axum::routing::post(handlers::enrollment::enroll_agent))
        .route("/agent/install", axum::routing::get(serve_install_script))
        .route("/agent/install.ps1", axum::routing::get(serve_install_ps1));

    // Protected routes (auth required)
    let protected_routes = Router::new()
        .nest("/users", handlers::users::routes())
        .nest("/alerts", handlers::alerts::routes())
        .nest("/agents", handlers::agents::routes())
        .route("/agents/tokens", axum::routing::post(handlers::enrollment::generate_token))
        .route("/agents/tokens", axum::routing::get(handlers::enrollment::list_tokens))
        .route("/agents/tokens/{token_id}", axum::routing::delete(handlers::enrollment::revoke_token))
        .route("/agents/{agent_id}/rotate-certificate", axum::routing::post(handlers::enrollment::rotate_certificate))
        .nest("/compliance", handlers::compliance::routes())
        .nest("/audit", handlers::audit::routes())
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            auth_layer,
        ));

    // Combine all routes under /api/v1
    let api = Router::new()
        .merge(public_routes)
        .merge(protected_routes);

    Router::new()
        .nest("/api/v1", api)
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(std::time::Duration::from_secs(30)))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            rate_limit_layer,
        ))
        .with_state(state)
}

/// Serve the Linux/macOS install script
async fn serve_install_script() -> impl IntoResponse {
    let script = include_str!("../../../scripts/install-agent.sh");
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/x-shellscript")],
        script,
    )
}

/// Serve the Windows PowerShell install script
async fn serve_install_ps1() -> impl IntoResponse {
    let script = include_str!("../../../scripts/install-agent.ps1");
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/plain")],
        script,
    )
}

