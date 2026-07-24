use sqlx::postgres::{PgPool, PgPoolOptions};

use crate::config::DatabaseConfig;

pub async fn create_pool(config: &DatabaseConfig) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .idle_timeout(std::time::Duration::from_secs(600))
        .connect(&config.url)
        .await
}
