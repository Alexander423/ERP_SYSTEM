use serde::{Deserialize, Serialize};
use std::fmt;

/// Standardized error codes for the ERP system
/// These are business-agnostic and represent technical error categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    // General System Errors (1000-1999)
    InternalServerError = 1000,
    ConfigurationError = 1001,
    ServiceUnavailable = 1002,
    Timeout = 1003,
    ResourceExhausted = 1004,

    // Database Errors (2000-2999)
    DatabaseConnectionError = 2000,
    DatabaseConstraintViolation = 2001,
    DatabaseTransactionError = 2002,
    DatabaseQueryError = 2003,
    DatabaseMigrationError = 2004,

    // Network & Communication Errors (3000-3999)
    NetworkError = 3000,
    NetworkTimeout = 3001,
    NetworkConnectionRefused = 3002,
    ExternalServiceError = 3003,
    SerializationError = 3004,

    // Security & Authentication Errors (4000-4999)
    AuthenticationRequired = 4000,
    AuthenticationFailed = 4001,
    InvalidCredentials = 4002,
    TokenExpired = 4003,
    TokenInvalid = 4004,
    AuthorizationFailed = 4005,
    PermissionDenied = 4006,
    SecurityPolicyViolation = 4007,

    // Input Validation Errors (5000-5999)
    ValidationFailed = 5000,
    InvalidInput = 5001,
    MissingRequiredField = 5002,
    InvalidFormat = 5003,
    ValueOutOfRange = 5004,
    DuplicateValue = 5005,

    // Resource Management Errors (6000-6999)
    ResourceNotFound = 6000,
    ResourceAlreadyExists = 6001,
    ResourceLocked = 6002,
    ResourceInUse = 6003,
    ResourceQuotaExceeded = 6004,
    NotFound = 6005,
    NotImplemented = 6006,

    // Rate Limiting & Throttling Errors (7000-7999)
    RateLimitExceeded = 7000,
    TooManyRequests = 7001,
    ConcurrencyLimitExceeded = 7002,

    // Cache & Storage Errors (8000-8999)
    CacheError = 8000,
    CacheMiss = 8001,
    StorageError = 8002,
    EncryptionError = 8003,
    DecryptionError = 8004,

    // Job & Queue Errors (9000-9999)
    JobQueueError = 9000,
    JobExecutionFailed = 9001,
    JobTimeout = 9002,
    JobDeserializationError = 9003,
}

impl ErrorCode {
    /// Get HTTP status code for this error
    pub fn http_status(&self) -> u16 {
        match self {
            // 500 - Internal Server Error
            ErrorCode::InternalServerError
            | ErrorCode::ConfigurationError
            | ErrorCode::DatabaseConnectionError
            | ErrorCode::DatabaseTransactionError
            | ErrorCode::DatabaseQueryError
            | ErrorCode::DatabaseMigrationError
            | ErrorCode::NetworkError
            | ErrorCode::ExternalServiceError
            | ErrorCode::SerializationError
            | ErrorCode::CacheError
            | ErrorCode::StorageError
            | ErrorCode::EncryptionError
            | ErrorCode::DecryptionError
            | ErrorCode::JobQueueError
            | ErrorCode::JobExecutionFailed => 500,

            // 503 - Service Unavailable
            ErrorCode::ServiceUnavailable
            | ErrorCode::NetworkConnectionRefused => 503,

            // 408 - Request Timeout
            ErrorCode::Timeout
            | ErrorCode::NetworkTimeout
            | ErrorCode::JobTimeout => 408,

            // 401 - Unauthorized
            ErrorCode::AuthenticationRequired
            | ErrorCode::AuthenticationFailed
            | ErrorCode::InvalidCredentials
            | ErrorCode::TokenExpired
            | ErrorCode::TokenInvalid => 401,

            // 403 - Forbidden
            ErrorCode::AuthorizationFailed
            | ErrorCode::PermissionDenied
            | ErrorCode::SecurityPolicyViolation => 403,

            // 400 - Bad Request
            ErrorCode::ValidationFailed
            | ErrorCode::InvalidInput
            | ErrorCode::MissingRequiredField
            | ErrorCode::InvalidFormat
            | ErrorCode::ValueOutOfRange
            | ErrorCode::JobDeserializationError => 400,

            // 404 - Not Found
            ErrorCode::ResourceNotFound
            | ErrorCode::CacheMiss => 404,

            // 409 - Conflict
            ErrorCode::ResourceAlreadyExists
            | ErrorCode::DuplicateValue
            | ErrorCode::DatabaseConstraintViolation => 409,

            // 423 - Locked
            ErrorCode::ResourceLocked => 423,

            // 429 - Too Many Requests
            ErrorCode::RateLimitExceeded
            | ErrorCode::TooManyRequests
            | ErrorCode::ConcurrencyLimitExceeded => 429,

            // 507 - Insufficient Storage
            ErrorCode::ResourceExhausted
            | ErrorCode::ResourceQuotaExceeded => 507,

            // 422 - Unprocessable Entity
            ErrorCode::ResourceInUse => 422,

            // 404 - Not Found
            ErrorCode::NotFound => 404,

            // 501 - Not Implemented
            ErrorCode::NotImplemented => 501,
        }
    }

