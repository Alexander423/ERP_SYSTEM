use crate::email::{EmailJobData, VerificationEmailTemplate, WelcomeEmailTemplate};
use crate::models::User;
use crate::repository::UserRepository;
use crate::tokens::{TokenManager, TokenPurpose};
use erp_core::{
    audit::{AuditEvent, AuditLogger, event::EventOutcome, EventSeverity, EventType},
    error::{Error, ErrorCode, Result},
    jobs::JobQueue,
    DatabasePool, TenantContext,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Configuration for email verification workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailVerificationConfig {
    /// Token expiry in hours (default: 24 hours)
    pub token_expiry_hours: u32,
    /// Maximum verification requests per hour per user
    pub max_requests_per_hour: u32,
    /// Company name for email templates
    pub company_name: String,
    /// Base URL for verification links
    pub base_url: String,
    /// Send welcome email after verification
    pub send_welcome_email: bool,
}

impl Default for EmailVerificationConfig {
    fn default() -> Self {
        Self {
            token_expiry_hours: 24,
            max_requests_per_hour: 5,
            company_name: "ERP System".to_string(),
            base_url: "https://localhost:3000".to_string(),
            send_welcome_email: true,
        }
    }
}

/// Request data for email verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailVerificationRequest {
    pub user_id: Uuid,
    pub client_ip: Option<String>,
}

/// Request data for email verification confirmation  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailVerificationConfirmation {
    pub token: String,
    pub client_ip: Option<String>,
}

/// Email verification workflow service
pub struct EmailVerificationWorkflow {
    config: EmailVerificationConfig,
    token_manager: Arc<TokenManager>,
    user_repository: Arc<UserRepository>,
    job_queue: Arc<dyn JobQueue>,
    audit_logger: Option<AuditLogger>,
    db: DatabasePool,
}

impl EmailVerificationWorkflow {
    pub fn new(
        config: EmailVerificationConfig,
        token_manager: Arc<TokenManager>,
        user_repository: Arc<UserRepository>,
        job_queue: Arc<dyn JobQueue>,
        audit_logger: Option<AuditLogger>,
        db: DatabasePool,
    ) -> Self {
        Self {
            config,
            token_manager,
            user_repository,
            job_queue,
            audit_logger,
            db,
        }
    }

    /// Send email verification to user
    pub async fn send_verification_email(
        &self,
        tenant: &TenantContext,
        request: EmailVerificationRequest,
    ) -> Result<()> {
        info!(
            tenant_id = %tenant.tenant_id.0,
            user_id = %request.user_id,
            client_ip = ?request.client_ip,
            "Processing email verification request"
        );

        // Get user
        let user = self.user_repository.find_by_id(tenant, request.user_id).await?
            .ok_or_else(|| Error::new(ErrorCode::ResourceNotFound, "User not found"))?;

        // Check if user is already verified
        if user.email_verified_at.is_some() {
            warn!(
                user_id = %user.id,
                email = %user.email,
                "Email verification requested for already verified user"
            );

            // Log this for security monitoring
            if let Some(audit_logger) = &self.audit_logger {
                audit_logger.log_event(
                    AuditEvent::builder(
                        EventType::SecurityPolicyViolation,
                        "Email verification requested for verified user"
                    )
                    .severity(EventSeverity::Warning)
                    .outcome(EventOutcome::Failure)
                    .resource("user", &user.id.to_string())
                    .metadata("email".to_string(), serde_json::Value::String(user.email.clone()))
                    .build()
                ).await?;
            }

            return Err(Error::new(
                ErrorCode::InvalidInput,
                "User email is already verified"
            ));
        }

        // Check rate limiting
        self.check_rate_limit(tenant, request.user_id).await?;

        // Send verification email
        self.send_verification_email_internal(tenant, &user, request.client_ip.clone()).await?;

        // Log successful request
        if let Some(audit_logger) = &self.audit_logger {
            audit_logger.log_event(
                AuditEvent::builder(
                    EventType::Custom("EMAIL_VERIFICATION_SENT".to_string()),
                    "Email verification sent"
                )
                .severity(EventSeverity::Info)
                .outcome(EventOutcome::Success)
                .resource("user", &user.id.to_string())
                .metadata("email".to_string(), serde_json::Value::String(user.email.clone()))
                .metadata("client_ip".to_string(), 
                    serde_json::Value::String(request.client_ip.clone().unwrap_or_default()))
                .build()
            ).await?;
        }

        info!(
            user_id = %user.id,
            email = %user.email,
            "Email verification sent successfully"
        );

        Ok(())
    }

