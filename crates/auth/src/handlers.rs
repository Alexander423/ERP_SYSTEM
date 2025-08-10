use crate::{
    dto::*,
    middleware::{auth_middleware, AuthState},
    service::{AuthService, LoginOrTwoFactorResponse},
};
use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    middleware,
    response::{IntoResponse, Response},
    Json,
    Router,
    routing::{get, post},
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use erp_core::{Error, RequestContext};
use serde::Deserialize;
use std::sync::Arc;
use time;
use tracing::{error, info};
use uuid::Uuid;
use validator::Validate;

pub type SharedAuthService = Arc<AuthService>;

#[derive(Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_page() -> u32 { 1 }
fn default_limit() -> u32 { 20 }

pub fn auth_routes() -> Router<SharedAuthService> {
    Router::new()
        // Public endpoints
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/verify-2fa", post(verify_2fa))
        .route("/auth/refresh-token", post(refresh_token))
        .route("/auth/forgot-password", post(forgot_password))
        .route("/auth/reset-password", post(reset_password))
        .route("/auth/verify-email", post(verify_email))
        .route("/auth/resend-verification", post(resend_verification))
        .route("/auth/validate-reset-token/:token", get(validate_reset_token))
        // Protected endpoints - will be protected when auth_routes_with_middleware is used
        .route("/auth/logout", post(logout))
        .route("/users", get(list_users).post(invite_user))
        .route("/users/:id", get(get_user).put(update_user).delete(delete_user))
        .route("/users/:id/roles", post(assign_role).delete(remove_role))
        .route("/users/:id/enable-2fa", post(enable_2fa))
        .route("/users/:id/disable-2fa", post(disable_2fa))
        .route("/roles", get(list_roles).post(create_role))
        .route("/roles/:id", get(get_role).put(update_role).delete(delete_role))
        .route("/permissions", get(list_permissions))
        .route("/auth/impersonate", post(impersonate))
        .route("/auth/stop-impersonation", post(stop_impersonation))
}

/// Creates auth routes with proper middleware applied to protected endpoints
pub fn auth_routes_with_middleware(auth_service: SharedAuthService) -> Router<SharedAuthService> {
    let public_routes = Router::new()
        // Public endpoints - no middleware
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/verify-2fa", post(verify_2fa))
        .route("/auth/refresh-token", post(refresh_token))
        .route("/auth/forgot-password", post(forgot_password))
        .route("/auth/reset-password", post(reset_password))
        .route("/auth/verify-email", post(verify_email))
        .route("/auth/resend-verification", post(resend_verification))
        .route("/auth/validate-reset-token/:token", get(validate_reset_token));

    let protected_routes = Router::new()
        // Basic protected endpoints - require authentication only
        .route("/auth/logout", post(logout))
        .route("/auth/stop-impersonation", post(stop_impersonation))
        // User management endpoints
        .route("/users", get(list_users).post(invite_user))
        .route("/users/:id", get(get_user).put(update_user).delete(delete_user))
        .route("/users/:id/roles", post(assign_role).delete(remove_role))
        .route("/users/:id/enable-2fa", post(enable_2fa))
        .route("/users/:id/disable-2fa", post(disable_2fa))
        // Role management endpoints
        .route("/roles", get(list_roles).post(create_role))
        .route("/roles/:id", get(get_role).put(update_role).delete(delete_role))
        // Permission management
        .route("/permissions", get(list_permissions))
        // Impersonation
        .route("/auth/impersonate", post(impersonate))
        // Apply auth middleware to all protected routes
        .layer(middleware::from_fn_with_state(
            AuthState {
                jwt_service: auth_service.jwt_service(),
                db: auth_service.db(),
                redis: auth_service.redis(),
            },
            auth_middleware
        ))
        .with_state(auth_service);

    // Combine public and protected routes
    public_routes.merge(protected_routes)
}

// Public handlers

async fn register(
    State(service): State<SharedAuthService>,
    Json(request): Json<RegisterRequest>,
) -> Result<Json<RegistrationResponse>, AppError> {
    info!("Registration request for company: {}", request.company_name);
    let response = service.register_tenant(request).await?;
    Ok(Json(response))
}

