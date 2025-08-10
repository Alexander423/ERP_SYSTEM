use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct RegisterRequest {
    #[validate(length(min = 1, max = 255))]
    pub company_name: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
    #[validate(length(min = 1, max = 100))]
    pub first_name: String,
    #[validate(length(min = 1, max = 100))]
    pub last_name: String,
}

#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
    pub access_token: String,
    #[serde(skip_serializing)]
    pub refresh_token: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TwoFactorRequiredResponse {
    pub two_factor_required: bool,
    pub login_session_token: String,
}

#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct Verify2FARequest {
    pub login_session_token: String,
    #[validate(length(equal = 6))]
    pub code: String,
}

#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct ForgotPasswordRequest {
    #[validate(email)]
    pub email: String,
}

#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct ResetPasswordRequest {
    pub token: String,
    #[validate(length(min = 8))]
    pub new_password: String,
    #[validate(length(min = 8))]
    pub confirm_password: String,
}

#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct VerifyEmailRequest {
    pub token: String,
    pub client_ip: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct ResendVerificationRequest {
    pub user_id: Uuid,
    pub client_ip: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EmailVerificationResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PasswordResetResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TokenValidationResponse {
    pub valid: bool,
    pub user_email: Option<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct InviteUserRequest {
    #[validate(email)]
    pub email: String,
    pub role_ids: Vec<Uuid>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct UpdateUserRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct CreateRoleRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    pub description: Option<String>,
    pub permission_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct UpdateRoleRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub permission_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ImpersonateRequest {
    pub user_id: Uuid,
    pub reason: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub is_active: bool,
    pub email_verified: bool,
    pub two_factor_enabled: bool,
    pub roles: Vec<RoleResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RoleResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_editable: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PermissionResponse {
    pub id: Uuid,
    pub resource: String,
    pub action: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RegistrationResponse {
    pub message: String,
    pub tenant_id: Uuid,
    pub user_id: Uuid,
}

// Role assignment DTOs
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct AssignRoleRequest {
    #[validate(length(min = 1, max = 10, message = "Must specify between 1 and 10 roles"))]
    pub role_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct RemoveRoleRequest {
    #[validate(length(min = 1, max = 10, message = "Must specify between 1 and 10 roles"))]
    pub role_ids: Vec<Uuid>,
}

// 2FA Management DTOs
#[derive(Debug, Serialize, ToSchema)]
pub struct Enable2FAResponse {
    pub secret: String,
    pub qr_code: String,
    pub backup_codes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct Disable2FARequest {
    #[validate(length(equal = 6, message = "2FA code must be exactly 6 digits"))]
    pub code: String,
    #[validate(length(max = 255, message = "Reason cannot exceed 255 characters"))]
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Disable2FAResponse {
    pub success: bool,
    pub message: String,
}

// Impersonation management DTOs
#[derive(Debug, Deserialize, ToSchema)]
pub struct StopImpersonationRequest {
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct StopImpersonationResponse {
    pub success: bool,
    pub message: String,
    pub original_user_id: Uuid,
}

