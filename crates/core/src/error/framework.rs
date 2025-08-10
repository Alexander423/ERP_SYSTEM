use super::{ErrorCode, ErrorContext};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error as ThisError;

pub type Result<T> = std::result::Result<T, Error>;

/// Severity levels for errors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ErrorSeverity {
    /// Low priority errors that don't affect system functionality
    Low,
    /// Medium priority errors that may degrade performance
    Medium,
    /// High priority errors that affect core functionality
    High,
    /// Critical errors that require immediate attention
    Critical,
}

/// Error categories for classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ErrorCategory {
    System,
    Database,
    Network,
    Security,
    Validation,
    Resource,
    RateLimit,
    Storage,
    Jobs,
}

/// Main error type for the ERP system
#[derive(Debug, ThisError, Clone)]
pub struct Error {
    /// Standardized error code
    pub code: ErrorCode,
    /// Human-readable error message
    pub message: String,
    /// Optional detailed description
    pub details: Option<String>,
    /// Error context for debugging
    pub context: ErrorContext,
    /// Error severity
    pub severity: ErrorSeverity,
    /// Optional cause chain
    pub cause: Option<Box<Error>>,
}

impl Error {
    /// Create a new error
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
            context: ErrorContext::new(),
            severity: Self::default_severity_for_code(code),
            cause: None,
        }
    }

    /// Create error with details
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    /// Set error context
    pub fn with_context(mut self, context: ErrorContext) -> Self {
        self.context = context;
        self
    }

    /// Set error severity
    pub fn with_severity(mut self, severity: ErrorSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Chain with another error as cause
    pub fn with_cause(mut self, cause: Error) -> Self {
        self.cause = Some(Box::new(cause));
        self
    }

    /// Add trace to context
    pub fn add_trace(mut self, trace: impl Into<String>) -> Self {
        self.context = self.context.add_trace(trace.into());
        self
    }

    /// Add metadata to context
    pub fn add_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.context.add_metadata(key.into(), value);
        self
    }

    /// Get HTTP status code
    pub fn http_status(&self) -> u16 {
        self.code.http_status()
    }

    /// Get error category
    pub fn category(&self) -> ErrorCategory {
        match self.code.category() {
            "system" => ErrorCategory::System,
            "database" => ErrorCategory::Database,
            "network" => ErrorCategory::Network,
            "security" => ErrorCategory::Security,
            "validation" => ErrorCategory::Validation,
            "resource" => ErrorCategory::Resource,
            "rate_limit" => ErrorCategory::RateLimit,
            "storage" => ErrorCategory::Storage,
            "jobs" => ErrorCategory::Jobs,
            _ => ErrorCategory::System,
        }
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        self.code.is_retryable()
    }

    /// Check if should be logged as error level
    pub fn should_log_as_error(&self) -> bool {
        self.code.should_log_as_error() || matches!(self.severity, ErrorSeverity::High | ErrorSeverity::Critical)
    }

    /// Convert to JSON for API responses (sanitized for security)
    pub fn to_api_response(&self) -> serde_json::Value {
        self.to_api_response_with_environment("development")
    }

    /// Convert to JSON for API responses with environment-specific sanitization
    pub fn to_api_response_with_environment(&self, environment: &str) -> serde_json::Value {
        let is_production = environment == "production";
        
        // In production, sanitize sensitive information
        let (message, details) = if is_production {
            self.sanitize_for_production()
        } else {
            (self.message.clone(), self.details.clone())
        };

        serde_json::json!({
            "error": {
                "code": self.code,
                "message": message,
                "details": if is_production { None } else { details },
                "error_id": self.context.error_id,
                "request_id": self.context.request_id,
                "timestamp": self.context.timestamp
            }
        })
    }

    /// Convert to full debug JSON (for internal logging only, never for API responses)
    pub fn to_debug_json(&self) -> serde_json::Value {
        serde_json::json!({
            "error": {
                "code": self.code,
                "message": self.message,
                "details": self.details,
                "context": self.context,
                "severity": self.severity,
                "cause": self.cause.as_ref().map(|c| c.to_debug_json())
            }
        })
    }

    /// Sanitize error messages and details for production API responses
    fn sanitize_for_production(&self) -> (String, Option<String>) {
        let sanitized_message = match self.code {
            // Authentication errors - generic message
            ErrorCode::AuthenticationFailed 
            | ErrorCode::InvalidCredentials 
            | ErrorCode::TokenExpired 
            | ErrorCode::TokenInvalid => "Authentication failed".to_string(),

            // Authorization errors - generic message
            ErrorCode::PermissionDenied 
            | ErrorCode::AuthorizationFailed => "Access denied".to_string(),

            // Validation errors - safe to show general validation info
            ErrorCode::ValidationFailed => "Input validation failed".to_string(),
            ErrorCode::InvalidInput => "Invalid input provided".to_string(),
            ErrorCode::MissingRequiredField => "Required field missing".to_string(),
            ErrorCode::InvalidFormat => "Invalid format provided".to_string(),
            ErrorCode::ValueOutOfRange => "Value out of acceptable range".to_string(),

            // Resource errors - generic messages
            ErrorCode::ResourceNotFound => "Resource not found".to_string(),
            ErrorCode::ResourceAlreadyExists => "Resource already exists".to_string(),
            ErrorCode::DuplicateValue => "Duplicate value detected".to_string(),

            // Rate limiting - safe to be specific
            ErrorCode::RateLimitExceeded 
            | ErrorCode::TooManyRequests => "Rate limit exceeded, please try again later".to_string(),

            // Server errors - generic message to prevent information disclosure
            ErrorCode::InternalServerError 
            | ErrorCode::DatabaseConnectionError 
            | ErrorCode::DatabaseQueryError 
            | ErrorCode::DatabaseTransactionError 
            | ErrorCode::DatabaseConstraintViolation 
            | ErrorCode::DatabaseMigrationError 
            | ErrorCode::NetworkError 
            | ErrorCode::NetworkConnectionRefused 
            | ErrorCode::NetworkTimeout 
            | ErrorCode::ExternalServiceError 
            | ErrorCode::ServiceUnavailable 
            | ErrorCode::ConfigurationError 
            | ErrorCode::CacheError 
            | ErrorCode::CacheMiss 
            | ErrorCode::SerializationError 
            | ErrorCode::JobDeserializationError 
            | ErrorCode::EncryptionError 
            | ErrorCode::DecryptionError 
            | ErrorCode::ResourceExhausted 
            | ErrorCode::SecurityPolicyViolation => "An internal error occurred. Please try again later".to_string(),

            _ => "An error occurred. Please try again later".to_string(),
        };

        // Never expose details in production
        (sanitized_message, None)
    }

    /// Get default severity for error code
    fn default_severity_for_code(code: ErrorCode) -> ErrorSeverity {
        match code {
            ErrorCode::ValidationFailed
            | ErrorCode::InvalidInput
            | ErrorCode::MissingRequiredField
            | ErrorCode::InvalidFormat
            | ErrorCode::ValueOutOfRange
            | ErrorCode::ResourceNotFound
            | ErrorCode::CacheMiss => ErrorSeverity::Low,

            ErrorCode::DuplicateValue
            | ErrorCode::ResourceAlreadyExists
            | ErrorCode::AuthenticationFailed
            | ErrorCode::InvalidCredentials
            | ErrorCode::PermissionDenied
            | ErrorCode::RateLimitExceeded
            | ErrorCode::TooManyRequests => ErrorSeverity::Medium,

            ErrorCode::DatabaseConnectionError
            | ErrorCode::NetworkError
            | ErrorCode::ExternalServiceError
            | ErrorCode::ServiceUnavailable
            | ErrorCode::AuthorizationFailed
            | ErrorCode::SecurityPolicyViolation
            | ErrorCode::ResourceExhausted => ErrorSeverity::High,

            ErrorCode::InternalServerError
            | ErrorCode::ConfigurationError
            | ErrorCode::DatabaseTransactionError
            | ErrorCode::DatabaseMigrationError
            | ErrorCode::EncryptionError
            | ErrorCode::DecryptionError => ErrorSeverity::Critical,

            _ => ErrorSeverity::Medium,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)?;
        if let Some(details) = &self.details {
            write!(f, ": {}", details)?;
        }
        Ok(())
    }
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct ErrorSer {
            code: ErrorCode,
            message: String,
            details: Option<String>,
            context: ErrorContext,
            severity: ErrorSeverity,
        }

        let error_ser = ErrorSer {
            code: self.code,
            message: self.message.clone(),
            details: self.details.clone(),
            context: self.context.clone(),
            severity: self.severity,
        };

        error_ser.serialize(serializer)
    }
}

