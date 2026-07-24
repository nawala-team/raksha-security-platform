//! Raksha Query Language (RQL) - Threat Hunting Query Engine
//!
//! A domain-specific query language for threat hunting across security events,
//! alerts, agent telemetry, and network flows. Compiles RQL queries into
//! OpenSearch DSL for execution against the SIEM data store.
//!
//! # Example Queries
//!
//! ```rql
//! events where severity = 'critical' and source_ip in ('10.0.0.0/8') time_range last 24h
//! alerts where status = 'open' group_by agent_id count > 5
//! network where dst_port = 443 and bytes_out > 1000000 time_range last 1h order_by bytes_out desc limit 50
//! ```

pub mod executor;
pub mod lexer;
pub mod models;
pub mod parser;
pub mod scheduler;

pub use executor::QueryExecutor;
pub use lexer::Lexer;
pub use models::*;
pub use parser::Parser;
pub use scheduler::QueryScheduler;
