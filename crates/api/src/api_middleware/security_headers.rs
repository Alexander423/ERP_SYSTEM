//! # Security Headers Middleware
//! 
//! This middleware module implements comprehensive HTTP security headers for the ERP API server.
//! It follows security best practices and helps protect against common web vulnerabilities:
//! 
//! ## Security Headers Implemented
//! 
//! - **HSTS (HTTP Strict Transport Security)**: Enforces HTTPS connections
//! - **CSP (Content Security Policy)**: Prevents XSS and data injection attacks
//! - **X-Frame-Options**: Protects against clickjacking attacks
//! - **X-Content-Type-Options**: Prevents MIME type sniffing
//! - **Referrer-Policy**: Controls referrer information leakage
//! - **Permissions-Policy**: Restricts browser feature access
//! - **X-XSS-Protection**: Legacy XSS protection (for older browsers)
//! 
//! ## Configuration Profiles
//! 
//! ### Development Profile
//! - **HSTS Disabled**: Allows HTTP during development
//! - **Permissive CSP**: Allows inline scripts/styles for dev tools
//! - **WebSocket Support**: Enables ws: and wss: connections
//! 
//! ### Production Profile
//! - **Strict HSTS**: 2-year max-age with preload and subdomains
//! - **Restrictive CSP**: No inline scripts, limited sources
//! - **Maximum Security**: All security headers at strictest settings
//! 
//! ## Security Benefits
//! 
//! - **XSS Prevention**: Content Security Policy blocks malicious scripts
//! - **Clickjacking Protection**: X-Frame-Options prevents embedding
//! - **HTTPS Enforcement**: HSTS ensures encrypted connections
//! - **Data Leakage Prevention**: Referrer policy controls information sharing
//! - **Feature Restriction**: Permissions policy limits browser capabilities
//! 
//! ## Usage
//! 
//! ```rust
//! use crate::middleware::SecurityHeadersMiddleware;
//! use axum::Router;
//! 
//! // Production setup
//! let app = Router::new()
//!     .route("/api", get(handler))
//!     .layer(SecurityHeadersMiddleware::for_production().layer());
//! 
//! // Development setup
//! let app = Router::new()
//!     .route("/api", get(handler))
//!     .layer(SecurityHeadersMiddleware::for_development().layer());
//! ```

use axum::{
    extract::Request,
    http::{header, HeaderValue, StatusCode},
    middleware::Next,
    response::Response,
};
use tracing::debug;

/// Configuration for HTTP security headers middleware.
/// 
/// This struct allows fine-grained control over security headers applied to HTTP responses.
/// Different environments (development, testing, production) require different security
/// postures, and this configuration enables those customizations.
/// 
/// # Security Headers Configured
/// 
/// - **HSTS**: HTTP Strict Transport Security for HTTPS enforcement
/// - **CSP**: Content Security Policy for XSS and injection protection
/// - **X-Frame-Options**: Clickjacking protection through frame restrictions
/// - **X-Content-Type-Options**: MIME type sniffing prevention
/// - **Referrer-Policy**: Controls referrer information in cross-origin requests
/// - **Permissions-Policy**: Browser feature restriction and privacy protection
/// 
/// # Environment-Specific Configurations
/// 
/// Different deployment environments require different security settings:
/// 
/// - **Development**: Permissive settings for debugging and hot-reload
/// - **Testing**: Balanced settings that don't break automated testing
/// - **Production**: Maximum security settings for live deployments
/// 
/// # Examples
/// 
/// ```rust
/// use crate::middleware::SecurityHeadersConfig;
/// 
/// // Custom configuration
/// let config = SecurityHeadersConfig {
///     enable_hsts: true,
///     hsts_max_age: 31536000, // 1 year
///     csp: Some("default-src 'self'".to_string()),
///     x_frame_options: Some("DENY".to_string()),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone)]
pub struct SecurityHeadersConfig {
    /// Enable HSTS (HTTP Strict Transport Security)
    pub enable_hsts: bool,
    /// HSTS max age in seconds
    pub hsts_max_age: u64,
    /// Include subdomains in HSTS
    pub hsts_include_subdomains: bool,
    /// Enable HSTS preload
    pub hsts_preload: bool,
    
    /// Content Security Policy
    pub csp: Option<String>,
    /// X-Frame-Options
    pub x_frame_options: Option<String>,
    /// X-Content-Type-Options
    pub x_content_type_options: bool,
    /// Referrer-Policy
    pub referrer_policy: Option<String>,
    /// Permissions-Policy
    pub permissions_policy: Option<String>,
    
    /// Enable secure cookies in development
    pub force_secure_cookies: bool,
}

