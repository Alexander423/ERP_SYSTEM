use super::common::{TestContext, init_test_logging};
use erp_auth::dto::RegisterRequest;
use validator::Validate;

#[tokio::test]
async fn test_complete_registration_workflow() {
    init_test_logging();
    let ctx = TestContext::new().await;

    // Test registration request
    let register_request = RegisterRequest {
        company_name: "Integration Test Company".to_string(),
        email: "admin@integrationtest.com".to_string(),
        password: "SecurePassword123!".to_string(),
        first_name: "Integration".to_string(),
        last_name: "Admin".to_string(),
    };

    // Validate request
    assert!(register_request.validate().is_ok(), "Registration request should be valid");

    // Register tenant
    let registration_result = ctx.auth_service
        .register_tenant(register_request)
        .await;

    assert!(registration_result.is_ok(), "Registration should succeed");
    
    let registration_response = registration_result.unwrap();
    assert!(!registration_response.user_id.to_string().is_empty(), "User ID should be returned");
    assert!(!registration_response.tenant_id.to_string().is_empty(), "Tenant ID should be returned");
    assert!(registration_response.message.contains("verification"), "Should mention email verification");

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_registration_validation() {
    init_test_logging();
    let ctx = TestContext::new().await;

    // Test invalid email
    let invalid_email_request = RegisterRequest {
        company_name: "Test Company".to_string(),
        email: "invalid-email".to_string(),
        password: "SecurePassword123!".to_string(),
        first_name: "Test".to_string(),
        last_name: "User".to_string(),
    };

    let result = ctx.auth_service.register_tenant(invalid_email_request).await;
    assert!(result.is_err(), "Should reject invalid email");

    // Test weak password
    let weak_password_request = RegisterRequest {
        company_name: "Test Company".to_string(),
        email: "test@example.com".to_string(),
        password: "weak".to_string(),
        first_name: "Test".to_string(),
        last_name: "User".to_string(),
    };

    let result = ctx.auth_service.register_tenant(weak_password_request).await;
    assert!(result.is_err(), "Should reject weak password");

    // Test empty company name
    let empty_company_request = RegisterRequest {
        company_name: "".to_string(),
        email: "test@example.com".to_string(),
        password: "SecurePassword123!".to_string(),
        first_name: "Test".to_string(),
        last_name: "User".to_string(),
    };

    let result = ctx.auth_service.register_tenant(empty_company_request).await;
    assert!(result.is_err(), "Should reject empty company name");

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_duplicate_email_registration() {
    init_test_logging();
    let ctx = TestContext::new().await;

    let register_request = RegisterRequest {
        company_name: "First Company".to_string(),
        email: "duplicate@test.com".to_string(),
        password: "SecurePassword123!".to_string(),
        first_name: "First".to_string(),
        last_name: "User".to_string(),
    };

    // First registration should succeed
    let first_result = ctx.auth_service
        .register_tenant(register_request.clone())
        .await;
    assert!(first_result.is_ok(), "First registration should succeed");

    // Second registration with same email should fail
    let duplicate_request = RegisterRequest {
        company_name: "Second Company".to_string(),
        email: "duplicate@test.com".to_string(),
        password: "AnotherPassword123!".to_string(),
        first_name: "Second".to_string(),
        last_name: "User".to_string(),
    };

    let second_result = ctx.auth_service
        .register_tenant(duplicate_request)
        .await;
    assert!(second_result.is_err(), "Duplicate email registration should fail");

    ctx.cleanup().await;
}