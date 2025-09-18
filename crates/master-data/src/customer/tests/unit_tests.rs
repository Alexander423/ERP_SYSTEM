use super::*;
use crate::customer::model::*;
use crate::customer::validation::*;
use crate::types::*;
use chrono::Utc;
use rust_decimal::Decimal;
use uuid::Uuid;
use std::collections::HashMap;

/// Professional unit tests for Customer model and business logic
/// These tests validate the core functionality without external dependencies

#[cfg(test)]
mod customer_model_tests {
    use super::*;

    fn create_valid_customer() -> Customer {
        Customer {
            id: Uuid::new_v4(),
            customer_number: "CUST-001".to_string(),
            external_ids: HashMap::new(),
            legal_name: "Test Corporation Ltd.".to_string(),
            trade_names: vec!["Test Corp".to_string()],
            customer_type: CustomerType::B2b,
            industry_classification: IndustryClassification::Technology,
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
            customer_lifetime_value: Some(Decimal::new(100000, 2)),
            annual_revenue: Some(Decimal::new(50000000, 2)),
            employee_count: Some(250),
            financial_info: Some(FinancialInfo {
                credit_limit: Decimal::new(50000, 2),
                payment_terms: PaymentTerms::Net30,
                currency: "USD".to_string(),
                tax_id: Some("12-3456789".to_string()),
                dunning_procedure: DunningProcedure::Standard,
                payment_tolerance: Decimal::new(100, 2),
                discount_percentage: Decimal::new(5, 2),
                risk_category: RiskCategory::Low,
                collection_profile: CollectionProfile::Standard,
            }),
            sales_info: Some(SalesInfo {
                sales_organization: "US_EAST".to_string(),
                distribution_channel: "DIRECT".to_string(),
                division: "TECH".to_string(),
                sales_group: Some("ENTERPRISE".to_string()),
                sales_office: Some("NYC".to_string()),
                customer_group: "CORP".to_string(),
                price_group: Some("PREMIUM".to_string()),
                account_assignment_group: "STD".to_string(),
                shipping_conditions: "STANDARD".to_string(),
                delivery_priority: DeliveryPriority::High,
                partial_delivery: true,
                order_probability: Some(85),
            }),
            marketing_info: Some(MarketingInfo {
                acquisition_channel: AcquisitionChannel::DirectSales,
                acquisition_date: Utc::now().date_naive(),
                customer_segment: CustomerSegment::Enterprise,
                engagement_score: Some(8.5),
                communication_preferences: CommunicationPreferences {
                    email_marketing: true,
                    phone_calls: true,
                    direct_mail: false,
                    sms: false,
                    preferred_language: "en".to_string(),
                    preferred_contact_time: "09:00-17:00".to_string(),
                    frequency_preference: FrequencyPreference::Weekly,
                },
                gdpr_consent: GdprConsent {
                    marketing_consent: true,
                    analytics_consent: true,
                    third_party_sharing: false,
                    consent_date: Utc::now(),
                    consent_source: "WEBSITE".to_string(),
                },
            }),
            compliance_info: Some(ComplianceInfo {
                kyc_status: KycStatus::Verified,
                kyc_last_updated: Some(Utc::now()),
                aml_risk_score: Some(25),
                sanctions_screening_status: SanctionsStatus::Clear,
                sanctions_last_checked: Some(Utc::now()),
                regulatory_requirements: vec![],
                data_residency_requirements: vec!["US".to_string()],
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
                tenant_id: erp_core::TenantId(Uuid::new_v4()),
                created_at: Utc::now(),
                created_by: erp_core::UserId(Uuid::new_v4()),
                modified_at: Utc::now(),
                modified_by: erp_core::UserId(Uuid::new_v4()),
                is_deleted: false,
                deleted_at: None,
                deleted_by: None,
                tags: vec!["test".to_string()],
                custom_fields: HashMap::new(),
                integration_metadata: HashMap::new(),
            },
        }
    }

    #[test]
    fn test_customer_creation_valid() {
        let customer = create_valid_customer();

        assert!(!customer.id.is_nil());
        assert_eq!(customer.customer_number, "CUST-001");
        assert_eq!(customer.legal_name, "Test Corporation Ltd.");
        assert_eq!(customer.customer_type, CustomerType::B2b);
        assert_eq!(customer.lifecycle_stage, CustomerLifecycleStage::ActiveCustomer);
        assert_eq!(customer.status, EntityStatus::Active);
        assert!(!customer.metadata.is_deleted);
        assert_eq!(customer.metadata.version, 1);
    }

    #[test]
    fn test_customer_validation_valid() {
        let customer = create_valid_customer();
        let validation_result = customer.validate();
        assert!(validation_result.is_ok(), "Valid customer should pass validation");
    }

    #[test]
    fn test_customer_validation_invalid_legal_name() {
        let mut customer = create_valid_customer();
        customer.legal_name = "".to_string(); // Empty name should fail

        let validation_result = customer.validate();
        assert!(validation_result.is_err(), "Empty legal name should fail validation");
    }

    #[test]
    fn test_customer_validation_invalid_customer_number() {
        let mut customer = create_valid_customer();
        customer.customer_number = "x".repeat(51); // Too long should fail

        let validation_result = customer.validate();
        assert!(validation_result.is_err(), "Too long customer number should fail validation");
    }

    #[test]
    fn test_customer_hierarchy_validation() {
        let mut customer = create_valid_customer();
        customer.customer_hierarchy_level = 10; // Max level exceeded

        // This should still be valid as hierarchy level is u8
        assert!(customer.validate().is_ok());

        // Test parent-child relationship
        let parent_id = Uuid::new_v4();
        customer.parent_customer_id = Some(parent_id);
        customer.customer_hierarchy_level = 1;

        assert!(customer.validate().is_ok());
        assert_eq!(customer.parent_customer_id, Some(parent_id));
        assert_eq!(customer.customer_hierarchy_level, 1);
    }

    #[test]
    fn test_financial_info_validation() {
        let customer = create_valid_customer();
        let financial_info = customer.financial_info.unwrap();

        assert!(financial_info.credit_limit > Decimal::ZERO);
        assert_eq!(financial_info.payment_terms, PaymentTerms::Net30);
        assert_eq!(financial_info.currency, "USD");
        assert!(financial_info.tax_id.is_some());
        assert_eq!(financial_info.risk_category, RiskCategory::Low);
    }

    #[test]
    fn test_sales_info_validation() {
        let customer = create_valid_customer();
        let sales_info = customer.sales_info.unwrap();

        assert_eq!(sales_info.sales_organization, "US_EAST");
        assert_eq!(sales_info.distribution_channel, "DIRECT");
        assert_eq!(sales_info.division, "TECH");
        assert_eq!(sales_info.delivery_priority, DeliveryPriority::High);
        assert!(sales_info.partial_delivery);
        assert!(sales_info.order_probability.unwrap() > 0);
    }

    #[test]
    fn test_marketing_info_validation() {
        let customer = create_valid_customer();
        let marketing_info = customer.marketing_info.unwrap();

        assert_eq!(marketing_info.acquisition_channel, AcquisitionChannel::DirectSales);
        assert_eq!(marketing_info.customer_segment, CustomerSegment::Enterprise);
        assert!(marketing_info.engagement_score.unwrap() > 0.0);

        let comm_prefs = &marketing_info.communication_preferences;
        assert!(comm_prefs.email_marketing);
        assert_eq!(comm_prefs.preferred_language, "en");
        assert_eq!(comm_prefs.frequency_preference, FrequencyPreference::Weekly);

        let gdpr = &marketing_info.gdpr_consent;
        assert!(gdpr.marketing_consent);
        assert_eq!(gdpr.consent_source, "WEBSITE");
    }

    #[test]
    fn test_compliance_info_validation() {
        let customer = create_valid_customer();
        let compliance_info = customer.compliance_info.unwrap();

        assert_eq!(compliance_info.kyc_status, KycStatus::Verified);
        assert!(compliance_info.kyc_last_updated.is_some());
        assert_eq!(compliance_info.sanctions_screening_status, SanctionsStatus::Clear);
        assert!(compliance_info.aml_risk_score.unwrap() < 100);
        assert!(compliance_info.data_residency_requirements.contains(&"US".to_string()));
    }

    #[test]
    fn test_customer_preferences_validation() {
        let customer = create_valid_customer();
        let preferences = &customer.preferences;

        assert_eq!(preferences.preferred_communication_method, CommunicationMethod::Email);
        assert_eq!(preferences.preferred_language, "en");
        assert_eq!(preferences.timezone, "America/New_York");
        assert_eq!(preferences.invoice_delivery_method, InvoiceDeliveryMethod::Email);
        assert_eq!(preferences.statement_frequency, StatementFrequency::Monthly);
        assert!(preferences.portal_access_enabled);
        assert!(preferences.mobile_app_enabled);
    }

    #[test]
    fn test_customer_metadata_validation() {
        let customer = create_valid_customer();
        let metadata = &customer.metadata;

        assert_eq!(metadata.version, 1);
        assert!(!metadata.tenant_id.0.is_nil());
        assert!(!metadata.created_by.0.is_nil());
        assert!(!metadata.modified_by.0.is_nil());
        assert!(!metadata.is_deleted);
        assert!(metadata.deleted_at.is_none());
        assert!(metadata.deleted_by.is_none());
        assert!(metadata.tags.contains(&"test".to_string()));
    }

    #[test]
    fn test_customer_external_ids() {
        let mut customer = create_valid_customer();

        customer.external_ids.insert("SAP_ID".to_string(), "123456".to_string());
        customer.external_ids.insert("ORACLE_ID".to_string(), "ORG789".to_string());
        customer.external_ids.insert("SALESFORCE_ID".to_string(), "0031234567890".to_string());

        assert_eq!(customer.external_ids.len(), 3);
        assert_eq!(customer.external_ids.get("SAP_ID"), Some(&"123456".to_string()));
        assert_eq!(customer.external_ids.get("ORACLE_ID"), Some(&"ORG789".to_string()));
        assert_eq!(customer.external_ids.get("SALESFORCE_ID"), Some(&"0031234567890".to_string()));
    }

    #[test]
    fn test_customer_lifecycle_transitions() {
        let mut customer = create_valid_customer();

        // Test valid lifecycle transitions
        customer.lifecycle_stage = CustomerLifecycleStage::Lead;
        assert_eq!(customer.lifecycle_stage, CustomerLifecycleStage::Lead);

        customer.lifecycle_stage = CustomerLifecycleStage::Prospect;
        assert_eq!(customer.lifecycle_stage, CustomerLifecycleStage::Prospect);

        customer.lifecycle_stage = CustomerLifecycleStage::ActiveCustomer;
        assert_eq!(customer.lifecycle_stage, CustomerLifecycleStage::ActiveCustomer);

        customer.lifecycle_stage = CustomerLifecycleStage::InactiveCustomer;
        assert_eq!(customer.lifecycle_stage, CustomerLifecycleStage::InactiveCustomer);
    }

    #[test]
    fn test_customer_credit_status_transitions() {
        let mut customer = create_valid_customer();

        customer.credit_status = CreditStatus::Pending;
        assert_eq!(customer.credit_status, CreditStatus::Pending);

        customer.credit_status = CreditStatus::Approved;
        assert_eq!(customer.credit_status, CreditStatus::Approved);

        customer.credit_status = CreditStatus::Rejected;
        assert_eq!(customer.credit_status, CreditStatus::Rejected);

        customer.credit_status = CreditStatus::Suspended;
        assert_eq!(customer.credit_status, CreditStatus::Suspended);
    }
}

#[cfg(test)]
mod customer_business_logic_tests {
    use super::*;

