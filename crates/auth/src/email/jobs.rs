use crate::email::{EmailService, EmailTemplate};
use erp_core::{
    audit::{AuditEvent, AuditLogger, EventType, EventSeverity, event::EventOutcome},
    jobs::{Job, traits::JobContext, JobResult, SerializableJob},
    Error, ErrorCode,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, info, warn};

/// Email job data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailJobData {
    /// Recipient email address
    pub to: String,
    /// Email subject
    pub subject: String,
    /// HTML body
    pub html_body: String,
    /// Text body (fallback)
    pub text_body: String,
    /// Template name for logging
    pub template_name: String,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Retry configuration
    pub max_retries: Option<u32>,
    /// Tenant ID for auditing
    pub tenant_id: Option<String>,
    /// User ID for auditing
    pub user_id: Option<String>,
}

impl EmailJobData {
    pub fn from_template(
        to: impl Into<String>,
        template: &dyn EmailTemplate,
        tenant_id: Option<String>,
        user_id: Option<String>,
    ) -> Self {
        Self {
            to: to.into(),
            subject: template.subject(),
            html_body: template.html_body(),
            text_body: template.text_body(),
            template_name: template.template_name().to_string(),
            metadata: HashMap::new(),
            max_retries: Some(3),
            tenant_id,
            user_id,
        }
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = Some(retries);
        self
    }
}

/// Background job for sending emails
pub struct EmailJob {
    data: EmailJobData,
    email_service: EmailService,
    audit_logger: Option<AuditLogger>,
}

impl EmailJob {
    pub fn new(
        data: EmailJobData,
        email_service: EmailService,
        audit_logger: Option<AuditLogger>,
    ) -> Self {
        Self {
            data,
            email_service,
            audit_logger,
        }
    }
}

#[async_trait]
impl Job for EmailJob {
    async fn execute(&self, context: &JobContext) -> JobResult {
        info!(
            job_id = %context.job_id,
            template = %self.data.template_name,
            recipient = %self.data.to,
            "Starting email job execution"
        );

        // Set up audit context
        if let Some(audit_logger) = &self.audit_logger {
            if let (Some(tenant_id), Some(user_id)) = (&self.data.tenant_id, &self.data.user_id) {
                audit_logger.set_context(
                    erp_core::audit::logger::AuditContext::new()
                        .with_tenant_id(tenant_id)
                        .with_actor_id(user_id)
                        .with_request_id(context.job_id.to_string())
                ).await;
            }
        }

        // Validate email data
        if self.data.to.is_empty() || self.data.subject.is_empty() {
            let error_msg = "Invalid email data: missing recipient or subject";
            error!(job_id = %context.job_id, error = error_msg);
            
            // Log audit event for failed email
            if let Some(audit_logger) = &self.audit_logger {
                let _ = audit_logger.log_event(
                    AuditEvent::builder(EventType::Custom("EMAIL_SEND_FAILED".to_string()), "Email validation failed")
                        .severity(EventSeverity::Warning)
                        .outcome(EventOutcome::Failure)
                        .metadata("error".to_string(), serde_json::Value::String(error_msg.to_string()))
                        .metadata("template".to_string(), serde_json::Value::String(self.data.template_name.clone()))
                        .metadata("recipient".to_string(), serde_json::Value::String(self.data.to.clone()))
                        .build()
                ).await;
            }

            return JobResult::failed(error_msg);
        }

        // Attempt to send email
        match self.email_service.send_email(
            &self.data.to,
            &self.data.subject,
            &self.data.html_body,
            Some(&self.data.text_body),
        ).await {
            Ok(_) => {
                info!(
                    job_id = %context.job_id,
                    template = %self.data.template_name,
                    recipient = %self.data.to,
                    "Email sent successfully"
                );

                // Log successful email send
                if let Some(audit_logger) = &self.audit_logger {
                    let _ = audit_logger.log_event(
                        AuditEvent::builder(EventType::Custom("EMAIL_SENT".to_string()), "Email sent successfully")
                            .severity(EventSeverity::Info)
                            .outcome(EventOutcome::Success)
                            .metadata("template".to_string(), serde_json::Value::String(self.data.template_name.clone()))
                            .metadata("recipient".to_string(), serde_json::Value::String(self.data.to.clone()))
                            .metadata("subject".to_string(), serde_json::Value::String(self.data.subject.clone()))
                            .build()
                    ).await;
                }

                JobResult::success_with_message("Email sent successfully")
            }
            
            Err(e) => {
                let error_msg = format!("Failed to send email: {}", e);
                
                // Check if this is a retryable error
                let is_retryable = match &e {
                    Error { code: ErrorCode::NetworkTimeout, .. } |
                    Error { code: ErrorCode::NetworkConnectionRefused, .. } |
                    Error { code: ErrorCode::ExternalServiceError, .. } |
                    Error { code: ErrorCode::ServiceUnavailable, .. } => true,
                    _ => false,
                };

                if is_retryable && context.attempt < context.max_attempts {
                    warn!(
                        job_id = %context.job_id,
                        attempt = context.attempt,
                        max_attempts = context.max_attempts,
                        error = %e,
                        "Email send failed, will retry"
                    );

                    // Log retry attempt
                    if let Some(audit_logger) = &self.audit_logger {
                        let _ = audit_logger.log_event(
                            AuditEvent::builder(EventType::Custom("EMAIL_SEND_RETRY".to_string()), "Email send failed, retrying")
                                .severity(EventSeverity::Warning)
                                .outcome(EventOutcome::Partial)
                                .metadata("error".to_string(), serde_json::Value::String(e.to_string()))
                                .metadata("attempt".to_string(), serde_json::Value::Number(context.attempt.into()))
                                .metadata("template".to_string(), serde_json::Value::String(self.data.template_name.clone()))
                                .build()
                        ).await;
                    }

                    // Calculate retry delay (exponential backoff)
                    let delay_seconds = (2_u64.pow(context.attempt).min(300)) * 60; // Minutes, max 5 hours
                    JobResult::retry_with_delay(error_msg, delay_seconds)
                } else {
                    error!(
                        job_id = %context.job_id,
                        attempt = context.attempt,
                        error = %e,
                        "Email send failed permanently"
                    );

                    // Log permanent failure
                    if let Some(audit_logger) = &self.audit_logger {
                        let _ = audit_logger.log_event(
                            AuditEvent::builder(EventType::Custom("EMAIL_SEND_FAILED".to_string()), "Email send failed permanently")
                                .severity(EventSeverity::Critical)
                                .outcome(EventOutcome::Failure)
                                .metadata("error".to_string(), serde_json::Value::String(e.to_string()))
                                .metadata("final_attempt".to_string(), serde_json::Value::Number(context.attempt.into()))
                                .metadata("template".to_string(), serde_json::Value::String(self.data.template_name.clone()))
                                .build()
                        ).await;
                    }

                    JobResult::failed(error_msg)
                }
            }
        }
    }

