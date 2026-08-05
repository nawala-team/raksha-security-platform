//! Server / infrastructure inventory endpoints.

use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use raksha_auth::Claims;
use raksha_core::error::{AppError, AppResult};
use raksha_core::models::{Pagination, PaginatedResponse, PaginationMeta, UserRole};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_servers).post(create_server))
        .route("/summary", get(server_summary))
        .route("/os-families", get(list_os_families))
        .route("/:id", get(get_server).delete(delete_server))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct ServerResponse {
    id: Uuid,
    agent_id: Option<Uuid>,
    hostname: String,
    display_name: Option<String>,
    environment: String,
    role: Option<String>,
    provider: Option<String>,
    region: Option<String>,
    ip_address: Option<String>,
    os_family: Option<String>,
    os_version: Option<String>,
    cpu_cores: Option<i32>,
    memory_mb: Option<i32>,
    disk_gb: Option<i32>,
    status: String,
    cpu_usage_pct: Option<f64>,
    memory_usage_pct: Option<f64>,
    disk_usage_pct: Option<f64>,
    uptime_secs: Option<i64>,
    last_seen_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

async fn list_servers(
    State(state): State<AppState>,
    Query(pagination): Query<Pagination>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<PaginatedResponse<ServerResponse>>> {
    let servers = sqlx::query_as::<_, ServerResponse>(
        r#"
        SELECT id, agent_id, hostname, display_name, environment, role,
               provider, region, ip_address::text, os_family, os_version,
               cpu_cores, memory_mb, disk_gb, status,
               cpu_usage_pct, memory_usage_pct, disk_usage_pct,
               uptime_secs, last_seen_at, created_at
        FROM servers
        ORDER BY hostname
        LIMIT $1 OFFSET $2
        "#
    )
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(&state.db)
    .await?;

    let total: i64 = sqlx::query_scalar(r#"SELECT COUNT(*) FROM servers"#)
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    Ok(Json(PaginatedResponse {
        data: servers,
        meta: PaginationMeta {
            page: pagination.page,
            per_page: pagination.per_page,
            total,
            total_pages: ((total as f64) / (pagination.limit() as f64)).ceil() as u32,
        },
    }))
}

async fn get_server(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<ServerResponse>> {
    let server = sqlx::query_as::<_, ServerResponse>(
        r#"
        SELECT id, agent_id, hostname, display_name, environment, role,
               provider, region, ip_address::text, os_family, os_version,
               cpu_cores, memory_mb, disk_gb, status,
               cpu_usage_pct, memory_usage_pct, disk_usage_pct,
               uptime_secs, last_seen_at, created_at
        FROM servers WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Server not found".to_string()))?;

    Ok(Json(server))
}

#[derive(Debug, Serialize)]
struct ServerSummary {
    total: i64,
    online: i64,
    offline: i64,
    degraded: i64,
    maintenance: i64,
    avg_cpu_usage: Option<f64>,
    avg_memory_usage: Option<f64>,
}

#[derive(Debug, sqlx::FromRow)]
struct ServerSummaryRow {
    total: i64,
    online: i64,
    offline: i64,
    degraded: i64,
    maintenance: i64,
    avg_cpu: Option<f64>,
    avg_mem: Option<f64>,
}

async fn server_summary(
    State(state): State<AppState>,
    _claims: axum::Extension<Claims>,
) -> AppResult<Json<ServerSummary>> {
    let row: ServerSummaryRow = sqlx::query_as(
        r#"
        SELECT
            COUNT(*)::bigint as total,
            COUNT(*) FILTER (WHERE status = 'online')::bigint as online,
            COUNT(*) FILTER (WHERE status = 'offline')::bigint as offline,
            COUNT(*) FILTER (WHERE status = 'degraded')::bigint as degraded,
            COUNT(*) FILTER (WHERE status = 'maintenance')::bigint as maintenance,
            AVG(cpu_usage_pct) as avg_cpu,
            AVG(memory_usage_pct) as avg_mem
        FROM servers
        "#
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(ServerSummaryRow {
        total: 0, online: 0, offline: 0, degraded: 0, maintenance: 0,
        avg_cpu: None, avg_mem: None,
    });

    Ok(Json(ServerSummary {
        total: row.total,
        online: row.online,
        offline: row.offline,
        degraded: row.degraded,
        maintenance: row.maintenance,
        avg_cpu_usage: row.avg_cpu,
        avg_memory_usage: row.avg_mem,
    }))
}

#[derive(Debug, Deserialize)]
struct CreateServerRequest {
    name: String,
    hostname: String,
    #[serde(default)]
    ip_address: Option<String>,
    #[serde(default)]
    os: Option<String>,
}

async fn create_server(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(payload): Json<CreateServerRequest>,
) -> AppResult<Json<serde_json::Value>> {
    if !claims.role.has_permission(&UserRole::Operator) {
        return Err(AppError::Forbidden("Operator access required".to_string()));
    }

    if payload.hostname.trim().is_empty() || payload.name.trim().is_empty() {
        return Err(AppError::Validation("Name and hostname are required".to_string()));
    }

    let id = Uuid::now_v7();
    let _row = sqlx::query(
        r#"
        INSERT INTO servers (id, name, hostname, ip_address, os, status, created_at)
        VALUES ($1, $2, $3, $4::inet, $5, 'online', NOW())
        RETURNING id
        "#
    )
    .bind(id)
    .bind(&payload.name)
    .bind(&payload.hostname)
    .bind(&payload.ip_address)
    .bind(&payload.os)
    .execute(&state.db)
    .await?;

    tracing::info!(server_id = %id, hostname = %payload.hostname, "Server created");
    Ok(Json(serde_json::json!({
        "id": id,
        "name": payload.name,
        "hostname": payload.hostname,
        "ip_address": payload.ip_address,
        "os": payload.os,
        "status": "online"
    })))
}

async fn delete_server(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> AppResult<Json<serde_json::Value>> {
    if !claims.role.has_permission(&UserRole::Admin) {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    let result = sqlx::query("DELETE FROM servers WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Server not found".to_string()));
    }

    tracing::info!(server_id = %id, deleted_by = %claims.sub, "Server deleted");
    Ok(Json(serde_json::json!({"status": "deleted", "id": id})))
}

// ============================================================
// OS Families endpoint
// ============================================================

#[derive(Debug, Serialize, sqlx::FromRow)]
struct OsFamilyResponse {
    id: String,
    display_name: String,
    category: String,
    vendor: Option<String>,
    icon: Option<String>,
    sort_order: i32,
}

#[derive(Debug, Deserialize)]
struct OsFamilyQuery {
    #[serde(default)]
    category: Option<String>,
}

/// List all supported OS families for dropdown/selection
async fn list_os_families(
    State(state): State<AppState>,
    Query(query): Query<OsFamilyQuery>,
) -> AppResult<Json<serde_json::Value>> {
    // Try to fetch from database first
    let db_families: Vec<OsFamilyResponse> = if let Some(cat) = &query.category {
        sqlx::query_as(
            r#"
            SELECT id, display_name, category, vendor, icon, sort_order
            FROM os_families
            WHERE is_active = true AND category = $1
            ORDER BY sort_order, display_name
            "#
        )
        .bind(cat)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default()
    } else {
        sqlx::query_as(
            r#"
            SELECT id, display_name, category, vendor, icon, sort_order
            FROM os_families
            WHERE is_active = true
            ORDER BY sort_order, display_name
            "#
        )
        .fetch_all(&state.db)
        .await
        .unwrap_or_default()
    };

    // If database has data, return it
    if !db_families.is_empty() {
        return Ok(Json(serde_json::json!({
            "os_families": db_families,
            "categories": ["linux", "windows", "macos", "bsd", "unix", "container_os"]
        })));
    }

    // Fallback to hardcoded list if table doesn't exist yet
    Ok(Json(serde_json::json!({
        "os_families": [
            // Linux distributions
            {"id": "linux", "display_name": "Linux", "category": "linux", "vendor": "Various"},
            {"id": "ubuntu", "display_name": "Ubuntu", "category": "linux", "vendor": "Canonical"},
            {"id": "debian", "display_name": "Debian", "category": "linux", "vendor": "Debian Project"},
            {"id": "rhel", "display_name": "Red Hat Enterprise Linux", "category": "linux", "vendor": "Red Hat"},
            {"id": "centos", "display_name": "CentOS", "category": "linux", "vendor": "CentOS Project"},
            {"id": "rocky", "display_name": "Rocky Linux", "category": "linux", "vendor": "Rocky Enterprise"},
            {"id": "alma", "display_name": "AlmaLinux", "category": "linux", "vendor": "AlmaLinux OS Foundation"},
            {"id": "fedora", "display_name": "Fedora", "category": "linux", "vendor": "Fedora Project"},
            {"id": "suse", "display_name": "SUSE Linux Enterprise", "category": "linux", "vendor": "SUSE"},
            {"id": "amazon", "display_name": "Amazon Linux", "category": "linux", "vendor": "Amazon"},
            {"id": "oracle_linux", "display_name": "Oracle Linux", "category": "linux", "vendor": "Oracle"},
            
            // Container OS
            {"id": "alpine", "display_name": "Alpine Linux", "category": "container_os", "vendor": "Alpine Linux"},
            {"id": "flatcar", "display_name": "Flatcar Container Linux", "category": "container_os", "vendor": "Kinvolk/Microsoft"},
            {"id": "bottlerocket", "display_name": "Bottlerocket", "category": "container_os", "vendor": "Amazon"},
            {"id": "coreos", "display_name": "CoreOS", "category": "container_os", "vendor": "Red Hat"},
            {"id": "photon", "display_name": "VMware Photon OS", "category": "container_os", "vendor": "VMware"},
            
            // Windows
            {"id": "windows", "display_name": "Windows", "category": "windows", "vendor": "Microsoft"},
            {"id": "windows_server", "display_name": "Windows Server", "category": "windows", "vendor": "Microsoft"},
            
            // macOS
            {"id": "macos", "display_name": "macOS", "category": "macos", "vendor": "Apple"},
            
            // BSD
            {"id": "freebsd", "display_name": "FreeBSD", "category": "bsd", "vendor": "FreeBSD Foundation"},
            {"id": "openbsd", "display_name": "OpenBSD", "category": "bsd", "vendor": "OpenBSD Project"},
            {"id": "netbsd", "display_name": "NetBSD", "category": "bsd", "vendor": "NetBSD Foundation"},
            
            // Enterprise Unix
            {"id": "solaris", "display_name": "Oracle Solaris", "category": "unix", "vendor": "Oracle"},
            {"id": "aix", "display_name": "IBM AIX", "category": "unix", "vendor": "IBM"},
            {"id": "hpux", "display_name": "HP-UX", "category": "unix", "vendor": "HPE"}
        ],
        "categories": ["linux", "windows", "macos", "bsd", "unix", "container_os"]
    })))
}
