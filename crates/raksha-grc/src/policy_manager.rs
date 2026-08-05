//! Policy Manager - CRUD, versioning, acknowledgment tracking, and review cycles.
//!
//! Manages the full policy lifecycle from draft through active to archived,
//! with user acknowledgment tracking and overdue alerts.

use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgPool;
use tracing::info;
use uuid::Uuid;

use crate::models::{Id, Policy, PolicyAcknowledgment, PolicyStatus};

/// Errors specific to policy management operations.
#[derive(Debug, thiserror::Error)]
pub enum PolicyManagerError {
    #[error("policy not found: {0}")]
    NotFound(Id),
    #[error("policy version conflict: {0}")]
    VersionConflict(String),
    #[error("invalid status transition from {from} to {to}")]
    InvalidTransition { from: String, to: String },
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// Overdue acknowledgment record.
#[derive(Debug, Clone, serde::Serialize)]
pub struct OverdueAcknowledgment {
    pub policy_id: Id,
    pub policy_title: String,
    pub policy_version: String,
    pub user_id: Id,
    pub effective_since: NaiveDate,
    pub days_overdue: i64,
}

/// Policy review due item.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PolicyReviewDue {
    pub policy_id: Id,
    pub title: String,
    pub last_updated: DateTime<Utc>,
    pub review_cycle_days: i32,
    pub days_until_due: i64,
}

/// Core policy management service.
pub struct PolicyManager {
    pool: PgPool,
}

