use raksha_core::error::AppResult;
use raksha_core::models::new_id;
use sqlx::PgPool;

use crate::hashchain::HashChain;
use crate::models::{AuditAction, AuditEntry};

/// Folds the request-scoped audit details into a single JSONB payload.
///
/// `audit_trail` has no columns for HTTP method/path/status/latency, so they
/// are preserved here together with the verbatim application action name
/// (which is more granular than the `audit_action_type` database enum).
fn build_metadata(entry: &AuditEntry) -> serde_json::Value {
    let mut map = match entry.metadata.clone() {
        Some(serde_json::Value::Object(map)) => map,
        _ => serde_json::Map::new(),
    };

    map.insert(
        "action".to_string(),
        serde_json::Value::String(entry.action.to_string()),
    );
    map.insert(
        "request_method".to_string(),
        serde_json::Value::String(entry.request_method.clone()),
    );
    map.insert(
        "request_path".to_string(),
        serde_json::Value::String(entry.request_path.clone()),
    );
    map.insert(
        "response_status".to_string(),
        serde_json::Value::from(entry.response_status),
    );
    map.insert(
        "duration_ms".to_string(),
        serde_json::Value::from(entry.duration_ms),
    );
    if let Some(user_agent) = &entry.user_agent {
        map.insert(
            "user_agent".to_string(),
            serde_json::Value::String(user_agent.clone()),
        );
    }

    serde_json::Value::Object(map)
}

#[derive(Clone)]
pub struct AuditStore {
    db: PgPool,
}

impl AuditStore {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn record(&self, entry: AuditEntry) -> AppResult<()> {
        // The request-scoped details (method, path, status, latency) have no
        // dedicated columns on `audit_trail`, so they are folded into
        // `metadata` alongside the verbatim application-level action name.
        let metadata = build_metadata(&entry);

        sqlx::query!(
            r#"
            INSERT INTO audit_trail (
                id, timestamp, actor_id, action_type, action_category,
                resource_type, resource_id, actor_ip, metadata, risk_level,
                integrity_hash, previous_hash
            )
            VALUES (
                $1, $2, $3,
                $4::text::audit_action_type,
                $5::text::audit_action_category,
                $6, $7,
                $8::text::inet,
                $9,
                $10::text::audit_risk_level,
                $11, $12
            )
            "#,
            entry.id,
            entry.timestamp,
            entry.user_id,
            entry.action.db_action_type(),
            entry.action.db_action_category(),
            entry.resource_type,
            entry.resource_id,
            entry.ip_address,
            metadata,
            entry.action.db_risk_level(),
            entry.hash,
            entry.previous_hash,
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn get_latest_hash(&self) -> AppResult<Option<String>> {
        let result = sqlx::query_scalar!(
            r#"SELECT integrity_hash FROM audit_trail ORDER BY timestamp DESC, id DESC LIMIT 1"#
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
