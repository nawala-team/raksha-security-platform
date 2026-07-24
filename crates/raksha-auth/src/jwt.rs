use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use raksha_core::config::JwtConfig;
use raksha_core::error::{AppError, AppResult};
use raksha_core::models::UserRole;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: Uuid,
    /// User role
    pub role: UserRole,
    /// Issuer
    pub iss: String,
    /// Issued at
    pub iat: i64,
    /// Expiration
    pub exp: i64,
    /// JWT ID
    pub jti: Uuid,
    /// Session ID
    pub sid: Uuid,
    /// Token type: "access" or "refresh"
    pub token_type: String,
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
    pub fn new(config: &JwtConfig) -> Self {
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
            iat: now.timestamp(),
            exp: (now + Duration::seconds(self.config.access_token_ttl_secs)).timestamp(),
            jti: Uuid::now_v7(),
            sid: session_id,
            token_type: "access".to_string(),
        };

        let refresh_claims = Claims {
            sub: user_id,
            role,
            iss: self.config.issuer.clone(),
            iat: now.timestamp(),
            exp: (now + Duration::seconds(self.config.refresh_token_ttl_secs)).timestamp(),
            jti: Uuid::now_v7(),
            sid: session_id,
            token_type: "refresh".to_string(),
        };

        let access_token = encode(&Header::default(), &access_claims, &self.encoding_key)
            .map_err(|e| AppError::Jwt(e.to_string()))?;

        let refresh_token = encode(&Header::default(), &refresh_claims, &self.encoding_key)
            .map_err(|e| AppError::Jwt(e.to_string()))?;

        Ok(TokenPair {
            access_token,
            refresh_token,
            expires_in: self.config.access_token_ttl_secs,
            token_type: "Bearer".to_string(),
        })
    }

    pub fn verify_token(&self, token: &str) -> AppResult<Claims> {
        let mut validation = Validation::default();
        validation.set_issuer(&[&self.config.issuer]);

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(|e| AppError::Jwt(e.to_string()))?;

        Ok(token_data.claims)
    }
}
