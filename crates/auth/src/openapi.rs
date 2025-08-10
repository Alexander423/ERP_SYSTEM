use crate::dto::*;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "ERP System API",
        version = "1.0.0",
        description = "Enterprise Resource Planning System - Authentication & User Management API\n\n## Overview\n\nThis API provides comprehensive authentication and user management capabilities for a multi-tenant ERP system. It supports secure registration, login, two-factor authentication, and role-based access control.\n\n## Authentication\n\nThe API uses JWT-based authentication with both access and refresh tokens:\n- **Access tokens**: Short-lived (15-30 minutes) for API requests\n- **Refresh tokens**: Longer-lived (7-30 days) for token renewal\n- **2FA**: Optional TOTP-based two-factor authentication\n\n## Multi-Tenancy\n\nAll requests must include the `X-Tenant-Id` header for tenant isolation.\n\n## Rate Limiting\n\nAPI endpoints are rate-limited. Exceeded limits return HTTP 429.",
        contact(
            name = "ERP System Support",
            email = "support@erp-system.com"
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    servers(
        (url = "http://localhost:3000", description = "Development server"),
        (url = "https://api.erp-system.com", description = "Production server")
    ),
    paths(
        // register,
        // login,
        // verify_2fa,
        // refresh_token,
        // forgot_password,
        // reset_password,
        // verify_email,
        // resend_verification,
        // validate_reset_token,
        // logout,
        // list_users,
        // get_user,
        // invite_user,
        // update_user,
        // delete_user,
        // list_roles,
        // create_role,
        // get_role,
        // update_role,
        // delete_role,
        // list_permissions,
        // impersonate,
    ),
    components(
        schemas(
            RegisterRequest,
            RegistrationResponse,
            LoginRequest,
            LoginResponse,
            TwoFactorRequiredResponse,
            Verify2FARequest,
            ForgotPasswordRequest,
            ResetPasswordRequest,
            VerifyEmailRequest,
            ResendVerificationRequest,
            EmailVerificationResponse,
            PasswordResetResponse,
            TokenValidationResponse,
            InviteUserRequest,
            UpdateUserRequest,
            CreateRoleRequest,
            UpdateRoleRequest,
            ImpersonateRequest,
            UserResponse,
            RoleResponse,
            PermissionResponse,
        )
    ),
    tags(
        (name = "auth", description = "Authentication operations - Registration, login, 2FA, password reset"),
        (name = "users", description = "User management operations - CRUD, invitations, profile management"),
        (name = "roles", description = "Role and permission management - RBAC operations"),
        (name = "health", description = "System health and monitoring endpoints")
    ),
    security(
        ("bearer_auth" = []),
        ("tenant_header" = [])
    )
)]
pub struct AuthApiDoc;

pub fn bearer_auth() -> utoipa::openapi::security::SecurityScheme {
    utoipa::openapi::security::SecurityScheme::Http(
        utoipa::openapi::security::Http::new(
            utoipa::openapi::security::HttpAuthScheme::Bearer
        )
    )
}

/// Register a new tenant with admin user
#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "Registration successful", body = RegistrationResponse),
        (status = 400, description = "Invalid input"),
        (status = 409, description = "Email already exists"),
    ),
    tag = "auth"
)]
async fn register() {}

/// Login to the system
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    request_body = LoginRequest,
    params(
        ("X-Tenant-Id" = String, Header, description = "Tenant ID")
    ),
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 200, description = "2FA required", body = TwoFactorRequiredResponse),
        (status = 401, description = "Invalid credentials"),
        (status = 403, description = "Account locked or disabled"),
    ),
    tag = "auth"
)]
async fn login() {}

/// Verify 2FA code
#[utoipa::path(
    post,
    path = "/api/v1/auth/verify-2fa",
    request_body = Verify2FARequest,
    responses(
        (status = 200, description = "2FA verification successful", body = LoginResponse),
        (status = 401, description = "Invalid or expired code"),
    ),
    tag = "auth"
)]
async fn verify_2fa() {}

/// Refresh access token
#[utoipa::path(
    post,
    path = "/api/v1/auth/refresh-token",
    responses(
        (status = 200, description = "Token refreshed successfully"),
        (status = 401, description = "Invalid or expired refresh token"),
    ),
    tag = "auth",
    security(
        ("cookie_auth" = [])
    )
)]
async fn refresh_token() {}

/// Logout from the system
#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    responses(
        (status = 204, description = "Logout successful"),
    ),
    tag = "auth",
    security(
        ("bearer_auth" = [])
    )
)]
async fn logout() {}

/// List all users
#[utoipa::path(
    get,
    path = "/api/v1/users",
    params(
        ("page" = u32, Query, description = "Page number"),
        ("limit" = u32, Query, description = "Items per page"),
    ),
    responses(
        (status = 200, description = "Users retrieved successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Insufficient permissions"),
    ),
    tag = "users",
    security(
        ("bearer_auth" = ["user:read"])
    )
)]
async fn list_users() {}

/// Get user by ID
#[utoipa::path(
    get,
    path = "/api/v1/users/{id}",
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User retrieved successfully", body = UserResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "User not found"),
    ),
    tag = "users",
    security(
        ("bearer_auth" = ["user:read"])
    )
)]
async fn get_user() {}

/// Invite a new user
#[utoipa::path(
    post,
    path = "/api/v1/users/invite",
    request_body = InviteUserRequest,
    responses(
        (status = 201, description = "User invited successfully"),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Insufficient permissions"),
    ),
    tag = "users",
    security(
        ("bearer_auth" = ["user:create"])
    )
)]
async fn invite_user() {}