// Convenience constructors for common errors
impl Error {
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::InternalServerError, message)
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::ValidationFailed, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::ResourceNotFound, message)
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::AuthenticationFailed, message)
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::PermissionDenied, message)
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::ResourceAlreadyExists, message)
    }

    pub fn rate_limited(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::RateLimitExceeded, message)
    }
}

// Implement From for common error types
impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        let code = match &err {
            sqlx::Error::Database(db_err) => {
                if db_err.is_unique_violation() || db_err.is_foreign_key_violation() {
                    ErrorCode::DatabaseConstraintViolation
                } else {
                    ErrorCode::DatabaseQueryError
                }
            }
            sqlx::Error::PoolTimedOut => ErrorCode::DatabaseConnectionError,
            sqlx::Error::Io(_) => ErrorCode::DatabaseConnectionError,
            _ => ErrorCode::DatabaseQueryError,
        };

        Self::new(code, err.to_string())
            .add_trace("sqlx::Error conversion")
    }
}

impl From<redis::RedisError> for Error {
    fn from(err: redis::RedisError) -> Self {
        let code = match err.kind() {
            redis::ErrorKind::IoError => ErrorCode::NetworkConnectionRefused,
            redis::ErrorKind::AuthenticationFailed => ErrorCode::InvalidCredentials,
            redis::ErrorKind::TypeError | redis::ErrorKind::ExecAbortError => ErrorCode::SerializationError,
            _ => ErrorCode::CacheError,
        };

        Self::new(code, err.to_string())
            .add_trace("redis::RedisError conversion")
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        // Determine if this is a serialization or deserialization error
        let code = if err.is_data() || err.is_syntax() {
            ErrorCode::JobDeserializationError
        } else {
            ErrorCode::SerializationError
        };
        
        Self::new(code, err.to_string())
            .add_trace("serde_json::Error conversion")
    }
}

impl From<config::ConfigError> for Error {
    fn from(err: config::ConfigError) -> Self {
        Self::new(ErrorCode::ConfigurationError, err.to_string())
            .add_trace("config::ConfigError conversion")
    }
}