    /// Complete email verification with token
    pub async fn verify_email(
        &self,
        tenant: &TenantContext,
        confirmation: EmailVerificationConfirmation,
    ) -> Result<User> {
        info!(
            tenant_id = %tenant.tenant_id.0,
            "Processing email verification confirmation"
        );

        // Validate and consume token
        let token_data = self.token_manager.validate_token(
            tenant,
            &confirmation.token,
            TokenPurpose::EmailVerification,
            confirmation.client_ip.clone(),
        ).await?;

        // Get user
        let user = self.user_repository.find_by_id(tenant, token_data.user_id).await?
            .ok_or_else(|| Error::new(ErrorCode::ResourceNotFound, "User not found"))?;

        // Check if already verified
        if user.email_verified_at.is_some() {
            warn!(
                user_id = %user.id,
                "Attempted verification of already verified user"
            );

            return Err(Error::new(
                ErrorCode::InvalidInput,
                "Email is already verified"
            ));
        }

        // Mark user as verified
        let updated_user = self.user_repository.mark_email_verified(tenant, user.id).await?;

        // Invalidate all existing verification tokens for this user
        let invalidated_count = self.token_manager.invalidate_user_tokens(
            tenant,
            user.id,
            TokenPurpose::EmailVerification,
        ).await?;

        // Send welcome email if configured
        if self.config.send_welcome_email {
            if let Err(e) = self.send_welcome_email(tenant, &updated_user).await {
                // Don't fail the verification if welcome email fails
                warn!(
                    user_id = %updated_user.id,
                    error = %e,
                    "Failed to send welcome email, but verification completed"
                );
            }
        }

        // Log successful verification
        if let Some(audit_logger) = &self.audit_logger {
            audit_logger.log_event(
                AuditEvent::builder(
                    EventType::Custom("EMAIL_VERIFIED".to_string()),
                    "Email verification completed successfully"
                )
                .severity(EventSeverity::Info)
                .outcome(EventOutcome::Success)
                .resource("user", &updated_user.id.to_string())
                .metadata("email".to_string(), serde_json::Value::String(updated_user.email.clone()))
                .metadata("invalidated_tokens".to_string(), 
                    serde_json::Value::Number(invalidated_count.into()))
                .metadata("client_ip".to_string(), 
                    serde_json::Value::String(confirmation.client_ip.unwrap_or_default()))
                .build()
            ).await?;
        }

        info!(
            user_id = %updated_user.id,
            email = %updated_user.email,
            invalidated_tokens = invalidated_count,
            "Email verification completed successfully"
        );

        Ok(updated_user)
    }

    /// Resend verification email (rate limited)
    pub async fn resend_verification_email(
        &self,
        tenant: &TenantContext,
        request: EmailVerificationRequest,
    ) -> Result<()> {
        info!(
            tenant_id = %tenant.tenant_id.0,
            user_id = %request.user_id,
            "Processing resend verification email request"
        );

        // This uses the same logic as initial send but with additional logging
        if let Some(audit_logger) = &self.audit_logger {
            audit_logger.log_event(
                AuditEvent::builder(
                    EventType::Custom("EMAIL_VERIFICATION_RESEND".to_string()),
                    "Email verification resend requested"
                )
                .severity(EventSeverity::Info)
                .outcome(EventOutcome::Success)
                .resource("user", &request.user_id.to_string())
                .build()
            ).await?;
        }

        self.send_verification_email(tenant, request).await
    }

    /// Check if a verification token is valid (without consuming it)
    pub async fn validate_verification_token(
        &self,
        tenant: &TenantContext,
        token: &str,
    ) -> Result<bool> {
        match self.token_manager.get_token(tenant, token, TokenPurpose::EmailVerification).await? {
            Some(token_data) => Ok(token_data.is_valid()),
            None => Ok(false),
        }
    }

