#[cfg(test)]
mod tests {
    use crate::security::*;
    use crate::config::*;
    use crate::{Permission, TenantId};

    #[test]
    fn test_password_validation() {
        use crate::utils::validate_password;

        // Valid passwords
        assert!(validate_password("SecurePass123!").is_ok());
        assert!(validate_password("Complex@Pass2024").is_ok());

        // Invalid passwords
        assert!(validate_password("short").is_err());
        assert!(validate_password("nouppercase123!").is_err());
        assert!(validate_password("NOLOWERCASE123!").is_err());
        assert!(validate_password("NoNumbers!").is_err());
        assert!(validate_password("NoSpecialChars123").is_err());
    }

    #[test]
    fn test_email_validation() {
        use crate::utils::validate_email;

        // Valid emails
        assert!(validate_email("user@example.com"));
        assert!(validate_email("test.email+tag@domain.co.uk"));

        // Invalid emails
        assert!(!validate_email("invalid-email"));
        assert!(!validate_email("@domain.com"));
        assert!(!validate_email("user@"));
    }

    #[test]
    fn test_schema_name_generation() {
        use crate::utils::generate_schema_name;

        let schema1 = generate_schema_name();
        let schema2 = generate_schema_name();

        assert!(schema1.starts_with("tenant_"));
        assert!(schema2.starts_with("tenant_"));
        assert_ne!(schema1, schema2); // Should be unique
        assert_eq!(schema1.len(), 15); // "tenant_" + 8 chars
    }

    #[tokio::test]
    async fn test_password_hashing() {
        let config = SecurityConfig {
            argon2_memory_cost: 65536,
            argon2_time_cost: 3,
            argon2_parallelism: 4,
            aes_encryption_key: "12345678901234567890123456789012".to_string(),
        };

        let hasher = PasswordHasher::new(&config).unwrap();
        let password = "TestPassword123!";

        let hash = hasher.hash_password(password).unwrap();
        assert!(!hash.is_empty());
        assert_ne!(hash, password);

        // Verify correct password
        assert!(hasher.verify_password(password, &hash).unwrap());

        // Verify incorrect password
        assert!(!hasher.verify_password("WrongPassword", &hash).unwrap());
    }

    #[test]
    fn test_encryption_service() {
        let config = SecurityConfig {
            argon2_memory_cost: 65536,
            argon2_time_cost: 3,
            argon2_parallelism: 4,
            aes_encryption_key: "12345678901234567890123456789012".to_string(),
        };

        let service = EncryptionService::new(&config).unwrap();
        let plaintext = "This is a secret message";

        let encrypted = service.encrypt_string(plaintext).unwrap();
        assert!(!encrypted.is_empty());
        assert_ne!(encrypted, plaintext);

        let decrypted = service.decrypt_string(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_permission_creation() {
        let permission = Permission::new("user", "read");
        assert_eq!(permission.resource, "user");
        assert_eq!(permission.action, "read");
        assert_eq!(permission.to_string(), "user:read");
    }

    #[test]
    fn test_tenant_id_serialization() {
        use serde_json;
        use uuid::Uuid;

        let tenant_id = TenantId(Uuid::new_v4());
        let serialized = serde_json::to_string(&tenant_id).unwrap();
        let deserialized: TenantId = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(tenant_id.0, deserialized.0);
    }
}