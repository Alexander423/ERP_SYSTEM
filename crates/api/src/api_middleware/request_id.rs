//! # Request ID Middleware
//! 
//! This middleware module provides request tracking capabilities for the ERP API server.
//! It generates or extracts unique request identifiers to enable:
//! 
//! - **Distributed Tracing**: Correlate logs across service boundaries
//! - **Request Correlation**: Link related operations in complex workflows
//! - **Debug Support**: Easily identify specific requests in logs
//! - **Audit Trails**: Track request flows for compliance and monitoring
//! 
//! ## Features
//! 
//! - **Automatic ID Generation**: Creates UUIDs when no ID is provided
//! - **Header Extraction**: Accepts IDs from various standard headers
//! - **Client IP Detection**: Extracts real client IP from proxy headers
//! - **Context Enrichment**: Adds user agent, correlation IDs to request context
//! - **Response Headers**: Returns request ID to client for tracking
//! 
//! ## Supported Headers
//! 
//! The middleware recognizes request IDs from multiple headers (in priority order):
//! - `x-request-id` (primary)
//! - `x-correlation-id`
//! - `x-trace-id`
//! - `request-id`
//! 
//! ## Client IP Detection
//! 
//! Real client IPs are extracted from proxy headers:
//! - `x-forwarded-for` (preferred, uses first IP)
//! - `x-real-ip`
//! - `cf-connecting-ip` (Cloudflare)
//! - `x-client-ip`
//! 
//! ## Usage
//! 
//! ```rust
//! use crate::middleware::RequestIdMiddleware;
//! use axum::Router;
//! 
//! let app = Router::new()
//!     .route("/api", get(handler))
//!     .layer(axum::middleware::from_fn(request_id_middleware));
//! ```

use axum::{
    extract::Request,
    http::{header::HeaderValue, HeaderName, StatusCode},
    middleware::Next,
    response::Response,
};
use erp_core::error::RequestContext;
use std::str::FromStr;
use tracing::{debug, Span};
use uuid::Uuid;

/// Request ID header name used for client communication.
/// 
/// This constant defines the primary header name used to communicate
/// request IDs between the client and server. It follows RFC 7231
/// conventions for custom headers.
pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// Request ID middleware that generates or extracts request IDs for distributed tracing.
/// 
/// This middleware automatically handles request identification by:
/// 1. Extracting existing request IDs from various headers
/// 2. Generating new UUIDs when no ID is present
/// 3. Enriching request context with additional metadata
/// 4. Adding request IDs to response headers for client tracking
/// 5. Integrating with the tracing system for log correlation
/// 
/// # Performance Characteristics
/// 
/// - **Low Overhead**: Minimal processing per request
/// - **Memory Efficient**: Uses string references where possible  
/// - **UUID Generation**: Fast UUID v4 generation when needed
/// - **Header Processing**: Efficient header iteration and validation
/// 
/// # Security Considerations
/// 
/// - **Input Validation**: Request IDs are validated for length and format
/// - **IP Extraction**: Real client IPs extracted from trusted proxy headers
/// - **No PII Leakage**: Request IDs don't contain sensitive information
/// - **Header Sanitization**: Invalid headers are rejected safely
/// Request ID middleware configuration and utilities
///
/// This struct provides configuration options and utility methods
/// for the request ID middleware system.
pub struct RequestIdMiddleware {
    /// Custom request ID header name (defaults to x-request-id)
    pub header_name: String,
    /// Whether to validate request ID format strictly
    pub strict_validation: bool,
    /// Whether to generate request IDs if none provided
    pub auto_generate: bool,
}

impl RequestIdMiddleware {
    /// Create a new RequestIdMiddleware with default configuration
    pub fn new() -> Self {
        Self {
            header_name: REQUEST_ID_HEADER.to_string(),
            strict_validation: true,
            auto_generate: true,
        }
    }

    /// Create middleware with custom header name
    pub fn with_header_name(mut self, header_name: impl Into<String>) -> Self {
        self.header_name = header_name.into();
        self
    }

    /// Enable or disable strict validation of request IDs
    pub fn with_strict_validation(mut self, strict: bool) -> Self {
        self.strict_validation = strict;
        self
    }

    /// Enable or disable automatic generation of request IDs
    pub fn with_auto_generate(mut self, auto_generate: bool) -> Self {
        self.auto_generate = auto_generate;
        self
    }

