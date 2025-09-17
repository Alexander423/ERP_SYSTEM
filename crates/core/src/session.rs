pub mod cleanup;

pub use cleanup::{SessionCleanupService, SessionStatsSnapshot, AggregatedSessionStats, CleanupServiceHealth};

use crate::{
    error::{Error, ErrorCode, Result},
    TenantContext,
};
use chrono::{DateTime, Duration, Utc};
use redis::{aio::ConnectionManager, AsyncCommands};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Session data stored in Redis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    /// Unique session identifier
    pub session_id: String,
    /// User ID associated with this session
    pub user_id: Uuid,
    /// Tenant ID for multi-tenancy
    pub tenant_id: Uuid,
    /// Session creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last activity timestamp (for sliding window timeout)
    pub last_activity: DateTime<Utc>,
    /// Session expiration timestamp
    pub expires_at: DateTime<Utc>,
    /// Client IP address
    pub client_ip: Option<String>,
    /// User agent string
    pub user_agent: Option<String>,
    /// Additional session metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Session state
    pub state: SessionState,
    /// JWT token version (for invalidation)
    pub token_version: u32,
    /// Device fingerprint for additional security
    pub device_fingerprint: Option<String>,
}

/// Session state enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionState {
    /// Active session
    Active,
    /// Session terminated by user logout
    LoggedOut,
    /// Session expired due to inactivity
    Expired,
    /// Session revoked by administrator or security policy
    Revoked,
    /// Session suspended due to suspicious activity
    Suspended,
}

/// Session configuration
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Session inactivity timeout (default: 30 minutes)
    pub inactivity_timeout: Duration,
    /// Absolute session timeout (default: 12 hours)
    pub absolute_timeout: Duration,
    /// Session cleanup interval (default: 5 minutes)
    pub cleanup_interval: Duration,
    /// Maximum sessions per user (default: 10)
    pub max_sessions_per_user: u32,
    /// Enable sliding window timeout
    pub enable_sliding_window: bool,
    /// Require device consistency
    pub require_device_consistency: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            inactivity_timeout: Duration::minutes(30),
            absolute_timeout: Duration::hours(12),
            cleanup_interval: Duration::minutes(5),
            max_sessions_per_user: 10,
            enable_sliding_window: true,
            require_device_consistency: false,
        }
    }
}

/// Session manager for handling user sessions with Redis storage
pub struct SessionManager {
    redis: ConnectionManager,
    config: SessionConfig,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(redis: ConnectionManager, config: SessionConfig) -> Self {
        Self { redis, config }
    }

    /// Create a new session for a user
    pub async fn create_session(
        &self,
        tenant: &TenantContext,
        user_id: Uuid,
        client_ip: Option<String>,
        user_agent: Option<String>,
        device_fingerprint: Option<String>,
    ) -> Result<SessionData> {
        let now = Utc::now();
        let session_id = Uuid::new_v4().to_string();

        // Clean up old sessions for this user if we've exceeded the limit
        self.enforce_session_limit(tenant, user_id).await?;

        let session = SessionData {
            session_id: session_id.clone(),
            user_id,
            tenant_id: tenant.tenant_id.0,
            created_at: now,
            last_activity: now,
            expires_at: now + self.config.absolute_timeout,
            client_ip: client_ip.clone(),
            user_agent: user_agent.clone(),
            metadata: HashMap::new(),
            state: SessionState::Active,
            token_version: 1,
            device_fingerprint: device_fingerprint.clone(),
        };

        // Store session in Redis
        self.store_session(&session).await?;

        // Add to user's session index
        self.add_to_user_sessions(tenant, user_id, &session_id).await?;

        info!(
            tenant_id = %tenant.tenant_id.0,
            user_id = %user_id,
            session_id = %session_id,
            client_ip = ?client_ip,
            "Session created successfully"
        );

        Ok(session)
    }

