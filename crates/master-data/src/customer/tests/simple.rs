use uuid::Uuid;
use chrono::Utc;
use crate::customer::validation::CustomerValidator;
use crate::customer::model::*;
use crate::types::*;

#[test]
fn test_customer_validator_basic() {
    let validator = CustomerValidator::new();

    // Test customer number validation
    assert!(validator.validate_customer_number("CUST-001").is_ok());
    assert!(validator.validate_customer_number("TEST-123").is_ok());
    assert!(validator.validate_customer_number("").is_err());
    assert!(validator.validate_customer_number("invalid!@#").is_err());

    // Test email validation
    assert!(validator.validate_email("test@example.com").is_ok());
    assert!(validator.validate_email("user@domain.org").is_ok());
    assert!(validator.validate_email("invalid-email").is_err());
    assert!(validator.validate_email("").is_err());

    // Test phone validation
    assert!(validator.validate_phone("+1-555-123-4567").is_ok());
    assert!(validator.validate_phone("555-123-4567").is_ok());
    assert!(validator.validate_phone("invalid-phone").is_err());
    assert!(validator.validate_phone("").is_err());

    // Test legal name validation
    assert!(validator.validate_legal_name("Test Company Ltd.").is_ok());
    assert!(validator.validate_legal_name("John Doe").is_ok());
    assert!(validator.validate_legal_name("").is_err());
    assert!(validator.validate_legal_name("   ").is_err());

    // Test tags validation
    assert!(validator.validate_tags(&[]).is_ok());
    assert!(validator.validate_tags(&["tag1".to_string(), "tag2".to_string()]).is_ok());
    assert!(validator.validate_tags(&["".to_string()]).is_err());
    assert!(validator.validate_tags(&["tag1".to_string(), "tag1".to_string()]).is_err());
}

#[test]
fn test_customer_creation() {
    let customer_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let now = Utc::now();

    let customer = Customer {
        id: customer_id,
        customer_number: "TEST-001".to_string(),
        external_ids: std::collections::HashMap::new(),
        legal_name: "Test Customer Ltd.".to_string(),
        trade_names: vec!["Test Corp".to_string()],
        customer_type: CustomerType::B2b,
        industry_classification: IndustryClassification::Technology,
        business_size: BusinessSize::Medium,
        parent_customer_id: None,
        corporate_group_id: None,
        customer_hierarchy_level: 1,
        consolidation_group: None,
        lifecycle_stage: CustomerLifecycleStage::Lead,
        status: EntityStatus::Active,
        credit_status: CreditStatus::Good,
        primary_address_id: None,
        billing_address_id: None,
        shipping_address_ids: vec![],
        addresses: vec![],
        primary_contact_id: None,
        contacts: vec![],
        tax_jurisdictions: vec![],
        tax_numbers: std::collections::HashMap::new(),
        regulatory_classifications: vec![],
        compliance_status: ComplianceStatus::Compliant,
        kyc_status: KycStatus::Completed,
        aml_risk_rating: RiskRating::Low,
        financial_info: FinancialInfo {
            currency_code: "USD".to_string(),
            credit_limit: Some(rust_decimal::Decimal::new(100000, 2)),
            payment_terms: Some(PaymentTerms {
                payment_method: PaymentMethod::BankTransfer,
                net_days: Some(30),
                discount_percentage: None,
                discount_days: None,
                late_fee_percentage: None,
            }),
            tax_exempt: false,
            tax_numbers: std::collections::HashMap::new(),
        },
        price_group_id: None,
        discount_group_id: None,
        sales_representative_id: None,
        account_manager_id: None,
        customer_segments: vec![CustomerSegment {
            segment_type: "BUSINESS_SIZE".to_string(),
            segment_value: "Enterprise".to_string(),
            confidence_score: Some(0.95),
            effective_date: now,
            expiry_date: None,
        }],
        acquisition_channel: Some(AcquisitionChannel::DirectSales),
        customer_lifetime_value: Some(rust_decimal::Decimal::new(50000, 0)),
        churn_probability: Some(0.1),
        performance_metrics: CustomerPerformanceMetrics {
            total_revenue: Some(rust_decimal::Decimal::new(25000, 0)),
            revenue_last_12_months: Some(25000.0),
            average_order_value: Some(rust_decimal::Decimal::new(2500, 0)),
            order_frequency: Some(0.83),
            total_orders: Some(10),
            last_order_date: Some(now),
            profit_margin: Some(0.2),
            last_purchase_date: Some(now),
            first_purchase_date: Some(now),
            customer_lifetime_value: Some(50000.0),
            predicted_churn_probability: Some(0.1),
            relationship_duration_days: Some(365),
            satisfaction_score: Some(4.5),
            net_promoter_score: Some(8),
            last_contact_date: Some(now),
            contact_frequency: Some(0.5),
            response_rate: Some(0.95),
            days_sales_outstanding: Some(30.0),
            payment_reliability_score: Some(0.98),
            support_ticket_count: Some(2),
            last_calculated: now,
        },
        behavioral_data: CustomerBehavioralData {
            preferred_purchase_channels: vec!["online".to_string(), "phone".to_string()],
            seasonal_purchase_patterns: std::collections::HashMap::new(),
            product_category_preferences: std::collections::HashMap::new(),
            purchase_frequency: Some(0.83),
            preferred_categories: std::collections::HashMap::new(),
            seasonal_trends: std::collections::HashMap::new(),
            price_sensitivity: Some(0.3),
            brand_loyalty: Some(0.8),
            preferred_contact_times: vec!["morning".to_string()],
            channel_engagement_rates: std::collections::HashMap::new(),
            communication_preferences: std::collections::HashMap::new(),
            support_ticket_frequency: Some(0.1),
            product_return_rate: Some(0.02),
            referral_activity: Some(0.15),
            website_engagement_score: Some(0.8),
            mobile_app_usage: Some(0.3),
            social_media_sentiment: Some(0.7),
            propensity_to_buy: Some(0.85),
            upsell_probability: Some(0.6),
            cross_sell_probability: Some(0.4),
            last_updated: now,
        },
        sync_info: SyncInfo {
            last_sync: Some(now),
            sync_source: Some("test".to_string()),
            sync_version: Some("1.0".to_string()),
            sync_status: SyncStatus::Success,
            external_references: std::collections::HashMap::new(),
        },
        contract_ids: Vec::new(),
        custom_fields: std::collections::HashMap::new(),
        audit: AuditFields {
            created_at: now,
            created_by: user_id,
            modified_at: now,
            modified_by: user_id,
            version: 1,
            is_deleted: false,
            deleted_at: None,
            deleted_by: None,
        },
    };

    // Basic assertions
    assert_eq!(customer.id, customer_id);
    assert_eq!(customer.legal_name, "Test Customer Ltd.");
    assert_eq!(customer.customer_type, CustomerType::B2b);
    assert_eq!(customer.lifecycle_stage, CustomerLifecycleStage::Lead);
    assert_eq!(customer.credit_status, CreditStatus::Good);
    assert!(!customer.audit.is_deleted);
    assert_eq!(customer.audit.version, 1);
}

