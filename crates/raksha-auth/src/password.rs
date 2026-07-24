use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Algorithm, Argon2, Params, Version,
};
use raksha_core::error::{AppError, AppResult};

// ─── Argon2id parameters (OWASP 2024 recommended) ───────────────────────────
// Reference: https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html

/// Memory cost: 64 MiB (65536 KiB)
const ARGON2_MEMORY_COST_KIB: u32 = 65_536;

/// Time cost (iterations): 3
const ARGON2_TIME_COST: u32 = 3;

/// Parallelism: 4 lanes
const ARGON2_PARALLELISM: u32 = 4;

/// Output hash length in bytes (256-bit)
const ARGON2_OUTPUT_LEN: usize = 32;

// ─── Password policy constants ───────────────────────────────────────────────

/// Minimum password length
const MIN_PASSWORD_LENGTH: usize = 12;

/// Maximum password length (prevents DoS via extremely long inputs to Argon2)
const MAX_PASSWORD_LENGTH: usize = 128;

// ─── Rate limiting constants (for use by callers) ────────────────────────────

/// Maximum login attempts before lockout
pub const MAX_LOGIN_ATTEMPTS: u32 = 5;

/// Lockout duration in seconds after max attempts exceeded
pub const LOCKOUT_DURATION_SECS: u64 = 900; // 15 minutes

/// Rate limit window for login attempts in seconds
pub const RATE_LIMIT_WINDOW_SECS: u64 = 300; // 5 minutes

#[derive(Clone)]
pub struct PasswordService;

impl PasswordService {
    /// Build the Argon2id hasher with hardened parameters.
    fn argon2_instance() -> Argon2<'static> {
        let params = Params::new(
            ARGON2_MEMORY_COST_KIB,
            ARGON2_TIME_COST,
            ARGON2_PARALLELISM,
            Some(ARGON2_OUTPUT_LEN),
        )
        .expect("Argon2 params are valid");

        Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
    }

    /// Validate password against policy before hashing.
    pub fn validate_password_policy(password: &str) -> AppResult<()> {
        if password.len() < MIN_PASSWORD_LENGTH {
            return Err(AppError::Validation(format!(
                "Password must be at least {} characters",
                MIN_PASSWORD_LENGTH
            )));
        }
        if password.len() > MAX_PASSWORD_LENGTH {
            return Err(AppError::Validation(format!(
                "Password must not exceed {} characters",
                MAX_PASSWORD_LENGTH
            )));
        }

        let has_upper = password.chars().any(|c| c.is_uppercase());
        let has_lower = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        let has_special = password.chars().any(|c| !c.is_alphanumeric());

        if !has_upper || !has_lower || !has_digit || !has_special {
            return Err(AppError::Validation(
                "Password must contain uppercase, lowercase, digit, and special character"
                    .to_string(),
            ));
        }

        Ok(())
    }

    /// Hash password using Argon2id with hardened parameters.
    /// Enforces password policy before hashing.
    pub fn hash_password(password: &str) -> AppResult<String> {
        Self::validate_password_policy(password)?;

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Self::argon2_instance();

        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AppError::Internal(format!("Password hashing failed: {e}")))?;

        Ok(hash.to_string())
    }

    /// Verify password against stored hash.
    /// Returns Ok(true) on match, Ok(false) on mismatch.
    /// The Argon2 verify operation is constant-time internally.
    pub fn verify_password(password: &str, hash: &str) -> AppResult<bool> {
        // Reject excessively long passwords before doing any work (DoS prevention)
        if password.len() > MAX_PASSWORD_LENGTH {
            return Ok(false);
        }

        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| AppError::Internal(format!("Invalid password hash: {e}")))?;

        Ok(Self::argon2_instance()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}