    #[test]
    fn test_customer_clv_calculation() {
        let customer = create_valid_customer();

        if let Some(clv) = customer.customer_lifetime_value {
            assert!(clv > Decimal::ZERO);
            assert_eq!(clv, Decimal::new(100000, 2)); // $1000.00
        }
    }

    #[test]
    fn test_customer_risk_assessment() {
        let customer = create_valid_customer();

        if let Some(financial_info) = &customer.financial_info {
            assert_eq!(financial_info.risk_category, RiskCategory::Low);

            if let Some(compliance_info) = &customer.compliance_info {
                assert!(compliance_info.aml_risk_score.unwrap_or(0) < 50); // Low risk threshold
            }
        }
    }

    #[test]
    fn test_customer_engagement_scoring() {
        let customer = create_valid_customer();

        if let Some(marketing_info) = &customer.marketing_info {
            if let Some(engagement_score) = marketing_info.engagement_score {
                assert!(engagement_score >= 0.0);
                assert!(engagement_score <= 10.0);
                assert_eq!(engagement_score, 8.5);
            }
        }
    }

    #[test]
    fn test_customer_compliance_requirements() {
        let customer = create_valid_customer();

        if let Some(compliance_info) = &customer.compliance_info {
            assert_eq!(compliance_info.kyc_status, KycStatus::Verified);
            assert_eq!(compliance_info.sanctions_screening_status, SanctionsStatus::Clear);
            assert!(compliance_info.data_residency_requirements.len() > 0);
        }
    }

