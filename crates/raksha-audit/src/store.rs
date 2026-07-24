use raksha_core::error::AppResult;
use raksha_core::models::new_id;
use sqlx::PgPool;

use crate::hashchain::HashChain;
use crate::models::{AuditAction, AuditEntry};

#[derive(Clone)]
pub struct AuditStore {
    db: PgPool,
}

impl AuditStore {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn record(&self, entry: AuditEntry) -> AppResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO audit_log (id, timestamp, user_id, action, resource_type, resource_id,
                ip_address, user_agent, request_method, request_path, response_status,
                duration_ms, metadata, hash, previous_hash)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
            entry.id,
            entry.timestamp,
            entry.user_id,
            entry.action.to_string(),
            entry.resource_type,
            entry.resource_id,
            entry.ip_address,
            entry.user_agent,
            entry.request_method,
            entry.request_path,
            entry.response_status as i32,
            entry.duration_ms as i64,
            entry.metadata,
            entry.hash,
            entry.previous_hash,
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn get_latest_hash(&self) -> AppResult<Option<String>> {
        let result = sqlx::query_scalar!(
            r#"SELECT hash FROM audit_log ORDER BY timestamp DESC LIMIT 1"#
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(result)
    }

    pub async fn create_entry(
        &self,
        user_id: Option<uuid::Uuid>,
        action: AuditAction,
        resource_type: String,
        resource_id: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
        request_method: String,
        request_path: String,
        response_status: u16,
        duration_ms: u64,
        metadata: Option<serde_json::Value>,
    ) -> AppResult<AuditEntry> {
        let now = chrono::Utc::now();
        let previous_hash = self.get_latest_hash().await?;

        let hash = HashChain::compute_hash(
            &now.to_rfc3339(),
            &user_id.map(|u| u.to_string()).unwrap_or_default(),
            &action.to_string(),
            &format!("{}:{}", resource_type, resource_id.as_deref().unwrap_or("")),
            previous_hash.as_deref(),
        );

        let entry = AuditEntry {
            id: new_id(),
            timestamp: now,
            user_id,
            action,
            resource_type,
            resource_id,
            ip_address,
            user_agent,
            request_method,
            request_path,
            response_status,
            duration_ms,
            metadata,
            hash,
            previous_hash,
        };

        self.record(entry.clone()).await?;
        Ok(entry)
    }
}
