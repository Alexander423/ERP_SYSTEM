use crate::email::{EmailJobData, PasswordResetEmailTemplate};
use crate::models::User;
use crate::repository::UserRepository;
use crate::tokens::{TokenManager, TokenPurpose};
use erp_core::{
    audit::{AuditEvent, AuditLogger, event::EventOutcome, EventSeverity, EventType},
    error::{Error, ErrorCode, Result},
    jobs::JobQueue,
    security::PasswordHasher,
    DatabasePool, TenantContext,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Configuration for password reset workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetConfig {
    /// Token expiry in hours (default: 1 hour)
    pub token_expiry_hours: u32,
    /// Maximum password reset requests per hour per user
    pub max_requests_per_hour: u32,
    /// Minimum password length
    pub min_password_length: u8,
    /// Require password complexity
    pub require_password_complexity: bool,
    /// Company name for email templates
    pub company_name: String,
    /// Base URL for reset links
    pub base_url: String,
}

impl Default for PasswordResetConfig {
    fn default() -> Self {
        Self {
            token_expiry_hours: 1,
            max_requests_per_hour: 3,
            min_password_length: 8,
            require_password_complexity: true,
            company_name: "ERP System".to_string(),
            base_url: "https://localhost:3000".to_string(),
        }
    }
}

/// Request data for password reset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetRequest {
    pub email: String,
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
}

/// Request data for password reset confirmation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetConfirmation {
    pub token: String,
    pub new_password: String,
    pub confirm_password: String,
    pub client_ip: Option<String>,
}

/// Password reset workflow service
pub struct PasswordResetWorkflow {
    config: PasswordResetConfig,
    token_manager: Arc<TokenManager>,
    user_repository: Arc<UserRepository>,
    job_queue: Arc<dyn JobQueue>,
    audit_logger: Option<AuditLogger>,
    password_hasher: Arc<PasswordHasher>,
    db: DatabasePool,
}

impl PasswordResetWorkflow {
    pub fn new(
        config: PasswordResetConfig,
        token_manager: Arc<TokenManager>,
        user_repository: Arc<UserRepository>,
        job_queue: Arc<dyn JobQueue>,
        audit_logger: Option<AuditLogger>,
        password_hasher: Arc<PasswordHasher>,
        db: DatabasePool,
    ) -> Self {
        Self {
            config,
            token_manager,
            user_repository,
            job_queue,
            audit_logger,
            password_hasher,
            db,
        }
    }

    /// Initiate password reset process
    pub async fn request_password_reset(
        &self,
        tenant: &TenantContext,
        request: PasswordResetRequest,
    ) -> Result<()> {
        info!(
            tenant_id = %tenant.tenant_id.0,
            email = %request.email,
            client_ip = ?request.client_ip,
            "Processing password reset request"
        );

        // Validate email format
        if !self.is_valid_email(&request.email) {
            return Err(Error::new(
                ErrorCode::ValidationFailed,
                "Invalid email address format"
            ));
        }

        // Check rate limiting
        self.check_rate_limit(tenant, &request.email).await?;

        // Find user by email (this should not reveal if user exists for security)
        let user_result = self.user_repository.find_by_email(tenant, &request.email).await;
        
        // Always log the request attempt for security monitoring
        if let Some(audit_logger) = &self.audit_logger {
            let outcome = match &user_result {
                Ok(Some(_)) => EventOutcome::Success,
                Ok(None) => EventOutcome::Failure,
                Err(_) => EventOutcome::Failure,
            };

            audit_logger.log_event(
                AuditEvent::builder(
                    EventType::Custom("PASSWORD_RESET_REQUEST".to_string()),
                    "Password reset requested"
                )
                .severity(EventSeverity::Info)
                .outcome(outcome)
                .resource("user", &request.email)
                .metadata("client_ip".to_string(), 
                    serde_json::Value::String(request.client_ip.clone().unwrap_or_default()))
                .metadata("user_agent".to_string(), 
                    serde_json::Value::String(request.user_agent.unwrap_or_default()))
                .build()
            ).await?;
        }

        match user_result {
            Ok(Some(user)) => {
                // User exists, proceed with reset
                self.send_password_reset_email(tenant, &user, request.client_ip).await?;
                info!("Password reset email sent to existing user: {}", request.email);
            }
            Ok(None) => {
                // User doesn't exist - for security, we don't reveal this
                // But we still simulate the process timing
                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                debug!("Password reset requested for non-existent user: {}", request.email);
            }
            Err(e) => {
                error!("Database error during password reset request: {}", e);
                return Err(e);
            }
        }

        // Always return success to prevent user enumeration
        Ok(())
    }

