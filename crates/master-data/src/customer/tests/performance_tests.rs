use super::*;
use crate::customer::model::*;
use crate::customer::tests::integration_tests::MockCustomerRepository;
use crate::types::*;
use erp_core::{TenantId, UserId};
use chrono::Utc;
use rust_decimal::Decimal;
use uuid::Uuid;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::timeout;

/// Professional performance tests for Customer operations
/// These tests validate performance characteristics and scalability

#[cfg(test)]
mod customer_performance_tests {
    use super::*;

    async fn create_test_customers(repository: &MockCustomerRepository, count: usize, tenant_id: TenantId) -> Vec<Uuid> {
        let mut customer_ids = Vec::new();

        for i in 0..count {
            let customer_id = Uuid::new_v4();
            let customer = Customer {
                id: customer_id,
                customer_number: format!("PERF-{:06}", i),
                external_ids: {
                    let mut map = HashMap::new();
                    map.insert("SAP_ID".to_string(), format!("SAP{:06}", i));
                    map.insert("CRM_ID".to_string(), format!("CRM{:06}", i));
                    map
                },
                legal_name: format!("Performance Test Corp {}", i),
                trade_names: vec![format!("PerfCorp {}", i)],
                customer_type: if i % 2 == 0 { CustomerType::B2b } else { CustomerType::B2c },
                industry_classification: match i % 5 {
                    0 => IndustryClassification::Technology,
                    1 => IndustryClassification::Manufacturing,
                    2 => IndustryClassification::Healthcare,
                    3 => IndustryClassification::Finance,
                    _ => IndustryClassification::Retail,
                },
                business_size: match i % 4 {
                    0 => BusinessSize::Small,
                    1 => BusinessSize::Medium,
                    2 => BusinessSize::Large,
                    _ => BusinessSize::Enterprise,
                },
                parent_customer_id: None,
                corporate_group_id: None,
                customer_hierarchy_level: 0,
                consolidation_group: None,
                lifecycle_stage: match i % 4 {
                    0 => CustomerLifecycleStage::Lead,
                    1 => CustomerLifecycleStage::Prospect,
                    2 => CustomerLifecycleStage::ActiveCustomer,
                    _ => CustomerLifecycleStage::InactiveCustomer,
                },
                status: EntityStatus::Active,
                credit_status: if i % 3 == 0 { CreditStatus::Approved } else { CreditStatus::Pending },
                primary_address_id: None,
                billing_address_id: None,
                shipping_address_ids: vec![],
                addresses: vec![],
                primary_contact_id: None,
                contacts: vec![],
                customer_lifetime_value: Some(Decimal::new((i as i64 + 1) * 1000, 2)),
                annual_revenue: Some(Decimal::new((i as i64 + 1) * 100000, 2)),
                employee_count: Some((i + 1) * 10),
                financial_info: Some(FinancialInfo {
                    credit_limit: Decimal::new((i as i64 + 1) * 5000, 2),
                    payment_terms: PaymentTerms::Net30,
                    currency: "USD".to_string(),
                    tax_id: Some(format!("TAX{:09}", i)),
                    dunning_procedure: DunningProcedure::Standard,
                    payment_tolerance: Decimal::new(100, 2),
                    discount_percentage: Decimal::new(5, 2),
                    risk_category: match i % 3 {
                        0 => RiskCategory::Low,
                        1 => RiskCategory::Medium,
                        _ => RiskCategory::High,
                    },
                    collection_profile: CollectionProfile::Standard,
                }),
                sales_info: None,
                marketing_info: None,
                compliance_info: None,
                preferences: CustomerPreferences {
                    preferred_communication_method: CommunicationMethod::Email,
                    preferred_language: "en".to_string(),
                    timezone: "UTC".to_string(),
                    invoice_delivery_method: InvoiceDeliveryMethod::Email,
                    statement_frequency: StatementFrequency::Monthly,
                    portal_access_enabled: true,
                    mobile_app_enabled: false,
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
                    tags: vec![format!("tag{}", i % 5)],
                    custom_fields: HashMap::new(),
                    integration_metadata: HashMap::new(),
                },
            };

            repository.save(&customer).await.unwrap();
            customer_ids.push(customer_id);
        }

        customer_ids
    }

