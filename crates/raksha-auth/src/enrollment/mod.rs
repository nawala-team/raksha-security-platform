//! Agent enrollment module - handles secure agent registration
//! 
//! Flow:
//! 1. Admin generates enrollment token via Portal UI
//! 2. Token contains org_id, is one-time-use, expires in 24h
//! 3. Agent sends enrollment request with token + machine fingerprint
//! 4. Portal verifies token → registers agent → issues mTLS certificate
//! 5. Agent uses certificate for all subsequent communication

pub mod token;
pub mod fingerprint;
pub mod certificate;

pub use token::{EnrollmentToken, EnrollmentTokenClaims, generate_enrollment_token, verify_enrollment_token};
pub use fingerprint::MachineFingerprint;
pub use certificate::AgentCertificate;
