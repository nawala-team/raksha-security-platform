use deadpool_redis::{Config as RedisConfig, Pool, Runtime};

use crate::config::RedisConfig as AppRedisConfig;

pub fn create_pool(config: &AppRedisConfig) -> Result<Pool, deadpool_redis::CreatePoolError> {
    let cfg = RedisConfig::from_url(&config.url);
    cfg.create_pool(Some(Runtime::Tokio1))
}
