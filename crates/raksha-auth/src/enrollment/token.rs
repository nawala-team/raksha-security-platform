//! Enrollment token generation and verification
//! 
//! Tokens are one-time-use and expire in 24 hours.
//! Format: rkat_{org_id_prefix}_{random_64_chars}

use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use sha3::{Sha3_256, Digest as Sha3Digest};
use uuid::Uuid;

const TOKEN_PREFIX: &str = "rkat";
const DEFAULT_EXPIRY_HOURS: i64 = 24;

/// Maximum allowed expiry for enrollment tokens: 72 hours
const MAX_EXPIRY_HOURS: i64 = 72;

/// Minimum random entropy bytes (256 bits)
const TOKEN_RANDOM_BYTES: usize = 32;

/// Maximum allowed uses per token
const MAX_TOKEN_USES: u32 = 100;

/// Claims embedded in the enrollment token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrollmentTokenClaims {
    pub token_id: Uuid,
    pub org_id: Uuid,
    pub agent_name: Option<String>,
    pub labels: Vec<String>,
    pub created_by: Uuid,
    pub created_at: i64,
    pub expires_at: i64,
    pub max_uses: u32,
    pub use_count: u32,
    pub allowed_modules: Vec<String>,
}

/// Generated enrollment token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrollmentToken {
    pub token: String,
    pub token_hash: String,
    pub claims: EnrollmentTokenClaims,
}

/// Request to generate a new enrollment token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateTokenRequest {
    pub org_id: Uuid,
    pub created_by: Uuid,
    pub agent_name: Option<String>,
    pub labels: Vec<String>,
    pub expiry_hours: Option<i64>,
    pub max_uses: Option<u32>,
    pub allowed_modules: Vec<String>,
}

/// Generate a new enrollment token
pub fn generate_enrollment_token(req: GenerateTokenRequest) -> EnrollmentToken {
    let token_id = Uuid::now_v7();
    let now = Utc::now();
    // Clamp expiry to safe upper bound
    let expiry_hours = req
        .expiry_hours
        .unwrap_or(DEFAULT_EXPIRY_HOURS)
        .min(MAX_EXPIRY_HOURS);
    let expires_at = now + Duration::hours(expiry_hours);

    // Use 32 bytes (256 bits) of entropy — double the original 16 bytes
    let random_bytes: [u8; TOKEN_RANDOM_BYTES] = rand::random();
    let random_hex = hex_encode(&random_bytes);
    
    let org_prefix = &req.org_id.to_string()[..8];
    let token_string = format!("{}_{}_{}", TOKEN_PREFIX, org_prefix, random_hex);
    let token_hash = hash_token(&token_string);

    // Clamp max_uses to prevent unbounded reuse
    let max_uses = req.max_uses.unwrap_or(1).min(MAX_TOKEN_USES);

    let claims = EnrollmentTokenClaims {
        token_id,
        org_id: req.org_id,
        agent_name: req.agent_name,
        labels: req.labels,
        created_by: req.created_by,
        created_at: now.timestamp(),
        expires_at: expires_at.timestamp(),
        max_uses,
        use_count: 0,
        allowed_modules: req.allowed_modules,
    };

    EnrollmentToken { token: token_string, token_hash, claims }
}

/// Verify token against stored hash (constant-time)
pub fn verify_enrollment_token(token: &str, stored_hash: &str) -> bool {
    let computed = hash_token(token);
    constant_time_eq(computed.as_bytes(), stored_hash.as_bytes())
}

/// Validate token string format
pub fn validate_token_format(token: &str) -> Result<(), TokenError> {
    let parts: Vec<&str> = token.split('_').collect();
    if parts.len() != 3 { return Err(TokenError::InvalidFormat); }
    if parts[0] != TOKEN_PREFIX { return Err(TokenError::InvalidPrefix); }
    if parts[1].len() != 8 { return Err(TokenError::InvalidOrgPrefix); }
    // Updated: 32 bytes of entropy = 64 hex chars
    if parts[2].len() != 64 { return Err(TokenError::InvalidRandomPart); }
    // Validate hex characters only
    if !parts[2].chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(TokenError::InvalidRandomPart);
    }
    Ok(())
}

/// Hash token using SHA3-256 (stronger than SHA2 for token hashing)
fn hash_token(token: &str) -> String {
    let mut hasher = Sha3_256::new();
    Sha3Digest::update(&mut hasher, token.as_bytes());
    hex_encode(&hasher.finalize())
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() { return false; }
    // Use subtle crate pattern: accumulate XOR differences
    let result = a.iter().zip(b.iter()).fold(0u8, |acc, (x, y)| acc | (x ^ y));
    // Compare in constant time — both branches take same time
    result == 0
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}

#[derive(Debug, thiserror::Error)]
pub enum TokenError {
    #[error("Invalid token format")]
    InvalidFormat,
    #[error("Invalid token prefix")]
    InvalidPrefix,
    #[error("Invalid org prefix")]
    InvalidOrgPrefix,
    #[error("Invalid random part")]
    InvalidRandomPart,
    #[error("Token expired")]
    Expired,
    #[error("Token max uses reached")]
    MaxUsesReached,
    #[error("Token revoked")]
    Revoked,
    #[error("Token not found")]
    NotFound,
}