    /// Get error category for metrics and logging
    pub fn category(&self) -> &'static str {
        match self {
            ErrorCode::InternalServerError
            | ErrorCode::ConfigurationError
            | ErrorCode::ServiceUnavailable
            | ErrorCode::Timeout
            | ErrorCode::ResourceExhausted => "system",

            ErrorCode::DatabaseConnectionError
            | ErrorCode::DatabaseConstraintViolation
            | ErrorCode::DatabaseTransactionError
            | ErrorCode::DatabaseQueryError
            | ErrorCode::DatabaseMigrationError => "database",

            ErrorCode::NetworkError
            | ErrorCode::NetworkTimeout
            | ErrorCode::NetworkConnectionRefused
            | ErrorCode::ExternalServiceError
            | ErrorCode::SerializationError => "network",

            ErrorCode::AuthenticationRequired
            | ErrorCode::AuthenticationFailed
            | ErrorCode::InvalidCredentials
            | ErrorCode::TokenExpired
            | ErrorCode::TokenInvalid
            | ErrorCode::AuthorizationFailed
            | ErrorCode::PermissionDenied
            | ErrorCode::SecurityPolicyViolation => "security",

            ErrorCode::ValidationFailed
            | ErrorCode::InvalidInput
            | ErrorCode::MissingRequiredField
            | ErrorCode::InvalidFormat
            | ErrorCode::ValueOutOfRange
            | ErrorCode::DuplicateValue => "validation",

            ErrorCode::ResourceNotFound
            | ErrorCode::ResourceAlreadyExists
            | ErrorCode::ResourceLocked
            | ErrorCode::ResourceInUse
            | ErrorCode::ResourceQuotaExceeded => "resource",

            ErrorCode::RateLimitExceeded
            | ErrorCode::TooManyRequests
            | ErrorCode::ConcurrencyLimitExceeded => "rate_limit",

            ErrorCode::CacheError
            | ErrorCode::CacheMiss
            | ErrorCode::StorageError
            | ErrorCode::EncryptionError
            | ErrorCode::DecryptionError => "storage",

            ErrorCode::JobQueueError
            | ErrorCode::JobExecutionFailed
            | ErrorCode::JobTimeout
            | ErrorCode::JobDeserializationError => "jobs",

            ErrorCode::NotFound => "resource",
            ErrorCode::NotImplemented => "system",
        }
    }

    /// Check if error should be retried
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            ErrorCode::NetworkTimeout
                | ErrorCode::NetworkConnectionRefused
                | ErrorCode::ServiceUnavailable
                | ErrorCode::DatabaseConnectionError
                | ErrorCode::CacheError
                | ErrorCode::JobTimeout
                | ErrorCode::ResourceExhausted
        )
    }

    /// Check if error should be logged at error level
    pub fn should_log_as_error(&self) -> bool {
        !matches!(
            self,
            ErrorCode::ValidationFailed
                | ErrorCode::InvalidInput
                | ErrorCode::MissingRequiredField
                | ErrorCode::InvalidFormat
                | ErrorCode::ValueOutOfRange
                | ErrorCode::ResourceNotFound
                | ErrorCode::AuthenticationFailed
                | ErrorCode::InvalidCredentials
                | ErrorCode::PermissionDenied
                | ErrorCode::RateLimitExceeded
                | ErrorCode::TooManyRequests
        )
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}