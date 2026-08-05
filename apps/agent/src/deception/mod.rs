//! Deception Technology Module - Honeypot Services
//!
//! Provides configurable honeypot services that detect unauthorized access attempts
//! by presenting fake services (SSH, HTTP, SMTP, MySQL) that log all interactions
//! and trigger immediate alerts. Any connection to a honeypot is inherently suspicious
//! since legitimate users have no reason to interact with these decoy services.

#![allow(dead_code)]

pub mod manager;
pub mod ssh_honeypot;
pub mod http_honeypot;
pub mod smtp_honeypot;
pub mod mysql_honeypot;

pub use manager::{HoneypotManager, DeceptionConfig};