async fn login(
    State(service): State<SharedAuthService>,
    jar: CookieJar,
    headers: HeaderMap,
    Json(request): Json<LoginRequest>,
) -> Result<Response, AppError> {
    let tenant_id = extract_tenant_id(&headers)?;
    let client_ip = extract_client_ip(&headers);
    let user_agent = extract_user_agent(&headers);
    info!("Login attempt for user: {} in tenant: {}", request.email, tenant_id);
    
    match service.login(tenant_id, request, client_ip, user_agent).await? {
        LoginOrTwoFactorResponse::Success(response) => {
            let refresh_cookie = Cookie::build(("refresh_token", response.refresh_token))
                .http_only(true)
                .secure(true)
                .same_site(SameSite::Strict)
                .path("/api/v1/auth")
                .max_age(time::Duration::days(30))
                .build();

            let jar = jar.add(refresh_cookie);
            
            Ok((
                jar,
                Json(serde_json::json!({
                    "access_token": response.access_token
                }))
            ).into_response())
        }
        LoginOrTwoFactorResponse::TwoFactorRequired(response) => {
            Ok(Json(response).into_response())
        }
    }
}

async fn verify_2fa(
    State(service): State<SharedAuthService>,
    jar: CookieJar,
    Json(request): Json<Verify2FARequest>,
) -> Result<Response, AppError> {
    info!("2FA verification attempt");
    let response = service.verify_2fa(request).await?;
    
    let refresh_cookie = Cookie::build(("refresh_token", response.refresh_token))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .path("/api/v1/auth")
        .max_age(time::Duration::days(30))
        .build();

    let jar = jar.add(refresh_cookie);
    
    Ok((
        jar,
        Json(serde_json::json!({
            "access_token": response.access_token
        }))
    ).into_response())
}

async fn refresh_token(
    State(service): State<SharedAuthService>,
    jar: CookieJar,
) -> Result<Json<serde_json::Value>, AppError> {
    let refresh_token = jar
        .get("refresh_token")
        .ok_or_else(|| Error::new(erp_core::ErrorCode::TokenInvalid, "No refresh token found"))?
        .value();
    
    let new_access_token = service.refresh_token(refresh_token).await?;
    
    Ok(Json(serde_json::json!({
        "access_token": new_access_token
    })))
}

async fn forgot_password(
    State(service): State<SharedAuthService>,
    headers: HeaderMap,
    Json(request): Json<ForgotPasswordRequest>,
) -> Result<Json<PasswordResetResponse>, AppError> {
    let tenant_id = extract_tenant_id(&headers)?;
    let client_ip = extract_client_ip(&headers);
    let user_agent = extract_user_agent(&headers);
    
    info!("Password reset requested for: {} in tenant: {}", request.email, tenant_id);
    
    service.request_password_reset(tenant_id, request, client_ip, user_agent).await?;
    
    Ok(Json(PasswordResetResponse {
        success: true,
        message: "If the email address is registered, you will receive a password reset link shortly.".to_string(),
    }))
}

async fn reset_password(
    State(service): State<SharedAuthService>,
    headers: HeaderMap,
    Json(request): Json<ResetPasswordRequest>,
) -> Result<Json<PasswordResetResponse>, AppError> {
    let tenant_id = extract_tenant_id(&headers)?;
    let client_ip = extract_client_ip(&headers);
    
    info!("Password reset attempt with token in tenant: {}", tenant_id);
    
    // Validate that passwords match
    if request.new_password != request.confirm_password {
        return Err(AppError(Error::new(erp_core::ErrorCode::ValidationFailed, "Passwords do not match")));
    }
    
    service.confirm_password_reset(tenant_id, request, client_ip).await?;
    
    Ok(Json(PasswordResetResponse {
        success: true,
        message: "Password has been reset successfully. You can now login with your new password.".to_string(),
    }))
}

// Protected handlers

async fn logout(
    State(service): State<SharedAuthService>,
    jar: CookieJar,
    headers: HeaderMap,
    ctx: RequestContext,
) -> Result<Response, AppError> {
    let jti = ctx.jti
        .ok_or_else(|| Error::new(erp_core::ErrorCode::MissingRequiredField, "Missing JWT ID in token"))?;
    let client_ip = extract_client_ip(&headers);
    
    service.logout(&jti, client_ip).await?;
    
    let jar = jar.remove(Cookie::from("refresh_token"));
    
    Ok((jar, StatusCode::NO_CONTENT).into_response())
}