#[test]
fn test_address_creation() {
    let address_id = Uuid::new_v4();
    let entity_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let now = Utc::now();

    let address = Address {
        id: address_id,
        entity_type: "customer".to_string(),
        entity_id,
        address_type: AddressType::Billing,
        street_line_1: "123 Main Street".to_string(),
        street_line_2: Some("Suite 456".to_string()),
        city: "New York".to_string(),
        state_province: Some("NY".to_string()),
        postal_code: "10001".to_string(),
        country_code: "US".to_string(),
        coordinates: None,
        is_primary: true,
        is_active: true,
        audit: AuditFields {
            created_at: now,
            created_by: user_id,
            modified_at: now,
            modified_by: user_id,
            version: 1,
            is_deleted: false,
            deleted_at: None,
            deleted_by: None,
        },
    };

    assert_eq!(address.address_type, AddressType::Billing);
    assert_eq!(address.street_line_1, "123 Main Street");
    assert_eq!(address.city, "New York");
    assert_eq!(address.country_code, "US");
    assert!(address.is_primary);
}

#[test]
fn test_contact_creation() {
    let contact_id = Uuid::new_v4();
    let entity_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let now = Utc::now();

    let contact = ContactInfo {
        id: contact_id,
        entity_type: "customer".to_string(),
        entity_id,
        contact_type: ContactType::Primary,
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        title: Some("CEO".to_string()),
        department: Some("Executive".to_string()),
        email: Some("john.doe@example.com".to_string()),
        phone: Some("+1-555-123-4567".to_string()),
        mobile: None,
        fax: None,
        preferred_language: None,
        communication_preferences: None,
        is_primary: true,
        is_active: true,
        audit: AuditFields {
            created_at: now,
            created_by: user_id,
            modified_at: now,
            modified_by: user_id,
            version: 1,
            is_deleted: false,
            deleted_at: None,
            deleted_by: None,
        },
    };

    assert_eq!(contact.contact_type, ContactType::Primary);
    assert_eq!(contact.first_name, "John");
    assert_eq!(contact.last_name, "Doe");
    assert_eq!(contact.email.as_deref(), Some("john.doe@example.com"));
    assert!(contact.is_primary);
}