    /// Extract request ID using this middleware's configuration
    pub fn extract_request_id(&self, request: &Request) -> Option<String> {
        if let Some(value) = request.headers().get(&self.header_name) {
            if let Ok(id_str) = value.to_str() {
                if !self.strict_validation || is_valid_request_id(id_str) {
                    return Some(id_str.to_string());
                }
            }
        }

        if self.auto_generate {
            Some(Uuid::new_v4().to_string())
        } else {
            None
        }
    }

    /// Check if a request has a valid request ID
    pub fn has_valid_request_id(&self, request: &Request) -> bool {
        if let Some(value) = request.headers().get(&self.header_name) {
            if let Ok(id_str) = value.to_str() {
                return !self.strict_validation || is_valid_request_id(id_str);
            }
        }
        false
    }
}

/// Middleware function that handles request ID generation and injection
pub async fn request_id_middleware(
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract or generate request ID
    let request_id = extract_or_generate_request_id(&request);
    
    // Create request context
    let request_context = RequestContext::new()
        .with_request_id(request_id.clone());
    
    // Extract additional context information from headers
    let request_context = enrich_request_context(request_context, &request);
    
    // Add request context to request extensions
    request.extensions_mut().insert(request_context.clone());
    
    // Add request ID to tracing span
    let span = Span::current();
    span.record("request_id", &request_id);
    
    // Process request
    let mut response = next.run(request).await;
    
    // Add request ID to response headers
    if let Ok(header_value) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert(
            HeaderName::from_str(REQUEST_ID_HEADER).unwrap(),
            header_value,
        );
    }
    
    debug!(
        request_id = %request_id,
        status = %response.status(),
        "Request completed"
    );
    
    Ok(response)
}

/// Extract request ID from headers or generate a new one
fn extract_or_generate_request_id(request: &Request) -> String {
    // Try to extract from various headers
    let possible_headers = [
        REQUEST_ID_HEADER,
        "x-correlation-id",
        "x-trace-id",
        "request-id",
    ];
    
    for header_name in &possible_headers {
        if let Some(value) = request.headers().get(*header_name) {
            if let Ok(id_str) = value.to_str() {
                if is_valid_request_id(id_str) {
                    debug!("Using existing request ID from header {}: {}", header_name, id_str);
                    return id_str.to_string();
                }
            }
        }
    }
    
    // Generate new request ID
    let new_id = Uuid::new_v4().to_string();
    debug!("Generated new request ID: {}", new_id);
    new_id
}

/// Enrich request context with additional information from headers
fn enrich_request_context(
    mut context: RequestContext,
    request: &Request,
) -> RequestContext {
    // Extract source IP
    if let Some(ip) = extract_client_ip(request) {
        context = context.with_source_ip(ip);
    }
    
    // Extract user agent
    if let Some(user_agent) = request.headers().get("user-agent") {
        if let Ok(ua_str) = user_agent.to_str() {
            context = context.with_user_agent(ua_str);
        }
    }
    
    // Extract correlation ID if different from request ID
    if let Some(correlation_id) = request.headers().get("x-correlation-id") {
        if let Ok(corr_str) = correlation_id.to_str() {
            context = context.with_correlation_id(corr_str);
        }
    }
    
    context
}

/// Extract client IP from various headers
fn extract_client_ip(request: &Request) -> Option<String> {
    // Try different headers in order of preference
    let ip_headers = [
        "x-forwarded-for",
        "x-real-ip", 
        "cf-connecting-ip", // Cloudflare
        "x-client-ip",
        "x-forwarded",
        "forwarded-for",
        "forwarded",
    ];
    
    for header_name in &ip_headers {
        if let Some(value) = request.headers().get(*header_name) {
            if let Ok(ip_str) = value.to_str() {
                // For X-Forwarded-For, take the first IP (client)
                let ip = if header_name == &"x-forwarded-for" {
                    ip_str.split(',').next().unwrap_or(ip_str).trim()
                } else {
                    ip_str.trim()
                };
                
                if is_valid_ip(ip) {
                    return Some(ip.to_string());
                }
            }
        }
    }
    
    // Fall back to connection info if available
    // Note: In practice, you might get this from connection metadata
    None
}

/// Validate that a string is a valid request ID
fn is_valid_request_id(id: &str) -> bool {
    // Check if it's a valid UUID or alphanumeric string
    if Uuid::from_str(id).is_ok() {
        return true;
    }
    
    // Allow alphanumeric with hyphens and underscores, reasonable length
    id.len() <= 128 
        && id.len() >= 8
        && id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
}

