use crate::{config::JwtConfig, error::Result, types::JwtClaims, Error};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenClaims {
    pub sub: String, // user_id
    pub tenant_id: String,
    pub exp: i64,
    pub iat: i64,
    pub jti: String,
    pub token_version: u32,
}

#[derive(Clone)]
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    access_token_expiry: Duration,
    refresh_token_expiry: Duration,
}

impl JwtService {
    pub fn new(config: &JwtConfig) -> Result<Self> {
        let encoding_key = EncodingKey::from_secret(config.secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(config.secret.as_bytes());

        Ok(Self {
            encoding_key,
            decoding_key,
            access_token_expiry: Duration::seconds(config.access_token_expiry),
            refresh_token_expiry: Duration::seconds(config.refresh_token_expiry),
        })
    }

    pub fn generate_token_pair(
        &self,
        user_id: &str,
        tenant_id: &str,
        roles: Vec<String>,
        permissions: Vec<String>,
        impersonator_id: Option<String>,
    ) -> Result<TokenPair> {
        let now = Utc::now();
        let access_jti = Uuid::new_v4().to_string();
        let refresh_jti = Uuid::new_v4().to_string();

        let access_claims = JwtClaims {
            sub: user_id.to_string(),
            tenant_id: tenant_id.to_string(),
            roles,
            permissions,
            exp: (now + self.access_token_expiry).timestamp(),
            iat: now.timestamp(),
            jti: access_jti,
            impersonator_id,
        };

        let refresh_claims = RefreshTokenClaims {
            sub: user_id.to_string(),
            tenant_id: tenant_id.to_string(),
            exp: (now + self.refresh_token_expiry).timestamp(),
            iat: now.timestamp(),
            jti: refresh_jti,
            token_version: 1,
        };

        let header = Header::new(Algorithm::HS512);

        let access_token = encode(&header, &access_claims, &self.encoding_key)
            .map_err(|e| Error::new(crate::error::ErrorCode::TokenInvalid, format!("Failed to generate access token: {}", e)))?;

        let refresh_token = encode(&header, &refresh_claims, &self.encoding_key)
            .map_err(|e| Error::new(crate::error::ErrorCode::TokenInvalid, format!("Failed to generate refresh token: {}", e)))?;

        Ok(TokenPair {
            access_token,
            refresh_token,
        })
    }

    pub fn verify_access_token(&self, token: &str) -> Result<JwtClaims> {
        let mut validation = Validation::new(Algorithm::HS512);
        validation.validate_exp = true;

        let token_data = decode::<JwtClaims>(token, &self.decoding_key, &validation)
            .map_err(|e| Error::new(crate::error::ErrorCode::TokenInvalid, format!("Invalid access token: {}", e)))?;

        Ok(token_data.claims)
    }

    pub fn verify_refresh_token(&self, token: &str) -> Result<RefreshTokenClaims> {
        let mut validation = Validation::new(Algorithm::HS512);
        validation.validate_exp = true;

        let token_data = decode::<RefreshTokenClaims>(token, &self.decoding_key, &validation)
            .map_err(|e| Error::new(crate::error::ErrorCode::TokenInvalid, format!("Invalid refresh token: {}", e)))?;

        Ok(token_data.claims)
    }

    pub fn generate_login_session_token(&self, user_id: &str, tenant_id: &str) -> Result<String> {
        let now = Utc::now();
        let exp = now + Duration::minutes(5);

        let claims = serde_json::json!({
            "sub": user_id,
            "tenant_id": tenant_id,
            "exp": exp.timestamp(),
            "iat": now.timestamp(),
            "jti": Uuid::new_v4().to_string(),
            "purpose": "2fa_verification"
        });

        let header = Header::new(Algorithm::HS512);
        encode(&header, &claims, &self.encoding_key)
            .map_err(|e| Error::new(crate::error::ErrorCode::TokenInvalid, format!("Failed to generate session token: {}", e)))
    }
}