    #[tokio::test]
    async fn test_customer_creation_performance() {
        let repository = MockCustomerRepository::new();
        let tenant_id = TenantId(Uuid::new_v4());

        let customer_count = 1000;
        let start_time = Instant::now();

        let customer_ids = create_test_customers(&repository, customer_count, tenant_id).await;

        let elapsed = start_time.elapsed();
        let ops_per_second = customer_count as f64 / elapsed.as_secs_f64();

        println!("Created {} customers in {:?} ({:.2} ops/sec)",
                customer_count, elapsed, ops_per_second);

        assert_eq!(customer_ids.len(), customer_count);
        assert!(ops_per_second > 100.0, "Should create at least 100 customers per second");
        assert!(elapsed < Duration::from_secs(30), "Should complete within 30 seconds");
    }

    #[tokio::test]
    async fn test_customer_bulk_retrieval_performance() {
        let repository = MockCustomerRepository::new();
        let tenant_id = TenantId(Uuid::new_v4());

        let customer_count = 500;
        let customer_ids = create_test_customers(&repository, customer_count, tenant_id).await;

        let start_time = Instant::now();

        // Test bulk retrieval by ID
        let mut retrieved_count = 0;
        for customer_id in &customer_ids {
            let customer = repository.find_by_id(*customer_id).await.unwrap();
            if customer.is_some() {
                retrieved_count += 1;
            }
        }

        let elapsed = start_time.elapsed();
        let ops_per_second = retrieved_count as f64 / elapsed.as_secs_f64();

        println!("Retrieved {} customers in {:?} ({:.2} ops/sec)",
                retrieved_count, elapsed, ops_per_second);

        assert_eq!(retrieved_count, customer_count);
        assert!(ops_per_second > 200.0, "Should retrieve at least 200 customers per second");
        assert!(elapsed < Duration::from_secs(10), "Should complete within 10 seconds");
    }

    #[tokio::test]
    async fn test_customer_search_performance() {
        let repository = MockCustomerRepository::new();
        let tenant_id = TenantId(Uuid::new_v4());

        let customer_count = 1000;
        let _customer_ids = create_test_customers(&repository, customer_count, tenant_id).await;

        let start_time = Instant::now();

        // Test various search criteria
        let criteria_sets = vec![
            CustomerSearchCriteria {
                customer_types: vec![CustomerType::B2b],
                limit: Some(100),
                offset: Some(0),
                ..Default::default()
            },
            CustomerSearchCriteria {
                industries: vec![IndustryClassification::Technology],
                limit: Some(100),
                offset: Some(0),
                ..Default::default()
            },
            CustomerSearchCriteria {
                lifecycle_stages: vec![CustomerLifecycleStage::ActiveCustomer],
                limit: Some(100),
                offset: Some(0),
                ..Default::default()
            },
        ];

        let mut total_results = 0;
        for criteria in criteria_sets {
            let results = repository.list(&criteria).await.unwrap();
            total_results += results.len();
        }

        let elapsed = start_time.elapsed();
        let searches_per_second = 3.0 / elapsed.as_secs_f64();

        println!("Executed 3 searches returning {} results in {:?} ({:.2} searches/sec)",
                total_results, elapsed, searches_per_second);

        assert!(total_results > 0, "Should return search results");
        assert!(searches_per_second > 10.0, "Should execute at least 10 searches per second");
        assert!(elapsed < Duration::from_secs(5), "Should complete within 5 seconds");
    }

