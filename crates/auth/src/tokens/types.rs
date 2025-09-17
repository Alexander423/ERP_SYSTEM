use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Purpose of a verification token
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenPurpose {
    EmailVerification,
    PasswordReset,
    InviteUser,
    ChangeEmail,
}

impl TokenPurpose {
    /// Get default expiry duration for this token purpose
    pub fn default_expiry_hours(&self) -> u32 {
        match self {
            TokenPurpose::EmailVerification => 24,  // 24 hours
            TokenPurpose::PasswordReset => 1,       // 1 hour
            TokenPurpose::InviteUser => 168,        // 7 days
            TokenPurpose::ChangeEmail => 24,        // 24 hours
        }
    }

    /// Check if this token purpose allows multiple active tokens
    pub fn allows_multiple_tokens(&self) -> bool {
        match self {
            TokenPurpose::EmailVerification => false,
            TokenPurpose::PasswordReset => false,    // Only latest reset token should be valid
            TokenPurpose::InviteUser => true,        // Can have multiple invites
            TokenPurpose::ChangeEmail => false,
        }
    }

    /// Get cache key prefix for this token purpose
    pub fn cache_prefix(&self) -> &'static str {
        match self {
            TokenPurpose::EmailVerification => "verify_email",
            TokenPurpose::PasswordReset => "reset_password",
            TokenPurpose::InviteUser => "invite_user",
            TokenPurpose::ChangeEmail => "change_email",
        }
    }
}

impl std::fmt::Display for TokenPurpose {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenPurpose::EmailVerification => write!(f, "email_verification"),
            TokenPurpose::PasswordReset => write!(f, "password_reset"),
            TokenPurpose::InviteUser => write!(f, "invite_user"),
            TokenPurpose::ChangeEmail => write!(f, "change_email"),
        }
    }
}

/// Token data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenData {
    /// Unique token identifier
    pub token: String,
    /// Token purpose
    pub purpose: TokenPurpose,
    /// User ID this token belongs to
    pub user_id: Uuid,
    /// Tenant ID
    pub tenant_id: Uuid,
    /// Email address (for verification tokens)
    pub email: Option<String>,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
    /// When the token was created
    pub created_at: DateTime<Utc>,
    /// When the token expires
    pub expires_at: DateTime<Utc>,
    /// Whether the token has been used
    pub used: bool,
    /// When the token was used (if applicable)
    pub used_at: Option<DateTime<Utc>>,
    /// IP address where token was created
    pub created_ip: Option<String>,
    /// IP address where token was used
    pub used_ip: Option<String>,
}

impl TokenData {
    pub fn new(
        purpose: TokenPurpose,
        user_id: Uuid,
        tenant_id: Uuid,
        expiry_hours: Option<u32>,
    ) -> Self {
        let token = Self::generate_secure_token();
        let expires_at = Utc::now() + chrono::Duration::hours(
            expiry_hours.unwrap_or_else(|| purpose.default_expiry_hours()) as i64
        );

        Self {
            token,
            purpose,
            user_id,
            tenant_id,
            email: None,
            metadata: std::collections::HashMap::new(),
            created_at: Utc::now(),
            expires_at,
            used: false,
            used_at: None,
            created_ip: None,
            used_ip: None,
        }
    }

    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    pub fn with_created_ip(mut self, ip: impl Into<String>) -> Self {
        self.created_ip = Some(ip.into());
        self
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Check if token is valid (not expired and not used)
    pub fn is_valid(&self) -> bool {
        !self.is_expired() && !self.used
    }

    /// Mark token as used
    pub fn mark_used(&mut self, used_ip: Option<String>) {
        self.used = true;
        self.used_at = Some(Utc::now());
        self.used_ip = used_ip;
    }

    /// Get time until expiry
    pub fn time_until_expiry(&self) -> Option<chrono::Duration> {
        if self.is_expired() {
            None
        } else {
            Some(self.expires_at - Utc::now())
        }
    }

    /// Generate a cryptographically secure token
    fn generate_secure_token() -> String {
        use rand::rngs::OsRng;
        use rand::RngCore;

        // Generate 32 random bytes using cryptographically secure OS RNG
        let mut rng = OsRng;
        let mut bytes = [0u8; 32];
        rng.fill_bytes(&mut bytes);

        // Use URL-safe base64 without padding
        base64_url::encode(&bytes)
    }

    /// Get cache key for storing this token in Redis
    pub fn cache_key(&self) -> String {
        format!("token:{}:{}:{}", self.purpose.cache_prefix(), self.tenant_id, self.token)
    }

    /// Get user tokens cache key (for finding all tokens for a user)
    pub fn user_tokens_key(&self) -> String {
        format!("user_tokens:{}:{}:{}", self.tenant_id, self.user_id, self.purpose.cache_prefix())
    }
}

/// Verification token for database storage
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct VerificationToken {
    pub id: Uuid,
    pub token: String,
    pub purpose: String, // Stored as string in DB
    pub user_id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub email: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub used: Option<bool>,
    pub used_at: Option<DateTime<Utc>>,
    pub created_ip: Option<String>,
    pub used_ip: Option<String>,
}

impl From<TokenData> for VerificationToken {
    fn from(token_data: TokenData) -> Self {
        Self {
            id: Uuid::new_v4(),
            token: token_data.token,
            purpose: token_data.purpose.to_string(),
            user_id: token_data.user_id,
            tenant_id: Some(token_data.tenant_id),
            email: token_data.email,
            metadata: if token_data.metadata.is_empty() {
                None
            } else {
                Some(serde_json::to_value(token_data.metadata).unwrap_or(serde_json::Value::Null))
            },
            created_at: token_data.created_at,
            expires_at: token_data.expires_at,
            used: Some(token_data.used),
            used_at: token_data.used_at,
            created_ip: token_data.created_ip,
            used_ip: token_data.used_ip,
        }
    }
}

impl TryFrom<VerificationToken> for TokenData {
    type Error = serde_json::Error;