    /// Get user information associated with a verification token
    pub async fn get_token_user_info(
        &self,
        tenant: &TenantContext,
        token: &str,
    ) -> Result<Option<TokenUserInfo>> {
        let token_data = match self.token_manager.get_token(tenant, token, TokenPurpose::EmailVerification).await? {
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

    async fn check_rate_limit(&self, tenant: &TenantContext, user_id: Uuid) -> Result<()> {
        let _pool = self.db.get_tenant_pool(tenant).await?;

        // TODO: Re-enable once sqlx query cache is fixed
        /*
        let recent_requests = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM verification_tokens
            WHERE tenant_id = $1
              AND user_id = $2
              AND purpose = 'email_verification'
              AND created_at > NOW() - INTERVAL '1 hour'
            "#,
            tenant.tenant_id.0,
            user_id
        )
        .fetch_one(pool.get())
        .await?;

        let count = recent_requests.count.unwrap_or(0) as u32;
        */
        let count = 0u32; // Temporary placeholder
        if count >= self.config.max_requests_per_hour {
            warn!(
                tenant_id = %tenant.tenant_id.0,
                user_id = %user_id,
                count = count,
                limit = self.config.max_requests_per_hour,
                "Email verification rate limit exceeded"
            );

            // Log security event
            if let Some(audit_logger) = &self.audit_logger {
                audit_logger.log_event(
                    AuditEvent::builder(
                        EventType::SecurityPolicyViolation,
                        "Email verification rate limit exceeded"
                    )
                    .severity(EventSeverity::Warning)
                    .outcome(EventOutcome::Failure)
                    .resource("user", &user_id.to_string())
                    .metadata("request_count".to_string(), serde_json::Value::Number(count.into()))
                    .metadata("limit".to_string(), serde_json::Value::Number(self.config.max_requests_per_hour.into()))
                    .build()
                ).await?;
            }

            return Err(Error::new(
                ErrorCode::RateLimitExceeded,
                "Too many verification email requests. Please try again later."
            ));
        }

        Ok(())
    }

    async fn send_verification_email_internal(
        &self,
        tenant: &TenantContext,
        user: &User,
        client_ip: Option<String>,
    ) -> Result<()> {
        // Create verification token
        let token_data = self.token_manager.create_token(
            tenant,
            TokenPurpose::EmailVerification,
            user.id,
            Some(user.email.clone()),
            Some(self.config.token_expiry_hours),
            client_ip,
            None,
        ).await?;

        // Create verification URL
        let verification_url = format!("{}/auth/verify-email?token={}", 
            self.config.base_url, token_data.token);

        // Create email template
        let email_template = VerificationEmailTemplate {
            user_name: format!("{} {}", user.first_name.clone().unwrap_or_default(), user.last_name.clone().unwrap_or_default()),
            company_name: self.config.company_name.clone(),
            verification_url,
            expires_in_hours: self.config.token_expiry_hours,
        };

        // Create email job
        let email_job = EmailJobData::from_template(
            &user.email,
            &email_template,
            Some(tenant.tenant_id.0.to_string()),
            Some(user.id.to_string()),
        ).with_metadata("workflow".to_string(), serde_json::Value::String("email_verification".to_string()));

        // Create a proper queued job from the serializable job
        let queued_job = erp_core::jobs::types::QueuedJob::new(&email_job)?;
        self.job_queue.enqueue(queued_job).await?;

        debug!(
            user_id = %user.id,
            email = %user.email,
            token_expires = %token_data.expires_at,
            "Email verification queued"
        );

        Ok(())
    }

    async fn send_welcome_email(&self, tenant: &TenantContext, user: &User) -> Result<()> {
        // Create login URL
        let login_url = format!("{}/auth/login", self.config.base_url);

        // Create welcome email template
        let email_template = WelcomeEmailTemplate {
            user_name: format!("{} {}", user.first_name.clone().unwrap_or_default(), user.last_name.clone().unwrap_or_default()),
            company_name: self.config.company_name.clone(),
            login_url,
        };

        // Create email job
        let email_job = EmailJobData::from_template(
            &user.email,
            &email_template,
            Some(tenant.tenant_id.0.to_string()),
            Some(user.id.to_string()),
        ).with_metadata("workflow".to_string(), serde_json::Value::String("welcome".to_string()));

        // Create a proper queued job from the serializable job
        let queued_job = erp_core::jobs::types::QueuedJob::new(&email_job)?;
        self.job_queue.enqueue(queued_job).await?;

        debug!(
            user_id = %user.id,
            email = %user.email,
            "Welcome email queued"
        );

        Ok(())
    }
}

/// User information associated with a verification token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUserInfo {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_email_verification_config_defaults() {
        let config = EmailVerificationConfig::default();
        assert_eq!(config.token_expiry_hours, 24);
        assert_eq!(config.max_requests_per_hour, 5);
        assert!(config.send_welcome_email);
    }

    /// Mock job queue for email verification testing - tracks queued jobs in memory
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

        /// Get all queued email jobs for testing verification
        pub fn get_queued_jobs(&self) -> Vec<erp_core::jobs::types::QueuedJob> {
            self.queued_jobs.lock().unwrap().clone()
        }

        /// Get only verification email jobs
        pub fn get_verification_emails(&self) -> Vec<erp_core::jobs::types::QueuedJob> {
            self.queued_jobs.lock().unwrap()
                .iter()
                .filter(|job| job.data.to_string().contains("verification"))
                .cloned()
                .collect()
        }

        /// Get only welcome email jobs
        pub fn get_welcome_emails(&self) -> Vec<erp_core::jobs::types::QueuedJob> {
            self.queued_jobs.lock().unwrap()
                .iter()
                .filter(|job| job.data.to_string().contains("welcome"))
                .cloned()
                .collect()
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

            Ok(JobId(format!("mock-email-job-{}", job_num)))
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