impl Default for SecurityHeadersConfig {
    fn default() -> Self {
        Self {
            enable_hsts: true,
            hsts_max_age: 31536000, // 1 year
            hsts_include_subdomains: true,
            hsts_preload: true,
            csp: Some(
                "default-src 'self'; \
                 script-src 'self' 'unsafe-inline'; \
                 style-src 'self' 'unsafe-inline'; \
                 img-src 'self' data: https:; \
                 font-src 'self'; \
                 connect-src 'self'; \
                 frame-ancestors 'none'; \
                 base-uri 'self'; \
                 form-action 'self'".to_string()
            ),
            x_frame_options: Some("DENY".to_string()),
            x_content_type_options: true,
            referrer_policy: Some("strict-origin-when-cross-origin".to_string()),
            permissions_policy: Some(
                "accelerometer=(), \
                 camera=(), \
                 geolocation=(), \
                 gyroscope=(), \
                 magnetometer=(), \
                 microphone=(), \
                 payment=(), \
                 usb=()".to_string()
            ),
            force_secure_cookies: false,
        }
    }
}

impl SecurityHeadersConfig {
    /// Create a development-friendly configuration
    pub fn development() -> Self {
        Self {
            enable_hsts: false, // Don't enforce HTTPS in development
            csp: Some(
                "default-src 'self'; \
                 script-src 'self' 'unsafe-inline' 'unsafe-eval'; \
                 style-src 'self' 'unsafe-inline'; \
                 img-src 'self' data: https: http:; \
                 font-src 'self' data:; \
                 connect-src 'self' ws: wss:; \
                 frame-ancestors 'self'; \
                 base-uri 'self'".to_string()
            ),
            ..Default::default()
        }
    }

    /// Create a production configuration with strict security
    pub fn production() -> Self {
        Self {
            enable_hsts: true,
            hsts_max_age: 63072000, // 2 years
            hsts_include_subdomains: true,
            hsts_preload: true,
            csp: Some(
                "default-src 'none'; \
                 script-src 'self'; \
                 style-src 'self'; \
                 img-src 'self' data:; \
                 font-src 'self'; \
                 connect-src 'self'; \
                 frame-ancestors 'none'; \
                 base-uri 'none'; \
                 form-action 'self'".to_string()
            ),
            force_secure_cookies: true,
            ..Default::default()
        }
    }
}

/// Security headers middleware
#[derive(Clone)]
pub struct SecurityHeadersMiddleware {
    config: SecurityHeadersConfig,
}

impl SecurityHeadersMiddleware {
    pub fn new(config: SecurityHeadersConfig) -> Self {
        Self { config }
    }

    pub fn with_default_config() -> Self {
        Self::new(SecurityHeadersConfig::default())
    }

    pub fn for_development() -> Self {
        Self::new(SecurityHeadersConfig::development())
    }

    pub fn for_production() -> Self {
        Self::new(SecurityHeadersConfig::production())
    }

    /// Create an axum layer for this middleware
    pub fn layer(self) -> axum::middleware::FromFn<impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, StatusCode>> + Send>>, Request> {
        let config = self.config.clone();
        axum::middleware::from_fn(move |request, next| {
            let config = config.clone();
            Box::pin(async move {
                security_headers_middleware_with_config(request, next, config).await
            }) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, StatusCode>> + Send>>
        })
    }

    /// Get the current configuration
    pub fn config(&self) -> &SecurityHeadersConfig {
        &self.config
    }

    /// Update the configuration
    pub fn with_config(mut self, config: SecurityHeadersConfig) -> Self {
        self.config = config;
        self
    }
}

/// Middleware function that adds security headers to responses using default config
pub async fn security_headers_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    security_headers_middleware_with_config(request, next, SecurityHeadersConfig::default()).await
}

/// Middleware function that adds security headers to responses with custom config
pub async fn security_headers_middleware_with_config(
    request: Request,
    next: Next,
    config: SecurityHeadersConfig,
) -> Result<Response, StatusCode> {
    // Process the request
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    // HSTS (HTTP Strict Transport Security)
    if config.enable_hsts {
        let mut hsts_value = format!("max-age={}", config.hsts_max_age);
        if config.hsts_include_subdomains {
            hsts_value.push_str("; includeSubDomains");
        }
        if config.hsts_preload {
            hsts_value.push_str("; preload");
        }
        
        if let Ok(hsts_header) = HeaderValue::from_str(&hsts_value) {
            headers.insert(header::STRICT_TRANSPORT_SECURITY, hsts_header);
        }
    }

    // Content Security Policy
    if let Some(csp) = &config.csp {
        if let Ok(csp_header) = HeaderValue::from_str(csp) {
            headers.insert(header::CONTENT_SECURITY_POLICY, csp_header);
        }
    }

    // X-Frame-Options
    if let Some(x_frame_options) = &config.x_frame_options {
        if let Ok(x_frame_header) = HeaderValue::from_str(x_frame_options) {
            headers.insert("x-frame-options", x_frame_header);
        }
    }

    // X-Content-Type-Options
    if config.x_content_type_options {
        headers.insert("x-content-type-options", HeaderValue::from_static("nosniff"));
    }

    // Referrer-Policy
    if let Some(referrer_policy) = &config.referrer_policy {
        if let Ok(referrer_header) = HeaderValue::from_str(referrer_policy) {
            headers.insert("referrer-policy", referrer_header);
        }
    }

    // Permissions-Policy
    if let Some(permissions_policy) = &config.permissions_policy {
        if let Ok(permissions_header) = HeaderValue::from_str(permissions_policy) {
            headers.insert("permissions-policy", permissions_header);
        }
    }

    // Additional security headers
    headers.insert("x-xss-protection", HeaderValue::from_static("1; mode=block"));
    headers.insert("x-dns-prefetch-control", HeaderValue::from_static("off"));
    headers.insert("x-download-options", HeaderValue::from_static("noopen"));
    headers.insert("x-permitted-cross-domain-policies", HeaderValue::from_static("none"));

    // Remove server information
    headers.remove(header::SERVER);
    headers.remove("x-powered-by");

    debug!("Added security headers to response");
    Ok(response)
}

