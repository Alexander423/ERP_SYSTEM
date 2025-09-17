use super::*;
use crate::security::{
    encryption::EncryptionService,
    access_control::{AccessControl, RoleBasedAccessControl, Permission, Resource, Action, AccessContext},
    audit::{AuditLogger, SecurityAuditLogger},
    data_masking::{DataMaskingService, MaskingPolicy, MaskingContext},
};
use uuid::Uuid;

#[tokio::test]
async fn test_field_level_encryption() {
    let encryption_service = EncryptionService::new().expect("Encryption service creation should succeed");

    let sensitive_data = "john.doe@example.com";
    let customer_id = Uuid::new_v4();
    let tenant_id = TenantId(Uuid::new_v4());

    // Test encryption
    let encrypted_result = encryption_service
        .encrypt_field(
            sensitive_data,
            "customers",
            "email",
            customer_id,
            tenant_id,
            crate::security::encryption::DataClassification::PersonalData,
        )
        .await
        .expect("Field encryption should succeed");

    assert_ne!(encrypted_result.encrypted_data, sensitive_data);
    assert!(!encrypted_result.nonce.is_empty());
    assert!(!encrypted_result.integrity_hash.is_empty());

    // Test decryption
    let decrypted_data = encryption_service
        .decrypt_field(
            &encrypted_result.encrypted_data,
            &encrypted_result.nonce,
            "customers",
            "email",
            customer_id,
            tenant_id,
            &encrypted_result.integrity_hash,
        )
        .await
        .expect("Field decryption should succeed");

    assert_eq!(decrypted_data, sensitive_data);
}

