use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use erp_core::Error;
use serde_json::json;
use std::env;
use tracing::{error, warn};

/// API error wrapper that provides secure error handling and response sanitization.
/// 
/// This wrapper ensures that sensitive information is never exposed in API responses,
/// while still providing useful debugging information in logs and development environments.
#[derive(Debug)]
pub struct ApiError {
    error: Error,
    request_id: Option<String>,
    environment: String,
}

impl ApiError {
    /// Create a new API error with environment detection
    pub fn new(error: Error) -> Self {
        let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
        Self {
            error,
            request_id: None,
            environment,
        }
    }

    /// Create a new API error with explicit environment
    pub fn new_with_environment(error: Error, environment: String) -> Self {
        Self {
            error,
            request_id: None,
            environment,
        }
    }

    /// Add request ID for correlation and debugging
    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }

    /// Check if this error should trigger security monitoring
    pub fn is_security_relevant(&self) -> bool {
        matches!(self.error.code, 
            erp_core::error::ErrorCode::AuthenticationFailed |
            erp_core::error::ErrorCode::AuthorizationFailed |
            erp_core::error::ErrorCode::PermissionDenied |
            erp_core::error::ErrorCode::SecurityPolicyViolation |
            erp_core::error::ErrorCode::RateLimitExceeded |
            erp_core::error::ErrorCode::TooManyRequests |
            erp_core::error::ErrorCode::InvalidCredentials
        )
    }

    /// Log error with appropriate level and security considerations
    fn log_error(&self) {
        let status_code = StatusCode::from_u16(self.error.http_status())
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        match status_code.as_u16() {
            500..=599 => {
                // Server errors - log full details for debugging
                error!(
                    error_code = %self.error.code,
                    error_id = %self.error.context.error_id,
                    request_id = ?self.request_id,
                    severity = ?self.error.severity,
                    "Internal server error: {}",
                    self.error
                );

                // In production, also log the full debug info to a separate channel
                if self.environment == "production" {
                    error!(target: "security_audit",
                        error_details = %serde_json::to_string(&self.error.to_debug_json()).unwrap_or_default(),
                        "Production server error - full details"
                    );
                }
            }
            400..=499 => {
                // Client errors - different handling for security-relevant errors
                if self.is_security_relevant() {
                    warn!(target: "security_audit",
                        error_code = %self.error.code,
                        error_id = %self.error.context.error_id,
                        request_id = ?self.request_id,
                        client_ip = ?self.error.context.metadata.get("client_ip"),
                        user_agent = ?self.error.context.metadata.get("user_agent"),
                        "Security-relevant client error: {}",
                        self.error
                    );
                } else {
                    tracing::debug!(
                        error_code = %self.error.code,
                        request_id = ?self.request_id,
                        "Client error: {}",
                        self.error
                    );
                }
            }
            _ => {
                tracing::info!(
                    error_code = %self.error.code,
                    request_id = ?self.request_id,
                    "Informational response: {}",
                    self.error
                );
            }
        }
    }
}

impl From<Error> for ApiError {
    fn from(error: Error) -> Self {
        Self::new(error)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status_code = StatusCode::from_u16(self.error.http_status())
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        // Log error with security considerations
        self.log_error();

        // Create sanitized response based on environment
        let error_response = self.error.to_api_response_with_environment(&self.environment);
        
        // Add request ID if present
        let mut response_json = error_response;
        if let Some(request_id) = &self.request_id {
            if let Some(error_obj) = response_json.get_mut("error") {
                error_obj["request_id"] = json!(request_id);
            }
        }

        // In production, ensure we're not leaking internal information
        if self.environment == "production" && status_code.is_server_error() {
            // Override with minimal information for server errors in production
            response_json = json!({
                "error": {
                    "code": self.error.code,
                    "message": "An internal error occurred. Please try again later.",
                    "error_id": self.error.context.error_id,
                    "request_id": self.request_id,
                    "timestamp": self.error.context.timestamp
                }
            });
        }

        (status_code, Json(response_json)).into_response()
    }
}