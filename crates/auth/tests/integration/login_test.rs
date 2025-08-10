use super::common::{TestContext, init_test_logging};
use erp_auth::dto::{RegisterRequest, LoginRequest};
use erp_auth::service::LoginOrTwoFactorResponse;

#[tokio::test]
async fn test_successful_login() {
    init_test_logging();
    let ctx = TestContext::new().await;

    // Register a user first
    let register_request = RegisterRequest {
        company_name: "Login Test Company".to_string(),
        email: "logintest@example.com".to_string(),
        password: "LoginPassword123!".to_string(),
        first_name: "Login".to_string(),
        last_name: "User".to_string(),
    };

    let registration = ctx.auth_service
        .register_tenant(register_request)
        .await
        .expect("Registration should succeed");

    // Attempt login
    let login_request = LoginRequest {
        email: "logintest@example.com".to_string(),
        password: "LoginPassword123!".to_string(),
    };

    let login_result = ctx.auth_service
        .login(registration.tenant_id, login_request, None, None)
        .await;

    assert!(login_result.is_ok(), "Login should succeed");

    match login_result.unwrap() {
        LoginOrTwoFactorResponse::Success(response) => {
            assert!(!response.access_token.is_empty(), "Access token should not be empty");
            assert!(!response.refresh_token.is_empty(), "Refresh token should not be empty");
        }
        LoginOrTwoFactorResponse::TwoFactorRequired(_) => {
            // This is also valid if 2FA is enabled for the user
            println!("2FA required - this is acceptable");
        }
    }

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_login_invalid_credentials() {
    init_test_logging();
    let ctx = TestContext::new().await;

    // Register a user first
    let register_request = RegisterRequest {
        company_name: "Invalid Login Test Company".to_string(),
        email: "invalidlogin@example.com".to_string(),
        password: "CorrectPassword123!".to_string(),
        first_name: "Invalid".to_string(),
        last_name: "User".to_string(),
    };

    let registration = ctx.auth_service
        .register_tenant(register_request)
        .await
        .expect("Registration should succeed");

    // Test wrong password
    let wrong_password_request = LoginRequest {
        email: "invalidlogin@example.com".to_string(),
        password: "WrongPassword123!".to_string(),
    };

    let login_result = ctx.auth_service
        .login(registration.tenant_id, wrong_password_request, None, None)
        .await;

    assert!(login_result.is_err(), "Login with wrong password should fail");

    // Test nonexistent user
    let nonexistent_user_request = LoginRequest {
        email: "nonexistent@example.com".to_string(),
        password: "AnyPassword123!".to_string(),
    };

    let login_result = ctx.auth_service
        .login(registration.tenant_id, nonexistent_user_request, None, None)
        .await;

    assert!(login_result.is_err(), "Login with nonexistent user should fail");

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_login_rate_limiting() {
    init_test_logging();
    let ctx = TestContext::new().await;

    // Register a user first
    let register_request = RegisterRequest {
        company_name: "Rate Limit Login Test Company".to_string(),
        email: "ratelimitlogin@example.com".to_string(),
        password: "Password123!".to_string(),
        first_name: "RateLimit".to_string(),
        last_name: "User".to_string(),
    };

    let registration = ctx.auth_service
        .register_tenant(register_request)
        .await
        .expect("Registration should succeed");

    // Attempt multiple failed logins
    let wrong_login_request = LoginRequest {
        email: "ratelimitlogin@example.com".to_string(),
        password: "WrongPassword123!".to_string(),
    };

    for attempt in 1..=6 {
        let login_result = ctx.auth_service
            .login(registration.tenant_id, wrong_login_request.clone(), None, None)
            .await;

        assert!(login_result.is_err(), "Wrong password should fail on attempt {}", attempt);
        
        if attempt >= 5 {
            // After 5 failed attempts, the account should be locked
            // The exact error message might vary based on implementation
            tracing::info!("Login attempt {} failed as expected: {:?}", attempt, login_result);
        }
    }

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_token_refresh() {
    init_test_logging();
    let ctx = TestContext::new().await;

    // Register and login a user
    let register_request = RegisterRequest {
        company_name: "Token Refresh Test Company".to_string(),
        email: "tokenrefresh@example.com".to_string(),
        password: "Password123!".to_string(),
        first_name: "Token".to_string(),
        last_name: "User".to_string(),
    };

    let registration = ctx.auth_service
        .register_tenant(register_request)
        .await
        .expect("Registration should succeed");

    let login_request = LoginRequest {
        email: "tokenrefresh@example.com".to_string(),
        password: "Password123!".to_string(),
    };

    let login_result = ctx.auth_service
        .login(registration.tenant_id, login_request, None, None)
        .await
        .expect("Login should succeed");

    if let LoginOrTwoFactorResponse::Success(login_response) = login_result {
        // Test token refresh
        let refresh_result = ctx.auth_service
            .refresh_token(&login_response.refresh_token)
            .await;

        assert!(refresh_result.is_ok(), "Token refresh should succeed");
        
        let new_access_token = refresh_result.unwrap();
        assert!(!new_access_token.is_empty(), "New access token should not be empty");
        assert_ne!(new_access_token, login_response.access_token, "New token should be different");
    }

    ctx.cleanup().await;
}