    #[tokio::test]
    async fn test_customer_update_performance() {
        let repository = MockCustomerRepository::new();
        let tenant_id = TenantId(Uuid::new_v4());

        let customer_count = 500;
        let customer_ids = create_test_customers(&repository, customer_count, tenant_id).await;

        let start_time = Instant::now();

        // Update all customers
        let mut updated_count = 0;
        for customer_id in &customer_ids {
            if let Some(mut customer) = repository.find_by_id(*customer_id).await.unwrap() {
                customer.legal_name = format!("Updated {}", customer.legal_name);
                customer.metadata.version += 1;
                customer.metadata.modified_at = Utc::now();

                repository.save(&customer).await.unwrap();
                updated_count += 1;
            }
        }

        let elapsed = start_time.elapsed();
        let ops_per_second = updated_count as f64 / elapsed.as_secs_f64();

        println!("Updated {} customers in {:?} ({:.2} ops/sec)",
                updated_count, elapsed, ops_per_second);

        assert_eq!(updated_count, customer_count);
        assert!(ops_per_second > 50.0, "Should update at least 50 customers per second");
        assert!(elapsed < Duration::from_secs(20), "Should complete within 20 seconds");
    }

    #[tokio::test]
    async fn test_customer_external_id_lookup_performance() {
        let repository = MockCustomerRepository::new();
        let tenant_id = TenantId(Uuid::new_v4());

        let customer_count = 1000;
        let _customer_ids = create_test_customers(&repository, customer_count, tenant_id).await;

        let start_time = Instant::now();

        // Test external ID lookups
        let mut found_count = 0;
        for i in 0..100 {
            let sap_id = format!("SAP{:06}", i);
            let customer = repository.find_by_external_id("SAP_ID", &sap_id).await.unwrap();
            if customer.is_some() {
                found_count += 1;
            }
        }

        let elapsed = start_time.elapsed();
        let lookups_per_second = 100.0 / elapsed.as_secs_f64();

        println!("Performed 100 external ID lookups, found {} customers in {:?} ({:.2} lookups/sec)",
                found_count, elapsed, lookups_per_second);

        assert_eq!(found_count, 100);
        assert!(lookups_per_second > 50.0, "Should perform at least 50 lookups per second");
        assert!(elapsed < Duration::from_secs(5), "Should complete within 5 seconds");
    }

    #[tokio::test]
    async fn test_customer_number_existence_check_performance() {
        let repository = MockCustomerRepository::new();
        let tenant_id = TenantId(Uuid::new_v4());

        let customer_count = 1000;
        let _customer_ids = create_test_customers(&repository, customer_count, tenant_id).await;

        let start_time = Instant::now();

        // Test existence checks
        let mut exists_count = 0;
        for i in 0..100 {
            let customer_number = format!("PERF-{:06}", i);
            let exists = repository.exists_by_customer_number(&customer_number).await.unwrap();
            if exists {
                exists_count += 1;
            }
        }

        let elapsed = start_time.elapsed();
        let checks_per_second = 100.0 / elapsed.as_secs_f64();

        println!("Performed 100 existence checks, found {} customers in {:?} ({:.2} checks/sec)",
                exists_count, elapsed, checks_per_second);

        assert_eq!(exists_count, 100);
        assert!(checks_per_second > 100.0, "Should perform at least 100 checks per second");
        assert!(elapsed < Duration::from_secs(3), "Should complete within 3 seconds");
    }

    #[tokio::test]
    async fn test_customer_pagination_performance() {
        let repository = MockCustomerRepository::new();
        let tenant_id = TenantId(Uuid::new_v4());

        let customer_count = 1000;
        let _customer_ids = create_test_customers(&repository, customer_count, tenant_id).await;

        let start_time = Instant::now();

        // Test pagination performance
        let page_size = 50;
        let mut total_retrieved = 0;
        let mut page = 0;

        loop {
            let criteria = CustomerSearchCriteria {
                limit: Some(page_size),
                offset: Some(page * page_size),
                ..Default::default()
            };

            let results = repository.list(&criteria).await.unwrap();
            if results.is_empty() {
                break;
            }

            total_retrieved += results.len();
            page += 1;

            if page >= 20 { // Limit to first 20 pages for performance test
                break;
            }
        }

        let elapsed = start_time.elapsed();
        let pages_per_second = page as f64 / elapsed.as_secs_f64();

        println!("Retrieved {} customers across {} pages in {:?} ({:.2} pages/sec)",
                total_retrieved, page, elapsed, pages_per_second);

        assert!(total_retrieved >= 1000, "Should retrieve all customers");
        assert!(pages_per_second > 5.0, "Should process at least 5 pages per second");
        assert!(elapsed < Duration::from_secs(10), "Should complete within 10 seconds");
    }

