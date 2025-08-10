use super::common::{TestContext, init_test_logging};
use erp_auth::dto::*;

#[tokio::test]
async fn test_email_verification_flow() {
    init_test_logging();
    let ctx = TestContext::new().await;
    
    // Register a user first
    let register_request = RegisterRequest {
        company_name: "Email Test Company".to_string(),
        email: "emailtest@example.com".to_string(),
        password: "EmailPassword123!".to_string(),
        first_name: "Email".to_string(),
        last_name: "User".to_string(),
    };

    let registration = ctx.auth_service
        .register_tenant(register_request)
        .await
        .expect("Registration should succeed");

    // Test email verification
    let verify_request = VerifyEmailRequest {
        token: "test_verification_token".to_string(),
        client_ip: Some("127.0.0.1".to_string()),
    };

    let verify_result = ctx.auth_service
        .verify_email(registration.tenant_id, verify_request)
        .await;

    // Since we don't have a real token, this should fail, but it tests the API
    assert!(verify_result.is_err(), "Invalid token should fail verification");

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_resend_verification() {
    init_test_logging();
    let ctx = TestContext::new().await;
    
    // Register a user first
    let register_request = RegisterRequest {
        company_name: "Resend Test Company".to_string(),
        email: "resendtest@example.com".to_string(),
        password: "ResendPassword123!".to_string(),
        first_name: "Resend".to_string(),
        last_name: "User".to_string(),
    };

    let registration = ctx.auth_service
        .register_tenant(register_request)
        .await
        .expect("Registration should succeed");

    // Test resending verification
    let resend_result = ctx.auth_service
        .resend_verification_email(
            registration.tenant_id,
            registration.user_id,
            Some("127.0.0.1".to_string())
        )
        .await;

    assert!(resend_result.is_ok(), "Resend verification should succeed");

    ctx.cleanup().await;
}