impl PolicyManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new policy in draft status.
    pub async fn create_policy(
        &self,
        tenant_id: Id,
        title: &str,
        version: &str,
        content: &str,
        effective_date: Option<NaiveDate>,
        review_cycle_days: i32,
    ) -> Result<Policy, PolicyManagerError> {
        let id = Uuid::now_v7();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO grc_policies (id, tenant_id, title, version, content, status,
                effective_date, review_cycle_days, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, 'draft', $6, $7, $8, $8)
            "#,
        )
        .bind(id)
        .bind(tenant_id)
        .bind(title)
        .bind(version)
        .bind(content)
        .bind(effective_date)
        .bind(review_cycle_days)
        .bind(now)
        .execute(&self.pool)
        .await?;

        info!(policy_id = %id, title = %title, "policy created");

        Ok(Policy {
            id,
            tenant_id,
            title: title.to_string(),
            version: version.to_string(),
            content: content.to_string(),
            status: PolicyStatus::Draft,
            approved_by: None,
            effective_date,
            review_cycle_days,
            created_at: now,
            updated_at: now,
        })
    }

    /// Activate a draft policy with an approver.
    pub async fn activate_policy(
        &self,
        tenant_id: Id,
        policy_id: Id,
        approved_by: Id,
    ) -> Result<(), PolicyManagerError> {
        let result = sqlx::query(
            r#"
            UPDATE grc_policies
            SET status = 'active',
                approved_by = $3,
                effective_date = COALESCE(effective_date, CURRENT_DATE),
                updated_at = NOW()
            WHERE id = $1 AND tenant_id = $2 AND status = 'draft'
            "#,
        )
        .bind(policy_id)
        .bind(tenant_id)
        .bind(approved_by)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(PolicyManagerError::InvalidTransition {
                from: "non-draft or not found".into(),
                to: "active".into(),
            });
        }

        info!(policy_id = %policy_id, approved_by = %approved_by, "policy activated");
        Ok(())
    }

    /// Archive an active policy.
    pub async fn archive_policy(
        &self,
        tenant_id: Id,
        policy_id: Id,
    ) -> Result<(), PolicyManagerError> {
        let result = sqlx::query(
            r#"
            UPDATE grc_policies
            SET status = 'archived', updated_at = NOW()
            WHERE id = $1 AND tenant_id = $2 AND status = 'active'
            "#,
        )
        .bind(policy_id)
        .bind(tenant_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(PolicyManagerError::InvalidTransition {
                from: "non-active or not found".into(),
                to: "archived".into(),
            });
        }

        info!(policy_id = %policy_id, "policy archived");
        Ok(())
    }

    /// Record a user's acknowledgment of a policy.
    pub async fn acknowledge_policy(
        &self,
        tenant_id: Id,
        policy_id: Id,
        user_id: Id,
    ) -> Result<PolicyAcknowledgment, PolicyManagerError> {
        // Get current policy version
        let version: String = sqlx::query_scalar(
            r#"
            SELECT version FROM grc_policies
            WHERE id = $1 AND tenant_id = $2 AND status = 'active'
            "#,
        )
        .bind(policy_id)
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(PolicyManagerError::NotFound(policy_id))?;

        let id = Uuid::now_v7();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO grc_policy_acknowledgments
                (id, tenant_id, policy_id, user_id, acknowledged_at, version_acknowledged)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (policy_id, user_id, version_acknowledged) DO UPDATE
                SET acknowledged_at = EXCLUDED.acknowledged_at
            "#,
        )
        .bind(id)
        .bind(tenant_id)
        .bind(policy_id)
        .bind(user_id)
        .bind(now)
        .bind(&version)
        .execute(&self.pool)
        .await?;

        Ok(PolicyAcknowledgment {
            id,
            tenant_id,
            policy_id,
            user_id,
            acknowledged_at: now,
            version_acknowledged: version,
        })
    }

    /// Find users who have not acknowledged the current version of active policies.
    pub async fn find_overdue_acknowledgments(
        &self,
        tenant_id: Id,
        user_ids: &[Id],
    ) -> Result<Vec<OverdueAcknowledgment>, PolicyManagerError> {
        let rows = sqlx::query_as::<_, (Uuid, String, String, NaiveDate, Uuid)>(
            r#"
            SELECT p.id, p.title, p.version, p.effective_date, u.user_id
            FROM grc_policies p
            CROSS JOIN UNNEST($3::uuid[]) AS u(user_id)
            LEFT JOIN grc_policy_acknowledgments a
                ON a.policy_id = p.id
                AND a.user_id = u.user_id
                AND a.version_acknowledged = p.version
            WHERE p.tenant_id = $1
              AND p.status = 'active'
              AND p.effective_date IS NOT NULL
              AND a.id IS NULL
            ORDER BY p.effective_date ASC
            "#,
        )
        .bind(tenant_id)
        .bind(tenant_id)
        .bind(user_ids)
        .fetch_all(&self.pool)
        .await?;

        let today = Utc::now().date_naive();
        let overdue = rows
            .into_iter()
            .map(|(policy_id, title, version, effective, user_id)| {
                let days_overdue = (today - effective).num_days();
                OverdueAcknowledgment {
                    policy_id,
                    policy_title: title,
                    policy_version: version,
                    user_id,
                    effective_since: effective,
                    days_overdue,
                }
            })
            .collect();

        Ok(overdue)
    }

    /// Get policies that are due for review based on their review cycle.
    pub async fn find_policies_due_for_review(
        &self,
        tenant_id: Id,
    ) -> Result<Vec<PolicyReviewDue>, PolicyManagerError> {
        let rows = sqlx::query_as::<_, (Uuid, String, DateTime<Utc>, i32)>(
            r#"
            SELECT id, title, updated_at, review_cycle_days
            FROM grc_policies
            WHERE tenant_id = $1
              AND status = 'active'
              AND review_cycle_days > 0
            ORDER BY (updated_at + (review_cycle_days || ' days')::interval) ASC
            "#,
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        let now = Utc::now();
        let reviews = rows
            .into_iter()
            .map(|(id, title, last_updated, cycle_days)| {
                let next_review = last_updated
                    + chrono::Duration::days(cycle_days as i64);
                let days_until_due = (next_review - now).num_days();
                PolicyReviewDue {
                    policy_id: id,
                    title,
                    last_updated,
                    review_cycle_days: cycle_days,
                    days_until_due,
                }
            })
            .collect();

        Ok(reviews)
    }

    /// Create a new version of an existing policy (versioning).
    pub async fn create_new_version(
        &self,
        tenant_id: Id,
        policy_id: Id,
        new_version: &str,
        new_content: &str,
    ) -> Result<Policy, PolicyManagerError> {
        // Archive old version
        self.archive_policy(tenant_id, policy_id).await?;

        // Get the original policy metadata
        let (title, review_cycle_days): (String, i32) = sqlx::query_as(
            r#"
            SELECT title, review_cycle_days
            FROM grc_policies
            WHERE id = $1 AND tenant_id = $2
            "#,
        )
        .bind(policy_id)
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(PolicyManagerError::NotFound(policy_id))?;

        // Create new version as draft
        self.create_policy(
            tenant_id,
            &title,
            new_version,
            new_content,
            None,
            review_cycle_days,
        )
        .await
    }
}