    #[tokio::test]
    async fn test_customer_count_performance() {
        let repository = MockCustomerRepository::new();
        let tenant_id = TenantId(Uuid::new_v4());

        let customer_count = 1000;
        let _customer_ids = create_test_customers(&repository, customer_count, tenant_id).await;

        let start_time = Instant::now();

        // Test count operations with different criteria
        let criteria_sets = vec![
            CustomerSearchCriteria::default(),
            CustomerSearchCriteria {
                customer_types: vec![CustomerType::B2b],
                ..Default::default()
            },
            CustomerSearchCriteria {
                industries: vec![IndustryClassification::Technology, IndustryClassification::Finance],
                ..Default::default()
            },
            CustomerSearchCriteria {
                lifecycle_stages: vec![CustomerLifecycleStage::ActiveCustomer],
                ..Default::default()
            },
        ];

        let mut total_count = 0;
        for criteria in criteria_sets {
            let count = repository.count(&criteria).await.unwrap();
            total_count += count;
        }

        let elapsed = start_time.elapsed();
        let counts_per_second = 4.0 / elapsed.as_secs_f64();

        println!("Performed 4 count operations returning {} total in {:?} ({:.2} counts/sec)",
                total_count, elapsed, counts_per_second);

        assert!(total_count > 0, "Should return count results");
        assert!(counts_per_second > 20.0, "Should perform at least 20 counts per second");
        assert!(elapsed < Duration::from_secs(2), "Should complete within 2 seconds");
    }

    #[tokio::test]
    async fn test_customer_memory_usage() {
        let repository = MockCustomerRepository::new();
        let tenant_id = TenantId(Uuid::new_v4());

        // Test memory efficiency with large number of customers
        let customer_count = 10000;

        let start_time = Instant::now();
        let _customer_ids = create_test_customers(&repository, customer_count, tenant_id).await;
        let creation_time = start_time.elapsed();

        // Measure memory footprint (approximation)
        let customers = repository.customers.read().unwrap();
        let customer_count_in_memory = customers.len();
        drop(customers);

        // Test that large dataset operations complete in reasonable time
        let search_start = Instant::now();
        let criteria = CustomerSearchCriteria {
            limit: Some(1000),
            offset: Some(0),
            ..Default::default()
        };
        let results = repository.list(&criteria).await.unwrap();
        let search_time = search_start.elapsed();

        println!("Memory test: {} customers created in {:?}, search returned {} results in {:?}",
                customer_count_in_memory, creation_time, results.len(), search_time);

        assert_eq!(customer_count_in_memory, customer_count);
        assert_eq!(results.len(), 1000);
        assert!(creation_time < Duration::from_secs(60), "Creation should complete within 60 seconds");
        assert!(search_time < Duration::from_secs(5), "Search should complete within 5 seconds");
    }

