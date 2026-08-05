//! Control Mapping - framework coverage analysis and gap identification.
//!
//! Maps internal controls to multiple compliance frameworks and provides
//! coverage percentage and gap analysis reporting.

use sqlx::PgPool;
use tracing::info;
use uuid::Uuid;

use crate::models::{
    ControlMapping, ControlStatus, CreateControlMappingRequest, Framework,
    FrameworkCoverage, Id,
};

/// Errors specific to control mapping operations.
#[derive(Debug, thiserror::Error)]
pub enum ControlMappingError {
    #[error("control not found: {0}")]
    ControlNotFound(Id),
    #[error("mapping not found: {0}")]
    MappingNotFound(Id),
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// Gap analysis result for a framework.
#[derive(Debug, Clone, serde::Serialize)]
pub struct GapAnalysis {
    pub framework: Framework,
    pub missing_controls: Vec<String>,
    pub partial_controls: Vec<GapItem>,
    pub total_gaps: u64,
}

/// Individual gap item.
#[derive(Debug, Clone, serde::Serialize)]
pub struct GapItem {
    pub framework_ref: String,
    pub control_id: Option<Id>,
    pub control_title: Option<String>,
    pub status: ControlStatus,
    pub recommendation: String,
}

/// Coverage report for a specific framework.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CoverageReport {
    pub framework: Framework,
    pub coverage: FrameworkCoverage,
    pub controls: Vec<ControlCoverageItem>,
}

/// Individual control coverage detail.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ControlCoverageItem {
    pub framework_ref: String,
    pub control_id: Option<Id>,
    pub control_title: Option<String>,
    pub status: ControlStatus,
}

/// Control mapper service for framework analysis.
pub struct ControlMapper {
    pool: PgPool,
}

impl ControlMapper {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Add a cross-framework mapping for a control.
    pub async fn add_mapping(
        &self,
        req: &CreateControlMappingRequest,
    ) -> Result<ControlMapping, ControlMappingError> {
        let id = Uuid::now_v7();

        sqlx::query(
            r#"
            INSERT INTO grc_control_mappings (id, control_id, framework, framework_ref, rationale)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(id)
        .bind(req.control_id)
        .bind(req.framework.to_string())
        .bind(&req.framework_ref)
        .bind(&req.rationale)
        .execute(&self.pool)
        .await?;

        info!(
            control_id = %req.control_id,
            framework = %req.framework,
            framework_ref = %req.framework_ref,
            "control mapping added"
        );

        Ok(ControlMapping {
            id,
            control_id: req.control_id,
            framework: req.framework,
            framework_ref: req.framework_ref.clone(),
            rationale: req.rationale.clone(),
        })
    }

    /// Get all mappings for a given control.
    pub async fn get_mappings_for_control(
        &self,
        control_id: Id,
    ) -> Result<Vec<ControlMapping>, ControlMappingError> {
        let rows = sqlx::query_as::<_, (Uuid, Uuid, String, String, Option<String>)>(
            r#"
            SELECT id, control_id, framework, framework_ref, rationale
            FROM grc_control_mappings
            WHERE control_id = $1
            ORDER BY framework, framework_ref
            "#,
        )
        .bind(control_id)
        .fetch_all(&self.pool)
        .await?;

        let mappings = rows
            .into_iter()
            .map(|(id, ctrl_id, fw, fw_ref, rationale)| ControlMapping {
                id,
                control_id: ctrl_id,
                framework: parse_framework(&fw),
                framework_ref: fw_ref,
                rationale,
            })
            .collect();

        Ok(mappings)
    }

    /// Calculate coverage percentage for a given framework.
    pub async fn get_coverage(
        &self,
        tenant_id: Id,
        framework: Framework,
    ) -> Result<FrameworkCoverage, ControlMappingError> {
        let fw_str = framework.to_string();

        let row = sqlx::query_as::<_, (i64, i64, i64, i64, i64)>(
            r#"
            SELECT
                COUNT(*)::bigint as total,
                COUNT(*) FILTER (WHERE c.status = 'implemented')::bigint,
                COUNT(*) FILTER (WHERE c.status = 'partial')::bigint,
                COUNT(*) FILTER (WHERE c.status = 'not_implemented')::bigint,
                COUNT(*) FILTER (WHERE c.status = 'not_applicable')::bigint
            FROM grc_controls c
            WHERE c.tenant_id = $1 AND c.framework = $2
            "#,
        )
        .bind(tenant_id)
        .bind(&fw_str)
        .fetch_one(&self.pool)
        .await?;

        let (total, implemented, partial, not_implemented, not_applicable) = row;
        let applicable = total - not_applicable;
        let coverage_percent = if applicable > 0 {
            ((implemented as f64 + partial as f64 * 0.5) / applicable as f64) * 100.0
        } else {
            0.0
        };

        Ok(FrameworkCoverage {
            framework,
            total_requirements: total as u64,
            implemented: implemented as u64,
            partial: partial as u64,
            not_implemented: not_implemented as u64,
            not_applicable: not_applicable as u64,
            coverage_percent,
        })
    }

    /// Perform gap analysis for a framework - identify missing/partial controls.
    pub async fn gap_analysis(
        &self,
        tenant_id: Id,
        framework: Framework,
    ) -> Result<GapAnalysis, ControlMappingError> {
        let fw_str = framework.to_string();

        let rows = sqlx::query_as::<_, (String, Option<Uuid>, Option<String>, String)>(
            r#"
            SELECT
                c.control_ref as framework_ref,
                c.id as control_id,
                c.title as control_title,
                c.status
            FROM grc_controls c
            WHERE c.tenant_id = $1
              AND c.framework = $2
              AND c.status IN ('not_implemented', 'partial')
            ORDER BY c.control_ref
            "#,
        )
        .bind(tenant_id)
        .bind(&fw_str)
        .fetch_all(&self.pool)
        .await?;

        let mut missing_controls = Vec::new();
        let mut partial_controls = Vec::new();

        for (fw_ref, control_id, title, status) in rows {
            let ctrl_status = parse_control_status(&status);
            match ctrl_status {
                ControlStatus::NotImplemented => {
                    missing_controls.push(fw_ref);
                }
                ControlStatus::Partial => {
                    partial_controls.push(GapItem {
                        framework_ref: fw_ref,
                        control_id,
                        control_title: title,
                        status: ctrl_status,
                        recommendation: "Complete implementation to achieve full coverage".into(),
                    });
                }
                _ => {}
            }
        }

        let total_gaps = missing_controls.len() as u64 + partial_controls.len() as u64;

        Ok(GapAnalysis {
            framework,
            missing_controls,
            partial_controls,
            total_gaps,
        })
    }
}

/// Parse a framework string from the database.
fn parse_framework(s: &str) -> Framework {
    match s {
        "CIS" => Framework::Cis,
        "NIST" => Framework::Nist,
        "PCI-DSS" => Framework::PciDss,
        "ISO-27001" => Framework::Iso27001,
        "SOC2" => Framework::Soc2,
        "HIPAA" => Framework::Hipaa,
        _ => Framework::Nist, // fallback
    }
}

/// Parse a control status string from the database.
fn parse_control_status(s: &str) -> ControlStatus {
    match s {
        "implemented" => ControlStatus::Implemented,
        "partial" => ControlStatus::Partial,
        "not_implemented" => ControlStatus::NotImplemented,
        "not_applicable" => ControlStatus::NotApplicable,
        _ => ControlStatus::NotImplemented, // fallback
    }
}

