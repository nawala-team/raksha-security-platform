//! Agent certificate management for mTLS
//! 
//! After successful enrollment, the portal issues a client certificate
//! to the agent. This certificate is used for all subsequent communication.
//! Certificates are rotated every 30 days automatically.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Default certificate validity: 30 days
const CERT_VALIDITY_DAYS: i64 = 30;

/// Certificate rotation trigger: 7 days before expiry
const ROTATION_THRESHOLD_DAYS: i64 = 7;

/// Agent certificate issued after enrollment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCertificate {
    /// Certificate serial number
    pub serial: String,
    /// Agent ID this cert belongs to
    pub agent_id: Uuid,
    /// Organization ID
    pub org_id: Uuid,
    /// Certificate fingerprint (SHA-256 of DER-encoded cert)
    pub fingerprint: String,
    /// Common Name (CN) - agent_id.org_id.raksha.internal
    pub common_name: String,
    /// Subject Alternative Names
    pub san: Vec<String>,
    /// Not valid before
    pub not_before: DateTime<Utc>,
    /// Not valid after  
    pub not_after: DateTime<Utc>,
    /// Certificate status
    pub status: CertificateStatus,
    /// Issued at timestamp
    pub issued_at: DateTime<Utc>,
    /// Who/what issued this cert
    pub issuer: String,
}

/// Certificate status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CertificateStatus {
    /// Certificate is active and valid
    Active,
    /// Certificate is pending rotation (new one issued, old still valid)
    PendingRotation,
    /// Certificate has been revoked
    Revoked,
    /// Certificate has expired
    Expired,
}

/// Request to issue a new agent certificate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueCertificateRequest {
    pub agent_id: Uuid,
    pub org_id: Uuid,
    pub hostname: String,
    /// Optional custom validity in days
    pub validity_days: Option<i64>,
}

/// Response after issuing a certificate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueCertificateResponse {
    /// The certificate in PEM format
    pub certificate_pem: String,
    /// The private key in PEM format (only sent once during enrollment)
    pub private_key_pem: String,
    /// CA certificate for verification
    pub ca_certificate_pem: String,
    /// Certificate metadata
    pub metadata: AgentCertificate,
}

/// Certificate rotation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotateCertificateRequest {
    pub agent_id: Uuid,
    pub current_serial: String,
    /// CSR (Certificate Signing Request) from agent
    pub csr_pem: String,
}

impl AgentCertificate {
    /// Create a new certificate metadata entry
    pub fn new(req: &IssueCertificateRequest) -> Self {
        let now = Utc::now();
        let validity = req.validity_days.unwrap_or(CERT_VALIDITY_DAYS);
        let not_after = now + Duration::days(validity);
        let serial = Uuid::now_v7().to_string();
        let common_name = format!("{}.{}.raksha.internal", req.agent_id, req.org_id);

        Self {
            serial,
            agent_id: req.agent_id,
            org_id: req.org_id,
            fingerprint: String::new(), // Set after cert generation
            common_name: common_name.clone(),
            san: vec![
                common_name,
                req.hostname.clone(),
            ],
            not_before: now,
            not_after,
            status: CertificateStatus::Active,
            issued_at: now,
            issuer: "raksha-ca".to_string(),
        }
    }

    /// Check if certificate needs rotation
    pub fn needs_rotation(&self) -> bool {
        let threshold = Utc::now() + Duration::days(ROTATION_THRESHOLD_DAYS);
        self.not_after <= threshold && self.status == CertificateStatus::Active
    }

    /// Check if certificate is currently valid
    pub fn is_valid(&self) -> bool {
        let now = Utc::now();
        self.status == CertificateStatus::Active
            && now >= self.not_before
            && now <= self.not_after
    }

    /// Revoke this certificate
    pub fn revoke(&mut self) {
        self.status = CertificateStatus::Revoked;
    }

    /// Mark as expired
    pub fn mark_expired(&mut self) {
        self.status = CertificateStatus::Expired;
    }
}

/// Certificate-related errors
#[derive(Debug, thiserror::Error)]
pub enum CertificateError {
    #[error("Certificate generation failed: {0}")]
    GenerationFailed(String),
    #[error("Certificate not found: {0}")]
    NotFound(String),
    #[error("Certificate has been revoked")]
    Revoked,
    #[error("Certificate has expired")]
    Expired,
    #[error("Invalid CSR: {0}")]
    InvalidCsr(String),
    #[error("CA not initialized")]
    CaNotInitialized,
    #[error("Certificate rotation failed: {0}")]
    RotationFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_certificate() {
        let req = IssueCertificateRequest {
            agent_id: Uuid::now_v7(),
            org_id: Uuid::now_v7(),
            hostname: "web-01.example.com".to_string(),
            validity_days: None,
        };

        let cert = AgentCertificate::new(&req);
        assert!(cert.is_valid());
        assert!(!cert.needs_rotation());
        assert_eq!(cert.status, CertificateStatus::Active);
        assert!(cert.common_name.contains("raksha.internal"));
    }

    #[test]
    fn test_revoke() {
        let req = IssueCertificateRequest {
            agent_id: Uuid::now_v7(),
            org_id: Uuid::now_v7(),
            hostname: "test".to_string(),
            validity_days: None,
        };

        let mut cert = AgentCertificate::new(&req);
        assert!(cert.is_valid());
        cert.revoke();
        assert!(!cert.is_valid());
        assert_eq!(cert.status, CertificateStatus::Revoked);
    }
}