    /// Retrieve and validate a session
    pub async fn get_session(
        &self,
        tenant: &TenantContext,
        session_id: &str,
    ) -> Result<Option<SessionData>> {
        let session_key = self.session_key(tenant, session_id);
        let mut conn = self.redis.clone();

        let session_data: Option<String> = conn.get(&session_key).await?;

        match session_data {
            Some(data) => {
                let mut session: SessionData = serde_json::from_str(&data)
                    .map_err(|e| Error::new(ErrorCode::SerializationError, e.to_string()))?;

                // Check if session is expired or invalid
                if !self.is_session_valid(&session) {
                    // Clean up invalid session
                    self.invalidate_session(tenant, session_id, SessionState::Expired)
                        .await?;
                    return Ok(None);
                }

                // Update last activity if sliding window is enabled
                if self.config.enable_sliding_window {
                    session.last_activity = Utc::now();
                    self.store_session(&session).await?;
                }

                Ok(Some(session))
            }
            None => Ok(None),
        }
    }

    /// Update session metadata
    pub async fn update_session_metadata(
        &self,
        tenant: &TenantContext,
        session_id: &str,
        key: String,
        value: serde_json::Value,
    ) -> Result<()> {
        let mut session = match self.get_session(tenant, session_id).await? {
            Some(s) => s,
            None => return Err(Error::new(ErrorCode::ResourceNotFound, "Session not found")),
        };

        session.metadata.insert(key, value);
        session.last_activity = Utc::now();
        
        self.store_session(&session).await?;
        Ok(())
    }

    /// Invalidate a specific session
    pub async fn invalidate_session(
        &self,
        tenant: &TenantContext,
        session_id: &str,
        reason: SessionState,
    ) -> Result<()> {
        let session_key = self.session_key(tenant, session_id);
        let mut conn = self.redis.clone();

        // Get session data directly from Redis without validation to avoid recursion
        let session_data: Option<String> = conn.get(&session_key).await?;
        
        if let Some(data) = session_data {
            if let Ok(session) = serde_json::from_str::<SessionData>(&data) {
                // Remove from user's session index
                self.remove_from_user_sessions(tenant, session.user_id, session_id)
                    .await?;

                info!(
                    tenant_id = %tenant.tenant_id.0,
                    user_id = %session.user_id,
                    session_id = %session_id,
                    reason = ?reason,
                    "Session invalidated"
                );
            }
        }

        // Remove session from Redis
        let _: u32 = conn.del(&session_key).await?;

        Ok(())
    }

    /// Invalidate all sessions for a user
    pub async fn invalidate_user_sessions(
        &self,
        tenant: &TenantContext,
        user_id: Uuid,
        reason: SessionState,
    ) -> Result<u32> {
        let user_sessions_key = self.user_sessions_key(tenant, user_id);
        let mut conn = self.redis.clone();

        // Get all session IDs for the user
        let session_ids: Vec<String> = conn.smembers(&user_sessions_key).await?;
        let mut invalidated_count = 0;

        for session_id in session_ids {
            if self.invalidate_session(tenant, &session_id, reason.clone()).await.is_ok() {
                invalidated_count += 1;
            }
        }

        // Clean up user session index
        let _: u32 = conn.del(&user_sessions_key).await?;

        info!(
            tenant_id = %tenant.tenant_id.0,
            user_id = %user_id,
            invalidated_count = invalidated_count,
            reason = ?reason,
            "All user sessions invalidated"
        );

        Ok(invalidated_count)
    }

    /// Get all active sessions for a user
    pub async fn get_user_sessions(
        &self,
        tenant: &TenantContext,
        user_id: Uuid,
    ) -> Result<Vec<SessionData>> {
        let user_sessions_key = self.user_sessions_key(tenant, user_id);
        let mut conn = self.redis.clone();

        let session_ids: Vec<String> = conn.smembers(&user_sessions_key).await?;
        let mut sessions = Vec::new();

        for session_id in session_ids {
            if let Some(session) = self.get_session(tenant, &session_id).await? {
                if session.state == SessionState::Active {
                    sessions.push(session);
                }
            }
        }

        Ok(sessions)
    }