    #[tokio::test]
    async fn test_concurrent_customer_operations() {
        let repository = std::sync::Arc::new(MockCustomerRepository::new());
        let tenant_id = TenantId(Uuid::new_v4());

        let start_time = Instant::now();

        // Spawn concurrent tasks
        let mut tasks = Vec::new();

        for i in 0..10 {
            let repo = repository.clone();
            let tid = tenant_id;

            let task = tokio::spawn(async move {
                let customer_id = Uuid::new_v4();
                let customer = Customer {
                    id: customer_id,
                    customer_number: format!("CONC-{:03}", i),
                    external_ids: HashMap::new(),
                    legal_name: format!("Concurrent Test Corp {}", i),
                    trade_names: vec![],
                    customer_type: CustomerType::B2b,
                    industry_classification: IndustryClassification::Technology,
                    business_size: BusinessSize::Medium,
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
                    customer_lifetime_value: None,
                    annual_revenue: None,
                    employee_count: None,
                    financial_info: None,
                    sales_info: None,
                    marketing_info: None,
                    compliance_info: None,
                    preferences: CustomerPreferences {
                        preferred_communication_method: CommunicationMethod::Email,
                        preferred_language: "en".to_string(),
                        timezone: "UTC".to_string(),
                        invoice_delivery_method: InvoiceDeliveryMethod::Email,
                        statement_frequency: StatementFrequency::Monthly,
                        portal_access_enabled: true,
                        mobile_app_enabled: false,
                        notification_preferences: HashMap::new(),
                    },
                    metadata: CustomerMetadata {
                        version: 1,
                        tenant_id: tid,
                        created_at: Utc::now(),
                        created_by: UserId(Uuid::new_v4()),
                        modified_at: Utc::now(),
                        modified_by: UserId(Uuid::new_v4()),
                        is_deleted: false,
                        deleted_at: None,
                        deleted_by: None,
                        tags: vec![],
                        custom_fields: HashMap::new(),
                        integration_metadata: HashMap::new(),
                    },
                };

                // Save customer
                repo.save(&customer).await.unwrap();

                // Perform some operations
                let found = repo.find_by_id(customer_id).await.unwrap();
                assert!(found.is_some());

                let exists = repo.exists_by_customer_number(&customer.customer_number).await.unwrap();
                assert!(exists);

                customer_id
            });

            tasks.push(task);
        }

        // Wait for all tasks to complete
        let mut completed_customers = Vec::new();
        for task in tasks {
            let customer_id = task.await.unwrap();
            completed_customers.push(customer_id);
        }

        let elapsed = start_time.elapsed();

        println!("Completed {} concurrent operations in {:?}",
                completed_customers.len(), elapsed);

        assert_eq!(completed_customers.len(), 10);
        assert!(elapsed < Duration::from_secs(10), "Concurrent operations should complete within 10 seconds");

        // Verify all customers were created
        for customer_id in completed_customers {
            let customer = repository.find_by_id(customer_id).await.unwrap();
            assert!(customer.is_some(), "All concurrently created customers should exist");
        }
    }

    #[tokio::test]
    async fn test_operation_timeout_handling() {
        let repository = MockCustomerRepository::new();
        let tenant_id = TenantId(Uuid::new_v4());

        // Test that operations complete within expected timeouts
        let customer_id = Uuid::new_v4();
        let customer = Customer {
            id: customer_id,
            customer_number: "TIMEOUT-001".to_string(),
            external_ids: HashMap::new(),
            legal_name: "Timeout Test Corp".to_string(),
            trade_names: vec![],
            customer_type: CustomerType::B2b,
            industry_classification: IndustryClassification::Technology,
            business_size: BusinessSize::Medium,
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
            customer_lifetime_value: None,
            annual_revenue: None,
            employee_count: None,
            financial_info: None,
            sales_info: None,
            marketing_info: None,
            compliance_info: None,
            preferences: CustomerPreferences {
                preferred_communication_method: CommunicationMethod::Email,
                preferred_language: "en".to_string(),
                timezone: "UTC".to_string(),
                invoice_delivery_method: InvoiceDeliveryMethod::Email,
                statement_frequency: StatementFrequency::Monthly,
                portal_access_enabled: true,
                mobile_app_enabled: false,
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
                tags: vec![],
                custom_fields: HashMap::new(),
                integration_metadata: HashMap::new(),
            },
        };

        // Test save operation with timeout
        let save_result = timeout(Duration::from_secs(5), repository.save(&customer)).await;
        assert!(save_result.is_ok(), "Save operation should complete within timeout");
        assert!(save_result.unwrap().is_ok(), "Save operation should succeed");

        // Test find operation with timeout
        let find_result = timeout(Duration::from_secs(5), repository.find_by_id(customer_id)).await;
        assert!(find_result.is_ok(), "Find operation should complete within timeout");
        assert!(find_result.unwrap().is_ok(), "Find operation should succeed");

        // Test search operation with timeout
        let criteria = CustomerSearchCriteria::default();
        let search_result = timeout(Duration::from_secs(5), repository.list(&criteria)).await;
        assert!(search_result.is_ok(), "Search operation should complete within timeout");
        assert!(search_result.unwrap().is_ok(), "Search operation should succeed");
    }
}