#[test]
fn test_customer_enums() {
    // Test CustomerType enum
    assert_eq!(format!("{:?}", CustomerType::B2b), "B2b");
    assert_eq!(format!("{:?}", CustomerType::Individual), "Individual");
    assert_eq!(format!("{:?}", CustomerType::Government), "Government");

    // Test CustomerLifecycleStage enum
    assert_eq!(format!("{:?}", CustomerLifecycleStage::Lead), "Lead");
    assert_eq!(format!("{:?}", CustomerLifecycleStage::Prospect), "Prospect");
    assert_eq!(format!("{:?}", CustomerLifecycleStage::Active), "Active");

    // Test CreditStatus enum
    assert_eq!(format!("{:?}", CreditStatus::Good), "Good");
    assert_eq!(format!("{:?}", CreditStatus::Fair), "Fair");
    assert_eq!(format!("{:?}", CreditStatus::Poor), "Poor");
}

#[test]
fn test_financial_info() {
    let financial_info = FinancialInfo {
        currency_code: "USD".to_string(),
        credit_limit: Some(rust_decimal::Decimal::new(50000, 2)),
        payment_terms: Some(PaymentTerms {
            payment_method: PaymentMethod::BankTransfer,
            net_days: Some(30),
            discount_percentage: None,
            discount_days: None,
            late_fee_percentage: None,
        }),
        tax_exempt: false,
        tax_numbers: std::collections::HashMap::new(),
    };

    assert_eq!(financial_info.currency_code, "USD");
    assert_eq!(financial_info.credit_limit, Some(rust_decimal::Decimal::new(50000, 2)));
    assert_eq!(financial_info.payment_terms.as_ref().map(|pt| &pt.payment_method), Some(&PaymentMethod::BankTransfer));
    assert_eq!(financial_info.tax_exempt, false);
}

#[test]
fn test_performance_metrics() {
    let now = Utc::now();
    let metrics = CustomerPerformanceMetrics {
        total_revenue: Some(rust_decimal::Decimal::new(100000, 0)),
        revenue_last_12_months: Some(100000.0),
        average_order_value: Some(rust_decimal::Decimal::new(5000, 0)),
        order_frequency: Some(0.27), // ~20 orders per 730 days
        total_orders: Some(20),
        last_order_date: Some(now),
        profit_margin: Some(0.25),
        last_purchase_date: Some(now),
        first_purchase_date: Some(now),
        customer_lifetime_value: Some(150000.0),
        predicted_churn_probability: Some(0.05),
        relationship_duration_days: Some(730),
        satisfaction_score: Some(4.8),
        net_promoter_score: Some(9),
        last_contact_date: Some(now),
        contact_frequency: Some(0.25),
        response_rate: Some(0.98),
        days_sales_outstanding: Some(25.0),
        payment_reliability_score: Some(0.99),
        support_ticket_count: Some(1),
        last_calculated: now,
    };

    assert_eq!(metrics.total_revenue, Some(rust_decimal::Decimal::new(100000, 0)));
    assert_eq!(metrics.total_orders, Some(20));
    assert_eq!(metrics.satisfaction_score, Some(4.8));
    assert!(metrics.payment_reliability_score.unwrap() > 0.95);
}

#[test]
fn test_create_customer_request() {
    let request = CreateCustomerRequest {
        customer_number: Some("NEW-001".to_string()),
        legal_name: "New Customer Inc.".to_string(),
        trade_names: Some(vec!["New Customer".to_string()]),
        customer_type: CustomerType::B2b,
        industry_classification: Some(IndustryClassification::Technology),
        business_size: Some(BusinessSize::Medium),
        parent_customer_id: None,
        corporate_group_id: None,
        lifecycle_stage: Some(CustomerLifecycleStage::Lead),
        status: Some(EntityStatus::Active),
        credit_status: Some(CreditStatus::Good),
        addresses: None,
        contacts: None,
        tax_jurisdictions: None,
        tax_numbers: None,
        financial_info: None,
        sales_representative_id: None,
        account_manager_id: None,
        acquisition_channel: None,
        external_ids: None,
        sync_info: None,
        customer_hierarchy_level: None,
        consolidation_group: None,
    };

    assert_eq!(request.customer_number.as_deref(), Some("NEW-001"));
    assert_eq!(request.legal_name, "New Customer Inc.");
    assert_eq!(request.customer_type, CustomerType::B2b);
    assert_eq!(request.lifecycle_stage, Some(CustomerLifecycleStage::Lead));
}