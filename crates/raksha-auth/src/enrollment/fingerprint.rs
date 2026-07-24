//! Machine fingerprinting for agent identity
//! 
//! Generates a unique fingerprint based on hardware/OS characteristics.
//! Used to prevent agent cloning and ensure one enrollment = one machine.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Machine fingerprint data collected during enrollment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineFingerprint {
    /// Hostname of the machine
    pub hostname: String,
    /// Operating system (e.g., "linux", "windows", "darwin")
    pub os: String,
    /// OS version/build
    pub os_version: String,
    /// Architecture (e.g., "x86_64", "aarch64")
    pub arch: String,
    /// Machine ID (from /etc/machine-id on Linux, registry on Windows)
    pub machine_id: String,
    /// Number of CPU cores
    pub cpu_cores: u32,
    /// Total memory in bytes
    pub total_memory: u64,
    /// Primary MAC address (hashed)
    pub mac_hash: String,
    /// Boot ID (changes on reboot - used for staleness detection, not identity)
    pub boot_id: Option<String>,
}

impl MachineFingerprint {
    /// Generate a stable fingerprint hash from the machine characteristics.
    /// This hash is used as the agent's identity anchor.
    /// 
    /// Components: hostname + machine_id + os + arch + mac_hash
    /// This ensures uniqueness while being stable across reboots.
    pub fn compute_identity_hash(&self, org_id: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.hostname.as_bytes());
        hasher.update(b"|");
        hasher.update(self.machine_id.as_bytes());
        hasher.update(b"|");
        hasher.update(self.os.as_bytes());
        hasher.update(b"|");
        hasher.update(self.arch.as_bytes());
        hasher.update(b"|");
        hasher.update(self.mac_hash.as_bytes());
        hasher.update(b"|");
        hasher.update(org_id.as_bytes());
        
        let result = hasher.finalize();
        hex_encode(&result)
    }

    /// Validate that the fingerprint has all required fields populated
    pub fn validate(&self) -> Result<(), FingerprintError> {
        if self.hostname.is_empty() {
            return Err(FingerprintError::MissingField("hostname".to_string()));
        }
        if self.machine_id.is_empty() {
            return Err(FingerprintError::MissingField("machine_id".to_string()));
        }
        if self.os.is_empty() {
            return Err(FingerprintError::MissingField("os".to_string()));
        }
        if self.arch.is_empty() {
            return Err(FingerprintError::MissingField("arch".to_string()));
        }
        Ok(())
    }

    /// Check if this fingerprint matches another (same machine)
    /// Allows some tolerance for minor changes (e.g., hostname change)
    pub fn similarity_score(&self, other: &MachineFingerprint) -> f64 {
        let mut score = 0.0;
        let mut weight_total = 0.0;

        // machine_id is the strongest signal (weight: 5)
        weight_total += 5.0;
        if self.machine_id == other.machine_id {
            score += 5.0;
        }

        // MAC hash (weight: 3)
        weight_total += 3.0;
        if self.mac_hash == other.mac_hash {
            score += 3.0;
        }

        // hostname (weight: 2)
        weight_total += 2.0;
        if self.hostname == other.hostname {
            score += 2.0;
        }

        // OS + arch (weight: 1 each)
        weight_total += 2.0;
        if self.os == other.os {
            score += 1.0;
        }
        if self.arch == other.arch {
            score += 1.0;
        }

        score / weight_total
    }
}

/// Fingerprint-related errors
#[derive(Debug, thiserror::Error)]
pub enum FingerprintError {
    #[error("Missing required fingerprint field: {0}")]
    MissingField(String),
    #[error("Fingerprint mismatch: machine identity changed")]
    IdentityMismatch,
    #[error("Possible clone detected: same fingerprint from different IP")]
    PossibleClone,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_fingerprint() -> MachineFingerprint {
        MachineFingerprint {
            hostname: "web-server-01".to_string(),
            os: "linux".to_string(),
            os_version: "6.1.0".to_string(),
            arch: "x86_64".to_string(),
            machine_id: "a1b2c3d4e5f6".to_string(),
            cpu_cores: 8,
            total_memory: 16_000_000_000,
            mac_hash: "abcdef123456".to_string(),
            boot_id: Some("boot-123".to_string()),
        }
    }

    #[test]
    fn test_identity_hash_stable() {
        let fp = sample_fingerprint();
        let hash1 = fp.compute_identity_hash("org_123");
        let hash2 = fp.compute_identity_hash("org_123");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_different_org_different_hash() {
        let fp = sample_fingerprint();
        let hash1 = fp.compute_identity_hash("org_123");
        let hash2 = fp.compute_identity_hash("org_456");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_similarity_same_machine() {
        let fp1 = sample_fingerprint();
        let fp2 = sample_fingerprint();
        assert_eq!(fp1.similarity_score(&fp2), 1.0);
    }

    #[test]
    fn test_validate() {
        let fp = sample_fingerprint();
        assert!(fp.validate().is_ok());

        let bad = MachineFingerprint {
            hostname: "".to_string(),
            ..sample_fingerprint()
        };
        assert!(bad.validate().is_err());
    }
}
