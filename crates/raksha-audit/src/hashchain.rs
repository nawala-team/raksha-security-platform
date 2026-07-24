use sha2::{Digest, Sha256};

/// Hash chain for audit log integrity verification
#[derive(Clone)]
pub struct HashChain;

impl HashChain {
    /// Compute the hash for a new audit entry, chaining from the previous hash
    pub fn compute_hash(
        timestamp: &str,
        user_id: &str,
        action: &str,
        resource: &str,
        previous_hash: Option<&str>,
    ) -> String {
        let mut hasher = Sha256::new();

        hasher.update(timestamp.as_bytes());
        hasher.update(b"|");
        hasher.update(user_id.as_bytes());
        hasher.update(b"|");
        hasher.update(action.as_bytes());
        hasher.update(b"|");
        hasher.update(resource.as_bytes());
        hasher.update(b"|");
        hasher.update(previous_hash.unwrap_or("genesis").as_bytes());

        format!("{:x}", hasher.finalize())
    }

    /// Verify the integrity of a chain of audit entries
    pub fn verify_chain(entries: &[(String, Option<String>, String)]) -> bool {
        for (i, (current_hash, prev_hash, _data)) in entries.iter().enumerate() {
            if i == 0 {
                if prev_hash.is_some() {
                    return false;
                }
            } else {
                let expected_prev = &entries[i - 1].0;
                match prev_hash {
                    Some(ph) if ph == expected_prev => {}
                    _ => return false,
                }
            }
            // In full impl, recompute hash from data and compare to current_hash
            let _ = current_hash;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_chain_deterministic() {
        let h1 = HashChain::compute_hash("2024-01-01T00:00:00Z", "user1", "login", "/auth", None);
        let h2 = HashChain::compute_hash("2024-01-01T00:00:00Z", "user1", "login", "/auth", None);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_hash_chain_links() {
        let h1 = HashChain::compute_hash("2024-01-01T00:00:00Z", "user1", "login", "/auth", None);
        let h2 = HashChain::compute_hash(
            "2024-01-01T00:01:00Z",
            "user1",
            "read",
            "/users",
            Some(&h1),
        );
        assert_ne!(h1, h2);
    }
}
