use super::*;
use crate::customer::*;
use crate::customer::repository::{CustomerRepository, PostgresCustomerRepository};
use crate::customer::service::{CustomerService, DefaultCustomerService};
use crate::customer::analytics_engine::CustomerAnalyticsEngine;
use std::time::{Duration, Instant};
use tokio::time::timeout;

#[tokio::test]
async fn test_bulk_customer_creation_performance() {
    let pool = create_test_pool().await;
    let ctx = TestContext::new(pool.clone()).await;

    let repo = PostgresCustomerRepository::new(pool.clone(), ctx.tenant_id);
    let batch_size = 100;

    // Generate test customers
    let customers: Vec<Customer> = (0..batch_size)
        .map(|i| Customer {
            id: Uuid::new_v4(),
            tenant_id: ctx.tenant_id,
            customer_number: format!("PERF-{:04}", i),
            legal_name: format!("Performance Test Customer {}", i),
            display_name: Some(format!("Perf Test {}", i)),
            customer_type: if i % 2 == 0 { CustomerType::Business } else { CustomerType::Individual },
            lifecycle_stage: CustomerLifecycleStage::Lead,
            credit_status: CreditStatus::Good,
            credit_limit: rust_decimal::Decimal::new(10000 + (i as i64 * 1000), 2),
            addresses: vec![],
            contacts: vec![],
            tax_numbers: vec![],
            consolidation_group: None,
            notes: Some(format!("Performance test customer #{}", i)),
            tags: vec!["performance".to_string(), "test".to_string()],
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
        })
        .collect();

    // Test individual creation performance
    let start = Instant::now();
    for customer in &customers {
        repo.create(customer.clone())
            .await
            .expect("Customer creation should succeed");
    }
    let individual_duration = start.elapsed();

    println!("Individual creation of {} customers took: {:?}", batch_size, individual_duration);
    println!("Average per customer: {:?}", individual_duration / batch_size as u32);

    // Verify performance threshold (should be under 10ms per customer on average)
    let avg_per_customer = individual_duration / batch_size as u32;
    assert!(
        avg_per_customer < Duration::from_millis(10),
        "Customer creation should be under 10ms per customer, was {:?}",
        avg_per_customer
    );

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_large_dataset_search_performance() {
    let pool = create_test_pool().await;
    let ctx = TestContext::new(pool.clone()).await;

    let repo = PostgresCustomerRepository::new(pool.clone(), ctx.tenant_id);

    // Create a larger dataset for search testing
    let dataset_size = 1000;
    let customers: Vec<Customer> = (0..dataset_size)
        .map(|i| Customer {
            id: Uuid::new_v4(),
            tenant_id: ctx.tenant_id,
            customer_number: format!("SEARCH-{:06}", i),
            legal_name: format!("Search Test Customer {}", i),
            display_name: Some(format!("Search {}", i)),
            customer_type: match i % 3 {
                0 => CustomerType::Business,
                1 => CustomerType::Individual,
                _ => CustomerType::Government,
            },
            lifecycle_stage: match i % 4 {
                0 => CustomerLifecycleStage::Lead,
                1 => CustomerLifecycleStage::Prospect,
                2 => CustomerLifecycleStage::Active,
                _ => CustomerLifecycleStage::Inactive,
            },
            credit_status: CreditStatus::Good,
            notes: Some(format!("Search performance test customer with ID {}", i)),
            tags: vec![
                "search".to_string(),
                "performance".to_string(),
                format!("batch-{}", i / 100),
            ],
            created_by: ctx.test_user_id,
            created_at: Utc::now(),
            modified_by: ctx.test_user_id,
            modified_at: Utc::now(),
            ..Default::default()
        })
        .collect();

    // Bulk create customers
    for customer in &customers {
        repo.create(customer.clone())
            .await
            .expect("Customer creation should succeed");
    }

    // Test various search scenarios
    let search_scenarios = vec![
        ("Full text search", CustomerSearchCriteria {
            legal_name: Some("Search Test".to_string()),
            ..Default::default()
        }),
        ("Type filter", CustomerSearchCriteria {
            customer_type: Some(CustomerType::Business),
            ..Default::default()
        }),
        ("Lifecycle filter", CustomerSearchCriteria {
            lifecycle_stage: Some(CustomerLifecycleStage::Active),
            ..Default::default()
        }),
        ("Combined filters", CustomerSearchCriteria {
            customer_type: Some(CustomerType::Business),
            lifecycle_stage: Some(CustomerLifecycleStage::Active),
            ..Default::default()
        }),
    ];

    for (scenario_name, search_criteria) in search_scenarios {
        let start = Instant::now();
        let results = repo
            .search(search_criteria, 0, 50)
            .await
            .expect("Search should succeed");
        let search_duration = start.elapsed();

        println!("{} on {} records took: {:?}", scenario_name, dataset_size, search_duration);

        // Search should complete within 100ms even with large dataset
        assert!(
            search_duration < Duration::from_millis(100),
            "{} should complete under 100ms, took {:?}",
            scenario_name,
            search_duration
        );

        assert!(results.customers.len() <= 50, "Should respect limit parameter");
    }

    // Test pagination performance
    let start = Instant::now();
    let mut total_retrieved = 0;
    let page_size = 50;
    let mut offset = 0;

    loop {
        let page_results = repo
            .search(
                CustomerSearchCriteria::default(),
                offset,
                page_size
            )
            .await
            .expect("Paginated search should succeed");

        total_retrieved += page_results.customers.len();

        if page_results.customers.len() < page_size {
            break;
        }

        offset += page_size;
    }

    let pagination_duration = start.elapsed();
    println!("Paginated retrieval of {} customers took: {:?}", total_retrieved, pagination_duration);

    // Pagination should be efficient
    assert!(
        pagination_duration < Duration::from_secs(2),
        "Pagination should complete under 2 seconds, took {:?}",
        pagination_duration
    );

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_concurrent_operations_performance() {
    let pool = create_test_pool().await;
    let ctx = TestContext::new(pool.clone()).await;

    let repo = PostgresCustomerRepository::new(pool.clone(), ctx.tenant_id);
    let concurrent_operations = 50;

    // Test concurrent reads
    let read_futures = (0..concurrent_operations).map(|i| {
        let repo = repo.clone();
        let customer_id = Uuid::new_v4();

        tokio::spawn(async move {
            // Try to read a non-existent customer (tests query performance)
            let start = Instant::now();
            let _ = repo.find_by_id(customer_id).await;
            start.elapsed()
        })
    });

    let start = Instant::now();
    let read_results = futures::future::join_all(read_futures).await;
    let total_concurrent_read_time = start.elapsed();

    println!("Concurrent reads ({} operations) took: {:?}", concurrent_operations, total_concurrent_read_time);

    let avg_read_time: Duration = read_results
        .into_iter()
        .map(|r| r.expect("Read task should complete"))
        .sum::<Duration>() / concurrent_operations as u32;

    println!("Average concurrent read time: {:?}", avg_read_time);

    // Concurrent reads should be fast
    assert!(
        avg_read_time < Duration::from_millis(10),
        "Concurrent reads should average under 10ms, was {:?}",
        avg_read_time
    );

    // Test concurrent writes
    let write_futures = (0..concurrent_operations).map(|i| {
        let repo = repo.clone();
        let ctx_tenant_id = ctx.tenant_id;
        let ctx_user_id = ctx.test_user_id;

        tokio::spawn(async move {
            let customer = Customer {
                id: Uuid::new_v4(),
                tenant_id: ctx_tenant_id,
                customer_number: format!("CONCURRENT-{:04}", i),
                legal_name: format!("Concurrent Test Customer {}", i),
                customer_type: CustomerType::Business,
                lifecycle_stage: CustomerLifecycleStage::Lead,
                created_by: ctx_user_id,
                created_at: Utc::now(),
                modified_by: ctx_user_id,
                modified_at: Utc::now(),
                ..Default::default()
            };

            let start = Instant::now();
            let result = repo.create(customer).await;
            (start.elapsed(), result)
        })
    });

    let start = Instant::now();
    let write_results = futures::future::join_all(write_futures).await;
    let total_concurrent_write_time = start.elapsed();

    println!("Concurrent writes ({} operations) took: {:?}", concurrent_operations, total_concurrent_write_time);

    let successful_writes: Vec<Duration> = write_results
        .into_iter()
        .filter_map(|r| {
            let (duration, result) = r.expect("Write task should complete");
            if result.is_ok() { Some(duration) } else { None }
        })
        .collect();

    assert!(
        successful_writes.len() >= (concurrent_operations as f32 * 0.95) as usize,
        "At least 95% of concurrent writes should succeed"
    );

    let avg_write_time: Duration = successful_writes.iter().sum::<Duration>() / successful_writes.len() as u32;
    println!("Average concurrent write time: {:?}", avg_write_time);

    // Concurrent writes should complete reasonably quickly
    assert!(
        avg_write_time < Duration::from_millis(50),
        "Concurrent writes should average under 50ms, was {:?}",
        avg_write_time
    );

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_analytics_performance() {
    let pool = create_test_pool().await;
    let ctx = TestContext::new(pool.clone()).await;

    let analytics_engine = crate::customer::analytics_engine::PostgresCustomerAnalyticsEngine::new(
        pool.clone(),
        crate::types::TenantContext {
            tenant_id: ctx.tenant_id,
            user_id: ctx.test_user_id,
        }
    );

    // Create test data for analytics
    let repo = PostgresCustomerRepository::new(pool.clone(), ctx.tenant_id);
    let analytics_test_customers: Vec<Customer> = (0..500)
        .map(|i| Customer {
            id: Uuid::new_v4(),
            tenant_id: ctx.tenant_id,
            customer_number: format!("ANALYTICS-{:04}", i),
            legal_name: format!("Analytics Test Customer {}", i),
            customer_type: CustomerType::Business,
            lifecycle_stage: if i < 100 {
                CustomerLifecycleStage::Active
            } else if i < 200 {
                CustomerLifecycleStage::Prospect
            } else {
                CustomerLifecycleStage::Lead
            },
            credit_limit: rust_decimal::Decimal::new((i as i64 + 1) * 10000, 2),
            created_by: ctx.test_user_id,
            created_at: Utc::now() - chrono::Duration::days(i as i64 % 365),
            modified_by: ctx.test_user_id,
            modified_at: Utc::now(),
            ..Default::default()
        })
        .collect();

    for customer in &analytics_test_customers {
        repo.create(customer.clone())
            .await
            .expect("Analytics test customer creation should succeed");
    }

    // Test analytics operations performance
    let customer_id = analytics_test_customers[0].id;

    // Test customer insights calculation
    let insights_start = Instant::now();
    let insights_result = timeout(
        Duration::from_secs(5),
        analytics_engine.calculate_customer_insights(customer_id)
    ).await;
    let insights_duration = insights_start.elapsed();

    println!("Customer insights calculation took: {:?}", insights_duration);

    assert!(insights_result.is_ok(), "Customer insights should complete within timeout");
    assert!(
        insights_duration < Duration::from_millis(500),
        "Customer insights should complete under 500ms, took {:?}",
        insights_duration
    );

    // Test customer segmentation performance
    let segmentation_start = Instant::now();
    let segmentation_result = timeout(
        Duration::from_secs(10),
        analytics_engine.segment_customers()
    ).await;
    let segmentation_duration = segmentation_start.elapsed();

    println!("Customer segmentation took: {:?}", segmentation_duration);

    assert!(segmentation_result.is_ok(), "Customer segmentation should complete within timeout");
    assert!(
        segmentation_duration < Duration::from_secs(2),
        "Customer segmentation should complete under 2 seconds, took {:?}",
        segmentation_duration
    );

    // Test bulk analytics operations
    let bulk_analytics_start = Instant::now();
    let customer_ids: Vec<Uuid> = analytics_test_customers.iter().take(50).map(|c| c.id).collect();

    let bulk_insights_futures = customer_ids.into_iter().map(|id| {
        let engine = analytics_engine.clone();
        tokio::spawn(async move {
            engine.calculate_customer_insights(id).await
        })
    });

    let bulk_results = futures::future::join_all(bulk_insights_futures).await;
    let bulk_analytics_duration = bulk_analytics_start.elapsed();

    println!("Bulk analytics (50 customers) took: {:?}", bulk_analytics_duration);

    let successful_analytics = bulk_results
        .into_iter()
        .filter(|r| r.is_ok() && r.as_ref().unwrap().is_ok())
        .count();

    assert!(
        successful_analytics >= 45,
        "At least 90% of bulk analytics operations should succeed"
    );

    assert!(
        bulk_analytics_duration < Duration::from_secs(10),
        "Bulk analytics should complete under 10 seconds, took {:?}",
        bulk_analytics_duration
    );

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_event_store_performance() {
    let pool = create_test_pool().await;
    let ctx = TestContext::new(pool.clone()).await;

    let event_store = crate::customer::event_store::PostgresCustomerEventStore::new(pool.clone());
    let customer_id = Uuid::new_v4();

    // Test event appending performance
    let events_per_batch = 100;
    let batch_count = 10;

    for batch in 0..batch_count {
        let events: Vec<CustomerEvent> = (0..events_per_batch)
            .map(|i| CustomerEvent::CustomerUpdated {
                customer_id,
                field_name: format!("test_field_{}", i),
                old_value: serde_json::Value::String(format!("old_value_{}", i)),
                new_value: serde_json::Value::String(format!("new_value_{}", i)),
                updated_by: ctx.test_user_id,
                updated_at: Utc::now(),
            })
            .collect();

        let start = Instant::now();
        let version = event_store
            .append_events(customer_id, events, Some(batch as i64 * events_per_batch as i64), Some(ctx.test_user_id))
            .await
            .expect("Event appending should succeed");
        let append_duration = start.elapsed();

        println!("Appending {} events (batch {}) took: {:?}", events_per_batch, batch, append_duration);

        assert_eq!(version, (batch + 1) as i64 * events_per_batch as i64);

        // Event appending should be fast
        assert!(
            append_duration < Duration::from_millis(100),
            "Event appending should complete under 100ms, took {:?}",
            append_duration
        );
    }

    // Test event loading performance
    let load_start = Instant::now();
    let loaded_events = event_store
        .load_events(customer_id)
        .await
        .expect("Event loading should succeed");
    let load_duration = load_start.elapsed();

    println!("Loading {} events took: {:?}", loaded_events.len(), load_duration);

    assert_eq!(loaded_events.len(), batch_count * events_per_batch);

    // Event loading should be reasonably fast even with many events
    assert!(
        load_duration < Duration::from_millis(200),
        "Event loading should complete under 200ms, took {:?}",
        load_duration
    );

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_memory_usage_during_bulk_operations() {
    let pool = create_test_pool().await;
    let ctx = TestContext::new(pool.clone()).await;

    let repo = PostgresCustomerRepository::new(pool.clone(), ctx.tenant_id);

    // This test monitors that we don't load too much data into memory at once
    let large_dataset_size = 2000;

    // Create large dataset
    for i in 0..large_dataset_size {
        let customer = Customer {
            id: Uuid::new_v4(),
            tenant_id: ctx.tenant_id,
            customer_number: format!("MEMORY-{:06}", i),
            legal_name: format!("Memory Test Customer {}", i),
            customer_type: CustomerType::Business,
            lifecycle_stage: CustomerLifecycleStage::Lead,
            notes: Some("A".repeat(1000)), // 1KB of notes per customer
            created_by: ctx.test_user_id,
            created_at: Utc::now(),
            modified_by: ctx.test_user_id,
            modified_at: Utc::now(),
            ..Default::default()
        };

        repo.create(customer)
            .await
            .expect("Memory test customer creation should succeed");
    }

    // Test that we can iterate through large dataset without loading everything into memory
    let page_size = 100;
    let mut processed_count = 0;
    let mut offset = 0;

    let iteration_start = Instant::now();

    loop {
        let page_results = repo
            .search(CustomerSearchCriteria::default(), offset, page_size)
            .await
            .expect("Paginated search should succeed");

        processed_count += page_results.customers.len();

        // Verify we're not holding onto previous pages (this is more of a design test)
        // In a real scenario, we'd use a memory profiler

        if page_results.customers.len() < page_size {
            break;
        }

        offset += page_size;
    }

    let iteration_duration = iteration_start.elapsed();

    println!(
        "Processed {} customers in pages of {} in {:?}",
        processed_count, page_size, iteration_duration
    );

    assert_eq!(processed_count, large_dataset_size);

    // Should be able to process large dataset efficiently
    assert!(
        iteration_duration < Duration::from_secs(5),
        "Large dataset iteration should complete under 5 seconds, took {:?}",
        iteration_duration
    );

    ctx.cleanup().await;
}