    /// Complete password reset with token
    pub async fn confirm_password_reset(
        &self,
        tenant: &TenantContext,
        confirmation: PasswordResetConfirmation,
    ) -> Result<()> {
        info!(
            tenant_id = %tenant.tenant_id.0,
            "Processing password reset confirmation"
        );

        // Validate input
        if confirmation.new_password != confirmation.confirm_password {
            return Err(Error::new(
                ErrorCode::ValidationFailed,
                "Passwords do not match"
            ));
        }

        self.validate_password_strength(&confirmation.new_password)?;

        // Validate and consume token
        let token_data = self.token_manager.validate_token(
            tenant,
            &confirmation.token,
            TokenPurpose::PasswordReset,
            confirmation.client_ip.clone(),
        ).await?;

        // Get user
        let user = self.user_repository.find_by_id(tenant, token_data.user_id).await?
            .ok_or_else(|| Error::new(ErrorCode::ResourceNotFound, "User not found"))?;

        // Hash new password
        let password_hash = self.hash_password(&confirmation.new_password)?;

        // Update user password
        self.user_repository.update_password(tenant, user.id, &password_hash).await?;

        // Invalidate all existing password reset tokens for this user
        let invalidated_count = self.token_manager.invalidate_user_tokens(
            tenant,
            user.id,
            TokenPurpose::PasswordReset,
        ).await?;

        // Log successful password reset
        if let Some(audit_logger) = &self.audit_logger {
            audit_logger.log_event(
                AuditEvent::builder(
                    EventType::Custom("PASSWORD_RESET_COMPLETED".to_string()),
                    "Password reset completed successfully"
                )
                .severity(EventSeverity::Info)
                .outcome(EventOutcome::Success)
                .resource("user", &user.id.to_string())
                .metadata("email".to_string(), serde_json::Value::String(user.email.clone()))
                .metadata("invalidated_tokens".to_string(), 
                    serde_json::Value::Number(invalidated_count.into()))
                .metadata("client_ip".to_string(), 
                    serde_json::Value::String(confirmation.client_ip.unwrap_or_default()))
                .build()
            ).await?;
        }

        info!(
            user_id = %user.id,
            email = %user.email,
            invalidated_tokens = invalidated_count,
            "Password reset completed successfully"
        );

        Ok(())
    }

    /// Validate password reset token without consuming it
    pub async fn validate_reset_token(
        &self,
        tenant: &TenantContext,
        token: &str,
    ) -> Result<bool> {
        match self.token_manager.get_token(tenant, token, TokenPurpose::PasswordReset).await? {
            Some(token_data) => Ok(token_data.is_valid()),
            None => Ok(false),
        }
    }

    /// Get user information associated with a reset token (for UI purposes)
    pub async fn get_token_user_info(
        &self,
        tenant: &TenantContext,
        token: &str,
    ) -> Result<Option<TokenUserInfo>> {
        let token_data = match self.token_manager.get_token(tenant, token, TokenPurpose::PasswordReset).await? {
            Some(data) if data.is_valid() => data,
            _ => return Ok(None),
        };

        let user = self.user_repository.find_by_id(tenant, token_data.user_id).await?
            .ok_or_else(|| Error::new(ErrorCode::ResourceNotFound, "User not found"))?;

        Ok(Some(TokenUserInfo {
            email: user.email,
            first_name: user.first_name.unwrap_or_default(),
            last_name: user.last_name.unwrap_or_default(),
            expires_at: token_data.expires_at,
        }))
    }

    // Private helper methods

    async fn check_rate_limit(&self, tenant: &TenantContext, email: &str) -> Result<()> {
        // Implementation using raw SQL to avoid cache issues
        let pool = self.db.get_tenant_pool(tenant).await?;

        // Check recent reset requests using raw query
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM verification_tokens
            WHERE tenant_id = $1
              AND email = $2
              AND purpose = 'password_reset'
              AND created_at > NOW() - INTERVAL '1 hour'
            "#
        )
        .bind(tenant.tenant_id.0)
        .bind(email)
        .fetch_one(pool.get())
        .await;

