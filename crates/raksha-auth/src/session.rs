use chrono::{DateTime, Utc};
use deadpool_redis::Pool as RedisPool;
use raksha_core::error::{AppError, AppResult};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub ip_address: String,
    pub user_agent: String,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    pub is_active: bool,
}

#[derive(Clone)]
pub struct SessionManager {
    redis_pool: RedisPool,
    prefix: String,
    ttl_secs: i64,
}

impl SessionManager {
    pub fn new(redis_pool: RedisPool, ttl_secs: i64) -> Self {
        Self {
            redis_pool,
            prefix: "raksha:session:".to_string(),
            ttl_secs,
        }
    }

    fn key(&self, session_id: &Uuid) -> String {
        format!("{}{}", self.prefix, session_id)
    }

    pub async fn create_session(
        &self,
        user_id: Uuid,
        ip_address: String,
        user_agent: String,
    ) -> AppResult<Session> {
        let session = Session {
            id: Uuid::now_v7(),
            user_id,
            ip_address,
            user_agent,
            created_at: Utc::now(),
            last_active: Utc::now(),
            is_active: true,
        };

        let serialized = serde_json::to_string(&session)
            .map_err(|e| AppError::Internal(format!("Session serialization error: {e}")))?;

        let mut conn = self
            .redis_pool
            .get()
            .await
            .map_err(|e| AppError::Redis(e.to_string()))?;

        let key = self.key(&session.id);
        conn.set_ex::<_, _, ()>(&key, &serialized, self.ttl_secs as u64)
            .await
            .map_err(|e| AppError::Redis(e.to_string()))?;

        // Track user sessions
        let user_sessions_key = format!("{}user:{}", self.prefix, user_id);
        conn.sadd::<_, _, ()>(&user_sessions_key, session.id.to_string())
            .await
            .map_err(|e| AppError::Redis(e.to_string()))?;

        Ok(session)
    }

    pub async fn get_session(&self, session_id: &Uuid) -> AppResult<Option<Session>> {
        let mut conn = self
            .redis_pool
            .get()
            .await
            .map_err(|e| AppError::Redis(e.to_string()))?;

        let key = self.key(session_id);
        let data: Option<String> = conn
            .get(&key)
            .await
            .map_err(|e| AppError::Redis(e.to_string()))?;

        match data {
            Some(json) => {
                let session: Session = serde_json::from_str(&json)
                    .map_err(|e| AppError::Internal(format!("Session deserialization: {e}")))?;
                Ok(Some(session))
            }
            None => Ok(None),
        }
    }

    pub async fn invalidate_session(&self, session_id: &Uuid) -> AppResult<()> {
        let mut conn = self
            .redis_pool
            .get()
            .await
            .map_err(|e| AppError::Redis(e.to_string()))?;

        let key = self.key(session_id);
        conn.del::<_, ()>(&key)
            .await
            .map_err(|e| AppError::Redis(e.to_string()))?;

        Ok(())
    }

    pub async fn invalidate_all_user_sessions(&self, user_id: &Uuid) -> AppResult<()> {
        let mut conn = self
            .redis_pool
            .get()
            .await
            .map_err(|e| AppError::Redis(e.to_string()))?;

        let user_sessions_key = format!("{}user:{}", self.prefix, user_id);
        let session_ids: Vec<String> = conn
            .smembers(&user_sessions_key)
            .await
            .map_err(|e| AppError::Redis(e.to_string()))?;

        for sid in &session_ids {
            let key = format!("{}{}", self.prefix, sid);
            conn.del::<_, ()>(&key)
                .await
                .map_err(|e| AppError::Redis(e.to_string()))?;
        }

        conn.del::<_, ()>(&user_sessions_key)
            .await
            .map_err(|e| AppError::Redis(e.to_string()))?;

        Ok(())
    }
}
