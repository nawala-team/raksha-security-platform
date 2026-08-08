use raksha_core::error::AppResult;
use raksha_core::models::{new_id, AlertStatus};
use sqlx::PgPool;
use once_cell::sync::Lazy;

use crate::models::{Alert, AlertFilter, CreateAlert};

// Pre-compiled regex patterns for sanitization
static HTML_TAG_RE: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"<[^>]*>").expect("Invalid HTML tag regex pattern")
});
static JS_URL_RE: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"(?i)javascript:").expect("Invalid JS URL regex pattern")
});

/// Sanitize input string to prevent XSS - strips HTML tags
fn sanitize_input(input: &str) -> String {
    // Remove HTML tags
    let no_tags = HTML_TAG_RE.replace_all(input, "");
    // Remove javascript: urls
    let cleaned = JS_URL_RE.replace_all(&no_tags, "");
    // Escape remaining special chars
    cleaned
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

#[derive(Clone)]
pub struct AlertEngine {
    db: PgPool,
}

impl AlertEngine {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn create_alert(&self, input: CreateAlert) -> AppResult<Alert> {
        let id = new_id();
        let now = chrono::Utc::now();
        
        // Sanitize inputs
        let title = sanitize_input(&input.title);
        let description = sanitize_input(&input.description);
        let source = sanitize_input(&input.source);

        let alert = sqlx::query_as!(
            Alert,
            r#"
            INSERT INTO alerts (id, title, description, severity, status, source, source_id, agent_id, rule_id, metadata, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING
                id, title, description,
                severity as "severity: _",
                status as "status: _",
                source, source_id, agent_id, assigned_to, rule_id, metadata,
                created_at, updated_at, resolved_at
            "#,
            id,
            title,
            description,
            input.severity as _,
            AlertStatus::Open as _,
            source,
            input.source_id,
            input.agent_id,
            input.rule_id,
            input.metadata,
            now,
            now,
        )
        .fetch_one(&self.db)
        .await?;

        Ok(alert)
    }

    pub async fn get_alert(&self, id: &uuid::Uuid) -> AppResult<Option<Alert>> {
        let alert = sqlx::query_as!(
            Alert,
            r#"
            SELECT
                id, title, description,
                severity as "severity: _",
                status as "status: _",
                source, source_id, agent_id, assigned_to, rule_id, metadata,
                created_at, updated_at, resolved_at
            FROM alerts WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(alert)
    }

    pub async fn list_alerts(
        &self,
        filter: &AlertFilter,
        limit: i64,
        offset: i64,
    ) -> AppResult<(Vec<Alert>, i64)> {
        // Build dynamic query - for production, consider a query builder
        let alerts = sqlx::query_as!(
            Alert,
            r#"
            SELECT
                id, title, description,
                severity as "severity: _",
                status as "status: _",
                source, source_id, agent_id, assigned_to, rule_id, metadata,
                created_at, updated_at, resolved_at
            FROM alerts
            WHERE ($1::text IS NULL OR source = $1)
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            filter.source,
            limit,
            offset,
        )
        .fetch_all(&self.db)
        .await?;

        let count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) as "count!" FROM alerts WHERE ($1::text IS NULL OR source = $1)"#,
            filter.source,
        )
        .fetch_one(&self.db)
        .await?;

        Ok((alerts, count))
    }

    pub async fn update_status(
        &self,
        id: &uuid::Uuid,
        status: AlertStatus,
    ) -> AppResult<Option<Alert>> {
        let now = chrono::Utc::now();
        let resolved_at = if status == AlertStatus::Resolved {
            Some(now)
        } else {
            None
        };

        let alert = sqlx::query_as!(
            Alert,
            r#"
            UPDATE alerts SET status = $2, updated_at = $3, resolved_at = COALESCE($4, resolved_at)
            WHERE id = $1
            RETURNING
                id, title, description,
                severity as "severity: _",
                status as "status: _",
                source, source_id, agent_id, assigned_to, rule_id, metadata,
                created_at, updated_at, resolved_at
            "#,
            id,
            status as _,
            now,
            resolved_at,
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(alert)
    }
}