async fn list_users(
    State(service): State<SharedAuthService>,
    ctx: RequestContext,
    Query(params): Query<PaginationParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Check permission
    check_permission(&ctx, "users", "read")?;
    
    let tenant_context = ctx.tenant_context
        .ok_or_else(|| Error::new(erp_core::ErrorCode::MissingRequiredField, "Missing tenant context"))?;
    
    // Page is 1-based in request, but 0-based for service
    let page = params.page.saturating_sub(1);
    let users = service.list_users(&tenant_context, page, params.limit).await?;
    
    Ok(Json(serde_json::json!({
        "users": users,
        "page": params.page,
        "limit": params.limit
    })))
}

async fn get_user(
    State(service): State<SharedAuthService>,
    ctx: RequestContext,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserResponse>, AppError> {
    // Check permission
    check_permission(&ctx, "users", "read")?;
    
    let tenant_context = ctx.tenant_context
        .ok_or_else(|| Error::new(erp_core::ErrorCode::MissingRequiredField, "Missing tenant context"))?;
    
    let user = service.get_user(&tenant_context, user_id).await?;
    Ok(Json(user))
}

async fn invite_user(
    State(service): State<SharedAuthService>,
    ctx: RequestContext,
    Json(request): Json<InviteUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    // Check permission
    check_permission(&ctx, "users", "create")?;
    
    let tenant_context = ctx.tenant_context
        .ok_or_else(|| Error::new(erp_core::ErrorCode::MissingRequiredField, "Missing tenant context"))?;
    
    let user = service.invite_user(&tenant_context, request).await?;
    Ok(Json(user))
}

async fn update_user(
    State(service): State<SharedAuthService>,
    ctx: RequestContext,
    Path(user_id): Path<Uuid>,
    Json(request): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    // Check permission
    check_permission(&ctx, "users", "update")?;
    
    let tenant_context = ctx.tenant_context
        .ok_or_else(|| Error::new(erp_core::ErrorCode::MissingRequiredField, "Missing tenant context"))?;
    
    let user = service.update_user(&tenant_context, user_id, request).await?;
    Ok(Json(user))
}

async fn delete_user(
    State(service): State<SharedAuthService>,
    ctx: RequestContext,
    Path(user_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    // Check permission
    check_permission(&ctx, "users", "delete")?;
    
    let tenant_context = ctx.tenant_context
        .ok_or_else(|| Error::new(erp_core::ErrorCode::MissingRequiredField, "Missing tenant context"))?;
    
    service.delete_user(&tenant_context, user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn assign_role(
    State(service): State<SharedAuthService>,
    ctx: RequestContext,
    Path(user_id): Path<Uuid>,
    Json(request): Json<AssignRoleRequest>,
) -> Result<StatusCode, AppError> {
    // Check permission
    check_permission(&ctx, "users", "assign_roles")?;
    
    // Validate request
    request.validate().map_err(|e| Error::new(erp_core::ErrorCode::ValidationFailed, e.to_string()))?;
    
    let tenant_context = ctx.tenant_context
        .ok_or_else(|| Error::new(erp_core::ErrorCode::MissingRequiredField, "Missing tenant context"))?;
    
    service.assign_roles_to_user(&tenant_context, user_id, request.role_ids).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn remove_role(
    State(service): State<SharedAuthService>,
    ctx: RequestContext,
    Path(user_id): Path<Uuid>,
    Json(request): Json<RemoveRoleRequest>,
) -> Result<StatusCode, AppError> {
    // Check permission
    check_permission(&ctx, "users", "assign_roles")?;
    
    let tenant_context = ctx.tenant_context
        .ok_or_else(|| Error::new(erp_core::ErrorCode::MissingRequiredField, "Missing tenant context"))?;
    
    service.remove_roles_from_user(&tenant_context, user_id, request.role_ids).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn enable_2fa(
    State(service): State<SharedAuthService>,
    ctx: RequestContext,
    Path(user_id): Path<Uuid>,
) -> Result<Json<Enable2FAResponse>, AppError> {
    // Check permission
    check_permission(&ctx, "users", "update")?;
    
    let tenant_context = ctx.tenant_context
        .ok_or_else(|| Error::new(erp_core::ErrorCode::MissingRequiredField, "Missing tenant context"))?;
    
    let response = service.enable_2fa_for_user(&tenant_context, user_id).await?;
    Ok(Json(response))
}

async fn disable_2fa(
    State(service): State<SharedAuthService>,
    ctx: RequestContext,
    Path(user_id): Path<Uuid>,
    Json(request): Json<Disable2FARequest>,
) -> Result<Json<Disable2FAResponse>, AppError> {
    // Check permission
    check_permission(&ctx, "users", "update")?;
    
    // Validate request
    request.validate().map_err(|e| Error::new(erp_core::ErrorCode::ValidationFailed, e.to_string()))?;
    
    let tenant_context = ctx.tenant_context
        .ok_or_else(|| Error::new(erp_core::ErrorCode::MissingRequiredField, "Missing tenant context"))?;
    
    service.disable_2fa_for_user(&tenant_context, user_id, &request.code).await?;
    
    Ok(Json(Disable2FAResponse {
        success: true,
        message: "2FA has been disabled successfully".to_string(),
    }))
}

async fn list_roles(
    State(service): State<SharedAuthService>,
    ctx: RequestContext,
) -> Result<Json<Vec<RoleResponse>>, AppError> {
    // Check permission
    check_permission(&ctx, "roles", "read")?;
    
    let tenant_context = ctx.tenant_context
        .ok_or_else(|| Error::new(erp_core::ErrorCode::MissingRequiredField, "Missing tenant context"))?;
    
    let roles = service.list_roles(&tenant_context).await?;
    Ok(Json(roles))
}

async fn create_role(
    State(service): State<SharedAuthService>,
    ctx: RequestContext,
    Json(request): Json<CreateRoleRequest>,
) -> Result<Json<RoleResponse>, AppError> {
    // Check permission
    check_permission(&ctx, "roles", "create")?;
    
    let tenant_context = ctx.tenant_context
        .ok_or_else(|| Error::new(erp_core::ErrorCode::MissingRequiredField, "Missing tenant context"))?;
    
    let role = service.create_role(&tenant_context, request).await?;
    Ok(Json(role))
}

async fn get_role(
    State(service): State<SharedAuthService>,
    ctx: RequestContext,
    Path(role_id): Path<Uuid>,
) -> Result<Json<RoleResponse>, AppError> {
    // Check permission
    check_permission(&ctx, "roles", "read")?;
    
    let tenant_context = ctx.tenant_context
        .ok_or_else(|| Error::new(erp_core::ErrorCode::MissingRequiredField, "Missing tenant context"))?;
    
    let role = service.get_role(&tenant_context, role_id).await?;
    Ok(Json(role))
}

async fn update_role(
    State(service): State<SharedAuthService>,
    ctx: RequestContext,
    Path(role_id): Path<Uuid>,
    Json(request): Json<UpdateRoleRequest>,
) -> Result<Json<RoleResponse>, AppError> {
    // Check permission
    check_permission(&ctx, "roles", "update")?;
    
    let tenant_context = ctx.tenant_context
        .ok_or_else(|| Error::new(erp_core::ErrorCode::MissingRequiredField, "Missing tenant context"))?;
    
    let role = service.update_role(&tenant_context, role_id, request).await?;
    Ok(Json(role))
}

async fn delete_role(
    State(service): State<SharedAuthService>,
    ctx: RequestContext,
    Path(role_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    // Check permission
    check_permission(&ctx, "roles", "delete")?;
    
    let tenant_context = ctx.tenant_context
        .ok_or_else(|| Error::new(erp_core::ErrorCode::MissingRequiredField, "Missing tenant context"))?;
    
    service.delete_role(&tenant_context, role_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_permissions(
    State(service): State<SharedAuthService>,
    ctx: RequestContext,
) -> Result<Json<Vec<PermissionResponse>>, AppError> {
    // Check permission
    check_permission(&ctx, "permissions", "read")?;
    
    let tenant_context = ctx.tenant_context
        .ok_or_else(|| Error::new(erp_core::ErrorCode::MissingRequiredField, "Missing tenant context"))?;
    
    let permissions = service.list_permissions(&tenant_context).await?;
    Ok(Json(permissions))
}

async fn impersonate(
    State(service): State<SharedAuthService>,
    ctx: RequestContext,
    Json(request): Json<ImpersonateRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Check permission
    check_permission(&ctx, "users", "impersonate")?;
    
    let tenant_context = ctx.tenant_context
        .ok_or_else(|| Error::new(erp_core::ErrorCode::MissingRequiredField, "Missing tenant context"))?;
    
    let admin_user_id = ctx.user_id
        .ok_or_else(|| Error::new(erp_core::ErrorCode::MissingRequiredField, "Missing user context"))?;
    
    let response = service.impersonate_user(
        &tenant_context,
        admin_user_id,
        request.user_id,
        request.reason,
    ).await?;
    
    Ok(Json(serde_json::json!({
        "access_token": response.access_token
    })))
}

async fn stop_impersonation(
    State(service): State<SharedAuthService>,
    ctx: RequestContext,
    Json(_request): Json<StopImpersonationRequest>,
) -> Result<Json<StopImpersonationResponse>, AppError> {
    let jti = ctx.jti
        .ok_or_else(|| Error::new(erp_core::ErrorCode::MissingRequiredField, "Missing JWT ID in token"))?;
    
    let admin_user_id = ctx.user_id
        .ok_or_else(|| Error::new(erp_core::ErrorCode::MissingRequiredField, "Missing user context"))?;
    
    let impersonator_id = ctx.impersonator_id
        .ok_or_else(|| Error::new(erp_core::ErrorCode::AuthorizationFailed, "Not in an impersonation session"))?;
    
    service.stop_impersonation(&jti, impersonator_id.0, admin_user_id).await?;
    
    Ok(Json(StopImpersonationResponse {
        success: true,
        message: "Impersonation session ended successfully".to_string(),
        original_user_id: impersonator_id.0,
    }))
}

// New email verification handlers

async fn verify_email(
    State(service): State<SharedAuthService>,
    headers: HeaderMap,
    Json(request): Json<VerifyEmailRequest>,
) -> Result<Json<EmailVerificationResponse>, AppError> {
    let tenant_id = extract_tenant_id(&headers)?;
    
    info!("Email verification attempt in tenant: {}", tenant_id);
    
    match service.verify_email(tenant_id, request).await {
        Ok(_) => Ok(Json(EmailVerificationResponse {
            success: true,
            message: "Email verified successfully! Your account is now active.".to_string(),
        })),
        Err(e) => {
            error!("Email verification failed: {}", e);
            Ok(Json(EmailVerificationResponse {
                success: false,
                message: "Email verification failed. The token may be invalid or expired.".to_string(),
            }))
        }
    }
}

async fn resend_verification(
    State(service): State<SharedAuthService>,
    headers: HeaderMap,
    Json(request): Json<ResendVerificationRequest>,
) -> Result<Json<EmailVerificationResponse>, AppError> {
    let tenant_id = extract_tenant_id(&headers)?;
    let client_ip = extract_client_ip(&headers);
    
    info!("Resend verification request for user: {} in tenant: {}", request.user_id, tenant_id);
    
    service.resend_verification_email(tenant_id, request.user_id, client_ip).await?;
    
    Ok(Json(EmailVerificationResponse {
        success: true,
        message: "Verification email has been sent. Please check your inbox.".to_string(),
    }))
}

async fn validate_reset_token(
    State(service): State<SharedAuthService>,
    headers: HeaderMap,
    Path(token): Path<String>,
) -> Result<Json<TokenValidationResponse>, AppError> {
    let tenant_id = extract_tenant_id(&headers)?;
    
    info!("Validating reset token in tenant: {}", tenant_id);
    
    let is_valid = service.validate_reset_token(tenant_id, &token).await?;
    let validation_response = TokenValidationResponse {
        valid: is_valid,
        user_email: None,
        expires_at: None,
    };
    Ok(Json(validation_response))
}

// Helper functions

/// Checks if the user has the required permission
fn check_permission(ctx: &RequestContext, resource: &str, action: &str) -> Result<(), Error> {
    let required_permission = format!("{}:{}", resource, action);
    let has_permission = ctx.permissions.iter()
        .any(|p| format!("{}:{}", p.resource, p.action) == required_permission);
    
    if !has_permission {
        return Err(Error::new(
            erp_core::ErrorCode::PermissionDenied, 
            format!("Missing required permission: {}", required_permission)
        ));
    }
    
    Ok(())
}

fn extract_tenant_id(headers: &HeaderMap) -> Result<Uuid, Error> {
    headers
        .get("X-Tenant-Id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| Error::new(erp_core::ErrorCode::MissingRequiredField, "Missing or invalid X-Tenant-Id header"))
}

fn extract_client_ip(headers: &HeaderMap) -> Option<String> {
    // Try common headers for client IP
    headers
        .get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .or_else(|| headers.get("cf-connecting-ip"))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
}

fn extract_user_agent(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

// Error handling

pub struct AppError(Error);

impl From<Error> for AppError {
    fn from(err: Error) -> Self {
        AppError(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.0.http_status()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        
        let body = Json(self.0.to_api_response());

        (status, body).into_response()
    }
}