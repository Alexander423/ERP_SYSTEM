use prometheus::{HistogramVec, IntCounterVec, Opts, Registry};

/// Authentication-specific metrics
#[derive(Debug, Clone)]
pub struct AuthMetrics {
    // Login metrics
    pub login_attempts_total: IntCounterVec,
    pub login_success_total: IntCounterVec,
    pub login_failures_total: IntCounterVec,
    pub login_duration_seconds: HistogramVec,
    
    // Registration metrics
    pub registration_attempts_total: IntCounterVec,
    pub registration_success_total: IntCounterVec,
    pub registration_failures_total: IntCounterVec,
    
    // Password reset metrics
    pub password_reset_requests_total: IntCounterVec,
    pub password_reset_completions_total: IntCounterVec,
    pub password_reset_failures_total: IntCounterVec,
    
    // Email verification metrics
    pub email_verification_sent_total: IntCounterVec,
    pub email_verification_completed_total: IntCounterVec,
    pub email_verification_failures_total: IntCounterVec,
    
    // Token metrics
    pub token_validations_total: IntCounterVec,
    pub token_validation_duration_seconds: HistogramVec,
    pub token_refresh_total: IntCounterVec,
    
    // Email metrics
    pub emails_sent_total: IntCounterVec,
    pub email_send_failures_total: IntCounterVec,
    pub email_send_duration_seconds: HistogramVec,
    
    // Security metrics
    pub rate_limit_exceeded_total: IntCounterVec,
    pub invalid_token_attempts_total: IntCounterVec,
    pub account_lockouts_total: IntCounterVec,
}

