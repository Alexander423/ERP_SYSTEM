//! Authentication handlers
//!
//! HTTP handlers for authentication endpoints including login, register, 2FA, etc.

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{post, Router},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_in: Option<i64>,
    pub requires_2fa: Option<bool>,
    pub session_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub company_name: String,
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Debug, Deserialize)]
pub struct Verify2FARequest {
    pub session_token: String,
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
    pub confirm_password: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyEmailRequest {
    pub token: String,
}

/// Create authentication routes
pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/verify-2fa", post(verify_2fa))
        .route("/refresh-token", post(refresh_token))
        .route("/forgot-password", post(forgot_password))
        .route("/reset-password", post(reset_password))
        .route("/verify-email", post(verify_email))
        .route("/logout", post(logout))
}

/// Register a new tenant and admin user
async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<Value>, StatusCode> {
    // Call the auth service to register a new tenant
    match state.auth_service.register_tenant(erp_auth::dto::RegisterRequest {
        company_name: payload.company_name,
        email: payload.email,
        password: payload.password,
        first_name: payload.first_name,
        last_name: payload.last_name,
    }).await {
        Ok(response) => Ok(Json(json!({
            "success": true,
            "tenant_id": response.tenant_id,
            "user_id": response.user_id,
            "message": response.message
        }))),
        Err(e) => {
            tracing::error!("Registration failed: {}", e);
            Ok(Json(json!({
                "success": false,
                "error": "Registration failed",
                "message": e.to_string()
            })))
        }
    }
}

/// User login
async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    // Validate input
    if payload.email.is_empty() || payload.password.is_empty() {
        return Ok(Json(LoginResponse {
            success: false,
            access_token: None,
            refresh_token: None,
            expires_in: None,
            requires_2fa: None,
            session_token: None,
        }));
    }

    // For now, use a default tenant ID (in production, this would come from subdomain or header)
    let tenant_id = uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000")
        .unwrap_or_else(|_| uuid::Uuid::new_v4());

    // Create login request for auth service
    let auth_request = erp_auth::dto::LoginRequest {
        email: payload.email,
        password: payload.password,
    };

    // Call the auth service
    match state.auth_service.login(tenant_id, auth_request, None, None).await {
        Ok(response) => {
            match response {
                erp_auth::LoginOrTwoFactorResponse::Success(login_resp) => {
                    Ok(Json(LoginResponse {
                        success: true,
                        access_token: Some(login_resp.access_token),
                        refresh_token: Some(login_resp.refresh_token),
                        expires_in: Some(1800), // 30 minutes
                        requires_2fa: Some(false),
                        session_token: None,
                    }))
                },
                erp_auth::LoginOrTwoFactorResponse::TwoFactorRequired(tfa_resp) => {
                    Ok(Json(LoginResponse {
                        success: true,
                        access_token: None,
                        refresh_token: None,
                        expires_in: None,
                        requires_2fa: Some(true),
                        session_token: Some(tfa_resp.login_session_token),
                    }))
                }
            }
        },
        Err(_) => {
            // Authentication failed
            Ok(Json(LoginResponse {
                success: false,
                access_token: None,
                refresh_token: None,
                expires_in: None,
                requires_2fa: None,
                session_token: None,
            }))
        }
    }
}

/// Verify 2FA token
async fn verify_2fa(
    State(state): State<AppState>,
    Json(payload): Json<Verify2FARequest>,
) -> Result<Json<Value>, StatusCode> {
    // Create verification request for auth service
    let verify_request = erp_auth::dto::Verify2FARequest {
        login_session_token: payload.session_token,
        code: payload.code,
    };

    // Call the auth service
    match state.auth_service.verify_2fa(verify_request).await {
        Ok(response) => Ok(Json(json!({
            "success": true,
            "access_token": response.access_token,
            "refresh_token": response.refresh_token,
            "expires_in": 1800
        }))),
        Err(e) => {
            tracing::error!("2FA verification failed: {}", e);
            Ok(Json(json!({
                "success": false,
                "error": "Invalid 2FA code"
            })))
        }
    }
}

