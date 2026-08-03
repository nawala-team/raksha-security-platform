//! Aggregated dashboard statistics.
//!
//! Backs the dashboard landing page: headline counters plus a composite
//! security score. Everything is derived from live tables so the numbers move
//! with the platform rather than being hardcoded in the UI.

use axum::{extract::State, routing::get, Json, Router};
use chrono::{DateTime, Duration, Utc};
use serde::Serialize;

use raksha_auth::Claims;
use raksha_core::error::AppResult;

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/stats", get(get_stats))
        .route("/security-score", get(get_security_score))
}

#[derive(Debug, Serialize)]
pub struct DashboardStats {
    pub active_alerts: i64,
    pub critical_alerts: i64,
    pub alerts_last_24h: i64,
    pub agents_total: i64,
    pub agents_online: i64,
    pub threats_blocked: i64,
    pub threats_blocked_today: i64,
    pub open_incidents: i64,
    pub compliance_score: f64,
    pub generated_at: DateTime<Utc>,
}

async fn get_stats(
    State(state): State<AppState>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<DashboardStats>> {
    let since_24h = Utc::now() - Duration::hours(24);

    // Alerts still needing attention (anything not closed out).
    let active_alerts = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as "count!"
        FROM alerts
        WHERE status NOT IN ('resolved', 'false_positive', 'suppressed')
        "#
    )
    .fetch_one(&state.db)
    .await?;

    let critical_alerts = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as "count!"
        FROM alerts
        WHERE severity = 'critical'
          AND status NOT IN ('resolved', 'false_positive', 'suppressed')
        "#
    )
    .fetch_one(&state.db)
    .await?;

    let alerts_last_24h = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM alerts WHERE created_at >= $1"#,
        since_24h
    )
    .fetch_one(&state.db)
    .await?;

    let agents_total = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM agents WHERE status <> 'decommissioned'"#
    )
    .fetch_one(&state.db)
    .await?;

    let agents_online =
        sqlx::query_scalar!(r#"SELECT COUNT(*) as "count!" FROM agents WHERE status = 'online'"#)
            .fetch_one(&state.db)
            .await?;

    // A threat-intel match is a threat we caught.
    let threats_blocked =
        sqlx::query_scalar!(r#"SELECT COUNT(*) as "count!" FROM threat_indicator_matches"#)
            .fetch_one(&state.db)
            .await?;

    let threats_blocked_today = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM threat_indicator_matches WHERE matched_at >= $1"#,
        since_24h
    )
    .fetch_one(&state.db)
    .await?;

    let open_incidents = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM incidents WHERE status <> 'closed'"#
    )
    .fetch_one(&state.db)
    .await?;

    // Latest assessment per standard, averaged. NULL when nothing is assessed yet.
    let compliance_score = sqlx::query_scalar!(
        r#"
        SELECT AVG(overall_score) as "avg"
        FROM (
            SELECT DISTINCT ON (standard_id) overall_score
            FROM compliance_scores
            ORDER BY standard_id, assessed_at DESC
        ) latest
        "#
    )
    .fetch_one(&state.db)
    .await?
    .unwrap_or(0.0);

    Ok(Json(DashboardStats {
        active_alerts,
        critical_alerts,
        alerts_last_24h,
        agents_total,
        agents_online,
        threats_blocked,
        threats_blocked_today,
        open_incidents,
        compliance_score: (compliance_score * 100.0).round() / 100.0,
        generated_at: Utc::now(),
    }))
}

#[derive(Debug, Serialize)]
pub struct ScoreComponent {
    pub name: String,
    pub score: f64,
    pub weight: f64,
    pub detail: String,
}

#[derive(Debug, Serialize)]
pub struct SecurityScore {
    pub score: f64,
    pub grade: String,
    pub components: Vec<ScoreComponent>,
    pub generated_at: DateTime<Utc>,
}

/// Composite posture score built from four weighted signals.
async fn get_security_score(
    State(state): State<AppState>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<SecurityScore>> {
    // 1. Compliance: average of the latest assessment per standard.
    let compliance = sqlx::query_scalar!(
        r#"
        SELECT AVG(overall_score) as "avg"
        FROM (
            SELECT DISTINCT ON (standard_id) overall_score
            FROM compliance_scores
            ORDER BY standard_id, assessed_at DESC
        ) latest
        "#
    )
    .fetch_one(&state.db)
    .await?
    .unwrap_or(0.0);

    // 2. Agent coverage: how much of the fleet is actually reporting.
    let agents_total = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM agents WHERE status <> 'decommissioned'"#
    )
    .fetch_one(&state.db)
    .await?;
    let agents_online =
        sqlx::query_scalar!(r#"SELECT COUNT(*) as "count!" FROM agents WHERE status = 'online'"#)
            .fetch_one(&state.db)
            .await?;
    // No agents enrolled yet is a neutral starting point, not a failure.
    let coverage = if agents_total == 0 {
        100.0
    } else {
        (agents_online as f64 / agents_total as f64) * 100.0
    };

    // 3. Alert hygiene: unresolved critical/high alerts pull the score down.
    let severe_open = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as "count!"
        FROM alerts
        WHERE severity IN ('critical', 'high')
          AND status NOT IN ('resolved', 'false_positive', 'suppressed')
        "#
    )
    .fetch_one(&state.db)
    .await?;
    let alert_hygiene = (100.0 - (severe_open as f64 * 5.0)).max(0.0);

    // 4. Vulnerability posture: weight critical findings heavier than high.
    let vuln = sqlx::query!(
        r#"
        SELECT
            COALESCE(SUM(critical_count), 0) as "critical!",
            COALESCE(SUM(high_count), 0) as "high!"
        FROM vulnerability_scans
        WHERE status = 'completed'
        "#
    )
    .fetch_one(&state.db)
    .await?;
    let vuln_score = (100.0 - (vuln.critical as f64 * 4.0) - (vuln.high as f64 * 1.5)).max(0.0);

    let components = vec![
        ScoreComponent {
            name: "Compliance".to_string(),
            score: (compliance * 100.0).round() / 100.0,
            weight: 0.30,
            detail: "Average of the latest assessment per standard".to_string(),
        },
        ScoreComponent {
            name: "Agent Coverage".to_string(),
            score: (coverage * 100.0).round() / 100.0,
            weight: 0.25,
            detail: format!("{agents_online} of {agents_total} agents online"),
        },
        ScoreComponent {
            name: "Alert Hygiene".to_string(),
            score: alert_hygiene,
            weight: 0.25,
            detail: format!("{severe_open} unresolved critical/high alerts"),
        },
        ScoreComponent {
            name: "Vulnerability Posture".to_string(),
            score: vuln_score,
            weight: 0.20,
            detail: format!("{} critical, {} high findings", vuln.critical, vuln.high),
        },
    ];

    let score: f64 = components.iter().map(|c| c.score * c.weight).sum();
    let score = (score * 100.0).round() / 100.0;

    let grade = match score {
        s if s >= 90.0 => "A",
        s if s >= 80.0 => "B",
        s if s >= 70.0 => "C",
        s if s >= 60.0 => "D",
        _ => "F",
    }
    .to_string();

    Ok(Json(SecurityScore {
        score,
        grade,
        components,
        generated_at: Utc::now(),
    }))
}