    /// Clean up expired sessions (should be run periodically)
    pub async fn cleanup_expired_sessions(&self, tenant: &TenantContext) -> Result<u32> {
        let pattern = format!("session:{}:*", tenant.tenant_id.0);
        let mut conn = self.redis.clone();

        // Use SCAN instead of KEYS to avoid blocking Redis
        let mut session_keys = Vec::new();
        let mut cursor = 0;
        const SCAN_BATCH_SIZE: usize = 100;

        loop {
            let (new_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(&pattern)
                .arg("COUNT")
                .arg(SCAN_BATCH_SIZE)
                .query_async(&mut conn)
                .await?;

            session_keys.extend(keys);
            cursor = new_cursor;
            
            if cursor == 0 {
                break;
            }
        }

        let mut cleaned_up = 0;

        for session_key in session_keys {
            if let Ok(Some(data)) = conn.get::<&str, Option<String>>(&session_key).await {
                if let Ok(session) = serde_json::from_str::<SessionData>(&data) {
                    if !self.is_session_valid(&session) {
                        // Remove expired session
                        let _: u32 = conn.del(&session_key).await?;

                        // Remove from user session index
                        let user_sessions_key = self.user_sessions_key(tenant, session.user_id);
                        let _: u32 = conn.srem(&user_sessions_key, &session.session_id).await?;

                        cleaned_up += 1;
                    }
                }
            }
        }

        if cleaned_up > 0 {
            info!(
                tenant_id = %tenant.tenant_id.0,
                cleaned_up_count = cleaned_up,
                "Cleaned up expired sessions"
            );
        }

        Ok(cleaned_up)
    }

    /// Get session statistics for a tenant
    pub async fn get_session_stats(&self, tenant: &TenantContext) -> Result<SessionStats> {
        let pattern = format!("session:{}:*", tenant.tenant_id.0);
        let mut conn = self.redis.clone();

        // Use SCAN instead of KEYS to avoid blocking Redis
        let session_keys = self.scan_keys(&mut conn, &pattern).await?;
        let mut stats = SessionStats::default();

        for session_key in session_keys {
            if let Ok(Some(data)) = conn.get::<&str, Option<String>>(&session_key).await {
                if let Ok(session) = serde_json::from_str::<SessionData>(&data) {
                    stats.total_sessions += 1;

                    match session.state {
                        SessionState::Active => {
                            if self.is_session_valid(&session) {
                                stats.active_sessions += 1;
                            } else {
                                stats.expired_sessions += 1;
                            }
                        }
                        SessionState::Expired => stats.expired_sessions += 1,
                        SessionState::LoggedOut => stats.logged_out_sessions += 1,
                        SessionState::Revoked => stats.revoked_sessions += 1,
                        SessionState::Suspended => stats.suspended_sessions += 1,
                    }
                }
            }
        }

        Ok(stats)
    }

    // Private helper methods

    /// Non-blocking scan for Redis keys matching a pattern
    async fn scan_keys(&self, conn: &mut redis::aio::ConnectionManager, pattern: &str) -> Result<Vec<String>> {
        use redis::{AsyncCommands, Cmd};

        let mut cursor: u64 = 0;
        let mut keys = Vec::new();

        loop {
            let mut cmd = Cmd::new();
            cmd.arg("SCAN").arg(cursor).arg("MATCH").arg(pattern).arg("COUNT").arg(100);

            let result: Vec<redis::Value> = cmd.query_async(conn).await
                .map_err(|e| Error::internal(format!("Redis SCAN failed: {}", e)))?;

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

    fn session_key(&self, tenant: &TenantContext, session_id: &str) -> String {
        format!("session:{}:{}", tenant.tenant_id.0, session_id)
    }

    fn user_sessions_key(&self, tenant: &TenantContext, user_id: Uuid) -> String {
        format!("user_sessions:{}:{}", tenant.tenant_id.0, user_id)
    }

    async fn store_session(&self, session: &SessionData) -> Result<()> {
        let tenant_context = TenantContext {
            tenant_id: crate::TenantId(session.tenant_id),
            schema_name: format!("tenant_{}", session.tenant_id),
        };
        let session_key = self.session_key(&tenant_context, &session.session_id);
        let mut conn = self.redis.clone();

        let serialized = serde_json::to_string(session)
            .map_err(|e| Error::new(ErrorCode::SerializationError, e.to_string()))?;

        // Calculate TTL based on absolute timeout
        let ttl = session.expires_at.signed_duration_since(Utc::now()).num_seconds().max(1) as u64;

        conn.set_ex::<_, _, ()>(&session_key, serialized, ttl).await?;
        
        debug!("Stored session: {} with TTL: {}s", session.session_id, ttl);
        Ok(())
    }

    async fn add_to_user_sessions(
        &self,
        tenant: &TenantContext,
        user_id: Uuid,
        session_id: &str,
    ) -> Result<()> {
        let user_sessions_key = self.user_sessions_key(tenant, user_id);
        let mut conn = self.redis.clone();

        let _: u32 = conn.sadd(&user_sessions_key, session_id).await?;
        
        // Set TTL for user sessions index
        let ttl = self.config.absolute_timeout.num_seconds().max(1) as i64;
        let _: u32 = conn.expire(&user_sessions_key, ttl).await?;

        Ok(())
    }

    async fn remove_from_user_sessions(
        &self,
        tenant: &TenantContext,
        user_id: Uuid,
        session_id: &str,
    ) -> Result<()> {
        let user_sessions_key = self.user_sessions_key(tenant, user_id);
        let mut conn = self.redis.clone();

        let _: u32 = conn.srem(&user_sessions_key, session_id).await?;
        Ok(())
    }

    async fn enforce_session_limit(&self, tenant: &TenantContext, user_id: Uuid) -> Result<()> {
        let user_sessions = self.get_user_sessions(tenant, user_id).await?;

        if user_sessions.len() >= self.config.max_sessions_per_user as usize {
            // Remove oldest sessions to make room for new one
            let mut sessions_to_remove = user_sessions;
            sessions_to_remove.sort_by(|a, b| a.created_at.cmp(&b.created_at));

            let excess_count = sessions_to_remove.len() - self.config.max_sessions_per_user as usize + 1;

            for session in sessions_to_remove.iter().take(excess_count) {
                warn!(
                    tenant_id = %tenant.tenant_id.0,
                    user_id = %user_id,
                    session_id = %session.session_id,
                    "Removing session due to session limit exceeded"
                );

                self.invalidate_session(tenant, &session.session_id, SessionState::Revoked)
                    .await?;
            }
        }

        Ok(())
    }

    fn is_session_valid(&self, session: &SessionData) -> bool {
        let now = Utc::now();

        // Check if session is in valid state
        if session.state != SessionState::Active {
            return false;
        }

        // Check absolute timeout
        if now > session.expires_at {
            return false;
        }

        // Check inactivity timeout if sliding window is enabled
        if self.config.enable_sliding_window {
            let inactivity_limit = session.last_activity + self.config.inactivity_timeout;
            if now > inactivity_limit {
                return false;
            }
        }

        true
    }
}

/// Session statistics
#[derive(Debug, Default, Clone, Serialize)]
pub struct SessionStats {
    pub total_sessions: u32,
    pub active_sessions: u32,
    pub expired_sessions: u32,
    pub logged_out_sessions: u32,
    pub revoked_sessions: u32,
    pub suspended_sessions: u32,
}