/// Refresh access token
async fn refresh_token(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<Json<Value>, StatusCode> {
    // Call the auth service to refresh the token
    match state.auth_service.refresh_token(&payload.refresh_token).await {
        Ok(token_pair) => Ok(Json(json!({
            "success": true,
            "access_token": token_pair.access_token,
            "refresh_token": token_pair.refresh_token,
            "expires_in": 1800
        }))),
        Err(e) => {
            tracing::error!("Token refresh failed: {}", e);
            Ok(Json(json!({
                "success": false,
                "error": "Invalid or expired refresh token"
            })))
        }
    }
}

/// Request password reset
async fn forgot_password(
    State(state): State<AppState>,
    Json(payload): Json<ForgotPasswordRequest>,
) -> Result<Json<Value>, StatusCode> {
    // For now, use a default tenant ID (in production, this would come from subdomain or header)
    let tenant_id = uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000")
        .unwrap_or_else(|_| uuid::Uuid::new_v4());

    // Create forgot password request for auth service
    let forgot_request = erp_auth::dto::ForgotPasswordRequest {
        email: payload.email,
    };

    // Call the auth service
    match state.auth_service.request_password_reset(tenant_id, forgot_request, None, None).await {
        Ok(_) => Ok(Json(json!({
            "success": true,
            "message": "If an account exists with this email, a password reset link has been sent."
        }))),
        Err(e) => {
            // Don't reveal if email exists or not for security
            tracing::error!("Password reset request failed: {}", e);
            Ok(Json(json!({
                "success": true,
                "message": "If an account exists with this email, a password reset link has been sent."
            })))
        }
    }
}

/// Reset password with token
async fn reset_password(
    State(state): State<AppState>,
    Json(payload): Json<ResetPasswordRequest>,
) -> Result<Json<Value>, StatusCode> {
    // Validate passwords match
    if payload.new_password != payload.confirm_password {
        return Ok(Json(json!({
            "success": false,
            "error": "Passwords do not match"
        })));
    }

    // For now, use a default tenant ID (in production, this would come from subdomain or header)
    let tenant_id = uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000")
        .unwrap_or_else(|_| uuid::Uuid::new_v4());

    // Create reset password request for auth service
    let reset_request = erp_auth::dto::ResetPasswordRequest {
        token: payload.token,
        new_password: payload.new_password,
        confirm_password: payload.confirm_password,
    };

    // Call the auth service
    match state.auth_service.confirm_password_reset(tenant_id, reset_request, None).await {
        Ok(_) => Ok(Json(json!({
            "success": true,
            "message": "Password has been successfully reset. You can now log in with your new password."
        }))),
        Err(e) => {
            tracing::error!("Password reset failed: {}", e);
            Ok(Json(json!({
                "success": false,
                "error": "Invalid or expired reset token"
            })))
        }
    }
}

/// Verify email address
async fn verify_email(
    State(state): State<AppState>,
    Json(payload): Json<VerifyEmailRequest>,
) -> Result<Json<Value>, StatusCode> {
    // For now, use a default tenant ID (in production, this would come from subdomain or header)
    let tenant_id = uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000")
        .unwrap_or_else(|_| uuid::Uuid::new_v4());

    // Create verify email request for auth service
    let verify_request = erp_auth::dto::VerifyEmailRequest {
        token: payload.token,
        client_ip: None,
    };

    // Call the auth service
    match state.auth_service.verify_email(tenant_id, verify_request).await {
        Ok(_) => Ok(Json(json!({
            "success": true,
            "message": "Email has been successfully verified. You can now log in."
        }))),
        Err(e) => {
            tracing::error!("Email verification failed: {}", e);
            Ok(Json(json!({
                "success": false,
                "error": "Invalid or expired verification token"
            })))
        }
    }
}

/// User logout
async fn logout(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    // In production, we would get the session ID from the token
    // For now, just return success
    // TODO: Implement proper session extraction from JWT and call state.auth_service.logout(session_id)
    Ok(Json(json!({
        "success": true,
        "message": "Logged out successfully"
    })))
}