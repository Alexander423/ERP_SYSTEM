use super::types::{TokenData, TokenPurpose, VerificationToken};
use erp_core::{
    audit::{AuditEvent, AuditLogger, event::EventOutcome, EventSeverity, EventType},
    error::{Error, ErrorCode, Result},
    DatabasePool, TenantContext,
};
use redis::{aio::ConnectionManager, AsyncCommands};
use sqlx::Row;
use std::collections::HashMap;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Token manager for handling verification tokens
pub struct TokenManager {
    db: DatabasePool,
    redis: ConnectionManager,
    audit_logger: Option<AuditLogger>,
}

impl TokenManager {
    pub fn new(
        db: DatabasePool,
        redis: ConnectionManager,
        audit_logger: Option<AuditLogger>,
    ) -> Self {
        Self {
            db,
            redis,
            audit_logger,
        }
    }

    /// Create a new verification token
    pub async fn create_token(
        &self,
        tenant: &TenantContext,
        purpose: TokenPurpose,
        user_id: Uuid,
        email: Option<String>,
        expiry_hours: Option<u32>,
        created_ip: Option<String>,
        metadata: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<TokenData> {
        info!(
            tenant_id = %tenant.tenant_id.0,
            user_id = %user_id,
            purpose = %purpose,
            "Creating verification token"
        );

        // If this token purpose doesn't allow multiple tokens, invalidate existing ones
        if !purpose.allows_multiple_tokens() {
            self.invalidate_user_tokens(tenant, user_id, purpose).await?;
        }

        // Create new token
        let mut token_data = TokenData::new(purpose, user_id, tenant.tenant_id.0, expiry_hours);
        
        if let Some(email) = email {
            token_data = token_data.with_email(email);
        }

        if let Some(ip) = created_ip {
            token_data = token_data.with_created_ip(ip);
        }

        if let Some(meta) = metadata {
            for (key, value) in meta {
                token_data = token_data.with_metadata(key, value);
            }
        }

        // Store in database
        self.store_token_in_db(tenant, &token_data).await?;

        // Cache in Redis
        self.cache_token(&token_data).await?;

        // Audit log
        if let Some(audit_logger) = &self.audit_logger {
            audit_logger.log_event(
                AuditEvent::builder(
                    EventType::Custom("TOKEN_CREATED".to_string()),
                    format!("Verification token created for purpose: {}", purpose)
                )
                .severity(EventSeverity::Info)
                .outcome(EventOutcome::Success)
                .resource("verification_token", &token_data.token)
                .metadata("purpose".to_string(), serde_json::Value::String(purpose.to_string()))
                .metadata("expires_at".to_string(), serde_json::Value::String(token_data.expires_at.to_rfc3339()))
                .build()
            ).await?;
        }

        debug!("Created verification token: {} for user: {}", token_data.token, user_id);
        Ok(token_data)
    }

    /// Validate and consume a token
    pub async fn validate_token(
        &self,
        tenant: &TenantContext,
        token: &str,
        purpose: TokenPurpose,
        used_ip: Option<String>,
    ) -> Result<TokenData> {
        debug!(
            tenant_id = %tenant.tenant_id.0,
            token = token,
            purpose = %purpose,
            "Validating token"
        );

        // Try to get from cache first
        let mut token_data = match self.get_cached_token(tenant, token, purpose).await {
            Ok(Some(data)) => data,
            Ok(None) => {
                // Not in cache, try database
                match self.get_token_from_db(tenant, token, purpose).await? {
                    Some(data) => {
                        // Cache it for faster future access
                        self.cache_token(&data).await?;
                        data
                    }
                    None => {
                        // Log invalid token attempt
                        if let Some(audit_logger) = &self.audit_logger {
                            audit_logger.log_event(
                                AuditEvent::builder(
                                    EventType::SecurityPolicyViolation,
                                    "Invalid verification token used"
                                )
                                .severity(EventSeverity::Warning)
                                .outcome(EventOutcome::Failure)
                                .resource("verification_token", token)
                                .metadata("purpose".to_string(), serde_json::Value::String(purpose.to_string()))
                                .build()
                            ).await?;
                        }
                        
                        return Err(Error::new(ErrorCode::ResourceNotFound, "Invalid token"));
                    }
                }
            }
            Err(e) => {
                warn!("Failed to get token from cache: {}", e);
                // Fall back to database
                match self.get_token_from_db(tenant, token, purpose).await? {
                    Some(data) => data,
                    None => return Err(Error::new(ErrorCode::ResourceNotFound, "Invalid token")),
                }
            }
        };

        // Validate token
        if token_data.used {
            if let Some(audit_logger) = &self.audit_logger {
                audit_logger.log_event(
                    AuditEvent::builder(
                        EventType::SecurityPolicyViolation,
                        "Attempted reuse of already used verification token"
                    )
                    .severity(EventSeverity::Warning)
                    .outcome(EventOutcome::Failure)
                    .resource("verification_token", token)
                    .metadata("original_used_at".to_string(), 
                        serde_json::Value::String(token_data.used_at.unwrap_or_default().to_rfc3339()))
                    .build()
                ).await?;
            }
            
            return Err(Error::new(ErrorCode::InvalidInput, "Token has already been used"));
        }

        if token_data.is_expired() {
            if let Some(audit_logger) = &self.audit_logger {
                audit_logger.log_event(
                    AuditEvent::builder(
                        EventType::SecurityPolicyViolation,
                        "Attempted use of expired verification token"
                    )
                    .severity(EventSeverity::Info)
                    .outcome(EventOutcome::Failure)
                    .resource("verification_token", token)
                    .metadata("expired_at".to_string(), 
                        serde_json::Value::String(token_data.expires_at.to_rfc3339()))
                    .build()
                ).await?;
            }
            
            return Err(Error::new(ErrorCode::TokenExpired, "Token has expired"));
        }

        // Mark token as used
        token_data.mark_used(used_ip.clone());

        // Update in database
        self.mark_token_used_in_db(tenant, &token_data).await?;

        // Remove from cache (used tokens shouldn't be cached)
        self.remove_token_from_cache(&token_data).await?;

        // Audit successful validation
        if let Some(audit_logger) = &self.audit_logger {
            audit_logger.log_event(
                AuditEvent::builder(
                    EventType::Custom("TOKEN_VALIDATED".to_string()),
                    format!("Verification token successfully validated for purpose: {}", purpose)
                )
                .severity(EventSeverity::Info)
                .outcome(EventOutcome::Success)
                .resource("verification_token", token)
                .metadata("purpose".to_string(), serde_json::Value::String(purpose.to_string()))
                .metadata("user_id".to_string(), serde_json::Value::String(token_data.user_id.to_string()))
                .build()
            ).await?;
        }

        info!("Token validated successfully: {} for user: {}", token, token_data.user_id);
        Ok(token_data)
    }

    /// Get a token without consuming it (for verification purposes)
    pub async fn get_token(
        &self,
        tenant: &TenantContext,
        token: &str,
        purpose: TokenPurpose,
    ) -> Result<Option<TokenData>> {
        // Try cache first
        match self.get_cached_token(tenant, token, purpose).await {
            Ok(cached) => Ok(cached),
            Err(_) => {
                // Fall back to database
                self.get_token_from_db(tenant, token, purpose).await
            }
        }
    }

    /// Invalidate all tokens for a user with specific purpose
    pub async fn invalidate_user_tokens(
        &self,
        tenant: &TenantContext,
        user_id: Uuid,
        purpose: TokenPurpose,
    ) -> Result<u32> {
        info!(
            tenant_id = %tenant.tenant_id.0,
            user_id = %user_id,
            purpose = %purpose,
            "Invalidating user tokens"
        );

        // Mark tokens as used in database
        let pool = self.db.get_tenant_pool(tenant).await?;
        let invalidated_count = sqlx::query(
            "UPDATE verification_tokens
             SET used = true, used_at = NOW()
             WHERE tenant_id = $1 AND user_id = $2 AND purpose = $3 AND used = false AND expires_at > NOW()"
        )
        .bind(tenant.tenant_id.0)
        .bind(user_id)
        .bind(purpose.to_string())
        .execute(pool.get())
        .await?
        .rows_affected() as u32;

        // Clear from cache
        self.clear_user_tokens_from_cache(tenant, user_id, purpose).await?;

        // Audit log
        if let Some(audit_logger) = &self.audit_logger {
            audit_logger.log_event(
                AuditEvent::builder(
                    EventType::Custom("TOKENS_INVALIDATED".to_string()),
                    format!("Invalidated {} tokens for user", invalidated_count)
                )
                .severity(EventSeverity::Info)
                .outcome(EventOutcome::Success)
                .metadata("invalidated_count".to_string(), serde_json::Value::Number(invalidated_count.into()))
                .metadata("purpose".to_string(), serde_json::Value::String(purpose.to_string()))
                .build()
            ).await?;
        }

        info!("Invalidated {} tokens for user: {}", invalidated_count, user_id);
        Ok(invalidated_count)
    }

    /// Clean up expired tokens
    pub async fn cleanup_expired_tokens(&self, tenant: &TenantContext) -> Result<u32> {
        info!("Cleaning up expired tokens");

        let pool = self.db.get_tenant_pool(tenant).await?;
        let deleted_count = sqlx::query(
            "DELETE FROM verification_tokens WHERE tenant_id = $1 AND expires_at < NOW()"
        )
        .bind(tenant.tenant_id.0)
        .execute(pool.get())
        .await?
        .rows_affected() as u32;

        info!("Cleaned up {} expired tokens", deleted_count);
        Ok(deleted_count)
    }

    /// Get token statistics
    pub async fn get_token_stats(&self, tenant: &TenantContext) -> Result<TokenStats> {
        let pool = self.db.get_tenant_pool(tenant).await?;
        
        let stats = sqlx::query(
            r#"
            SELECT
                purpose,
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE used = false AND expires_at > NOW()) as active,
                COUNT(*) FILTER (WHERE used = true) as used,
                COUNT(*) FILTER (WHERE expires_at < NOW()) as expired
            FROM verification_tokens
            WHERE tenant_id = $1
            GROUP BY purpose
            "#
        )
        .bind(tenant.tenant_id.0)
        .fetch_all(pool.get())
        .await?;

        let mut by_purpose = HashMap::new();
        for row in stats {
            by_purpose.insert(row.try_get("purpose")?, PurposeStats {
                total: row.try_get::<Option<i64>, _>("total")?.unwrap_or(0) as u32,
                active: row.try_get::<Option<i64>, _>("active")?.unwrap_or(0) as u32,
                used: row.try_get::<Option<i64>, _>("used")?.unwrap_or(0) as u32,
                expired: row.try_get::<Option<i64>, _>("expired")?.unwrap_or(0) as u32,
            });
        }

        Ok(TokenStats { by_purpose })
    }

