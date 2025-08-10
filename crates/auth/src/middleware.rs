use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use erp_core::{
    security::JwtService,
    DatabasePool, Error, Permission, RequestContext, TenantContext, TenantId, UserId,
};
use redis::aio::ConnectionManager;
use std::sync::Arc;
use tracing::{error, warn};
use uuid::Uuid;

#[derive(Clone)]
pub struct AuthState {
    pub jwt_service: Arc<JwtService>,
    pub db: Arc<DatabasePool>,
    pub redis: ConnectionManager,
}

pub async fn auth_middleware(
    State(state): State<AuthState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = match extract_token(&request) {
        Some(token) => token,
        None => {
            return Ok(unauthorized_response("Missing authorization token"));
        }
    };

    let claims = match state.jwt_service.verify_access_token(&token) {
        Ok(claims) => claims,
        Err(e) => {
            warn!("Token verification failed: {}", e);
            return Ok(unauthorized_response("Invalid or expired token"));
        }
    };

    // Check if token is revoked
    let is_revoked = check_token_revoked(&state.redis, &claims.jti).await;
    if is_revoked {
        return Ok(unauthorized_response("Token has been revoked"));
    }

    // Parse IDs
    let tenant_id = match Uuid::parse_str(&claims.tenant_id) {
        Ok(id) => id,
        Err(_) => {
            error!("Invalid tenant ID in token: {}", claims.tenant_id);
            return Ok(unauthorized_response("Invalid token claims"));
        }
    };

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            error!("Invalid user ID in token: {}", claims.sub);
            return Ok(unauthorized_response("Invalid token claims"));
        }
    };

    // Get tenant context
    let tenant = match get_tenant_context(&state.db, tenant_id).await {
        Ok(tenant) => tenant,
        Err(e) => {
            error!("Failed to get tenant context: {}", e);
            return Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response());
        }
    };

    // Parse permissions
    let permissions: Vec<Permission> = claims
        .permissions
        .iter()
        .filter_map(|p| {
            let parts: Vec<&str> = p.split(':').collect();
            if parts.len() == 2 {
                Some(Permission::new(parts[0], parts[1]))
            } else {
                warn!("Invalid permission format: {}", p);
                None
            }
        })
        .collect();

    // Parse impersonator ID if present
    let impersonator_id = claims
        .impersonator_id
        .and_then(|id| Uuid::parse_str(&id).ok())
        .map(UserId);

    // Create request context
    let context = RequestContext {
        tenant_context: Some(tenant),
        user_id: Some(user_id),
        jti: Some(claims.jti.clone()),
        permissions,
        impersonator_id,
        request_id: Uuid::new_v4().to_string(),
    };

    // Insert context into request extensions
    request.extensions_mut().insert(context);

    Ok(next.run(request).await)
}

pub async fn require_permission_middleware(
    required_permission: String,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let context = match request.extensions().get::<RequestContext>() {
        Some(ctx) => ctx,
        None => {
            error!("Request context not found in require_permission middleware");
            return Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response());
        }
    };

    let has_permission = context.permissions.iter().any(|p| p.to_string() == required_permission);

    if !has_permission {
        warn!(
            "User {:?} lacks required permission: {}",
            context.user_id, required_permission
        );
        return Ok(forbidden_response(&format!(
            "Missing required permission: {}",
            required_permission
        )));
    }

    Ok(next.run(request).await)
}

// Helper for creating permission middleware closures
pub fn require_permission(perm: &str) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, StatusCode>> + Send>> + Clone {
    let permission = perm.to_string();
    move |request: Request, next: Next| {
        let perm_clone = permission.clone();
        Box::pin(async move {
            require_permission_middleware(perm_clone, request, next).await
        })
    }
}

pub async fn rate_limit_middleware(
    State(redis): State<ConnectionManager>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract client identifier (IP or user ID)
    let client_id = extract_client_id(&request);
    
    // Check rate limit
    let is_allowed = check_rate_limit(redis, &client_id).await;
    
    if !is_allowed {
        return Ok(too_many_requests_response());
    }

    Ok(next.run(request).await)
}

// Helper functions

fn extract_token(request: &Request) -> Option<String> {
    request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| {
            if value.starts_with("Bearer ") {
                Some(value[7..].to_string())
            } else {
                None
            }
        })
}

async fn check_token_revoked(redis: &ConnectionManager, jti: &str) -> bool {
    let key = format!("revoked_token:{}", jti);
    
    let mut conn = redis.clone();
    match redis::AsyncCommands::exists::<_, bool>(&mut conn, &key).await {
        Ok(exists) => exists,
        Err(e) => {
            error!("Failed to check token revocation: {}", e);
            false // Allow on Redis error to prevent complete lockout
        }
    }
}

async fn get_tenant_context(db: &DatabasePool, tenant_id: Uuid) -> Result<TenantContext, Error> {
    let tenant = sqlx::query!(
        "SELECT schema_name FROM public.tenants WHERE id = $1 AND status = 'active'",
        tenant_id
    )
    .fetch_optional(&db.main_pool)
    .await?
    .ok_or_else(|| Error::new(erp_core::ErrorCode::ResourceNotFound, "Tenant not found or inactive"))?;

    Ok(TenantContext {
        tenant_id: TenantId(tenant_id),
        schema_name: tenant.schema_name,
    })
}

fn extract_client_id(request: &Request) -> String {
    // Try to get user ID from context first
    if let Some(ctx) = request.extensions().get::<RequestContext>() {
        if let Some(user_id) = ctx.user_id {
            return format!("user:{}", user_id);
        }
    }

    // Fall back to IP address
    request
        .headers()
        .get("X-Real-IP")
        .or_else(|| request.headers().get("X-Forwarded-For"))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

async fn check_rate_limit(mut redis: ConnectionManager, client_id: &str) -> bool {
    let key = format!("rate_limit:{}", client_id);
    let window = 60; // 1 minute window
    let max_requests = 60; // 60 requests per minute

    match redis::AsyncCommands::incr::<_, _, i32>(&mut redis, &key, 1).await {
        Ok(count) => {
            if count == 1 {
                // Set expiry on first request
                let _: Result<(), _> = redis::AsyncCommands::expire(&mut redis, &key, window).await;
            }
            count <= max_requests
        }
        Err(e) => {
            error!("Rate limit check failed: {}", e);
            true // Allow on error
        }
    }
}

// Response helpers

fn unauthorized_response(message: &str) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({
            "error": message
        })),
    )
        .into_response()
}

fn forbidden_response(message: &str) -> Response {
    (
        StatusCode::FORBIDDEN,
        Json(serde_json::json!({
            "error": message
        })),
    )
        .into_response()
}

fn too_many_requests_response() -> Response {
    (
        StatusCode::TOO_MANY_REQUESTS,
        Json(serde_json::json!({
            "error": "Rate limit exceeded"
        })),
    )
        .into_response()
}