//! Tenant Context Middleware
//!
//! This middleware extracts tenant information from incoming requests and makes it
//! available to request handlers. It supports multiple tenant identification methods:
//! - X-Tenant-ID header (for API clients)
//! - Subdomain extraction (for web applications)
//! - JWT claims (for authenticated requests)

use axum::{
    extract::{Host, Request},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use erp_core::TenantContext;
use serde_json::json;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Extract tenant context from the request
pub async fn tenant_context_middleware(
    headers: HeaderMap,
    Host(host): Host,
    mut req: Request,
    next: Next,
) -> Response {
    // Try to extract tenant ID from multiple sources
    let tenant_id = extract_tenant_id(&headers, &host).await;

    match tenant_id {
        Some(tid) => {
            // Create tenant context
            let tenant_context = TenantContext {
                tenant_id: erp_core::TenantId(tid),
                schema_name: format!("tenant_{}", tid.to_string().replace('-', "_")),
            };

            info!(
                tenant_id = %tid,
                schema = %tenant_context.schema_name,
                "Tenant context established"
            );

            // Insert tenant context into request extensions
            req.extensions_mut().insert(tenant_context);

            // Continue with the request
            next.run(req).await
        }
        None => {
            // For now, we'll allow requests without tenant context for public endpoints
            // In production, you might want to be more strict
            warn!("Request without tenant context");
            next.run(req).await
        }
    }
}

/// Extract tenant ID from various sources
async fn extract_tenant_id(headers: &HeaderMap, host: &str) -> Option<Uuid> {
    // 1. Try X-Tenant-ID header (highest priority)
    if let Some(header_value) = headers.get("x-tenant-id") {
        if let Ok(header_str) = header_value.to_str() {
            if let Ok(tenant_id) = Uuid::parse_str(header_str) {
                info!("Tenant ID extracted from X-Tenant-ID header: {}", tenant_id);
                return Some(tenant_id);
            } else {
                warn!("Invalid UUID in X-Tenant-ID header: {}", header_str);
            }
        }
    }

    // 2. Try subdomain extraction (e.g., tenant1.erp.example.com)
    if let Some(tenant_id) = extract_from_subdomain(host) {
        info!("Tenant ID extracted from subdomain: {}", tenant_id);
        return Some(tenant_id);
    }

    // 3. Try JWT token (if present)
    if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = &auth_str[7..];
                if let Some(tenant_id) = extract_from_jwt(token) {
                    info!("Tenant ID extracted from JWT: {}", tenant_id);
                    return Some(tenant_id);
                }
            }
        }
    }

    None
}

/// Extract tenant ID from subdomain
fn extract_from_subdomain(host: &str) -> Option<Uuid> {
    // Split the host by dots
    let parts: Vec<&str> = host.split('.').collect();

    // Check if we have a subdomain (at least 3 parts for subdomain.domain.tld)
    if parts.len() >= 3 {
        let subdomain = parts[0];

        // Try to parse subdomain as UUID
        if let Ok(tenant_id) = Uuid::parse_str(subdomain) {
            return Some(tenant_id);
        }

        // You could also map subdomain names to tenant IDs here
        // For example: "acme" -> specific UUID
        match subdomain {
            "demo" => Some(Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()),
            _ => None,
        }
    } else {
        None
    }
}

/// Extract tenant ID from JWT token claims
fn extract_from_jwt(token: &str) -> Option<Uuid> {
    // This is a simplified version - in production, you'd properly validate the JWT
    // For now, we'll just try to decode the claims without verification

    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }

    // Decode the claims (middle part)
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

    if let Ok(claims_bytes) = URL_SAFE_NO_PAD.decode(parts[1]) {
        if let Ok(claims_str) = String::from_utf8(claims_bytes) {
            if let Ok(claims) = serde_json::from_str::<serde_json::Value>(&claims_str) {
                // Try to get tenant_id from claims
                if let Some(tenant_id_str) = claims.get("tenant_id").and_then(|v| v.as_str()) {
                    if let Ok(tenant_id) = Uuid::parse_str(tenant_id_str) {
                        return Some(tenant_id);
                    }
                }
            }
        }
    }

    None
}

/// Middleware that requires a valid tenant context
pub async fn require_tenant_context(
    req: Request,
    next: Next,
) -> Response {
    // Check if tenant context exists in extensions
    if req.extensions().get::<TenantContext>().is_none() {
        error!("Request missing required tenant context");
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Missing tenant context",
                "message": "This endpoint requires a valid tenant context. Please provide X-Tenant-ID header or use a tenant-specific subdomain."
            }))
        ).into_response();
    }

    next.run(req).await
}

/// Extract tenant context from request extensions
pub fn extract_tenant_context(req: &Request) -> Option<TenantContext> {
    req.extensions().get::<TenantContext>().cloned()
}