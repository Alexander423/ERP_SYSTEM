//! # Security Headers Middleware

use axum::{
    extract::{Request, State},
    http::{header, HeaderValue},
    middleware::Next,
    response::Response,
};
use tracing::debug;

#[derive(Debug, Clone)]
pub struct SecurityHeadersConfig {
    pub enable_hsts: bool,
    pub hsts_max_age: u64,
    pub hsts_include_subdomains: bool,
    pub hsts_preload: bool,
    pub csp: Option<String>,
    pub x_frame_options: Option<String>,
    pub x_content_type_options: bool,
    pub referrer_policy: Option<String>,
    pub permissions_policy: Option<String>,
}

impl Default for SecurityHeadersConfig {
    fn default() -> Self {
        Self {
            enable_hsts: true,
            hsts_max_age: 31536000, // 1 year
            hsts_include_subdomains: true,
            hsts_preload: true,
            csp: Some(
                "default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data: https:; font-src 'self'; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'".to_string(),
            ),
            x_frame_options: Some("DENY".to_string()),
            x_content_type_options: true,
            referrer_policy: Some("strict-origin-when-cross-origin".to_string()),
            permissions_policy: Some(
                "accelerometer=(), camera=(), geolocation=(), gyroscope=(), magnetometer=(), microphone=(), payment=(), usb=()".to_string(),
            ),
        }
    }
}

impl SecurityHeadersConfig {
    pub fn development() -> Self {
        Self {
            enable_hsts: false,
            csp: Some(
                "default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https: http:; font-src 'self' data:; connect-src 'self' ws: wss:; frame-ancestors 'self'; base-uri 'self'".to_string(),
            ),
            ..Default::default()
        }
    }

    pub fn production() -> Self {
        Self::default()
    }
}

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

}

/// Simple security headers middleware function that can be used directly with from_fn
pub async fn security_headers_middleware(
    request: Request,
    next: Next,
) -> Response {
    let config = SecurityHeadersConfig::production();
    apply_security_headers(config, request, next).await
}

/// Development security headers middleware function
pub async fn security_headers_middleware_dev(
    request: Request,
    next: Next,
) -> Response {
    let config = SecurityHeadersConfig::development();
    apply_security_headers(config, request, next).await
}

async fn apply_security_headers(
    config: SecurityHeadersConfig,
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

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

    if let Some(csp) = &config.csp {
        if let Ok(csp_header) = HeaderValue::from_str(csp) {
            headers.insert(header::CONTENT_SECURITY_POLICY, csp_header);
        }
    }

    if let Some(x_frame_options) = &config.x_frame_options {
        if let Ok(x_frame_header) = HeaderValue::from_str(x_frame_options) {
            headers.insert(header::X_FRAME_OPTIONS, x_frame_header);
        }
    }

    if config.x_content_type_options {
        headers.insert(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    }

    if let Some(referrer_policy) = &config.referrer_policy {
        if let Ok(referrer_header) = HeaderValue::from_str(referrer_policy) {
            headers.insert(header::REFERRER_POLICY, referrer_header);
        }
    }

    if let Some(permissions_policy) = &config.permissions_policy {
        if let Ok(permissions_header) = HeaderValue::from_str(permissions_policy) {
            headers.insert("Permissions-Policy", permissions_header);
        }
    }

    headers.insert("x-xss-protection", HeaderValue::from_static("1; mode=block"));
    headers.remove(header::SERVER);

    debug!("Added security headers to response");
    response
}

/// State-based security headers middleware (for use with from_fn_with_state)
pub async fn security_headers_middleware_with_config(
    State(config): State<SecurityHeadersConfig>,
    request: Request,
    next: Next,
) -> Response {
    apply_security_headers(config, request, next).await
}