#[tokio::test]
async fn test_access_control_permissions() {
    let pool = create_test_pool().await;
    let ctx = TestContext::new(pool.clone()).await;

    let access_control = RoleBasedAccessControl::new(pool.clone());
    let user_id = Uuid::new_v4();
    let customer_id = Uuid::new_v4();

    // Create test permission
    let permission = Permission {
        id: Uuid::new_v4(),
        resource_type: "customers".to_string(),
        action: Action::Read,
        scope: "tenant".to_string(),
        field_restrictions: None,
        conditions: None,
        time_restrictions: None,
    };

    let resource = Resource {
        resource_type: "customers".to_string(),
        resource_id: customer_id,
        tenant_id: ctx.tenant_id,
        attributes: std::collections::HashMap::new(),
    };

    let access_context = AccessContext {
        user_id,
        tenant_id: ctx.tenant_id,
        session_id: Some("test-session".to_string()),
        ip_address: Some("127.0.0.1".parse().unwrap()),
        user_agent: Some("test-agent".to_string()),
        timestamp: Utc::now(),
        additional_context: std::collections::HashMap::new(),
    };

    // Test permission check without role assignment (should fail)
    let has_permission = access_control
        .check_permission(user_id, &permission, &resource, &access_context)
        .await
        .expect("Permission check should not error");

    assert!(!has_permission, "User should not have permission without role assignment");

    // Create and assign role with permission
    let role = crate::security::access_control::Role {
        id: Uuid::new_v4(),
        name: "customer_reader".to_string(),
        description: Some("Can read customer data".to_string()),
        priority: 1,
        is_system_role: false,
        is_active: true,
        tenant_id: ctx.tenant_id,
        permissions: vec![permission.clone()],
        metadata: std::collections::HashMap::new(),
        created_by: ctx.test_user_id,
        created_at: Utc::now(),
        modified_by: ctx.test_user_id,
        modified_at: Utc::now(),
        version: 1,
    };

    access_control
        .assign_role(user_id, &role, ctx.test_user_id)
        .await
        .expect("Role assignment should succeed");

    // Test permission check with role assignment (should succeed)
    let has_permission_after_role = access_control
        .check_permission(user_id, &permission, &resource, &access_context)
        .await
        .expect("Permission check should not error");

    assert!(has_permission_after_role, "User should have permission after role assignment");

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_audit_logging() {
    let pool = create_test_pool().await;
    let ctx = TestContext::new(pool.clone()).await;

    let audit_logger = SecurityAuditLogger::new(pool.clone());
    let user_id = Uuid::new_v4();
    let customer_id = Uuid::new_v4();

    // Test successful access audit
    let success_event = crate::security::audit::AuditEvent {
        event_type: "customer_access".to_string(),
        event_category: crate::security::audit::EventCategory::DataAccess,
        user_id: Some(user_id),
        tenant_id: ctx.tenant_id,
        resource_type: Some("customers".to_string()),
        resource_id: Some(customer_id),
        action: "read".to_string(),
        outcome: crate::security::audit::AuditOutcome::Success,
        risk_level: crate::security::audit::RiskLevel::Low,
        event_data: serde_json::json!({
            "customer_id": customer_id,
            "fields_accessed": ["id", "legal_name", "customer_number"]
        }),
        ip_address: Some("127.0.0.1".parse().unwrap()),
        user_agent: Some("test-agent".to_string()),
        session_id: Some("test-session".to_string()),
        correlation_id: Some(Uuid::new_v4()),
        source_system: "erp_system".to_string(),
        timestamp: Utc::now(),
        retention_until: None,
    };

    audit_logger
        .log_event(success_event.clone())
        .await
        .expect("Audit logging should succeed");

    // Test failed access audit
    let failed_event = crate::security::audit::AuditEvent {
        outcome: crate::security::audit::AuditOutcome::Failure,
        risk_level: crate::security::audit::RiskLevel::Medium,
        event_data: serde_json::json!({
            "customer_id": customer_id,
            "error": "insufficient_permissions",
            "attempted_action": "delete"
        }).as_object().unwrap().clone().into_iter().collect(),
        ..success_event
    };

    audit_logger
        .log_event(failed_event)
        .await
        .expect("Failed access audit logging should succeed");

    // Query audit logs
    let audit_events = audit_logger
        .get_events_for_user(user_id, ctx.tenant_id, None, None, 10, 0)
        .await
        .expect("Audit event retrieval should succeed");

    assert_eq!(audit_events.len(), 2);
    assert!(audit_events.iter().any(|e| e.outcome == crate::security::audit::AuditOutcome::Success));
    assert!(audit_events.iter().any(|e| e.outcome == crate::security::audit::AuditOutcome::Failure));

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_data_masking() {
    let pool = create_test_pool().await;
    let ctx = TestContext::new(pool.clone()).await;

    let masking_service = DataMaskingService::new(pool.clone());
    let user_id = Uuid::new_v4();

    // Create masking policy for email
    let email_policy = MaskingPolicy {
        id: Uuid::new_v4(),
        name: "email_masking".to_string(),
        description: Some("Mask email addresses for privacy".to_string()),
        table_name: "customers".to_string(),
        column_name: "email".to_string(),
        masking_type: crate::security::data_masking::MaskingType::PartialMasking,
        masking_config: serde_json::json!({
            "preserve_start": 2,
            "preserve_end": 0,
            "mask_char": "*",
            "preserve_domain": true
        }),
        conditions: Some(serde_json::json!({
            "user_role": "!admin"
        })),
        exemptions: Some(vec![
            crate::security::data_masking::MaskingExemption::Role("admin".to_string()),
            crate::security::data_masking::MaskingExemption::Permission("view_unmasked_pii".to_string()),
        ]),
        is_active: true,
        tenant_id: ctx.tenant_id,
        created_by: ctx.test_user_id,
        created_at: Utc::now(),
        modified_by: ctx.test_user_id,
        modified_at: Utc::now(),
    };

    masking_service
        .create_policy(email_policy.clone())
        .await
        .expect("Masking policy creation should succeed");

    // Test masking for regular user
    let masking_context = MaskingContext {
        user_id,
        tenant_id: ctx.tenant_id,
        user_roles: vec!["user".to_string()],
        user_permissions: vec![],
        request_context: std::collections::HashMap::new(),
    };

    let original_email = "john.doe@example.com";
    let masked_email = masking_service
        .mask_field(original_email, &email_policy, &masking_context)
        .await
        .expect("Email masking should succeed");

    assert_ne!(masked_email, original_email);
    assert!(masked_email.starts_with("jo"));
    assert!(masked_email.contains("@example.com"));
    assert!(masked_email.contains("*"));

    // Test no masking for admin user
    let admin_context = MaskingContext {
        user_id,
        tenant_id: ctx.tenant_id,
        user_roles: vec!["admin".to_string()],
        user_permissions: vec![],
        request_context: std::collections::HashMap::new(),
    };

    let unmasked_email = masking_service
        .mask_field(original_email, &email_policy, &admin_context)
        .await
        .expect("Email masking for admin should succeed");

    assert_eq!(unmasked_email, original_email);

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_security_policy_enforcement() {
    let pool = create_test_pool().await;
    let ctx = TestContext::new(pool.clone()).await;

    let access_control = RoleBasedAccessControl::new(pool.clone());
    let audit_logger = SecurityAuditLogger::new(pool.clone());

    let user_id = Uuid::new_v4();
    let customer_id = Uuid::new_v4();

    // Test time-based access restriction
    let time_restricted_permission = Permission {
        id: Uuid::new_v4(),
        resource_type: "customers".to_string(),
        action: Action::Delete,
        scope: "tenant".to_string(),
        field_restrictions: None,
        conditions: None,
        time_restrictions: Some(serde_json::json!({
            "allowed_hours": [9, 10, 11, 12, 13, 14, 15, 16, 17], // 9 AM to 5 PM
            "allowed_days": [1, 2, 3, 4, 5], // Monday to Friday
            "timezone": "UTC"
        })),
    };

    let resource = Resource {
        resource_type: "customers".to_string(),
        resource_id: customer_id,
        tenant_id: ctx.tenant_id,
        attributes: std::collections::HashMap::new(),
    };

    // Test access during restricted hours (assuming test runs outside business hours)
    let mut access_context = AccessContext {
        user_id,
        tenant_id: ctx.tenant_id,
        session_id: Some("security-test-session".to_string()),
        ip_address: Some("127.0.0.1".parse().unwrap()),
        user_agent: Some("security-test-agent".to_string()),
        timestamp: Utc::now(),
        additional_context: std::collections::HashMap::new(),
    };

    // Override timestamp to a weekend (should be denied)
    access_context.timestamp = chrono::DateTime::parse_from_rfc3339("2024-12-15T10:00:00Z")
        .unwrap()
        .with_timezone(&Utc); // Sunday

    let weekend_access = access_control
        .check_permission(user_id, &time_restricted_permission, &resource, &access_context)
        .await
        .expect("Time-restricted permission check should not error");

    assert!(!weekend_access, "Access should be denied on weekends");

    // Test conditional access based on resource attributes
    let conditional_permission = Permission {
        id: Uuid::new_v4(),
        resource_type: "customers".to_string(),
        action: Action::Update,
        scope: "tenant".to_string(),
        field_restrictions: None,
        conditions: Some(serde_json::json!({
            "resource_classification": "!confidential",
            "user_department": "sales"
        })),
        time_restrictions: None,
    };

    let confidential_resource = Resource {
        resource_type: "customers".to_string(),
        resource_id: customer_id,
        tenant_id: ctx.tenant_id,
        attributes: [("classification".to_string(), "confidential".to_string())]
            .iter()
            .cloned()
            .collect(),
    };

    let conditional_access = access_control
        .check_permission(user_id, &conditional_permission, &confidential_resource, &access_context)
        .await
        .expect("Conditional permission check should not error");

    assert!(!conditional_access, "Access to confidential resources should be denied");

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_sql_injection_prevention() {
    let pool = create_test_pool().await;
    let ctx = TestContext::new(pool.clone()).await;

    let repo = PostgresCustomerRepository::new(pool.clone(), ctx.tenant_id);

    // Test search with potential SQL injection
    let malicious_search = CustomerSearchCriteria {
        legal_name: Some("'; DROP TABLE customers; --".to_string()),
        customer_number: Some("1' OR '1'='1".to_string()),
        ..Default::default()
    };

    // This should not cause an error or SQL injection
    let search_result = repo
        .search(malicious_search, 0, 10)
        .await
        .expect("Search with malicious input should not cause SQL injection");

    // Should return no results (not everything)
    assert_eq!(search_result.customers.len(), 0);

    // Verify customers table still exists
    let count_result = sqlx::query("SELECT COUNT(*) as count FROM customers WHERE tenant_id = $1")
        .bind(ctx.tenant_id.0)
        .fetch_one(&pool)
        .await
        .expect("Table should still exist after injection attempt");

    // This verifies the table wasn't dropped
    assert!(count_result.try_get::<Option<i64>, _>("count").unwrap_or(Some(0)).is_some());

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_concurrent_access_control() {
    let pool = create_test_pool().await;
    let ctx = TestContext::new(pool.clone()).await;

    let access_control = RoleBasedAccessControl::new(pool.clone());

    // Test concurrent permission checks
    let user_id = Uuid::new_v4();
    let permission = Permission {
        id: Uuid::new_v4(),
        resource_type: "customers".to_string(),
        action: Action::Read,
        scope: "tenant".to_string(),
        field_restrictions: None,
        conditions: None,
        time_restrictions: None,
    };

    let resource = Resource {
        resource_type: "customers".to_string(),
        resource_id: Uuid::new_v4(),
        tenant_id: ctx.tenant_id,
        attributes: std::collections::HashMap::new(),
    };

    let access_context = AccessContext {
        user_id,
        tenant_id: ctx.tenant_id,
        session_id: Some("concurrent-test-session".to_string()),
        ip_address: Some("127.0.0.1".parse().unwrap()),
        user_agent: Some("concurrent-test-agent".to_string()),
        timestamp: Utc::now(),
        additional_context: std::collections::HashMap::new(),
    };

    // Launch multiple concurrent permission checks
    let futures = (0..10).map(|_| {
        let ac = access_control.clone();
        let perm = permission.clone();
        let res = resource.clone();
        let ctx = access_context.clone();

        tokio::spawn(async move {
            ac.check_permission(user_id, &perm, &res, &ctx).await
        })
    });

    let results = futures::future::join_all(futures).await;

    // All should complete without panicking
    for result in results {
        let permission_result = result
            .expect("Concurrent permission check should not panic")
            .expect("Permission check should not error");

        // All should return false (no role assigned)
        assert!(!permission_result);
    }

    ctx.cleanup().await;
}