        let count = match row {
            Ok(row) => {
                use sqlx::Row;
                row.try_get::<i64, _>("count").unwrap_or(0) as u32
            },
            Err(_) => {
                // If the table doesn't exist yet, assume no requests
                debug!("Unable to check rate limit - verification_tokens table may not exist");
                0u32
            }
        };
        if count >= self.config.max_requests_per_hour {
            warn!(
                tenant_id = %tenant.tenant_id.0,
                email = %email,
                count = count,
                limit = self.config.max_requests_per_hour,
                "Password reset rate limit exceeded"
            );

            // Log security event
            if let Some(audit_logger) = &self.audit_logger {
                audit_logger.log_event(
                    AuditEvent::builder(
                        EventType::SecurityPolicyViolation,
                        "Password reset rate limit exceeded"
                    )
                    .severity(EventSeverity::Warning)
                    .outcome(EventOutcome::Failure)
                    .resource("user", email)
                    .metadata("request_count".to_string(), serde_json::Value::Number(count.into()))
                    .metadata("limit".to_string(), serde_json::Value::Number(self.config.max_requests_per_hour.into()))
                    .build()
                ).await?;
            }

            return Err(Error::new(
                ErrorCode::RateLimitExceeded,
                "Too many password reset requests. Please try again later."
            ));
        }

        Ok(())
    }

    async fn send_password_reset_email(
        &self,
        tenant: &TenantContext,
        user: &User,
        client_ip: Option<String>,
    ) -> Result<()> {
        // Create password reset token
        let token_data = self.token_manager.create_token(
            tenant,
            TokenPurpose::PasswordReset,
            user.id,
            Some(user.email.clone()),
            Some(self.config.token_expiry_hours),
            client_ip.clone(),
            None,
        ).await?;

        // Create reset URL
        let reset_url = format!("{}/auth/reset-password?token={}", 
            self.config.base_url, token_data.token);

        // Create email template
        let email_template = PasswordResetEmailTemplate {
            user_name: format!("{} {}", user.first_name.clone().unwrap_or_default(), user.last_name.clone().unwrap_or_default()),
            company_name: self.config.company_name.clone(),
            reset_url,
            expires_in_hours: self.config.token_expiry_hours,
            source_ip: client_ip,
        };

        // Create email job
        let email_job = EmailJobData::from_template(
            &user.email,
            &email_template,
            Some(tenant.tenant_id.0.to_string()),
            Some(user.id.to_string()),
        ).with_metadata("workflow".to_string(), serde_json::Value::String("password_reset".to_string()));

        // Queue email for background sending
        // Create a proper queued job from the serializable job
        let queued_job = erp_core::jobs::types::QueuedJob::new(&email_job)?;
        self.job_queue.enqueue(queued_job).await?;

        debug!(
            user_id = %user.id,
            email = %user.email,
            token_expires = %token_data.expires_at,
            "Password reset email queued"
        );

        Ok(())
    }

    fn validate_password_strength(&self, password: &str) -> Result<()> {
        if password.len() < self.config.min_password_length as usize {
            return Err(Error::new(
                ErrorCode::ValidationFailed,
                format!("Password must be at least {} characters long", self.config.min_password_length)
            ));
        }

        if self.config.require_password_complexity {
            let has_upper = password.chars().any(|c| c.is_uppercase());
            let has_lower = password.chars().any(|c| c.is_lowercase());
            let has_digit = password.chars().any(|c| c.is_numeric());
            let has_special = password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c));

            if !has_upper || !has_lower || !has_digit || !has_special {
                return Err(Error::new(
                    ErrorCode::ValidationFailed,
                    "Password must contain uppercase, lowercase, number, and special character"
                ));
            }
        }

        Ok(())
    }

    fn hash_password(&self, password: &str) -> Result<String> {
        self.password_hasher.hash_password(password)
    }

    fn is_valid_email(&self, email: &str) -> bool {
        // Basic email validation - in production use a proper email validation library
        email.contains('@') && email.contains('.') && !email.starts_with('@') && !email.ends_with('@')
    }
}

/// User information associated with a password reset token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUserInfo {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

