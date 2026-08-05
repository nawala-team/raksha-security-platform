//! Settings endpoints: notification channels, rules and templates.

use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use raksha_auth::Claims;
use raksha_core::error::{AppError, AppResult};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_channels))
        .route("/channels", get(list_channels))
        .route("/channels/:id", get(get_channel))
        .route("/rules", get(list_rules))
        .route("/templates", get(list_templates))
        .route("/summary", get(settings_summary))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct ChannelResponse {
    id: Uuid,
    name: String,
    channel_type: String,
    is_enabled: bool,
    is_default: bool,
    config: serde_json::Value,
    has_secrets: bool,
    last_test_at: Option<DateTime<Utc>>,
    last_test_ok: Option<bool>,
    last_error: Option<String>,
    send_count: i64,
    error_count: i64,
    rate_limit_per_hour: Option<i32>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

async fn list_channels(
    State(state): State<AppState>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<Vec<ChannelResponse>>> {
    let channels = sqlx::query_as::<_, ChannelResponse>(
        r#"
        SELECT id, name, channel_type, is_enabled, is_default, config,
               (secrets_enc IS NOT NULL) as has_secrets,
               last_test_at, last_test_ok, last_error, send_count, error_count,
               rate_limit_per_hour, created_at, updated_at
        FROM notification_channels
        ORDER BY name
        "#
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(channels))
}

async fn get_channel(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<ChannelResponse>> {
    let channel = sqlx::query_as::<_, ChannelResponse>(
        r#"
        SELECT id, name, channel_type, is_enabled, is_default, config,
               (secrets_enc IS NOT NULL) as has_secrets,
               last_test_at, last_test_ok, last_error, send_count, error_count,
               rate_limit_per_hour, created_at, updated_at
        FROM notification_channels WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Channel not found".to_string()))?;

    Ok(Json(channel))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct RuleResponse {
    id: Uuid,
    name: String,
    description: Option<String>,
    is_enabled: bool,
    channel_id: Uuid,
    severity_filter: Option<Vec<String>>,
    category_filter: Option<Vec<String>>,
    template_id: Option<Uuid>,
    cooldown_mins: Option<i32>,
    priority: String,
    created_at: DateTime<Utc>,
}

async fn list_rules(
    State(state): State<AppState>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<Vec<RuleResponse>>> {
    let rules = sqlx::query_as::<_, RuleResponse>(
        r#"
        SELECT id, name, description, is_enabled, channel_id,
               severity_filter, category_filter, template_id,
               cooldown_mins, priority, created_at
        FROM notification_rules
        ORDER BY name
        "#
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(rules))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct TemplateResponse {
    id: Uuid,
    name: String,
    channel_type: String,
    subject_template: Option<String>,
    format: String,
    is_default: bool,
    created_at: DateTime<Utc>,
}

async fn list_templates(
    State(state): State<AppState>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<Vec<TemplateResponse>>> {
    let templates = sqlx::query_as::<_, TemplateResponse>(
        r#"
        SELECT id, name, channel_type, subject_template, format,
               is_default, created_at
        FROM notification_templates
        ORDER BY channel_type, name
        "#
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(templates))
}

#[derive(Debug, Serialize)]
struct SettingsSummary {
    total_channels: i64,
    enabled_channels: i64,
    failing_channels: i64,
    total_rules: i64,
    enabled_rules: i64,
    total_templates: i64,
    notifications_sent: i64,
}

#[derive(Debug, sqlx::FromRow)]
struct ChannelSummaryRow {
    total: i64,
    enabled: i64,
    failing: i64,
    sent: i64,
}

#[derive(Debug, sqlx::FromRow)]
struct RuleSummaryRow {
    total: i64,
    enabled: i64,
}

async fn settings_summary(
    State(state): State<AppState>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<SettingsSummary>> {
    let channels: ChannelSummaryRow = sqlx::query_as(
        r#"
        SELECT
            COUNT(*)::bigint as total,
            COUNT(*) FILTER (WHERE is_enabled)::bigint as enabled,
            COUNT(*) FILTER (WHERE last_test_ok = false)::bigint as failing,
            COALESCE(SUM(send_count), 0)::bigint as sent
        FROM notification_channels
        "#
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(ChannelSummaryRow { total: 0, enabled: 0, failing: 0, sent: 0 });

    let rules: RuleSummaryRow = sqlx::query_as(
        r#"
        SELECT
            COUNT(*)::bigint as total,
            COUNT(*) FILTER (WHERE is_enabled)::bigint as enabled
        FROM notification_rules
        "#
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(RuleSummaryRow { total: 0, enabled: 0 });

    let templates: i64 = sqlx::query_scalar(r#"SELECT COUNT(*) FROM notification_templates"#)
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    Ok(Json(SettingsSummary {
        total_channels: channels.total,
        enabled_channels: channels.enabled,
        failing_channels: channels.failing,
        total_rules: rules.total,
        enabled_rules: rules.enabled,
        total_templates: templates,
        notifications_sent: channels.sent,
    }))
}
