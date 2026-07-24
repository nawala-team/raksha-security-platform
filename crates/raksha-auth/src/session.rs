use chrono::{DateTime, Utc};
use deadpool_redis::Pool as RedisPool;
use raksha_core::error::{AppError, AppResult};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Maximum concurrent sessions per user (prevents session exhaustion)
const MAX_SESSIONS_PER_USER: usize = 10;

/// Minimum session TTL: 5 minutes
const MIN_SESSION_TTL_SECS: i64 = 300;

/// Maximum session TTL: 24 hours
const MAX_SESSION_TTL_SECS: i64 = 86_400;

/// Maximum length for user_agent string stored in session
const MAX_USER_AGENT_LEN: usize = 512;

/// Maximum length for IP address string
const MAX_IP_ADDRESS_LEN: usize = 45; // IPv6 max length

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub ip_address: String,
    pub user_agent: String,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    pub is_active: bool,
    /// Fingerprint for binding detection (hash of ip + user_agent)
    pub fingerprint: String,
}

#[derive(Clone)]
pub struct SessionManager {
    redis_pool: RedisPool,
    prefix: String,
    ttl_secs: i64,
}

impl SessionManager {
    pub fn new(redis_pool: RedisPool, ttl_secs: i64) -> Self {
        // Enforce TTL bounds at construction
        let clamped_ttl = ttl_secs.clamp(MIN_SESSION_TTL_SECS, MAX_SESSION_TTL_SECS);
        if clamped_ttl != ttl_secs {
            tracing::warn!(
                requested = ttl_secs,
                applied = clamped_ttl,
                "Session TTL clamped to safe bounds"
            );
        }

        Self {
            redis_pool,
            prefix: "raksha:session:".to_string(),
            ttl_secs: clamped_ttl,
        }
    }

    fn key(&self, session_id: &Uuid) -> String {
        format!("{}{}", self.prefix, session_id)
    }

    /// Compute a session fingerprint from IP and user agent.
    fn compute_fingerprint(ip_address: &str, user_agent: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(ip_address.as_bytes());
        hasher.update(b"|");
        hasher.update(user_agent.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub async fn create_session(
        &self,
        user_id: Uuid,
        ip_address: String,
        user_agent: String,
    ) -> AppResult<Session> {
        // Sanitize inputs — truncate to safe lengths
        let ip_address = if ip_address.len() > MAX_IP_ADDRESS_LEN {
            ip_address[..MAX_IP_ADDRESS_LEN].to_string()
        } else {
            ip_address
        };

        let user_agent = if user_agent.len() > MAX_USER_AGENT_LEN {
            user_agent[..MAX_USER_AGENT_LEN].to_string()
        } else {
            user_agent
        };

        // Enforce max sessions per user
        let existing_count = self.get_user_session_count(&user_id).await?;
        if existing_count >= MAX_SESSIONS_PER_USER {
            tracing::warn!(
                user_id = %user_id,
                session_count = existing_count,
                "Max sessions reached — evicting oldest"
            );
            self.evict_oldest_user_session(&user_id).await?;
        }

        let fingerprint = Self::compute_fingerprint(&ip_address, &user_agent);

        let session = Session {
            id: Uuid::now_v7(),
            user_id,
            ip_address,
            user_agent,
            created_at: Utc::now(),
            last_active: Utc::now(),
            is_active: true,
            fingerprint,
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

        // Get session first to clean up user index
        let key = self.key(session_id);
        let data: Option<String> = conn
            .get(&key)
            .await
            .map_err(|e| AppError::Redis(e.to_string()))?;

        if let Some(json) = &data {
            if let Ok(session) = serde_json::from_str::<Session>(json) {
                let user_sessions_key = format!("{}user:{}", self.prefix, session.user_id);
                conn.srem::<_, _, ()>(&user_sessions_key, session_id.to_string())
                    .await
                    .map_err(|e| AppError::Redis(e.to_string()))?;
            }
        }

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

        tracing::info!(
            user_id = %user_id,
            sessions_invalidated = session_ids.len(),
            "All user sessions invalidated"
        );

        Ok(())
    }

    /// Get the number of active sessions for a user
    async fn get_user_session_count(&self, user_id: &Uuid) -> AppResult<usize> {
        let mut conn = self
            .redis_pool
            .get()
            .await
            .map_err(|e| AppError::Redis(e.to_string()))?;

        let user_sessions_key = format!("{}user:{}", self.prefix, user_id);
        let count: usize = conn
            .scard(&user_sessions_key)
            .await
            .map_err(|e| AppError::Redis(e.to_string()))?;

        Ok(count)
    }

    /// Evict the oldest session for a user when max is reached
    async fn evict_oldest_user_session(&self, user_id: &Uuid) -> AppResult<()> {
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

        // Find oldest session by created_at
        let mut oldest_id: Option<String> = None;
        let mut oldest_time: Option<DateTime<Utc>> = None;

        for sid in &session_ids {
            let key = format!("{}{}", self.prefix, sid);
            let data: Option<String> = conn
                .get(&key)
                .await
                .map_err(|e| AppError::Redis(e.to_string()))?;

            if let Some(json) = data {
                if let Ok(session) = serde_json::from_str::<Session>(&json) {
                    if oldest_time.is_none() || session.created_at < oldest_time.unwrap() {
                        oldest_time = Some(session.created_at);
                        oldest_id = Some(sid.clone());
                    }
                }
            }
        }

        if let Some(sid) = oldest_id {
            let key = format!("{}{}", self.prefix, sid);
            conn.del::<_, ()>(&key)
                .await
                .map_err(|e| AppError::Redis(e.to_string()))?;
            conn.srem::<_, _, ()>(&user_sessions_key, &sid)
                .await
                .map_err(|e| AppError::Redis(e.to_string()))?;
        }

        Ok(())
    }
}
