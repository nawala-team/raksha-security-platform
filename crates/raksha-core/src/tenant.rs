//! Multi-tenancy support for Raksha Security Platform.
//!
//! Provides tenant isolation through:
//! - `Tenant` struct representing a tenant entity in the database
//! - `TenantContext` extracted from JWT claims and propagated to all queries
//! - `TenantFilter` trait for automatic tenant scoping on queries

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ============================================================
// Tenant Status
// ============================================================

/// Tenant lifecycle status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "tenant_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TenantStatus {
    Active,
    Suspended,
    Deleted,
}

// ============================================================
// Tenant Model
// ============================================================

/// A tenant represents an isolated organizational unit within the platform.
/// All data access is scoped to a tenant unless the caller is a superadmin.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Tenant {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub settings: serde_json::Value,
    pub status: TenantStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================
// Tenant Context
// ============================================================

/// Runtime context extracted from JWT claims and injected into request extensions.
/// Handlers use this to scope database queries to the correct tenant.
#[derive(Debug, Clone)]
pub struct TenantContext {
    /// The tenant ID this request is scoped to.
    /// `None` means the caller is a superadmin operating without tenant scope.
    pub tenant_id: Option<Uuid>,
    /// Whether the caller can bypass tenant filtering (superadmin).
    pub is_superadmin: bool,
}

impl TenantContext {
    /// Create a tenant-scoped context for a regular user.
    pub fn scoped(tenant_id: Uuid) -> Self {
        Self {
            tenant_id: Some(tenant_id),
            is_superadmin: false,
        }
    }

    /// Create an unscoped context for superadmin access.
    pub fn superadmin() -> Self {
        Self {
            tenant_id: None,
            is_superadmin: true,
        }
    }

    /// Returns the tenant ID or an error if no tenant scope is set
    /// and the caller is not a superadmin.
    pub fn require_tenant_id(&self) -> Result<Uuid, crate::error::AppError> {
        self.tenant_id.ok_or_else(|| {
            crate::error::AppError::Validation(
                "Tenant context is required for this operation".to_string(),
            )
        })
    }
}

// ============================================================
// Tenant Filter Trait
// ============================================================

/// Trait for building tenant-scoped query fragments.
///
/// Implementations add `WHERE tenant_id = $N` clauses to queries,
/// unless the context is a superadmin bypass.
pub trait TenantFilter {
    /// Returns a SQL WHERE clause fragment for tenant filtering.
    /// The `param_index` is the bind parameter position (e.g., `$1`, `$2`).
    ///
    /// Returns `None` if the caller has superadmin bypass.
    fn tenant_where_clause(&self, param_index: u32) -> Option<String>;

    /// Returns the tenant_id value to bind, or `None` if bypassed.
    fn tenant_bind_value(&self) -> Option<Uuid>;
}

impl TenantFilter for TenantContext {
    fn tenant_where_clause(&self, param_index: u32) -> Option<String> {
        if self.is_superadmin {
            None
        } else {
            self.tenant_id
                .map(|_| format!("tenant_id = ${}", param_index))
        }
    }

    fn tenant_bind_value(&self) -> Option<Uuid> {
        if self.is_superadmin {
            None
        } else {
            self.tenant_id
        }
    }
}

// ============================================================
// Request/Response DTOs
// ============================================================

/// Request payload for creating a new tenant.
#[derive(Debug, Deserialize, validator::Validate)]
pub struct CreateTenantRequest {
    #[validate(length(min = 2, max = 100))]
    pub name: String,
    #[validate(length(min = 2, max = 50))]
    pub slug: String,
    pub settings: Option<serde_json::Value>,
}

/// Request payload for updating a tenant.
#[derive(Debug, Deserialize, validator::Validate)]
pub struct UpdateTenantRequest {
    #[validate(length(min = 2, max = 100))]
    pub name: Option<String>,
    pub settings: Option<serde_json::Value>,
}

/// Response for tenant statistics.
#[derive(Debug, Serialize)]
pub struct TenantStats {
    pub tenant_id: Uuid,
    pub agent_count: i64,
    pub alert_count: i64,
    pub user_count: i64,
}

/// Validate a tenant slug: lowercase alphanumeric + hyphens,
/// no leading/trailing hyphens, minimum 2 characters.
pub fn is_valid_slug(slug: &str) -> bool {
    if slug.len() < 2 || slug.starts_with('-') || slug.ends_with('-') {
        return false;
    }
    slug.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}
