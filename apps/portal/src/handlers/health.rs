use axum::{extract::State, routing::get, Json, Router};
use serde::Serialize;

use crate::state::AppState;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
    uptime_secs: u64,
}

#[derive(Serialize)]
struct ReadyResponse {
    database: bool,
    redis: bool,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(health_check))
        .route("/ready", get(readiness_check))
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy",
        version: env!("CARGO_PKG_VERSION"),
        uptime_secs: 0, // Would track actual uptime via state
    })
}

async fn readiness_check(State(state): State<AppState>) -> Json<ReadyResponse> {
    let db_ok = sqlx::query("SELECT 1")
        .execute(&state.db)
        .await
        .is_ok();

    let redis_ok = state.redis.get().await.is_ok();

    Json(ReadyResponse {
        database: db_ok,
        redis: redis_ok,
    })
}
