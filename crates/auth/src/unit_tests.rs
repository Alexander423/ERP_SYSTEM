#[cfg(test)]
mod unit_tests {
    use crate::dto::*;
    use crate::models::*;
    use validator::Validate;
    use uuid::Uuid;

    #[test]
    fn test_register_request_validation() {
        let valid_request = RegisterRequest {
            company_name: "Test Company".to_string(),
            email: "admin@test.com".to_string(),
            password: "SecurePass123!".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
        };
        assert!(valid_request.validate().is_ok());

        // Invalid email
        let invalid_email = RegisterRequest {
            email: "invalid-email".to_string(),
            ..valid_request.clone()
        };
        assert!(invalid_email.validate().is_err());

        // Short password
        let short_password = RegisterRequest {
            password: "short".to_string(),
            ..valid_request.clone()
        };
        assert!(short_password.validate().is_err());

        // Empty company name
        let empty_company = RegisterRequest {
            company_name: "".to_string(),
            ..valid_request
        };
        assert!(empty_company.validate().is_err());
    }

    #[test]
    fn test_login_request_validation() {
        let valid_request = LoginRequest {
            email: "user@test.com".to_string(),
            password: "password123".to_string(),
        };
        assert!(valid_request.validate().is_ok());

        let invalid_email = LoginRequest {
            email: "not-an-email".to_string(),
            password: "password123".to_string(),
        };
        assert!(invalid_email.validate().is_err());
    }

    #[test]
    fn test_permission_model() {
        let permission = Permission::new("user", "read");
        assert_eq!(permission.resource, "user");
        assert_eq!(permission.action, "read");
        assert_eq!(permission.to_string(), "user:read");
    }

    #[test]
    fn test_user_model_methods() {
        use chrono::{Duration, Utc};

        let mut user = User {
            id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            password_hash: Some("hashed_password".to_string()),
            first_name: Some("John".to_string()),
            last_name: Some("Doe".to_string()),
            is_active: true,
            locked_until: None,
            email_verified_at: None,
            two_factor_secret_encrypted: None,
            two_factor_enabled_at: None,
            last_login_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Test full_name method
        assert_eq!(user.full_name(), "John Doe");

        user.first_name = None;
        assert_eq!(user.full_name(), "Doe");

        user.last_name = None;
        assert_eq!(user.full_name(), "test@example.com");

        // Test is_locked method
        assert!(!user.is_locked());

        user.locked_until = Some(Utc::now() + Duration::minutes(15));
        assert!(user.is_locked());

        user.locked_until = Some(Utc::now() - Duration::minutes(15));
        assert!(!user.is_locked());

        // Test has_2fa_enabled method
        assert!(!user.has_2fa_enabled());

        user.two_factor_secret_encrypted = Some("encrypted_secret".to_string());
        assert!(!user.has_2fa_enabled()); // Still need enabled_at

        user.two_factor_enabled_at = Some(Utc::now());
        assert!(user.has_2fa_enabled());
    }

    #[test]
    fn test_tenant_status_serialization() {
        use serde_json;

        let status = TenantStatus::Active;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"Active\"");

        let deserialized: TenantStatus = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, TenantStatus::Active));
    }
}