/// Middleware that enforces secure cookie settings
pub async fn secure_cookies_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let mut response = next.run(request).await;

    // Check if we should force secure cookies (based on config or HTTPS)
    let force_secure = request.extensions()
        .get::<SecurityHeadersConfig>()
        .map(|config| config.force_secure_cookies)
        .unwrap_or(false);

    let is_https = request.uri().scheme_str() == Some("https");

    // Modify Set-Cookie headers to ensure security attributes
    let headers = response.headers_mut();
    let mut cookie_headers_to_update = Vec::new();

    // Collect all Set-Cookie headers
    for (name, value) in headers.iter() {
        if name == "set-cookie" {
            if let Ok(cookie_str) = value.to_str() {
                cookie_headers_to_update.push(cookie_str.to_string());
            }
        }
    }

    // Remove existing Set-Cookie headers
    headers.remove("set-cookie");

    // Add back modified Set-Cookie headers with security attributes
    for cookie_str in cookie_headers_to_update {
        let mut secure_cookie = enhance_cookie_security(&cookie_str, is_https || force_secure);

        if let Ok(header_value) = HeaderValue::from_str(&secure_cookie) {
            headers.append("set-cookie", header_value);
        }
    }

    debug!("Enhanced cookie security attributes");
    Ok(response)
}

/// Enhance a cookie string with security attributes
fn enhance_cookie_security(cookie: &str, force_secure: bool) -> String {
    let mut parts: Vec<&str> = cookie.split(';').map(|s| s.trim()).collect();
    let mut has_secure = false;
    let mut has_httponly = false;
    let mut has_samesite = false;

    // Check existing attributes
    for part in &parts {
        let lower = part.to_lowercase();
        if lower == "secure" {
            has_secure = true;
        } else if lower == "httponly" {
            has_httponly = true;
        } else if lower.starts_with("samesite=") {
            has_samesite = true;
        }
    }

    // Add missing security attributes
    if force_secure && !has_secure {
        parts.push("Secure");
    }

    if !has_httponly {
        parts.push("HttpOnly");
    }

    if !has_samesite {
        parts.push("SameSite=Strict");
    }

    parts.join("; ")
}

/// Helper function to create a security event log entry
pub fn log_security_event(event: &str, details: Option<&str>) {
    tracing::warn!(
        security_event = event,
        details = details,
        "Security event detected"
    );
}



#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::StatusCode, routing::get, Router};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_security_headers_applied() {
        let app = Router::new()
            .route("/", get(|| async { "Hello, World!" }))
            .layer(axum::middleware::from_fn(security_headers_middleware));

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
        
        let headers = response.headers();
        
        // Check that security headers are present
        assert!(headers.contains_key("x-content-type-options"));
        assert!(headers.contains_key("x-frame-options"));
        assert!(headers.contains_key("referrer-policy"));
        
        // Check that server headers are removed
        assert!(!headers.contains_key(header::SERVER));
        assert!(!headers.contains_key("x-powered-by"));
    }

    #[tokio::test]
    async fn test_development_config() {
        let dev_config = SecurityHeadersConfig::development();
        
        // HSTS should be disabled in development
        assert!(!dev_config.enable_hsts);
        
        // CSP should be more permissive
        assert!(dev_config.csp.as_ref().unwrap().contains("'unsafe-inline'"));
    }

    #[tokio::test]
    async fn test_production_config() {
        let prod_config = SecurityHeadersConfig::production();
        
        // HSTS should be enabled with long max-age
        assert!(prod_config.enable_hsts);
        assert_eq!(prod_config.hsts_max_age, 63072000);
        
        // CSP should be strict
        assert!(prod_config.csp.as_ref().unwrap().contains("default-src 'none'"));
    }
}