    fn job_type(&self) -> &'static str {
        "email_send"
    }

    fn priority(&self) -> erp_core::jobs::JobPriority {
        // Email jobs have normal priority by default
        erp_core::jobs::JobPriority::Normal
    }

    fn max_attempts(&self) -> u32 {
        self.data.max_retries.unwrap_or(3)
    }

    fn timeout(&self) -> Option<u64> {
        Some(60) // 1 minute timeout for email sending
    }

    fn metadata(&self) -> HashMap<String, serde_json::Value> {
        let mut metadata = self.data.metadata.clone();
        metadata.insert("template_name".to_string(), serde_json::Value::String(self.data.template_name.clone()));
        metadata.insert("recipient".to_string(), serde_json::Value::String(self.data.to.clone()));
        metadata
    }

    async fn should_execute(&self, _context: &JobContext) -> bool {
        // Basic validation - could be extended with more checks
        !self.data.to.is_empty() && !self.data.subject.is_empty()
    }
}

impl SerializableJob for EmailJobData {
    fn job_type(&self) -> &'static str {
        "email_send"
    }

    fn serialize(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::to_value(self)
    }

    fn deserialize(data: &serde_json::Value) -> Result<Box<dyn SerializableJob>, serde_json::Error>
    where
        Self: Sized,
    {
        let email_data: EmailJobData = serde_json::from_value(data.clone())?;
        Ok(Box::new(email_data))
    }

    fn priority(&self) -> erp_core::jobs::JobPriority {
        // Critical emails (password reset, security alerts) get high priority
        if self.template_name == "password_reset" || self.template_name == "security_alert" {
            erp_core::jobs::JobPriority::High
        } else {
            erp_core::jobs::JobPriority::Normal
        }
    }

    fn max_attempts(&self) -> u32 {
        self.max_retries.unwrap_or(3)
    }

    fn retry_delay(&self, attempt: u32) -> u64 {
        // Exponential backoff in minutes, max 5 hours
        (2_u64.pow(attempt).min(300)) * 60
    }

    fn timeout(&self) -> Option<u64> {
        Some(60) // 1 minute
    }

    fn metadata(&self) -> HashMap<String, serde_json::Value> {
        let mut metadata = self.metadata.clone();
        metadata.insert("template_name".to_string(), serde_json::Value::String(self.template_name.clone()));
        metadata.insert("recipient".to_string(), serde_json::Value::String(self.to.clone()));
        metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::email::VerificationEmailTemplate;

    #[test]
    fn test_email_job_data_creation() {
        let template = VerificationEmailTemplate {
            user_name: "Test User".to_string(),
            company_name: "Test Company".to_string(),
            verification_url: "https://example.com/verify".to_string(),
            expires_in_hours: 24,
        };

        let job_data = EmailJobData::from_template(
            "test@example.com",
            &template,
            Some("tenant-123".to_string()),
            Some("user-456".to_string()),
        );

        assert_eq!(job_data.to, "test@example.com");
        assert_eq!(job_data.template_name, "email_verification");
        assert!(job_data.subject.contains("Test Company"));
        assert!(job_data.html_body.contains("Test User"));
    }

    #[test]
    fn test_email_job_serialization() {
        let job_data = EmailJobData {
            to: "test@example.com".to_string(),
            subject: "Test Subject".to_string(),
            html_body: "<p>Test HTML</p>".to_string(),
            text_body: "Test text".to_string(),
            template_name: "test_template".to_string(),
            metadata: HashMap::new(),
            max_retries: Some(3),
            tenant_id: None,
            user_id: None,
        };

        let serialized = <EmailJobData as erp_core::SerializableJob>::serialize(&job_data).unwrap();
        let _deserialized = <EmailJobData as erp_core::SerializableJob>::deserialize(&serialized).unwrap();
        
        // The deserialized result is a Box<dyn SerializableJob>, 
        // so we can't directly compare, but we can test the serialization worked
        assert!(serialized.get("to").unwrap().as_str().unwrap() == "test@example.com");
    }
}