use chrono::Utc;
use raksha_core::error::AppResult;
use raksha_core::models::{new_id, ComplianceStandard};
use sqlx::PgPool;

use crate::models::{ComplianceFinding, ComplianceReport, ComplianceRule, ComplianceStatus};

#[derive(Clone)]
pub struct ComplianceChecker {
    db: PgPool,
}

impl ComplianceChecker {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Run compliance checks for a given standard
    pub async fn run_assessment(
        &self,
        standard: ComplianceStandard,
    ) -> AppResult<ComplianceReport> {
        let rules = self.get_rules_for_standard(&standard).await?;
        let mut findings = Vec::new();

        for rule in &rules {
            let finding = self.evaluate_rule(rule).await;
            findings.push(finding);
        }

        let total = findings.len() as u32;
        let compliant = findings
            .iter()
            .filter(|f| f.status == ComplianceStatus::Compliant)
            .count() as u32;
        let non_compliant = findings
            .iter()
            .filter(|f| f.status == ComplianceStatus::NonCompliant)
            .count() as u32;
        let partially = findings
            .iter()
            .filter(|f| f.status == ComplianceStatus::PartiallyCompliant)
            .count() as u32;
        let na = findings
            .iter()
            .filter(|f| f.status == ComplianceStatus::NotApplicable)
            .count() as u32;

        let score = if total > 0 {
            ((compliant as f64 + partially as f64 * 0.5) / (total - na) as f64) * 100.0
        } else {
            0.0
        };

        Ok(ComplianceReport {
            id: new_id(),
            standard,
            overall_score: score,
            total_controls: total,
            compliant,
            non_compliant,
            partially_compliant: partially,
            not_applicable: na,
            findings,
            generated_at: Utc::now(),
        })
    }

    async fn get_rules_for_standard(
        &self,
        standard: &ComplianceStandard,
    ) -> AppResult<Vec<ComplianceRule>> {
        let rules = sqlx::query_as!(
            ComplianceRule,
            r#"
            SELECT id, standard as "standard: _", control_id, title, description, category, severity, automated, check_query
            FROM compliance_rules
            WHERE standard = $1
            ORDER BY control_id
            "#,
            standard as &ComplianceStandard,
        )
        .fetch_all(&self.db)
        .await?;

        Ok(rules)
    }

    async fn evaluate_rule(&self, rule: &ComplianceRule) -> ComplianceFinding {
        // In production, this would execute check_query or call external checks
        ComplianceFinding {
            rule_id: rule.id,
            control_id: rule.control_id.clone(),
            status: ComplianceStatus::Unknown,
            evidence: None,
            remediation: None,
            checked_at: Utc::now(),
        }
    }
}
