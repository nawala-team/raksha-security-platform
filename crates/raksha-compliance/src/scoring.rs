use serde::Serialize;

use crate::models::{ComplianceReport, ComplianceStatus};

#[derive(Debug, Clone, Serialize)]
pub struct ComplianceScorecard {
    pub overall_score: f64,
    pub grade: ComplianceGrade,
    pub trend: ScoreTrend,
    pub by_category: Vec<CategoryScore>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ComplianceGrade {
    A,
    B,
    C,
    D,
    F,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScoreTrend {
    Improving,
    Stable,
    Declining,
}

#[derive(Debug, Clone, Serialize)]
pub struct CategoryScore {
    pub category: String,
    pub score: f64,
    pub total: u32,
    pub passing: u32,
}

pub struct ComplianceScorer;

impl ComplianceScorer {
    pub fn grade_from_score(score: f64) -> ComplianceGrade {
        match score as u32 {
            90..=100 => ComplianceGrade::A,
            80..=89 => ComplianceGrade::B,
            70..=79 => ComplianceGrade::C,
            60..=69 => ComplianceGrade::D,
            _ => ComplianceGrade::F,
        }
    }

    pub fn generate_scorecard(report: &ComplianceReport) -> ComplianceScorecard {
        let grade = Self::grade_from_score(report.overall_score);

        // Group findings by category
        let mut categories: std::collections::HashMap<String, (u32, u32)> =
            std::collections::HashMap::new();

        for finding in &report.findings {
            let entry = categories.entry(finding.control_id.split('.').next().unwrap_or("unknown").to_string()).or_insert((0, 0));
            entry.0 += 1;
            if finding.status == ComplianceStatus::Compliant {
                entry.1 += 1;
            }
        }

        let by_category = categories
            .into_iter()
            .map(|(category, (total, passing))| CategoryScore {
                category,
                score: if total > 0 {
                    (passing as f64 / total as f64) * 100.0
                } else {
                    0.0
                },
                total,
                passing,
            })
            .collect();

        ComplianceScorecard {
            overall_score: report.overall_score,
            grade,
            trend: ScoreTrend::Stable, // Would compare with historical data
            by_category,
        }
    }
}
