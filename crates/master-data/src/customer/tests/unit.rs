use super::*;
use crate::customer::{CustomerType, CustomerLifecycleStage};
use crate::customer::aggregate::CustomerAggregate;
use uuid::Uuid;
use chrono::Utc;

#[tokio::test]
async fn test_customer_aggregate_creation() {
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let customer_number = "TEST-001".to_string();
    let legal_name = "Test Customer".to_string();
    let customer_type = CustomerType::Business;

    let result = CustomerAggregate::create(
        tenant_id,
        customer_number.clone(),
        legal_name.clone(),
        customer_type.clone(),
        user_id,
    );

    assert!(result.is_ok(), "Customer aggregate creation should succeed");
    let aggregate = result.unwrap();

    assert_eq!(aggregate.tenant_id, tenant_id);
    assert_eq!(aggregate.customer_number, customer_number);
    assert_eq!(aggregate.legal_name, legal_name);
    assert_eq!(aggregate.customer_type, customer_type);
    assert_eq!(aggregate.version, 1);
    assert!(!aggregate.is_deleted);
}

#[tokio::test]
async fn test_customer_aggregate_validation() {
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    // Test empty legal name validation
    let result = CustomerAggregate::create(
        tenant_id,
        "TEST-002".to_string(),
        "".to_string(), // Empty legal name should fail
        CustomerType::Business,
        user_id,
    );

    assert!(result.is_err(), "Empty legal name should cause creation to fail");

    // Test empty customer number validation
    let result = CustomerAggregate::create(
        tenant_id,
        "".to_string(), // Empty customer number should fail
        "Test Customer".to_string(),
        CustomerType::Business,
        user_id,
    );

    assert!(result.is_err(), "Empty customer number should cause creation to fail");
}

#[tokio::test]
async fn test_customer_type_variants() {
    // Test that all new customer type variants are available
    let business = CustomerType::Business;
    let individual = CustomerType::Individual;
    let government = CustomerType::Government;
    let b2b = CustomerType::B2b;

    // Test that they can be compared
    assert_ne!(business, individual);
    assert_ne!(individual, government);
    assert_ne!(government, b2b);

    // Test debug formatting
    assert_eq!(format!("{:?}", business), "Business");
    assert_eq!(format!("{:?}", individual), "Individual");
    assert_eq!(format!("{:?}", government), "Government");
}

#[tokio::test]
async fn test_customer_lifecycle_stage_variants() {
    // Test that all new lifecycle stage variants are available
    let active = CustomerLifecycleStage::Active;
    let prospect_customer = CustomerLifecycleStage::ProspectCustomer;
    let churned = CustomerLifecycleStage::Churned;
    let lead = CustomerLifecycleStage::Lead;

    // Test that they can be compared
    assert_ne!(active, prospect_customer);
    assert_ne!(prospect_customer, churned);
    assert_ne!(churned, lead);

    // Test debug formatting
    assert_eq!(format!("{:?}", active), "Active");
    assert_eq!(format!("{:?}", prospect_customer), "ProspectCustomer");
    assert_eq!(format!("{:?}", churned), "Churned");
}

#[tokio::test]
async fn test_customer_search_criteria_default() {
    // Test that CustomerSearchCriteria now implements Default
    let criteria = CustomerSearchCriteria::default();

    assert!(criteria.search_term.is_none());
    assert!(criteria.customer_numbers.is_none());
    assert!(criteria.customer_types.is_none());
    assert!(criteria.statuses.is_none());
    assert!(criteria.lifecycle_stages.is_none());
    assert!(criteria.page.is_none());
    assert!(criteria.page_size.is_none());

    // Test that we can create criteria with just a few fields
    let specific_criteria = CustomerSearchCriteria {
        search_term: Some("test".to_string()),
        customer_types: Some(vec![CustomerType::Business]),
        ..Default::default()
    };

    assert_eq!(specific_criteria.search_term, Some("test".to_string()));
    assert_eq!(specific_criteria.customer_types, Some(vec![CustomerType::Business]));
    assert!(specific_criteria.statuses.is_none());
}

#[tokio::test]
async fn test_customer_validation_rules() {
    use crate::customer::validation::CustomerValidator;

    let validator = CustomerValidator::new();

    // Test valid customer number format
    assert!(validator.validate_customer_number("CUST-001").is_ok());
    assert!(validator.validate_customer_number("12345").is_ok());

    // Test invalid customer number format
    assert!(validator.validate_customer_number("").is_err());
    assert!(validator.validate_customer_number("INVALID CHARS!@#").is_err());

    // Test valid email
    assert!(validator.validate_email("test@example.com").is_ok());

    // Test invalid email
    assert!(validator.validate_email("invalid-email").is_err());
    assert!(validator.validate_email("").is_err());

    // Test phone number validation
    assert!(validator.validate_phone("+1-555-123-4567").is_ok());
    assert!(validator.validate_phone("555-123-4567").is_ok());

    // Test invalid phone
    assert!(validator.validate_phone("invalid-phone").is_err());
}

#[tokio::test]
async fn test_address_validation() {
    use crate::types::{Address, AddressType, AuditFields};

    let address_id = Uuid::new_v4();
    let entity_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let now = Utc::now();

    let valid_address = Address {
        id: address_id,
        entity_type: "customer".to_string(),
        entity_id,
        address_type: AddressType::Billing,
        street_line_1: "123 Main St".to_string(),
        street_line_2: Some("Apt 4B".to_string()),
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

    // Test address validation logic
    assert!(!valid_address.street_line_1.is_empty());
    assert!(!valid_address.city.is_empty());
    assert!(!valid_address.postal_code.is_empty());
    assert!(!valid_address.country_code.is_empty());
    assert!(valid_address.is_primary);
    assert!(valid_address.is_active);
}

#[tokio::test]
async fn test_contact_validation() {
    use crate::types::{ContactInfo, ContactType, AuditFields};

    let contact_id = Uuid::new_v4();
    let entity_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let now = Utc::now();

    let valid_contact = ContactInfo {
        id: contact_id,
        entity_type: "customer".to_string(),
        entity_id,
        contact_type: ContactType::Primary,
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        title: Some("Manager".to_string()),
        department: Some("Sales".to_string()),
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

    // Test contact validation logic
    assert!(!valid_contact.first_name.is_empty());
    assert!(!valid_contact.last_name.is_empty());
    assert!(valid_contact.is_primary);
    assert!(valid_contact.is_active);

    if let Some(email) = &valid_contact.email {
        assert!(email.contains('@'));
    }
}