impl AuthMetrics {
    pub fn new(namespace: &str) -> Result<Self, prometheus::Error> {
        let login_attempts_total = IntCounterVec::new(
            Opts::new(
                format!("{}_login_attempts_total", namespace),
                "Total number of login attempts"
            ),
            &["tenant_id", "status"]
        )?;

        let login_success_total = IntCounterVec::new(
            Opts::new(
                format!("{}_login_success_total", namespace),
                "Total number of successful logins"
            ),
            &["tenant_id"]
        )?;

        let login_failures_total = IntCounterVec::new(
            Opts::new(
                format!("{}_login_failures_total", namespace),
                "Total number of failed logins"
            ),
            &["tenant_id", "reason"]
        )?;

        let login_duration_seconds = HistogramVec::new(
            prometheus::HistogramOpts::new(
                format!("{}_login_duration_seconds", namespace),
                "Time spent processing login requests"
            ).buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0]),
            &["tenant_id"]
        )?;

        let registration_attempts_total = IntCounterVec::new(
            Opts::new(
                format!("{}_registration_attempts_total", namespace),
                "Total number of registration attempts"
            ),
            &["tenant_id"]
        )?;

        let registration_success_total = IntCounterVec::new(
            Opts::new(
                format!("{}_registration_success_total", namespace),
                "Total number of successful registrations"
            ),
            &["tenant_id"]
        )?;

        let registration_failures_total = IntCounterVec::new(
            Opts::new(
                format!("{}_registration_failures_total", namespace),
                "Total number of failed registrations"
            ),
            &["tenant_id", "reason"]
        )?;

        let password_reset_requests_total = IntCounterVec::new(
            Opts::new(
                format!("{}_password_reset_requests_total", namespace),
                "Total number of password reset requests"
            ),
            &["tenant_id"]
        )?;

        let password_reset_completions_total = IntCounterVec::new(
            Opts::new(
                format!("{}_password_reset_completions_total", namespace),
                "Total number of completed password resets"
            ),
            &["tenant_id"]
        )?;

        let password_reset_failures_total = IntCounterVec::new(
            Opts::new(
                format!("{}_password_reset_failures_total", namespace),
                "Total number of failed password resets"
            ),
            &["tenant_id", "reason"]
        )?;

        let email_verification_sent_total = IntCounterVec::new(
            Opts::new(
                format!("{}_email_verification_sent_total", namespace),
                "Total number of verification emails sent"
            ),
            &["tenant_id"]
        )?;

        let email_verification_completed_total = IntCounterVec::new(
            Opts::new(
                format!("{}_email_verification_completed_total", namespace),
                "Total number of completed email verifications"
            ),
            &["tenant_id"]
        )?;

        let email_verification_failures_total = IntCounterVec::new(
            Opts::new(
                format!("{}_email_verification_failures_total", namespace),
                "Total number of failed email verifications"
            ),
            &["tenant_id", "reason"]
        )?;

        let token_validations_total = IntCounterVec::new(
            Opts::new(
                format!("{}_token_validations_total", namespace),
                "Total number of token validations"
            ),
            &["tenant_id", "token_type", "status"]
        )?;

        let token_validation_duration_seconds = HistogramVec::new(
            prometheus::HistogramOpts::new(
                format!("{}_token_validation_duration_seconds", namespace),
                "Time spent validating tokens"
            ).buckets(vec![0.0001, 0.0005, 0.001, 0.005, 0.01, 0.025, 0.05, 0.1]),
            &["tenant_id", "token_type"]
        )?;

        let token_refresh_total = IntCounterVec::new(
            Opts::new(
                format!("{}_token_refresh_total", namespace),
                "Total number of token refreshes"
            ),
            &["tenant_id", "status"]
        )?;

        let emails_sent_total = IntCounterVec::new(
            Opts::new(
                format!("{}_emails_sent_total", namespace),
                "Total number of emails sent"
            ),
            &["tenant_id", "template", "provider"]
        )?;

        let email_send_failures_total = IntCounterVec::new(
            Opts::new(
                format!("{}_email_send_failures_total", namespace),
                "Total number of failed email sends"
            ),
            &["tenant_id", "template", "provider", "reason"]
        )?;

        let email_send_duration_seconds = HistogramVec::new(
            prometheus::HistogramOpts::new(
                format!("{}_email_send_duration_seconds", namespace),
                "Time spent sending emails"
            ).buckets(vec![0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 25.0]),
            &["template", "provider"]
        )?;

        let rate_limit_exceeded_total = IntCounterVec::new(
            Opts::new(
                format!("{}_rate_limit_exceeded_total", namespace),
                "Total number of rate limit violations"
            ),
            &["tenant_id", "endpoint"]
        )?;

        let invalid_token_attempts_total = IntCounterVec::new(
            Opts::new(
                format!("{}_invalid_token_attempts_total", namespace),
                "Total number of invalid token attempts"
            ),
            &["tenant_id", "token_type"]
        )?;

        let account_lockouts_total = IntCounterVec::new(
            Opts::new(
                format!("{}_account_lockouts_total", namespace),
                "Total number of account lockouts"
            ),
            &["tenant_id", "reason"]
        )?;

        Ok(Self {
            login_attempts_total,
            login_success_total,
            login_failures_total,
            login_duration_seconds,
            registration_attempts_total,
            registration_success_total,
            registration_failures_total,
            password_reset_requests_total,
            password_reset_completions_total,
            password_reset_failures_total,
            email_verification_sent_total,
            email_verification_completed_total,
            email_verification_failures_total,
            token_validations_total,
            token_validation_duration_seconds,
            token_refresh_total,
            emails_sent_total,
            email_send_failures_total,
            email_send_duration_seconds,
            rate_limit_exceeded_total,
            invalid_token_attempts_total,
            account_lockouts_total,
        })
    }

    pub fn register_all(&self, registry: &Registry) -> Result<(), prometheus::Error> {
        registry.register(Box::new(self.login_attempts_total.clone()))?;
        registry.register(Box::new(self.login_success_total.clone()))?;
        registry.register(Box::new(self.login_failures_total.clone()))?;
        registry.register(Box::new(self.login_duration_seconds.clone()))?;
        registry.register(Box::new(self.registration_attempts_total.clone()))?;
        registry.register(Box::new(self.registration_success_total.clone()))?;
        registry.register(Box::new(self.registration_failures_total.clone()))?;
        registry.register(Box::new(self.password_reset_requests_total.clone()))?;
        registry.register(Box::new(self.password_reset_completions_total.clone()))?;
        registry.register(Box::new(self.password_reset_failures_total.clone()))?;
        registry.register(Box::new(self.email_verification_sent_total.clone()))?;
        registry.register(Box::new(self.email_verification_completed_total.clone()))?;
        registry.register(Box::new(self.email_verification_failures_total.clone()))?;
        registry.register(Box::new(self.token_validations_total.clone()))?;
        registry.register(Box::new(self.token_validation_duration_seconds.clone()))?;
        registry.register(Box::new(self.token_refresh_total.clone()))?;
        registry.register(Box::new(self.emails_sent_total.clone()))?;
        registry.register(Box::new(self.email_send_failures_total.clone()))?;
        registry.register(Box::new(self.email_send_duration_seconds.clone()))?;
        registry.register(Box::new(self.rate_limit_exceeded_total.clone()))?;
        registry.register(Box::new(self.invalid_token_attempts_total.clone()))?;
        registry.register(Box::new(self.account_lockouts_total.clone()))?;
        
        Ok(())
    }
}