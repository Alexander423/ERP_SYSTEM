use super::*;
use crate::customer::*;
use crate::customer::repository::{CustomerRepository, PostgresCustomerRepository};
use crate::customer::service::{CustomerService, DefaultCustomerService};
use crate::customer::event_store::{CustomerEventStore, PostgresCustomerEventStore};
use sqlx::PgPool;

#[tokio::test]
async fn test_customer_full_lifecycle_integration() {
    let pool = create_test_pool().await;
    let ctx = TestContext::new(pool.clone()).await;

    let repo = PostgresCustomerRepository::new(pool.clone(), ctx.tenant_id);
    let event_store = PostgresCustomerEventStore::new(pool.clone());
    let service = DefaultCustomerService::new(repo, event_store);

    // Create customer
    let create_request = ctx.create_test_customer_request();
    let created_customer = service
        .create_customer(create_request, ctx.test_user_id)
        .await
        .expect("Customer creation should succeed");

    assert!(!created_customer.id.is_nil());
    assert_eq!(created_customer.customer_type, CustomerType::Business);
    assert_eq!(created_customer.lifecycle_stage, CustomerLifecycleStage::Lead);

    // Update customer
    let update_request = UpdateCustomerRequest {
        display_name: Some("Updated Test Customer".to_string()),
        lifecycle_stage: Some(CustomerLifecycleStage::Prospect),
        notes: Some("Updated notes for testing".to_string()),
        tags: Some(vec!["updated".to_string(), "integration-test".to_string()]),
        ..Default::default()
    };

    let updated_customer = service
        .update_customer(created_customer.id, update_request, ctx.test_user_id)
        .await
        .expect("Customer update should succeed");

    assert_eq!(updated_customer.display_name.as_deref(), Some("Updated Test Customer"));
    assert_eq!(updated_customer.lifecycle_stage, CustomerLifecycleStage::Prospect);

    // Search for customer
    let search_criteria = CustomerSearchCriteria {
        legal_name: Some("Test Customer".to_string()),
        customer_type: Some(CustomerType::Business),
        lifecycle_stage: Some(CustomerLifecycleStage::Prospect),
        ..Default::default()
    };

    let search_results = service
        .search_customers(search_criteria, 0, 10)
        .await
        .expect("Customer search should succeed");

    assert!(search_results.customers.len() >= 1);
    assert!(search_results.customers.iter().any(|c| c.id == created_customer.id));

    // Get customer by ID
    let retrieved_customer = service
        .get_customer(created_customer.id)
        .await
        .expect("Customer retrieval should succeed");

    assert_eq!(retrieved_customer.id, created_customer.id);
    assert_eq!(retrieved_customer.lifecycle_stage, CustomerLifecycleStage::Prospect);

    // Archive customer
    service
        .archive_customer(created_customer.id, "Integration test cleanup".to_string(), ctx.test_user_id)
        .await
        .expect("Customer archival should succeed");

    // Verify customer is archived
    let archived_customer = service
        .get_customer(created_customer.id)
        .await
        .expect("Archived customer should still be retrievable");

    assert!(archived_customer.is_archived);

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_customer_event_sourcing_integration() {
    let pool = create_test_pool().await;
    let ctx = TestContext::new(pool.clone()).await;

    let event_store = PostgresCustomerEventStore::new(pool.clone());
    let customer_id = Uuid::new_v4();

    // Create events
    let events = vec![
        CustomerEvent::CustomerCreated {
            customer_id,
            tenant_id: ctx.tenant_id,
            customer_number: "EVENT-001".to_string(),
            legal_name: "Event Sourcing Test Customer".to_string(),
            customer_type: CustomerType::Business,
            created_by: ctx.test_user_id,
            created_at: Utc::now(),
        },
        CustomerEvent::LifecycleStageChanged {
            customer_id,
            previous_stage: CustomerLifecycleStage::Lead,
            new_stage: CustomerLifecycleStage::Prospect,
            reason: Some("Event sourcing test".to_string()),
            changed_by: ctx.test_user_id,
            changed_at: Utc::now(),
        },
    ];

    // Append events
    let version = event_store
        .append_events(customer_id, events.clone(), None, Some(ctx.test_user_id))
        .await
        .expect("Event appending should succeed");

    assert_eq!(version, events.len() as i64);

    // Load events
    let loaded_events = event_store
        .load_events(customer_id)
        .await
        .expect("Event loading should succeed");

    assert_eq!(loaded_events.len(), events.len());

    // Verify event data
    match &loaded_events[0].event {
        CustomerEvent::CustomerCreated {
            customer_id: event_customer_id,
            customer_number,
            legal_name,
            ..
        } => {
            assert_eq!(*event_customer_id, customer_id);
            assert_eq!(customer_number, "EVENT-001");
            assert_eq!(legal_name, "Event Sourcing Test Customer");
        }
        _ => panic!("Expected CustomerCreated event"),
    }

    match &loaded_events[1].event {
        CustomerEvent::LifecycleStageChanged {
            customer_id: event_customer_id,
            previous_stage,
            new_stage,
            ..
        } => {
            assert_eq!(*event_customer_id, customer_id);
            assert_eq!(*previous_stage, CustomerLifecycleStage::Lead);
            assert_eq!(*new_stage, CustomerLifecycleStage::Prospect);
        }
        _ => panic!("Expected LifecycleStageChanged event"),
    }

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_customer_repository_operations() {
    let pool = create_test_pool().await;
    let ctx = TestContext::new(pool.clone()).await;

    let repo = PostgresCustomerRepository::new(pool.clone(), ctx.tenant_id);

    // Create test customer data
    let customer = Customer {
        id: Uuid::new_v4(),
        tenant_id: ctx.tenant_id,
        customer_number: "REPO-001".to_string(),
        legal_name: "Repository Test Customer".to_string(),
        display_name: Some("Repo Test".to_string()),
        customer_type: CustomerType::Business,
        lifecycle_stage: CustomerLifecycleStage::Lead,
        credit_status: CreditStatus::Good,
        credit_limit: rust_decimal::Decimal::new(100000, 2),
        addresses: vec![],
        contacts: vec![],
        tax_numbers: vec![],
        consolidation_group: None,
        notes: Some("Repository integration test".to_string()),
        tags: vec!["repository".to_string(), "test".to_string()],
        custom_fields: std::collections::HashMap::new(),
        financial_info: None,
        metadata: std::collections::HashMap::new(),
        is_archived: false,
        is_deleted: false,
        created_by: ctx.test_user_id,
        created_at: Utc::now(),
        modified_by: ctx.test_user_id,
        modified_at: Utc::now(),
        version: 1,
        last_sync_timestamp: None,
    };

    // Test create
    let created_customer = repo
        .create(customer.clone())
        .await
        .expect("Customer creation should succeed");

    assert_eq!(created_customer.id, customer.id);
    assert_eq!(created_customer.legal_name, customer.legal_name);

    // Test find_by_id
    let found_customer = repo
        .find_by_id(customer.id)
        .await
        .expect("Customer retrieval should succeed")
        .expect("Customer should exist");

    assert_eq!(found_customer.id, customer.id);

    // Test find_by_customer_number
    let found_by_number = repo
        .find_by_customer_number(&customer.customer_number)
        .await
        .expect("Customer lookup by number should succeed")
        .expect("Customer should exist");

    assert_eq!(found_by_number.customer_number, customer.customer_number);

    // Test update
    let mut updated_customer = found_customer.clone();
    updated_customer.display_name = Some("Updated Repo Test".to_string());
    updated_customer.version += 1;

    let update_result = repo
        .update(updated_customer.clone())
        .await
        .expect("Customer update should succeed");

    assert_eq!(update_result.display_name.as_deref(), Some("Updated Repo Test"));

    // Test search
    let search_criteria = CustomerSearchCriteria {
        legal_name: Some("Repository".to_string()),
        customer_type: Some(CustomerType::Business),
        ..Default::default()
    };

    let search_results = repo
        .search(search_criteria, 0, 10)
        .await
        .expect("Customer search should succeed");

    assert!(search_results.customers.len() >= 1);
    assert!(search_results.customers.iter().any(|c| c.id == customer.id));

    // Test archive
    repo
        .archive(customer.id, ctx.test_user_id)
        .await
        .expect("Customer archival should succeed");

    let archived_customer = repo
        .find_by_id(customer.id)
        .await
        .expect("Archived customer retrieval should succeed")
        .expect("Archived customer should exist");

    assert!(archived_customer.is_archived);

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_customer_search_functionality() {
    let pool = create_test_pool().await;
    let ctx = TestContext::new(pool.clone()).await;

    let repo = PostgresCustomerRepository::new(pool.clone(), ctx.tenant_id);

    // Create multiple test customers
    let customers = vec![
        Customer {
            id: Uuid::new_v4(),
            tenant_id: ctx.tenant_id,
            customer_number: "SEARCH-001".to_string(),
            legal_name: "Alpha Corporation".to_string(),
            customer_type: CustomerType::Business,
            lifecycle_stage: CustomerLifecycleStage::Active,
            credit_status: CreditStatus::Good,
            tags: vec!["alpha".to_string(), "corporation".to_string()],
            created_by: ctx.test_user_id,
            created_at: Utc::now(),
            modified_by: ctx.test_user_id,
            modified_at: Utc::now(),
            ..Default::default()
        },
        Customer {
            id: Uuid::new_v4(),
            tenant_id: ctx.tenant_id,
            customer_number: "SEARCH-002".to_string(),
            legal_name: "Beta Industries".to_string(),
            customer_type: CustomerType::Business,
            lifecycle_stage: CustomerLifecycleStage::Prospect,
            credit_status: CreditStatus::Good,
            tags: vec!["beta".to_string(), "industries".to_string()],
            created_by: ctx.test_user_id,
            created_at: Utc::now(),
            modified_by: ctx.test_user_id,
            modified_at: Utc::now(),
            ..Default::default()
        },
        Customer {
            id: Uuid::new_v4(),
            tenant_id: ctx.tenant_id,
            customer_number: "SEARCH-003".to_string(),
            legal_name: "John Smith".to_string(),
            customer_type: CustomerType::Individual,
            lifecycle_stage: CustomerLifecycleStage::Lead,
            credit_status: CreditStatus::Pending,
            tags: vec!["individual".to_string(), "smith".to_string()],
            created_by: ctx.test_user_id,
            created_at: Utc::now(),
            modified_by: ctx.test_user_id,
            modified_at: Utc::now(),
            ..Default::default()
        },
    ];

    // Create all customers
    for customer in &customers {
        repo.create(customer.clone())
            .await
            .expect("Customer creation should succeed");
    }

    // Test search by customer type
    let business_search = CustomerSearchCriteria {
        customer_type: Some(CustomerType::Business),
        ..Default::default()
    };

    let business_results = repo
        .search(business_search, 0, 10)
        .await
        .expect("Business customer search should succeed");

    assert!(business_results.customers.len() >= 2);
    assert!(business_results.customers.iter().all(|c| c.customer_type == CustomerType::Business));

    // Test search by lifecycle stage
    let active_search = CustomerSearchCriteria {
        lifecycle_stage: Some(CustomerLifecycleStage::Active),
        ..Default::default()
    };

    let active_results = repo
        .search(active_search, 0, 10)
        .await
        .expect("Active customer search should succeed");

    assert!(active_results.customers.len() >= 1);
    assert!(active_results.customers.iter().all(|c| c.lifecycle_stage == CustomerLifecycleStage::Active));

    // Test search by name
    let name_search = CustomerSearchCriteria {
        legal_name: Some("Alpha".to_string()),
        ..Default::default()
    };

    let name_results = repo
        .search(name_search, 0, 10)
        .await
        .expect("Name search should succeed");

    assert!(name_results.customers.len() >= 1);
    assert!(name_results.customers.iter().any(|c| c.legal_name.contains("Alpha")));

    // Test pagination
    let paginated_search = CustomerSearchCriteria {
        customer_type: Some(CustomerType::Business),
        ..Default::default()
    };

    let page1 = repo
        .search(paginated_search.clone(), 0, 1)
        .await
        .expect("First page should succeed");

    let page2 = repo
        .search(paginated_search, 1, 1)
        .await
        .expect("Second page should succeed");

    assert_eq!(page1.customers.len(), 1);
    assert_eq!(page2.customers.len(), 1);
    assert_ne!(page1.customers[0].id, page2.customers[0].id);

    ctx.cleanup().await;
}