/// Update user information
#[utoipa::path(
    put,
    path = "/api/v1/users/{id}",
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "User updated successfully", body = UserResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "User not found"),
    ),
    tag = "users",
    security(
        ("bearer_auth" = ["user:update"])
    )
)]
async fn update_user() {}

/// Delete a user
#[utoipa::path(
    delete,
    path = "/api/v1/users/{id}",
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 204, description = "User deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "User not found"),
    ),
    tag = "users",
    security(
        ("bearer_auth" = ["user:delete"])
    )
)]
async fn delete_user() {}

/// List all roles
#[utoipa::path(
    get,
    path = "/api/v1/roles",
    responses(
        (status = 200, description = "Roles retrieved successfully", body = Vec<RoleResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Insufficient permissions"),
    ),
    tag = "roles",
    security(
        ("bearer_auth" = ["role:manage"])
    )
)]
async fn list_roles() {}

/// Create a new role
#[utoipa::path(
    post,
    path = "/api/v1/roles",
    request_body = CreateRoleRequest,
    responses(
        (status = 201, description = "Role created successfully", body = RoleResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Insufficient permissions"),
        (status = 409, description = "Role already exists"),
    ),
    tag = "roles",
    security(
        ("bearer_auth" = ["role:manage"])
    )
)]
async fn create_role() {}

/// Get role by ID
#[utoipa::path(
    get,
    path = "/api/v1/roles/{id}",
    params(
        ("id" = Uuid, Path, description = "Role ID")
    ),
    responses(
        (status = 200, description = "Role retrieved successfully", body = RoleResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Role not found"),
    ),
    tag = "roles",
    security(
        ("bearer_auth" = ["role:manage"])
    )
)]
async fn get_role() {}

/// Update role information
#[utoipa::path(
    put,
    path = "/api/v1/roles/{id}",
    params(
        ("id" = Uuid, Path, description = "Role ID")
    ),
    request_body = UpdateRoleRequest,
    responses(
        (status = 200, description = "Role updated successfully", body = RoleResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Role not found"),
    ),
    tag = "roles",
    security(
        ("bearer_auth" = ["role:manage"])
    )
)]
async fn update_role() {}

/// Delete a role
#[utoipa::path(
    delete,
    path = "/api/v1/roles/{id}",
    params(
        ("id" = Uuid, Path, description = "Role ID")
    ),
    responses(
        (status = 204, description = "Role deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Role not found"),
    ),
    tag = "roles",
    security(
        ("bearer_auth" = ["role:manage"])
    )
)]
async fn delete_role() {}

/// List all permissions
#[utoipa::path(
    get,
    path = "/api/v1/permissions",
    responses(
        (status = 200, description = "Permissions retrieved successfully", body = Vec<PermissionResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Insufficient permissions"),
    ),
    tag = "roles",
    security(
        ("bearer_auth" = ["permission:read"])
    )
)]
async fn list_permissions() {}

/// Start impersonating another user
#[utoipa::path(
    post,
    path = "/api/v1/auth/impersonate",
    request_body = ImpersonateRequest,
    responses(
        (status = 200, description = "Impersonation started successfully"),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "User not found"),
    ),
    tag = "auth",
    security(
        ("bearer_auth" = ["user:impersonate"])
    )
)]
async fn impersonate() {}

/// Request password reset
#[utoipa::path(
    post,
    path = "/api/v1/auth/forgot-password",
    request_body = ForgotPasswordRequest,
    params(
        ("X-Tenant-Id" = String, Header, description = "Tenant ID")
    ),
    responses(
        (status = 200, description = "Password reset email sent (if email exists)", body = PasswordResetResponse),
        (status = 400, description = "Invalid input"),
        (status = 429, description = "Rate limit exceeded"),
    ),
    tag = "auth"
)]
async fn forgot_password() {}

/// Reset password with token
#[utoipa::path(
    post,
    path = "/api/v1/auth/reset-password",
    request_body = ResetPasswordRequest,
    params(
        ("X-Tenant-Id" = String, Header, description = "Tenant ID")
    ),
    responses(
        (status = 200, description = "Password reset successful", body = PasswordResetResponse),
        (status = 400, description = "Invalid input or passwords don't match"),
        (status = 401, description = "Invalid or expired token"),
    ),
    tag = "auth"
)]
async fn reset_password() {}

/// Verify email address
#[utoipa::path(
    post,
    path = "/api/v1/auth/verify-email",
    request_body = VerifyEmailRequest,
    params(
        ("X-Tenant-Id" = String, Header, description = "Tenant ID")
    ),
    responses(
        (status = 200, description = "Email verification result", body = EmailVerificationResponse),
        (status = 400, description = "Invalid input"),
    ),
    tag = "auth"
)]
async fn verify_email() {}

/// Resend email verification
#[utoipa::path(
    post,
    path = "/api/v1/auth/resend-verification",
    request_body = ResendVerificationRequest,
    params(
        ("X-Tenant-Id" = String, Header, description = "Tenant ID")
    ),
    responses(
        (status = 200, description = "Verification email sent", body = EmailVerificationResponse),
        (status = 400, description = "Invalid input"),
        (status = 429, description = "Rate limit exceeded"),
    ),
    tag = "auth"
)]
async fn resend_verification() {}

/// Validate password reset token
#[utoipa::path(
    get,
    path = "/api/v1/auth/validate-reset-token/{token}",
    params(
        ("token" = String, Path, description = "Password reset token"),
        ("X-Tenant-Id" = String, Header, description = "Tenant ID")
    ),
    responses(
        (status = 200, description = "Token validation result", body = TokenValidationResponse),
        (status = 400, description = "Invalid input"),
    ),
    tag = "auth"
)]
async fn validate_reset_token() {}