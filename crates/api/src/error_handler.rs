//! Error handling utilities that make use of ApiError methods
//!
//! This module provides convenience functions for creating and handling
//! API errors with proper context and environment awareness.

use crate::error::ApiError;
use axum::{extract::Request, response::Response};
use erp_core::Error;
use std::env;

/// Create an API error with environment detection
pub fn create_api_error(error: Error) -> ApiError {
    let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
    ApiError::new_with_environment(error, environment)
}

/// Create an API error with request ID from request context
pub fn create_api_error_with_request_id(error: Error, request: &Request) -> ApiError {
    let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
    let mut api_error = ApiError::new_with_environment(error, environment);

    // Extract request ID from extensions if available
    if let Some(request_context) = request.extensions().get::<erp_core::error::RequestContext>() {
        api_error = api_error.with_request_id(request_context.request_id.clone());
    }

    api_error
}

/// Handle error in middleware context with proper request ID correlation
pub fn handle_middleware_error(error: Error, request: &Request) -> Response {
    let api_error = create_api_error_with_request_id(error, request);
    api_error.into_response()
}

/// Create production-ready API error with minimal information disclosure
pub fn create_production_error(error: Error, request_id: Option<String>) -> ApiError {
    let mut api_error = ApiError::new_with_environment(error, "production".to_string());

    if let Some(id) = request_id {
        api_error = api_error.with_request_id(id);
    }

    api_error
}

/// Create development-friendly API error with detailed information
pub fn create_development_error(error: Error, request_id: Option<String>) -> ApiError {
    let mut api_error = ApiError::new_with_environment(error, "development".to_string());

    if let Some(id) = request_id {
        api_error = api_error.with_request_id(id);
    }

    api_error
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use erp_core::error::{ErrorCode, RequestContext};

    #[test]
    fn test_create_api_error() {
        let error = Error::new(ErrorCode::ValidationFailed, "Test error");
        let api_error = create_api_error(error);

        // Should work without panicking
        assert!(format!("{:?}", api_error).contains("Test error"));
    }

    #[test]
    fn test_create_production_error() {
        let error = Error::new(ErrorCode::InternalServerError, "Internal error");
        let api_error = create_production_error(error, Some("test-request-123".to_string()));

        assert!(format!("{:?}", api_error).contains("test-request-123"));
    }

    #[test]
    fn test_create_development_error() {
        let error = Error::new(ErrorCode::NotFound, "Resource not found");
        let api_error = create_development_error(error, Some("dev-request-456".to_string()));

        assert!(format!("{:?}", api_error).contains("dev-request-456"));
    }

    #[tokio::test]
    async fn test_create_api_error_with_request_id() {
        let mut request = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        let request_context = RequestContext::new()
            .with_request_id("test-request-789".to_string());
        request.extensions_mut().insert(request_context);

        let error = Error::new(ErrorCode::AuthenticationFailed, "Auth failed");
        let api_error = create_api_error_with_request_id(error, &request);

        assert!(format!("{:?}", api_error).contains("test-request-789"));
    }
}