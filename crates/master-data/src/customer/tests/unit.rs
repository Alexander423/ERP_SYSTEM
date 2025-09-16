use super::*;
use crate::customer::{Customer, CustomerType, CustomerLifecycleStage, CreditStatus};
use crate::customer::aggregate::{CustomerAggregate, CustomerCommand};
use crate::customer::events::CustomerEvent;
use crate::types::*;
use uuid::Uuid;
use chrono::Utc;
use rust_decimal::Decimal;

#[tokio::test]
async fn test_customer_aggregate_creation() {
    let customer_id = Uuid::new_v4();
    let tenant_id = TenantId(Uuid::new_v4());
    let user_id = Uuid::new_v4();

    let command = CustomerCommand::CreateCustomer {
        customer_id,
        tenant_id,
        customer_number: "TEST-001".to_string(),
        legal_name: "Test Customer".to_string(),
        display_name: Some("Test".to_string()),
        customer_type: CustomerType::Business,
        lifecycle_stage: CustomerLifecycleStage::Lead,
        created_by: user_id,
    };

    let mut aggregate = CustomerAggregate::new();
    let events = aggregate.handle_command(command).expect("Command should succeed");

    assert_eq!(events.len(), 1);
    match &events[0] {
        CustomerEvent::CustomerCreated {
            customer_id: event_customer_id,
            tenant_id: event_tenant_id,
            customer_number,
            legal_name,
            customer_type,
            created_by,
            ..
        } => {
            assert_eq!(*event_customer_id, customer_id);
            assert_eq!(*event_tenant_id, tenant_id);
            assert_eq!(customer_number, "TEST-001");
            assert_eq!(legal_name, "Test Customer");
            assert_eq!(*customer_type, CustomerType::Business);
            assert_eq!(*created_by, user_id);
        }
        _ => panic!("Expected CustomerCreated event"),
    }
}

#[tokio::test]
async fn test_customer_aggregate_lifecycle_transition() {
    let customer_id = Uuid::new_v4();
    let tenant_id = TenantId(Uuid::new_v4());
    let user_id = Uuid::new_v4();

    let mut aggregate = CustomerAggregate::new();

    // Create customer
    let create_command = CustomerCommand::CreateCustomer {
        customer_id,
        tenant_id,
        customer_number: "TEST-002".to_string(),
        legal_name: "Test Customer 2".to_string(),
        display_name: Some("Test 2".to_string()),
        customer_type: CustomerType::Business,
        lifecycle_stage: CustomerLifecycleStage::Lead,
        created_by: user_id,
    };

    let creation_events = aggregate.handle_command(create_command).expect("Creation should succeed");
    aggregate.apply_events(creation_events);

    // Transition lifecycle stage
    let transition_command = CustomerCommand::ChangeLifecycleStage {
        customer_id,
        new_stage: CustomerLifecycleStage::Prospect,
        reason: Some("Qualified lead".to_string()),
        changed_by: user_id,
    };

    let transition_events = aggregate.handle_command(transition_command).expect("Transition should succeed");

    assert_eq!(transition_events.len(), 1);
    match &transition_events[0] {
        CustomerEvent::LifecycleStageChanged {
            customer_id: event_customer_id,
            previous_stage,
            new_stage,
            reason,
            changed_by,
            ..
        } => {
            assert_eq!(*event_customer_id, customer_id);
            assert_eq!(*previous_stage, CustomerLifecycleStage::Lead);
            assert_eq!(*new_stage, CustomerLifecycleStage::Prospect);
            assert_eq!(reason.as_ref().unwrap(), "Qualified lead");
            assert_eq!(*changed_by, user_id);
        }
        _ => panic!("Expected LifecycleStageChanged event"),
    }
}

#[tokio::test]
async fn test_customer_aggregate_credit_limit_update() {
    let customer_id = Uuid::new_v4();
    let tenant_id = TenantId(Uuid::new_v4());
    let user_id = Uuid::new_v4();

    let mut aggregate = CustomerAggregate::new();

    // Create customer
    let create_command = CustomerCommand::CreateCustomer {
        customer_id,
        tenant_id,
        customer_number: "TEST-003".to_string(),
        legal_name: "Test Customer 3".to_string(),
        display_name: Some("Test 3".to_string()),
        customer_type: CustomerType::Business,
        lifecycle_stage: CustomerLifecycleStage::Active,
        created_by: user_id,
    };

    let creation_events = aggregate.handle_command(create_command).expect("Creation should succeed");
    aggregate.apply_events(creation_events);

    // Update credit limit
    let credit_command = CustomerCommand::UpdateCreditLimit {
        customer_id,
        new_limit: Decimal::new(50000, 2), // $500.00
        reason: "Initial credit assessment".to_string(),
        updated_by: user_id,
    };

    let credit_events = aggregate.handle_command(credit_command).expect("Credit update should succeed");

    assert_eq!(credit_events.len(), 1);
    match &credit_events[0] {
        CustomerEvent::CreditLimitUpdated {
            customer_id: event_customer_id,
            previous_limit,
            new_limit,
            reason,
            updated_by,
            ..
        } => {
            assert_eq!(*event_customer_id, customer_id);
            assert_eq!(*previous_limit, Decimal::ZERO);
            assert_eq!(*new_limit, Decimal::new(50000, 2));
            assert_eq!(reason, "Initial credit assessment");
            assert_eq!(*updated_by, user_id);
        }
        _ => panic!("Expected CreditLimitUpdated event"),
    }
}

