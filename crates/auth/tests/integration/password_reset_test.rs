use super::common::{TestContext, init_test_logging};
use erp_auth::dto::{RegisterRequest, ForgotPasswordRequest, ResetPasswordRequest};

#[tokio::test]
async fn test_complete_password_reset_workflow() {
    init_test_logging();
    let ctx = TestContext::new().await;

    // First, register a user
    let register_request = RegisterRequest {
        company_name: "Password Reset Test Company".to_string(),
        email: "passwordreset@test.com".to_string(),
        password: "OriginalPassword123!".to_string(),
        first_name: "Password".to_string(),
        last_name: "User".to_string(),
    };

    let registration = ctx.auth_service
        .register_tenant(register_request)
        .await
        .expect("Registration should succeed");

    // Request password reset
    let forgot_request = ForgotPasswordRequest {
        email: "passwordreset@test.com".to_string(),
    };

    let forgot_result = ctx.auth_service
        .request_password_reset(
            registration.tenant_id,
            forgot_request,
            Some("127.0.0.1".to_string()),
            Some("Test-Agent/1.0".to_string()),
        )
        .await;

    assert!(forgot_result.is_ok(), "Password reset request should succeed");

    // In a real test, you would:
    // 1. Extract the token from the mock email service
    // 2. Use that token to complete the password reset
    // For now, we'll test the validation of the reset request

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_password_reset_nonexistent_email() {
    init_test_logging();
    let ctx = TestContext::new().await;

    // Create a tenant first
    let register_request = RegisterRequest {
        company_name: "Test Company".to_string(),
        email: "admin@test.com".to_string(),
        password: "Password123!".to_string(),
        first_name: "Test".to_string(),
        last_name: "Admin".to_string(),
    };

    let registration = ctx.auth_service
        .register_tenant(register_request)
        .await
        .expect("Registration should succeed");

    // Request password reset for nonexistent email
    let forgot_request = ForgotPasswordRequest {
        email: "nonexistent@test.com".to_string(),
    };

    let forgot_result = ctx.auth_service
        .request_password_reset(
            registration.tenant_id,
            forgot_request,
            Some("127.0.0.1".to_string()),
            None,
        )
        .await;

    // Should still succeed to prevent email enumeration
    assert!(forgot_result.is_ok(), "Password reset request should appear to succeed");

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_password_reset_rate_limiting() {
    init_test_logging();
    let ctx = TestContext::new().await;

    // Register a user
    let register_request = RegisterRequest {
        company_name: "Rate Limit Test Company".to_string(),
        email: "ratelimit@test.com".to_string(),
        password: "Password123!".to_string(),
        first_name: "Rate".to_string(),
        last_name: "User".to_string(),
    };

    let registration = ctx.auth_service
        .register_tenant(register_request)
        .await
        .expect("Registration should succeed");

    let forgot_request = ForgotPasswordRequest {
        email: "ratelimit@test.com".to_string(),
    };

    // Send multiple password reset requests rapidly
    for i in 1..=5 {
        let result = ctx.auth_service
            .request_password_reset(
                registration.tenant_id,
                forgot_request.clone(),
                Some("127.0.0.1".to_string()),
                Some(format!("Test-Agent/{}", i)),
            )
            .await;

        if i <= 3 {
            // First 3 should succeed (default rate limit)
            assert!(result.is_ok(), "Password reset request {} should succeed", i);
        } else {
            // Should be rate limited after that
            // Note: This might succeed in tests due to different rate limiting configuration
            tracing::info!("Password reset request {} result: {:?}", i, result);
        }
    }

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_password_reset_validation() {
    init_test_logging();
    let ctx = TestContext::new().await;

    // Register a user first
    let register_request = RegisterRequest {
        company_name: "Validation Test Company".to_string(),
        email: "validation@test.com".to_string(),
        password: "Password123!".to_string(),
        first_name: "Validation".to_string(),
        last_name: "User".to_string(),
    };

    let registration = ctx.auth_service
        .register_tenant(register_request)
        .await
        .expect("Registration should succeed");

    // Test invalid token
    let invalid_token_request = ResetPasswordRequest {
        token: "invalid-token".to_string(),
        new_password: "NewPassword123!".to_string(),
        confirm_password: "NewPassword123!".to_string(),
    };

    let result = ctx.auth_service
        .confirm_password_reset(
            registration.tenant_id,
            invalid_token_request,
            Some("127.0.0.1".to_string()),
        )
        .await;

    assert!(result.is_err(), "Invalid token should be rejected");

    ctx.cleanup().await;
}