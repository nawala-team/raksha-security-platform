//! Raksha Threat Intelligence Module
//!
//! Fetches, normalizes, and correlates Indicators of Compromise (IOCs)
//! from multiple open-source feeds. Auto-syncs on schedule.

pub mod feeds;
pub mod ioc;
pub mod matcher;

pub use feeds::FeedManager;
pub use ioc::{IOC, IOCType, ThreatSeverity};
pub use matcher::IOCMatcher;