    fn try_from(db_token: VerificationToken) -> Result<Self, Self::Error> {
        let purpose = match db_token.purpose.as_str() {
            "email_verification" => TokenPurpose::EmailVerification,
            "password_reset" => TokenPurpose::PasswordReset,
            "invite_user" => TokenPurpose::InviteUser,
            "change_email" => TokenPurpose::ChangeEmail,
            _ => return Err(serde_json::Error::io(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid token purpose"))),
        };

        let metadata = if let Some(meta) = db_token.metadata {
            serde_json::from_value(meta)?
        } else {
            std::collections::HashMap::new()
        };

        Ok(Self {
            token: db_token.token,
            purpose,
            user_id: db_token.user_id,
            tenant_id: db_token.tenant_id.unwrap_or_else(Uuid::new_v4),
            email: db_token.email,
            metadata,
            created_at: db_token.created_at,
            expires_at: db_token.expires_at,
            used: db_token.used.unwrap_or(false),
            used_at: db_token.used_at,
            created_ip: db_token.created_ip,
            used_ip: db_token.used_ip,
        })
    }
}

// Helper module for URL-safe base64 encoding
mod base64_url {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    
    pub fn encode(data: &[u8]) -> String {
        URL_SAFE_NO_PAD.encode(data)
    }
    
    #[allow(dead_code)]
    pub fn decode(data: &str) -> Result<Vec<u8>, base64::DecodeError> {
        URL_SAFE_NO_PAD.decode(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_purpose_defaults() {
        assert_eq!(TokenPurpose::EmailVerification.default_expiry_hours(), 24);
        assert_eq!(TokenPurpose::PasswordReset.default_expiry_hours(), 1);
        assert_eq!(TokenPurpose::InviteUser.default_expiry_hours(), 168);
        
        assert!(!TokenPurpose::EmailVerification.allows_multiple_tokens());
        assert!(!TokenPurpose::PasswordReset.allows_multiple_tokens());
        assert!(TokenPurpose::InviteUser.allows_multiple_tokens());
    }

    #[test]
    fn test_token_data_creation() {
        let user_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();
        
        let token = TokenData::new(
            TokenPurpose::EmailVerification,
            user_id,
            tenant_id,
            Some(48), // 48 hours
        );

        assert_eq!(token.purpose, TokenPurpose::EmailVerification);
        assert_eq!(token.user_id, user_id);
        assert_eq!(token.tenant_id, tenant_id);
        assert!(!token.token.is_empty());
        assert!(!token.is_expired());
        assert!(token.is_valid());
        assert!(!token.used);
    }

    #[test]
    fn test_token_expiry() {
        let user_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();
        
        // Create token that expires immediately
        let mut token = TokenData::new(
            TokenPurpose::PasswordReset,
            user_id,
            tenant_id,
            Some(0),
        );
        token.expires_at = Utc::now() - chrono::Duration::seconds(1);

        assert!(token.is_expired());
        assert!(!token.is_valid());
    }

    #[test]
    fn test_token_usage() {
        let user_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();
        
        let mut token = TokenData::new(
            TokenPurpose::EmailVerification,
            user_id,
            tenant_id,
            None,
        );

        assert!(!token.used);
        assert!(token.used_at.is_none());

        token.mark_used(Some("192.168.1.1".to_string()));

        assert!(token.used);
        assert!(token.used_at.is_some());
        assert_eq!(token.used_ip, Some("192.168.1.1".to_string()));
        assert!(!token.is_valid()); // Used tokens are not valid
    }

    #[test]
    fn test_cache_keys() {
        let user_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();
        
        let token = TokenData::new(
            TokenPurpose::EmailVerification,
            user_id,
            tenant_id,
            None,
        );

        let cache_key = token.cache_key();
        assert!(cache_key.contains("verify_email"));
        assert!(cache_key.contains(&tenant_id.to_string()));
        assert!(cache_key.contains(&token.token));

        let user_key = token.user_tokens_key();
        assert!(user_key.contains("user_tokens"));
        assert!(user_key.contains(&tenant_id.to_string()));
        assert!(user_key.contains(&user_id.to_string()));
    }

    #[test]
    fn test_token_db_conversion() {
        let user_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();
        
        let original = TokenData::new(
            TokenPurpose::EmailVerification,
            user_id,
            tenant_id,
            None,
        ).with_email("test@example.com");

        let db_token: VerificationToken = original.clone().into();
        let converted: TokenData = db_token.try_into().unwrap();

        assert_eq!(original.token, converted.token);
        assert_eq!(original.purpose, converted.purpose);
        assert_eq!(original.user_id, converted.user_id);
        assert_eq!(original.email, converted.email);
    }
}