//! Raksha GRC (Governance, Risk & Compliance) Module
//!
//! Provides risk register management, policy lifecycle with versioning,
//! control framework mapping, and compliance coverage analysis.

pub mod control_mapping;
pub mod handlers;
pub mod models;
pub mod policy_manager;
pub mod risk_engine;

pub use control_mapping::{ControlMapper, CoverageReport, GapAnalysis};
pub use models::{
    Control, ControlMapping, ControlStatus, Framework, Policy, PolicyAcknowledgment,
    PolicyStatus, RiskCategory, RiskItem, RiskStatus,
};
pub use policy_manager::PolicyManager;
pub use risk_engine::RiskEngine;
