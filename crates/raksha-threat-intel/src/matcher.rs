use std::collections::HashSet;
use std::net::IpAddr;
use std::str::FromStr;

use crate::ioc::{IOC, IOCType};

/// Fast IOC matching engine for real-time correlation
pub struct IOCMatcher {
    ip_set: HashSet<String>,
    domain_set: HashSet<String>,
    hash_set: HashSet<String>,
    url_set: HashSet<String>,
}

/// Match result when an IOC is found
#[derive(Debug, Clone)]
pub struct MatchResult {
    pub matched_ioc: IOC,
    pub context: String,
}

impl IOCMatcher {
    /// Build matcher from a list of IOCs
    pub fn from_iocs(iocs: &[IOC]) -> Self {
        let mut ip_set = HashSet::new();
        let mut domain_set = HashSet::new();
        let mut hash_set = HashSet::new();
        let mut url_set = HashSet::new();

        for ioc in iocs {
            match ioc.ioc_type {
                IOCType::IPv4 | IOCType::IPv6 => { ip_set.insert(ioc.value.clone()); }
                IOCType::Domain => { domain_set.insert(ioc.value.to_lowercase()); }
                IOCType::Sha256 | IOCType::Md5 | IOCType::Sha1 => { hash_set.insert(ioc.value.to_lowercase()); }
                IOCType::Url => { url_set.insert(ioc.value.to_lowercase()); }
                _ => {}
            }
        }

        Self { ip_set, domain_set, hash_set, url_set }
    }

    /// Check if an IP address matches any known bad IOC
    pub fn match_ip(&self, ip: &str) -> bool {
        self.ip_set.contains(ip)
    }

    /// Check if a domain matches any known bad IOC
    pub fn match_domain(&self, domain: &str) -> bool {
        let lower = domain.to_lowercase();
        self.domain_set.contains(&lower)
    }

    /// Check if a hash matches any known bad IOC
    pub fn match_hash(&self, hash: &str) -> bool {
        let lower = hash.to_lowercase();
        self.hash_set.contains(&lower)
    }

    /// Check if a URL matches any known bad IOC
    pub fn match_url(&self, url: &str) -> bool {
        let lower = url.to_lowercase();
        self.url_set.contains(&lower)
    }

    /// Match against any value (auto-detects type)
    pub fn match_any(&self, value: &str) -> bool {
        // Try IP first
        if IpAddr::from_str(value).is_ok() {
            return self.match_ip(value);
        }
        // Try hash (64 hex chars = sha256, 32 = md5)
        let lower = value.to_lowercase();
        if lower.len() == 64 && lower.chars().all(|c| c.is_ascii_hexdigit()) {
            return self.match_hash(&lower);
        }
        if lower.len() == 32 && lower.chars().all(|c| c.is_ascii_hexdigit()) {
            return self.match_hash(&lower);
        }
        // Try URL
        if lower.starts_with("http://") || lower.starts_with("https://") {
            return self.match_url(&lower);
        }
        // Default to domain
        self.match_domain(&lower)
    }

    /// Get total number of loaded indicators
    pub fn total_indicators(&self) -> usize {
        self.ip_set.len() + self.domain_set.len() + self.hash_set.len() + self.url_set.len()
    }
}
