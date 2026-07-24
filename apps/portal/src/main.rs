mod handlers;
mod middleware;
mod routes;
mod services;
mod state;

use std::net::SocketAddr;

use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration
    let config = raksha_core::AppConfig::load()?;

    // Initialize tracing
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            EnvFilter::new(&config.app.log_level)
        }))
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    info!(
        app = %config.app.name,
        env = ?config.app.environment,
        "Starting Raksha Security Platform"
    );

    // Build application state
    let state = AppState::new(&config).await?;

    // Build router
    let app = routes::build_router(state);

    // Start server
    let addr = SocketAddr::new(
        config.server.host.parse()?,
        config.server.port,
    );
    let listener = TcpListener::bind(addr).await?;

    info!(%addr, "Server listening");

    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .await?;

    Ok(())
}
