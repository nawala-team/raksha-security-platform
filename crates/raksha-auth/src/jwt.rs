use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use raksha_core::config::JwtConfig;
use raksha_core::error::{AppError, AppResult};
use raksha_core::models::UserRole;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Minimum HMAC key length (256 bits for HS256)
const MIN_SECRET_KEY_BYTES: usize = 32;

/// Maximum allowed access token TTL: 15 minutes
const MAX_ACCESS_TOKEN_TTL_SECS: i64 = 900;

/// Maximum allowed refresh token TTL: 7 days
const MAX_REFRESH_TOKEN_TTL_SECS: i64 = 604_800;

/// Clock skew tolerance for token validation (seconds)
const LEEWAY_SECS: u64 = 30;

/// Only algorithm we accept — prevents algorithm confusion attacks
const ALLOWED_ALGORITHM: Algorithm = Algorithm::HS256;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: Uuid,
    /// User role
    pub role: UserRole,
    /// Issuer
    pub iss: String,
    /// Audience
    pub aud: String,
    /// Issued at
    pub iat: i64,
    /// Not before
    pub nbf: i64,
    /// Expiration
    pub exp: i64,
    /// JWT ID (for revocation tracking)
    pub jti: Uuid,
    /// Session ID
    pub sid: Uuid,
    /// Token type: "access" or "refresh"
    pub token_type: String,
    /// Tenant ID (for multi-tenancy)
    #[serde(default)]
    pub tenant_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub token_type: String,
}

#[derive(Clone)]
pub struct TokenService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    config: JwtConfig,
}

impl TokenService {
    /// Create a new TokenService. Panics if the secret key is too short or TTLs exceed bounds.
    pub fn new(config: &JwtConfig) -> Self {
        // Enforce minimum key length to prevent weak HMAC signatures
        assert!(
            config.secret.as_bytes().len() >= MIN_SECRET_KEY_BYTES,
            "JWT secret must be at least {} bytes (got {})",
            MIN_SECRET_KEY_BYTES,
            config.secret.as_bytes().len()
        );

        // Enforce sane TTL upper bounds
        assert!(
            config.access_token_ttl_secs <= MAX_ACCESS_TOKEN_TTL_SECS,
            "Access token TTL must not exceed {} seconds",
            MAX_ACCESS_TOKEN_TTL_SECS
        );
        assert!(
            config.refresh_token_ttl_secs <= MAX_REFRESH_TOKEN_TTL_SECS,
            "Refresh token TTL must not exceed {} seconds",
            MAX_REFRESH_TOKEN_TTL_SECS
        );

        Self {
            encoding_key: EncodingKey::from_secret(config.secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(config.secret.as_bytes()),
            config: config.clone(),
        }
    }

    pub fn generate_token_pair(
        &self,
        user_id: Uuid,
        role: UserRole,
        session_id: Uuid,
    ) -> AppResult<TokenPair> {
        let now = Utc::now();

        let access_claims = Claims {
            sub: user_id,
            role: role.clone(),
            iss: self.config.issuer.clone(),
            aud: self.config.audience.clone(),
            iat: now.timestamp(),
            nbf: now.timestamp(),
            exp: (now + Duration::seconds(self.config.access_token_ttl_secs)).timestamp(),
            jti: Uuid::now_v7(),
            sid: session_id,
            token_type: "access".to_string(),
            tenant_id: None,
        };

        let refresh_claims = Claims {
            sub: user_id,
            role,
            iss: self.config.issuer.clone(),
            aud: self.config.audience.clone(),
            iat: now.timestamp(),
            nbf: now.timestamp(),
            exp: (now + Duration::seconds(self.config.refresh_token_ttl_secs)).timestamp(),
            jti: Uuid::now_v7(),
            sid: session_id,
            token_type: "refresh".to_string(),
            tenant_id: None,
        };

        // Explicitly specify algorithm — prevents algorithm confusion attacks
        let header = Header::new(ALLOWED_ALGORITHM);

        let access_token = encode(&header, &access_claims, &self.encoding_key)
            .map_err(|e| AppError::Jwt(e.to_string()))?;

        let refresh_token = encode(&header, &refresh_claims, &self.encoding_key)
            .map_err(|e| AppError::Jwt(e.to_string()))?;

        Ok(TokenPair {
            access_token,
            refresh_token,
            expires_in: self.config.access_token_ttl_secs,
            token_type: "Bearer".to_string(),
        })
    }

    pub fn verify_token(&self, token: &str) -> AppResult<Claims> {
        let mut validation = Validation::new(ALLOWED_ALGORITHM);
        validation.set_issuer(&[&self.config.issuer]);
        validation.set_audience(&[&self.config.audience]);
        validation.leeway = LEEWAY_SECS;
        // Require these claims to be present and valid
        validation.set_required_spec_claims(&["exp", "iss", "aud", "nbf", "sub", "iat"]);
        // Only allow our specific algorithm — blocks "none" and RS/ES confusion
        validation.algorithms = vec![ALLOWED_ALGORITHM];

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(|e| AppError::Jwt(e.to_string()))?;

        Ok(token_data.claims)
    }

    /// Verify specifically an access token (rejects refresh tokens)
    pub fn verify_access_token(&self, token: &str) -> AppResult<Claims> {
        let claims = self.verify_token(token)?;
        if claims.token_type != "access" {
            return Err(AppError::Jwt("Invalid token type: expected access token".to_string()));
        }
        Ok(claims)
    }

    /// Verify specifically a refresh token (rejects access tokens)
    pub fn verify_refresh_token(&self, token: &str) -> AppResult<Claims> {
        let claims = self.verify_token(token)?;
        if claims.token_type != "refresh" {
            return Err(AppError::Jwt("Invalid token type: expected refresh token".to_string()));
        }
        Ok(claims)
    }
}