    // Private helper methods

    async fn store_token_in_db(&self, tenant: &TenantContext, token_data: &TokenData) -> Result<()> {
        let pool = self.db.get_tenant_pool(tenant).await?;
        let db_token: VerificationToken = token_data.clone().into();

        sqlx::query(
            r#"
            INSERT INTO verification_tokens (
                id, token, purpose, user_id, tenant_id, email, metadata,
                created_at, expires_at, used, used_at, created_ip, used_ip
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#
        )
        .bind(db_token.id)
        .bind(&db_token.token)
        .bind(&db_token.purpose)
        .bind(db_token.user_id)
        .bind(db_token.tenant_id)
        .bind(&db_token.email)
        .bind(&db_token.metadata)
        .bind(db_token.created_at)
        .bind(db_token.expires_at)
        .bind(db_token.used)
        .bind(db_token.used_at)
        .bind(&db_token.created_ip)
        .bind(&db_token.used_ip)
        .execute(pool.get())
        .await?;

        debug!("Stored token in database: {}", token_data.token);
        Ok(())
    }

    async fn get_token_from_db(
        &self,
        tenant: &TenantContext,
        token: &str,
        purpose: TokenPurpose,
    ) -> Result<Option<TokenData>> {
        let pool = self.db.get_tenant_pool(tenant).await?;
        
        let row = sqlx::query(
            r#"
            SELECT id, token, purpose, user_id, tenant_id, email, metadata,
                   created_at, expires_at, used, used_at, created_ip, used_ip
            FROM verification_tokens
            WHERE token = $1 AND purpose = $2 AND tenant_id = $3
            "#
        )
        .bind(token)
        .bind(purpose.to_string())
        .bind(tenant.tenant_id.0)
        .fetch_optional(pool.get())
        .await?;

        let db_token: Option<VerificationToken> = match row {
            Some(row) => {
                Some(VerificationToken {
                    id: row.try_get("id")?,
                    token: row.try_get("token")?,
                    purpose: row.try_get("purpose")?,
                    user_id: row.try_get("user_id")?,
                    tenant_id: row.try_get("tenant_id")?,
                    email: row.try_get("email")?,
                    metadata: row.try_get("metadata")?,
                    created_at: row.try_get("created_at")?,
                    expires_at: row.try_get("expires_at")?,
                    used: row.try_get("used")?,
                    used_at: row.try_get("used_at")?,
                    created_ip: row.try_get("created_ip")?,
                    used_ip: row.try_get("used_ip")?,
                })
            }
            None => None,
        };

        match db_token {
            Some(db_token) => {
                let token_data = TokenData::try_from(db_token)
                    .map_err(|e| Error::new(ErrorCode::SerializationError, e.to_string()))?;
                Ok(Some(token_data))
            }
            None => Ok(None),
        }
    }

    async fn mark_token_used_in_db(&self, tenant: &TenantContext, token_data: &TokenData) -> Result<()> {
        let pool = self.db.get_tenant_pool(tenant).await?;

        sqlx::query(
            "UPDATE verification_tokens SET used = $1, used_at = $2, used_ip = $3 WHERE token = $4 AND tenant_id = $5"
        )
        .bind(token_data.used)
        .bind(token_data.used_at)
        .bind(&token_data.used_ip)
        .bind(&token_data.token)
        .bind(tenant.tenant_id.0)
        .execute(pool.get())
        .await?;

        Ok(())
    }

    async fn cache_token(&self, token_data: &TokenData) -> Result<()> {
        let mut conn = self.redis.clone();
        let cache_key = token_data.cache_key();
        let ttl = token_data.time_until_expiry()
            .map(|d| d.num_seconds().max(1))
            .unwrap_or(3600) as usize; // Default to 1 hour if calculation fails

        let serialized = serde_json::to_string(token_data)
            .map_err(|e| Error::new(ErrorCode::SerializationError, e.to_string()))?;

        conn.set_ex::<_, _, ()>(&cache_key, serialized, ttl as u64).await?;
        debug!("Cached token: {} with TTL: {}s", token_data.token, ttl);
        Ok(())
    }

    async fn get_cached_token(
        &self,
        tenant: &TenantContext,
        token: &str,
        purpose: TokenPurpose,
    ) -> Result<Option<TokenData>> {
        let mut conn = self.redis.clone();
        let cache_key = format!("token:{}:{}:{}", purpose.cache_prefix(), tenant.tenant_id.0, token);

        let cached: Option<String> = conn.get(&cache_key).await?;
        match cached {
            Some(data) => {
                let token_data: TokenData = serde_json::from_str(&data)
                    .map_err(|e| Error::new(ErrorCode::SerializationError, e.to_string()))?;
                Ok(Some(token_data))
            }
            None => Ok(None),
        }
    }

    async fn remove_token_from_cache(&self, token_data: &TokenData) -> Result<()> {
        let mut conn = self.redis.clone();
        let cache_key = token_data.cache_key();
        let _: u32 = conn.del(&cache_key).await?;
        debug!("Removed token from cache: {}", token_data.token);
        Ok(())
    }

    async fn clear_user_tokens_from_cache(
        &self,
        tenant: &TenantContext,
        user_id: Uuid,
        purpose: TokenPurpose,
    ) -> Result<()> {
        let mut conn = self.redis.clone();
        let pattern = format!("token:{}:{}:*", purpose.cache_prefix(), tenant.tenant_id.0);

        // Use SCAN instead of KEYS for better performance in production
        let keys = self.scan_keys(&mut conn, &pattern).await?;

        if !keys.is_empty() {
            let _: u32 = conn.del(&keys).await?;
            debug!("Cleared {} cached tokens for user: {}", keys.len(), user_id);
        }

        Ok(())
    }

    /// Non-blocking scan for Redis keys matching a pattern
    async fn scan_keys(&self, conn: &mut redis::aio::ConnectionManager, pattern: &str) -> Result<Vec<String>> {
        use redis::{AsyncCommands, Cmd};

        let mut cursor: u64 = 0;
        let mut keys = Vec::new();

        loop {
            let mut cmd = Cmd::new();
            cmd.arg("SCAN").arg(cursor).arg("MATCH").arg(pattern).arg("COUNT").arg(100);

            let result: Vec<redis::Value> = cmd.query_async(conn).await
                .map_err(|e| Error::new(ErrorCode::InternalServerError, format!("Redis SCAN failed: {}", e)))?;

            if let [redis::Value::BulkString(cursor_bytes), redis::Value::Array(key_values)] = &result[..] {
                cursor = String::from_utf8_lossy(cursor_bytes).parse().unwrap_or(0);

                for key_value in key_values {
                    if let redis::Value::BulkString(key_bytes) = key_value {
                        keys.push(String::from_utf8_lossy(key_bytes).to_string());
                    }
                }
            }

            // If cursor is 0, we've completed the scan
            if cursor == 0 {
                break;
            }
        }

        Ok(keys)
    }
}

/// Token statistics by purpose
#[derive(Debug, Clone)]
pub struct TokenStats {
    pub by_purpose: HashMap<String, PurposeStats>,
}

#[derive(Debug, Clone)]
pub struct PurposeStats {
    pub total: u32,
    pub active: u32,
    pub used: u32,
    pub expired: u32,
}