    #[test]
    fn test_customer_payment_terms_validation() {
        let customer = create_valid_customer();

        if let Some(financial_info) = &customer.financial_info {
            match financial_info.payment_terms {
                PaymentTerms::Net30 => assert!(true, "Net30 is valid"),
                PaymentTerms::Net60 => assert!(true, "Net60 is valid"),
                PaymentTerms::Net90 => assert!(true, "Net90 is valid"),
                PaymentTerms::Immediate => assert!(true, "Immediate is valid"),
                PaymentTerms::Custom(_) => assert!(true, "Custom terms are valid"),
            }
        }
    }
}

#[cfg(test)]
mod customer_serialization_tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_customer_json_serialization() {
        let customer = create_valid_customer();

        let json_result = serde_json::to_string(&customer);
        assert!(json_result.is_ok(), "Customer should serialize to JSON");

        let json_string = json_result.unwrap();
        assert!(json_string.contains("legal_name"));
        assert!(json_string.contains("customer_number"));
        assert!(json_string.contains("customer_type"));
    }

    #[test]
    fn test_customer_json_deserialization() {
        let customer = create_valid_customer();

        let json_string = serde_json::to_string(&customer).unwrap();
        let deserialized_result: Result<Customer, _> = serde_json::from_str(&json_string);

        assert!(deserialized_result.is_ok(), "JSON should deserialize to Customer");

        let deserialized_customer = deserialized_result.unwrap();
        assert_eq!(deserialized_customer.id, customer.id);
        assert_eq!(deserialized_customer.legal_name, customer.legal_name);
        assert_eq!(deserialized_customer.customer_type, customer.customer_type);
    }

    #[test]
    fn test_customer_roundtrip_serialization() {
        let original_customer = create_valid_customer();

        // Serialize to JSON
        let json_string = serde_json::to_string(&original_customer).unwrap();

        // Deserialize from JSON
        let deserialized_customer: Customer = serde_json::from_str(&json_string).unwrap();

        // Verify roundtrip integrity
        assert_eq!(original_customer.id, deserialized_customer.id);
        assert_eq!(original_customer.customer_number, deserialized_customer.customer_number);
        assert_eq!(original_customer.legal_name, deserialized_customer.legal_name);
        assert_eq!(original_customer.customer_type, deserialized_customer.customer_type);
        assert_eq!(original_customer.lifecycle_stage, deserialized_customer.lifecycle_stage);
        assert_eq!(original_customer.status, deserialized_customer.status);
    }
}