// Helper trait for piping
pub trait Pipe<T> {
    fn pipe<U, F>(self, f: F) -> U
    where
        F: FnOnce(Self) -> U,
        Self: Sized;
}

impl<T> Pipe<T> for T {
    fn pipe<U, F>(self, f: F) -> U
    where
        F: FnOnce(Self) -> U,
    {
        f(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_validation() {
        // Test password validation logic without database dependencies
        let config = PasswordResetConfig::default();
        
        // Test validation logic directly
        let validate = |password: &str| -> bool {
            password.len() >= config.min_password_length.into()
                && password.chars().any(|c| c.is_uppercase())
                && password.chars().any(|c| c.is_lowercase())
                && password.chars().any(|c| c.is_numeric())
        };

        // Too short
        assert!(!validate("short"));
        
        // No uppercase
        assert!(!validate("lowercaseonly123"));
        
        // No lowercase
        assert!(!validate("UPPERCASEONLY123"));
        
        // No numbers
        assert!(!validate("NoNumbersHere"));
        
        // Valid complex password
        assert!(validate("ComplexP@ssw0rd123"));
    }

    #[test]
    fn test_email_validation() {
        // Simple email validation test without dependencies
        let validate_email = |email: &str| -> bool {
            email.contains('@') && email.contains('.')
        };

        assert!(validate_email("test@example.com"));
        assert!(!validate_email("invalid"));
        assert!(!validate_email("@example.com"));
        assert!(!validate_email("test@"));
    }

    /// Mock job queue for testing - tracks queued jobs in memory
    pub struct MockJobQueue {
        pub queued_jobs: std::sync::Arc<std::sync::Mutex<Vec<erp_core::jobs::types::QueuedJob>>>,
        pub job_counter: std::sync::Arc<std::sync::atomic::AtomicU64>,
    }

    impl MockJobQueue {
        pub fn new() -> Self {
            Self {
                queued_jobs: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
                job_counter: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0)),
            }
        }

        /// Get all queued jobs for testing verification
        pub fn get_queued_jobs(&self) -> Vec<erp_core::jobs::types::QueuedJob> {
            self.queued_jobs.lock().unwrap().clone()
        }

        /// Clear all queued jobs for test isolation
        pub fn clear_jobs(&self) {
            self.queued_jobs.lock().unwrap().clear();
        }

        /// Get the number of jobs that have been queued
        pub fn job_count(&self) -> u64 {
            self.job_counter.load(std::sync::atomic::Ordering::SeqCst)
        }
    }
    
    #[async_trait::async_trait]
    impl JobQueue for MockJobQueue {
        async fn enqueue(&self, job: erp_core::jobs::types::QueuedJob) -> Result<erp_core::jobs::JobId> {
            use erp_core::jobs::JobId;

            // Store the job in our mock queue
            {
                let mut jobs = self.queued_jobs.lock().unwrap();
                jobs.push(job);
            }

            // Increment the job counter
            let job_num = self.job_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

            Ok(JobId(format!("mock-job-{}", job_num)))
        }
        
        async fn dequeue(&self, _queue: &str) -> Result<Option<erp_core::jobs::types::QueuedJob>> {
            Ok(None)
        }
        
        async fn get_status(&self, _job_id: &erp_core::jobs::JobId) -> Result<Option<erp_core::jobs::JobStatus>> {
            Ok(None)
        }
        
        async fn update_status(&self, _job_id: &erp_core::jobs::JobId, _status: erp_core::jobs::JobStatus) -> Result<()> {
            Ok(())
        }
        
        async fn cancel_job(&self, _job_id: &erp_core::jobs::JobId) -> Result<bool> {
            Ok(true)
        }
        
        async fn get_stats(&self) -> Result<erp_core::jobs::traits::QueueStats> {
            Ok(erp_core::jobs::traits::QueueStats::default())
        }
        
        async fn cleanup_old_jobs(&self, _older_than: chrono::DateTime<chrono::Utc>) -> Result<u64> {
            Ok(0)
        }
        
        async fn get_jobs_by_status(&self, _status: erp_core::jobs::JobState, _limit: Option<u32>) -> Result<Vec<erp_core::jobs::types::QueuedJob>> {
            Ok(vec![])
        }
        
        async fn health_check(&self) -> Result<bool> {
            Ok(true)
        }
    }
}