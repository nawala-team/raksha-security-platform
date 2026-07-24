//! Risk Engine - scoring, heatmap generation, trending, and escalation.
//!
//! Provides risk quantification and visualization data for the GRC dashboard.

use chrono::{NaiveDate, Utc};
use sqlx::PgPool;
use tracing::{info, warn};
use uuid::Uuid;

use crate::models::{HeatmapCell, Id, RiskHeatmap, RiskItem};

/// Errors specific to risk engine operations.
#[derive(Debug, thiserror::Error)]
pub enum RiskEngineError {
    #[error("invalid likelihood value {0}: must be 1-5")]
    InvalidLikelihood(u8),
    #[error("invalid impact value {0}: must be 1-5")]
    InvalidImpact(u8),
    #[error("risk not found: {0}")]
    NotFound(Id),
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// Risk trending data point.
#[derive(Debug, Clone, serde::Serialize)]
pub struct RiskTrendPoint {
    pub date: NaiveDate,
    pub total_risks: u64,
    pub critical_count: u64,
    pub high_count: u64,
    pub medium_count: u64,
    pub low_count: u64,
    pub average_score: f64,
}

/// Overdue risk review item for escalation.
#[derive(Debug, Clone, serde::Serialize)]
pub struct OverdueRisk {
    pub risk_id: Id,
    pub title: String,
    pub owner: Id,
    pub review_date: NaiveDate,
    pub days_overdue: i64,
}

/// Core risk engine for score calculation, heatmap, and trending.
pub struct RiskEngine {
    pool: PgPool,
}

impl RiskEngine {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Calculate risk score with validation.
    pub fn calculate_score(likelihood: u8, impact: u8) -> Result<u8, RiskEngineError> {
        if !(1..=5).contains(&likelihood) {
            return Err(RiskEngineError::InvalidLikelihood(likelihood));
        }
        if !(1..=5).contains(&impact) {
            return Err(RiskEngineError::InvalidImpact(impact));
        }
        Ok(likelihood * impact)
    }

