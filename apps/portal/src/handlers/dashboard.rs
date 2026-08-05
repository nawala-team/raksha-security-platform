//! Aggregated dashboard statistics.

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

    let active_alerts: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM alerts WHERE status NOT IN ('resolved', 'false_positive', 'suppressed')"
    ).fetch_one(&state.db).await.unwrap_or(0);

    let critical_alerts: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM alerts WHERE severity = 'critical' AND status NOT IN ('resolved', 'false_positive', 'suppressed')"
    ).fetch_one(&state.db).await.unwrap_or(0);

    let alerts_last_24h: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM alerts WHERE created_at >= $1")
        .bind(since_24h)
        .fetch_one(&state.db).await.unwrap_or(0);

    let agents_total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM agents WHERE status <> 'decommissioned'"
    ).fetch_one(&state.db).await.unwrap_or(0);

    let agents_online: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM agents WHERE status = 'online'"
    ).fetch_one(&state.db).await.unwrap_or(0);

    let threats_blocked: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM threat_indicator_matches"
    ).fetch_one(&state.db).await.unwrap_or(0);

    let threats_blocked_today: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM threat_indicator_matches WHERE matched_at >= $1"
    ).bind(since_24h).fetch_one(&state.db).await.unwrap_or(0);

    let open_incidents: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM incidents WHERE status <> 'closed'"
    ).fetch_one(&state.db).await.unwrap_or(0);

    let compliance_score: f64 = sqlx::query_scalar(
        r#"SELECT COALESCE(AVG(overall_score), 0.0) FROM (
            SELECT DISTINCT ON (standard_id) overall_score
            FROM compliance_scores ORDER BY standard_id, assessed_at DESC
        ) latest"#
    ).fetch_one(&state.db).await.unwrap_or(0.0);

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

async fn get_security_score(
    State(state): State<AppState>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<SecurityScore>> {
    let compliance: f64 = sqlx::query_scalar(
        r#"SELECT COALESCE(AVG(overall_score), 0.0) FROM (
            SELECT DISTINCT ON (standard_id) overall_score
            FROM compliance_scores ORDER BY standard_id, assessed_at DESC
        ) latest"#
    ).fetch_one(&state.db).await.unwrap_or(0.0);

    let agents_total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM agents WHERE status <> 'decommissioned'"
    ).fetch_one(&state.db).await.unwrap_or(0);
    
    let agents_online: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM agents WHERE status = 'online'"
    ).fetch_one(&state.db).await.unwrap_or(0);
    
    let coverage = if agents_total == 0 { 100.0 } else {
        (agents_online as f64 / agents_total as f64) * 100.0
    };

    let severe_open: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM alerts WHERE severity IN ('critical', 'high') AND status NOT IN ('resolved', 'false_positive', 'suppressed')"
    ).fetch_one(&state.db).await.unwrap_or(0);
    let alert_hygiene: f64 = (100.0_f64 - (severe_open as f64 * 5.0)).max(0.0);

    let vuln_critical: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(critical_count), 0) FROM vulnerability_scans WHERE status = 'completed'"
    ).fetch_one(&state.db).await.unwrap_or(0);
    let vuln_high: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(high_count), 0) FROM vulnerability_scans WHERE status = 'completed'"
    ).fetch_one(&state.db).await.unwrap_or(0);
    let vuln_score: f64 = (100.0_f64 - (vuln_critical as f64 * 4.0) - (vuln_high as f64 * 1.5)).max(0.0);

    let components = vec![
        ScoreComponent { name: "Compliance".into(), score: (compliance * 100.0).round() / 100.0, weight: 0.30, detail: "Average of latest assessment per standard".into() },
        ScoreComponent { name: "Agent Coverage".into(), score: (coverage * 100.0).round() / 100.0, weight: 0.25, detail: format!("{agents_online} of {agents_total} agents online") },
        ScoreComponent { name: "Alert Hygiene".into(), score: alert_hygiene, weight: 0.25, detail: format!("{severe_open} unresolved critical/high alerts") },
        ScoreComponent { name: "Vulnerability Posture".into(), score: vuln_score, weight: 0.20, detail: format!("{vuln_critical} critical, {vuln_high} high findings") },
    ];

    let score: f64 = components.iter().map(|c| c.score * c.weight).sum();
    let score = (score * 100.0).round() / 100.0;
    let grade = match score { s if s >= 90.0 => "A", s if s >= 80.0 => "B", s if s >= 70.0 => "C", s if s >= 60.0 => "D", _ => "F" }.to_string();

    Ok(Json(SecurityScore { score, grade, components, generated_at: Utc::now() }))
}