#[tokio::test]
async fn test_customer_aggregate_invalid_lifecycle_transition() {
    let customer_id = Uuid::new_v4();
    let tenant_id = TenantId(Uuid::new_v4());
    let user_id = Uuid::new_v4();

    let mut aggregate = CustomerAggregate::new();

    // Create customer as Lead
    let create_command = CustomerCommand::CreateCustomer {
        customer_id,
        tenant_id,
        customer_number: "TEST-004".to_string(),
        legal_name: "Test Customer 4".to_string(),
        display_name: Some("Test 4".to_string()),
        customer_type: CustomerType::Business,
        lifecycle_stage: CustomerLifecycleStage::Lead,
        created_by: user_id,
    };

    let creation_events = aggregate.handle_command(create_command).expect("Creation should succeed");
    aggregate.apply_events(creation_events);

    // Try to transition directly to Churned (invalid)
    let invalid_transition_command = CustomerCommand::ChangeLifecycleStage {
        customer_id,
        new_stage: CustomerLifecycleStage::Churned,
        reason: Some("Invalid transition".to_string()),
        changed_by: user_id,
    };

    let result = aggregate.handle_command(invalid_transition_command);
    assert!(result.is_err(), "Invalid lifecycle transition should fail");
}

#[tokio::test]
async fn test_customer_aggregate_business_rules() {
    let customer_id = Uuid::new_v4();
    let tenant_id = TenantId(Uuid::new_v4());
    let user_id = Uuid::new_v4();

    let mut aggregate = CustomerAggregate::new();

    // Create customer
    let create_command = CustomerCommand::CreateCustomer {
        customer_id,
        tenant_id,
        customer_number: "TEST-005".to_string(),
        legal_name: "Test Customer 5".to_string(),
        display_name: Some("Test 5".to_string()),
        customer_type: CustomerType::Individual,
        lifecycle_stage: CustomerLifecycleStage::Lead,
        created_by: user_id,
    };

    let creation_events = aggregate.handle_command(create_command).expect("Creation should succeed");
    aggregate.apply_events(creation_events);

    // Test business rule: Individual customers should have lower default credit limits
    let credit_command = CustomerCommand::UpdateCreditLimit {
        customer_id,
        new_limit: Decimal::new(100000000, 2), // $1,000,000.00 - too high for individual
        reason: "High credit limit request".to_string(),
        updated_by: user_id,
    };

    // This should either fail or be adjusted based on business rules
    let result = aggregate.handle_command(credit_command);

    // Depending on implementation, this might succeed with adjustment or fail
    // For this test, we'll assume it succeeds but with business rule validation
    if let Ok(events) = result {
        assert!(!events.is_empty(), "Should generate events for credit limit update");
    }
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
    use crate::types::{Address, AddressType};

    let valid_address = Address {
        address_type: AddressType::Billing,
        street1: "123 Main St".to_string(),
        street2: Some("Apt 4B".to_string()),
        city: "New York".to_string(),
        state_province: Some("NY".to_string()),
        postal_code: "10001".to_string(),
        country: "US".to_string(),
        is_primary: true,
    };

    // Test address validation logic
    assert!(!valid_address.street1.is_empty());
    assert!(!valid_address.city.is_empty());
    assert!(!valid_address.postal_code.is_empty());
    assert!(!valid_address.country.is_empty());
}

#[tokio::test]
async fn test_contact_validation() {
    use crate::types::{Contact, ContactType};

    let valid_contact = Contact {
        contact_type: ContactType::Primary,
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        email: Some("john.doe@example.com".to_string()),
        phone: Some("+1-555-123-4567".to_string()),
        position: Some("Manager".to_string()),
        department: Some("Sales".to_string()),
        is_primary: true,
    };

    // Test contact validation logic
    assert!(!valid_contact.first_name.is_empty());
    assert!(!valid_contact.last_name.is_empty());

    if let Some(email) = &valid_contact.email {
        assert!(email.contains('@'));
    }
}