    /// Generate a 5x5 risk heatmap for a tenant.
    pub async fn generate_heatmap(&self, tenant_id: Id) -> Result<RiskHeatmap, RiskEngineError> {
        let rows = sqlx::query_as::<_, (i16, i16, i64, Vec<Uuid>)>(
            r#"
            SELECT
                likelihood::smallint,
                impact::smallint,
                COUNT(*)::bigint as count,
                ARRAY_AGG(id) as risk_ids
            FROM grc_risks
            WHERE tenant_id = $1
              AND status NOT IN ('closed')
            GROUP BY likelihood, impact
            ORDER BY likelihood, impact
            "#,
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        let mut cells = Vec::with_capacity(25);
        let mut total_risks = 0u64;

        // Initialize all 25 cells
        for l in 1..=5u8 {
            for i in 1..=5u8 {
                cells.push(HeatmapCell {
                    likelihood: l,
                    impact: i,
                    count: 0,
                    risk_ids: Vec::new(),
                });
            }
        }

        // Fill in actual data
        for (likelihood, impact, count, risk_ids) in rows {
            let idx = ((likelihood as u8 - 1) * 5 + (impact as u8 - 1)) as usize;
            if idx < 25 {
                cells[idx].count = count as u64;
                cells[idx].risk_ids = risk_ids;
                total_risks += count as u64;
            }
        }

        Ok(RiskHeatmap { cells, total_risks })
    }

    /// Get risk trending data over a specified number of days.
    pub async fn get_trend(
        &self,
        tenant_id: Id,
        days: i32,
    ) -> Result<Vec<RiskTrendPoint>, RiskEngineError> {
        let rows = sqlx::query_as::<_, (NaiveDate, i64, i64, i64, i64, i64, f64)>(
            r#"
            SELECT
                d::date as trend_date,
                COALESCE(SUM(1) FILTER (WHERE r.id IS NOT NULL), 0)::bigint,
                COALESCE(SUM(1) FILTER (WHERE r.risk_score >= 16), 0)::bigint,
                COALESCE(SUM(1) FILTER (WHERE r.risk_score BETWEEN 10 AND 15), 0)::bigint,
                COALESCE(SUM(1) FILTER (WHERE r.risk_score BETWEEN 5 AND 9), 0)::bigint,
                COALESCE(SUM(1) FILTER (WHERE r.risk_score BETWEEN 1 AND 4), 0)::bigint,
                COALESCE(AVG(r.risk_score::float), 0.0)
            FROM generate_series(
                CURRENT_DATE - $2::int * INTERVAL '1 day',
                CURRENT_DATE,
                '1 day'::interval
            ) d
            LEFT JOIN grc_risks r
                ON r.tenant_id = $1
                AND r.created_at::date <= d::date
                AND (r.status != 'closed' OR r.updated_at::date > d::date)
            GROUP BY d::date
            ORDER BY d::date
            "#,
        )
        .bind(tenant_id)
        .bind(days)
        .fetch_all(&self.pool)
        .await?;

        let trend = rows
            .into_iter()
            .map(|(date, total, critical, high, medium, low, avg)| {
                RiskTrendPoint {
                    date,
                    total_risks: total as u64,
                    critical_count: critical as u64,
                    high_count: high as u64,
                    medium_count: medium as u64,
                    low_count: low as u64,
                    average_score: avg,
                }
            })
            .collect();

        Ok(trend)
    }

    /// Find all risks with overdue reviews for escalation.
    pub async fn find_overdue_reviews(
        &self,
        tenant_id: Id,
    ) -> Result<Vec<OverdueRisk>, RiskEngineError> {
        let today = Utc::now().date_naive();

        let rows = sqlx::query_as::<_, (Uuid, String, Uuid, NaiveDate)>(
            r#"
            SELECT id, title, owner, review_date
            FROM grc_risks
            WHERE tenant_id = $1
              AND status NOT IN ('closed', 'accepted')
              AND review_date < $2
            ORDER BY review_date ASC
            "#,
        )
        .bind(tenant_id)
        .bind(today)
        .fetch_all(&self.pool)
        .await?;

        let overdue = rows
            .into_iter()
            .map(|(risk_id, title, owner, review_date)| {
                let days_overdue = (today - review_date).num_days();
                OverdueRisk {
                    risk_id,
                    title,
                    owner,
                    review_date,
                    days_overdue,
                }
            })
            .collect();

        Ok(overdue)
    }

    /// Auto-escalate risks that are overdue by more than the given threshold.
    pub async fn auto_escalate(
        &self,
        tenant_id: Id,
        threshold_days: i64,
    ) -> Result<Vec<OverdueRisk>, RiskEngineError> {
        let overdue = self.find_overdue_reviews(tenant_id).await?;

        let escalated: Vec<OverdueRisk> = overdue
            .into_iter()
            .filter(|r| r.days_overdue > threshold_days)
            .collect();

        if !escalated.is_empty() {
            warn!(
                tenant_id = %tenant_id,
                count = escalated.len(),
                "auto-escalating overdue risk reviews"
            );
        }

        Ok(escalated)
    }

    /// Process risk acceptance workflow.
    pub async fn accept_risk(
        &self,
        tenant_id: Id,
        risk_id: Id,
        accepted_by: Id,
        justification: &str,
    ) -> Result<(), RiskEngineError> {
        let result = sqlx::query(
            r#"
            UPDATE grc_risks
            SET status = 'accepted',
                mitigation_plan = COALESCE(mitigation_plan, '') ||
                    E'\n\n--- Risk Accepted ---\nBy: ' || $3::text ||
                    E'\nJustification: ' || $4::text ||
                    E'\nDate: ' || NOW()::text,
                updated_at = NOW()
            WHERE id = $1
              AND tenant_id = $2
              AND status IN ('identified', 'assessed')
            "#,
        )
        .bind(risk_id)
        .bind(tenant_id)
        .bind(accepted_by.to_string())
        .bind(justification)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(RiskEngineError::NotFound(risk_id));
        }

        info!(risk_id = %risk_id, accepted_by = %accepted_by, "risk accepted");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{RiskCategory, RiskStatus};

    #[test]
    fn test_score_calculation() {
        assert_eq!(RiskEngine::calculate_score(1, 1).unwrap(), 1);
        assert_eq!(RiskEngine::calculate_score(5, 5).unwrap(), 25);
        assert_eq!(RiskEngine::calculate_score(3, 4).unwrap(), 12);
    }

    #[test]
    fn test_score_validation() {
        assert!(RiskEngine::calculate_score(0, 3).is_err());
        assert!(RiskEngine::calculate_score(6, 3).is_err());
        assert!(RiskEngine::calculate_score(3, 0).is_err());
        assert!(RiskEngine::calculate_score(3, 6).is_err());
    }

    #[test]
    fn test_risk_level() {
        let make_risk = |score: u8| RiskItem {
            id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            title: "test".into(),
            description: "test".into(),
            category: RiskCategory::Technical,
            likelihood: 1,
            impact: 1,
            risk_score: score,
            owner: Uuid::new_v4(),
            status: RiskStatus::Identified,
            mitigation_plan: None,
            review_date: Utc::now().date_naive(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert_eq!(make_risk(2).risk_level(), "low");
        assert_eq!(make_risk(6).risk_level(), "medium");
        assert_eq!(make_risk(12).risk_level(), "high");
        assert_eq!(make_risk(20).risk_level(), "critical");
    }
}