/// Simple IP validation
fn is_valid_ip(ip: &str) -> bool {
    ip.parse::<std::net::IpAddr>().is_ok()
}

/// Extension trait to easily get request ID from extensions
///
/// This trait provides convenient methods to extract request IDs and context
/// from HTTP requests. It's designed to be used throughout the application
/// where request correlation is needed.
pub trait RequestIdExt {
    /// Get the request ID if available in the request extensions
    fn request_id(&self) -> Option<&str>;

    /// Get the full request context if available in the request extensions
    fn request_context(&self) -> Option<&RequestContext>;

    /// Get the correlation ID if different from request ID
    fn correlation_id(&self) -> Option<&str>;

    /// Get the source IP address from the request context
    fn source_ip(&self) -> Option<&str>;

    /// Get the user agent from the request context
    fn user_agent(&self) -> Option<&str>;
}

impl RequestIdExt for Request {
    fn request_id(&self) -> Option<&str> {
        self.extensions()
            .get::<RequestContext>()
            .map(|ctx| ctx.request_id.as_str())
    }

    fn request_context(&self) -> Option<&RequestContext> {
        self.extensions().get::<RequestContext>()
    }

    fn correlation_id(&self) -> Option<&str> {
        self.extensions()
            .get::<RequestContext>()
            .and_then(|ctx| ctx.correlation_id.as_deref())
    }

    fn source_ip(&self) -> Option<&str> {
        self.extensions()
            .get::<RequestContext>()
            .and_then(|ctx| ctx.source_ip.as_deref())
    }

    fn user_agent(&self) -> Option<&str> {
        self.extensions()
            .get::<RequestContext>()
            .and_then(|ctx| ctx.user_agent.as_deref())
    }
}

/// Helper macro for logging with request ID
#[macro_export]
macro_rules! log_with_request_id {
    ($request:expr, $level:ident, $($args:tt)*) => {
        if let Some(request_id) = $request.request_id() {
            tracing::$level!(request_id = request_id, $($args)*);
        } else {
            tracing::$level!($($args)*);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::StatusCode, routing::get, Router};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_request_id_generation() {
        let app = Router::new()
            .route("/", get(|| async { "OK" }))
            .layer(axum::middleware::from_fn(request_id_middleware));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        // Check that request ID header is present
        let request_id = response.headers().get(REQUEST_ID_HEADER);
        assert!(request_id.is_some());
        
        let request_id_str = request_id.unwrap().to_str().unwrap();
        assert!(is_valid_request_id(request_id_str));
    }

    #[tokio::test]
    async fn test_existing_request_id_preserved() {
        let existing_id = "test-request-id-12345";
        
        let app = Router::new()
            .route("/", get(|| async { "OK" }))
            .layer(axum::middleware::from_fn(request_id_middleware));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/")
                    .header(REQUEST_ID_HEADER, existing_id)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let returned_id = response.headers()
            .get(REQUEST_ID_HEADER)
            .unwrap()
            .to_str()
            .unwrap();
        
        assert_eq!(returned_id, existing_id);
    }

    #[test]
    fn test_valid_request_id() {
        // Valid UUIDs
        assert!(is_valid_request_id("550e8400-e29b-41d4-a716-446655440000"));
        
        // Valid alphanumeric strings
        assert!(is_valid_request_id("test-request-123"));
        assert!(is_valid_request_id("abc_123_def"));
        
        // Invalid - too short
        assert!(!is_valid_request_id("abc"));
        
        // Invalid - too long
        let too_long = "a".repeat(129);
        assert!(!is_valid_request_id(&too_long));
        
        // Invalid - special characters
        assert!(!is_valid_request_id("test@request.id"));
    }

    #[test]
    fn test_ip_validation() {
        assert!(is_valid_ip("192.168.1.1"));
        assert!(is_valid_ip("10.0.0.1"));
        assert!(is_valid_ip("2001:db8::1"));
        assert!(!is_valid_ip("not.an.ip"));
        assert!(!is_valid_ip("999.999.999.999"));
    }

    #[test]
    fn test_client_ip_extraction() {
        let request = Request::builder()
            .uri("/")
            .header("x-forwarded-for", "203.0.113.1, 70.41.3.18, 150.172.238.178")
            .body(Body::empty())
            .unwrap();

        let ip = extract_client_ip(&request);
        assert_eq!(ip, Some("203.0.113.1".to_string()));
    }
}