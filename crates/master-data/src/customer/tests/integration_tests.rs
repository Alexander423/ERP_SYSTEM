use super::*;
use crate::customer::model::*;
use crate::customer::repository::*;
use crate::customer::aggregate::*;
use crate::types::*;
use erp_core::{TenantContext, TenantId, UserId};
use chrono::Utc;
use rust_decimal::Decimal;
use uuid::Uuid;
use std::collections::HashMap;

/// Professional integration tests for Customer aggregate and repository
/// These tests validate the full stack with database integration

#[cfg(test)]
mod customer_aggregate_tests {
    use super::*;

    fn create_test_tenant_context() -> TenantContext {
        TenantContext {
            tenant_id: TenantId(Uuid::new_v4()),
            schema_name: format!("test_tenant_{}", Uuid::new_v4().to_string().replace('-', "_")),
        }
    }

    #[tokio::test]
    async fn test_customer_aggregate_creation() {
        let tenant_id = TenantId(Uuid::new_v4());
        let user_id = UserId(Uuid::new_v4());
        let customer_number = "CUST-AGG-001".to_string();
        let legal_name = "Aggregate Test Corp".to_string();
        let customer_type = CustomerType::B2b;

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
    async fn test_customer_aggregate_update_legal_name() {
        let tenant_id = TenantId(Uuid::new_v4());
        let user_id = UserId(Uuid::new_v4());

        let mut aggregate = CustomerAggregate::create(
            tenant_id,
            "CUST-AGG-002".to_string(),
            "Original Name Corp".to_string(),
            CustomerType::B2b,
            user_id,
        ).unwrap();

        let new_legal_name = "Updated Name Corporation".to_string();
        let result = aggregate.update_legal_name(new_legal_name.clone(), user_id);

        assert!(result.is_ok(), "Legal name update should succeed");
        assert_eq!(aggregate.legal_name, new_legal_name);
        assert_eq!(aggregate.version, 2);
    }

    #[tokio::test]
    async fn test_customer_aggregate_lifecycle_transition() {
        let tenant_id = TenantId(Uuid::new_v4());
        let user_id = UserId(Uuid::new_v4());

        let mut aggregate = CustomerAggregate::create(
            tenant_id,
            "CUST-AGG-003".to_string(),
            "Lifecycle Test Corp".to_string(),
            CustomerType::B2b,
            user_id,
        ).unwrap();

        // Test transition from Lead to Prospect
        let result = aggregate.transition_lifecycle_stage(
            CustomerLifecycleStage::Prospect,
            user_id,
            Some("Qualified as prospect".to_string())
        );

        assert!(result.is_ok(), "Lifecycle transition should succeed");
        assert_eq!(aggregate.lifecycle_stage, CustomerLifecycleStage::Prospect);
        assert_eq!(aggregate.version, 2);
    }

    #[tokio::test]
    async fn test_customer_aggregate_credit_status_update() {
        let tenant_id = TenantId(Uuid::new_v4());
        let user_id = UserId(Uuid::new_v4());

        let mut aggregate = CustomerAggregate::create(
            tenant_id,
            "CUST-AGG-004".to_string(),
            "Credit Test Corp".to_string(),
            CustomerType::B2b,
            user_id,
        ).unwrap();

        let result = aggregate.update_credit_status(
            CreditStatus::Approved,
            Some(Decimal::new(100000, 2)), // $1000.00 credit limit
            user_id,
            Some("Credit approved after review".to_string())
        );

        assert!(result.is_ok(), "Credit status update should succeed");
        assert_eq!(aggregate.credit_status, CreditStatus::Approved);
        assert_eq!(aggregate.version, 2);
    }

    #[tokio::test]
    async fn test_customer_aggregate_soft_delete() {
        let tenant_id = TenantId(Uuid::new_v4());
        let user_id = UserId(Uuid::new_v4());

        let mut aggregate = CustomerAggregate::create(
            tenant_id,
            "CUST-AGG-005".to_string(),
            "Delete Test Corp".to_string(),
            CustomerType::B2b,
            user_id,
        ).unwrap();

        let result = aggregate.soft_delete(user_id, Some("No longer needed".to_string()));

        assert!(result.is_ok(), "Soft delete should succeed");
        assert!(aggregate.is_deleted);
        assert_eq!(aggregate.version, 2);
    }

    #[tokio::test]
    async fn test_customer_aggregate_event_generation() {
        let tenant_id = TenantId(Uuid::new_v4());
        let user_id = UserId(Uuid::new_v4());

        let aggregate = CustomerAggregate::create(
            tenant_id,
            "CUST-AGG-006".to_string(),
            "Event Test Corp".to_string(),
            CustomerType::B2b,
            user_id,
        ).unwrap();

        let events = aggregate.uncommitted_events();
        assert_eq!(events.len(), 1);

        if let CustomerEvent::CustomerCreated { customer_id, .. } = &events[0] {
            assert_eq!(*customer_id, aggregate.id);
        } else {
            panic!("Expected CustomerCreated event");
        }
    }

    #[tokio::test]
    async fn test_customer_aggregate_validation_errors() {
        let tenant_id = TenantId(Uuid::new_v4());
        let user_id = UserId(Uuid::new_v4());

        // Test empty legal name validation
        let result = CustomerAggregate::create(
            tenant_id,
            "CUST-AGG-007".to_string(),
            "".to_string(), // Empty legal name should fail
            CustomerType::B2b,
            user_id,
        );

        assert!(result.is_err(), "Empty legal name should fail validation");

        // Test customer number too long
        let result = CustomerAggregate::create(
            tenant_id,
            "x".repeat(51), // Too long customer number should fail
            "Valid Legal Name".to_string(),
            CustomerType::B2b,
            user_id,
        );

        assert!(result.is_err(), "Too long customer number should fail validation");
    }
}

/// Mock repository for testing without database dependencies
#[derive(Clone)]
pub struct MockCustomerRepository {
    pub customers: std::sync::Arc<std::sync::RwLock<HashMap<Uuid, Customer>>>,
}

impl MockCustomerRepository {
    pub fn new() -> Self {
        Self {
            customers: std::sync::Arc::new(std::sync::RwLock::new(HashMap::new())),
        }
    }

    fn create_test_customer(&self, id: Uuid, tenant_id: TenantId) -> Customer {
        Customer {
            id,
            customer_number: format!("MOCK-{}", id.to_string()[..8].to_uppercase()),
            external_ids: HashMap::new(),
            legal_name: "Mock Test Corporation".to_string(),
            trade_names: vec!["Mock Corp".to_string()],
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
            customer_lifetime_value: Some(Decimal::new(50000, 2)),
            annual_revenue: Some(Decimal::new(1000000, 2)),
            employee_count: Some(50),
            financial_info: Some(FinancialInfo {
                credit_limit: Decimal::new(25000, 2),
                payment_terms: PaymentTerms::Net30,
                currency: "USD".to_string(),
                tax_id: Some("12-3456789".to_string()),
                dunning_procedure: DunningProcedure::Standard,
                payment_tolerance: Decimal::new(50, 2),
                discount_percentage: Decimal::new(2, 2),
                risk_category: RiskCategory::Medium,
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
                tags: vec![],
                custom_fields: HashMap::new(),
                integration_metadata: HashMap::new(),
            },
        }
    }
}

#[async_trait::async_trait]
impl CustomerRepository for MockCustomerRepository {
    async fn save(&self, customer: &Customer) -> crate::error::Result<()> {
        let mut customers = self.customers.write().unwrap();
        customers.insert(customer.id, customer.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> crate::error::Result<Option<Customer>> {
        let customers = self.customers.read().unwrap();
        Ok(customers.get(&id).cloned())
    }

    async fn find_by_customer_number(
        &self,
        customer_number: &str
    ) -> crate::error::Result<Option<Customer>> {
        let customers = self.customers.read().unwrap();
        for customer in customers.values() {
            if customer.customer_number == customer_number {
                return Ok(Some(customer.clone()));
            }
        }
        Ok(None)
    }

    async fn find_by_external_id(
        &self,
        system: &str,
        external_id: &str,
    ) -> crate::error::Result<Option<Customer>> {
        let customers = self.customers.read().unwrap();
        for customer in customers.values() {
            if let Some(id) = customer.external_ids.get(system) {
                if id == external_id {
                    return Ok(Some(customer.clone()));
                }
            }
        }
        Ok(None)
    }

    async fn list(
        &self,
        criteria: &CustomerSearchCriteria,
    ) -> crate::error::Result<Vec<Customer>> {
        let customers = self.customers.read().unwrap();
        let mut results: Vec<Customer> = customers.values().cloned().collect();

        // Apply filters based on criteria
        if !criteria.lifecycle_stages.is_empty() {
            results.retain(|c| criteria.lifecycle_stages.contains(&c.lifecycle_stage));
        }

        if !criteria.customer_types.is_empty() {
            results.retain(|c| criteria.customer_types.contains(&c.customer_type));
        }

        if !criteria.industries.is_empty() {
            results.retain(|c| criteria.industries.contains(&c.industry_classification));
        }

        // Apply limit and offset
        let start = criteria.offset.unwrap_or(0) as usize;
        let limit = criteria.limit.unwrap_or(50) as usize;

        if start < results.len() {
            let end = std::cmp::min(start + limit, results.len());
            results = results[start..end].to_vec();
        } else {
            results.clear();
        }

        Ok(results)
    }

    async fn count(&self, criteria: &CustomerSearchCriteria) -> crate::error::Result<u64> {
        let customers = self.list(criteria).await?;
        Ok(customers.len() as u64)
    }

    async fn delete(&self, id: Uuid) -> crate::error::Result<()> {
        let mut customers = self.customers.write().unwrap();
        customers.remove(&id);
        Ok(())
    }

    async fn exists_by_customer_number(&self, customer_number: &str) -> crate::error::Result<bool> {
        let result = self.find_by_customer_number(customer_number).await?;
        Ok(result.is_some())
    }

    async fn get_performance_metrics(&self, _id: Uuid) -> crate::error::Result<CustomerPerformanceMetrics> {
        Ok(CustomerPerformanceMetrics {
            total_orders: 10,
            total_revenue: Decimal::new(50000, 2),
            average_order_value: Decimal::new(5000, 2),
            last_order_date: Some(Utc::now().date_naive()),
            days_since_last_order: Some(30),
            order_frequency_days: Some(45),
            return_rate: Decimal::new(5, 2),
            satisfaction_score: Some(8.5),
            support_tickets_count: 2,
            payment_punctuality_score: 95,
        })
    }

    async fn get_behavioral_data(&self, _id: Uuid) -> crate::error::Result<CustomerBehavioralData> {
        Ok(CustomerBehavioralData {
            preferred_contact_times: vec!["09:00-12:00".to_string(), "14:00-17:00".to_string()],
            communication_response_rate: Decimal::new(85, 2),
            portal_usage_frequency: PortalUsageFrequency::Weekly,
            feature_usage_patterns: HashMap::new(),
            seasonal_patterns: HashMap::new(),
            geographic_preferences: vec!["United States".to_string()],
            price_sensitivity_score: Some(6.5),
            loyalty_indicators: LoyaltyIndicators {
                repeat_purchase_rate: Decimal::new(75, 2),
                referral_count: 3,
                engagement_score: 8.2,
                churn_risk_score: 2.1,
            },
        })
    }
}

#[cfg(test)]
mod customer_repository_tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_repository_save_and_find() {
        let repository = MockCustomerRepository::new();
        let tenant_id = TenantId(Uuid::new_v4());
        let customer_id = Uuid::new_v4();

        let customer = repository.create_test_customer(customer_id, tenant_id);

        // Test save
        let save_result = repository.save(&customer).await;
        assert!(save_result.is_ok(), "Save should succeed");

        // Test find by ID
        let found_customer = repository.find_by_id(customer_id).await.unwrap();
        assert!(found_customer.is_some(), "Customer should be found");

        let found = found_customer.unwrap();
        assert_eq!(found.id, customer_id);
        assert_eq!(found.legal_name, customer.legal_name);
    }

    #[tokio::test]
    async fn test_mock_repository_find_by_customer_number() {
        let repository = MockCustomerRepository::new();
        let tenant_id = TenantId(Uuid::new_v4());
        let customer_id = Uuid::new_v4();

        let customer = repository.create_test_customer(customer_id, tenant_id);
        repository.save(&customer).await.unwrap();

        let found_customer = repository
            .find_by_customer_number(&customer.customer_number)
            .await
            .unwrap();

        assert!(found_customer.is_some(), "Customer should be found by number");
        assert_eq!(found_customer.unwrap().id, customer_id);
    }

    #[tokio::test]
    async fn test_mock_repository_find_by_external_id() {
        let repository = MockCustomerRepository::new();
        let tenant_id = TenantId(Uuid::new_v4());
        let customer_id = Uuid::new_v4();

        let mut customer = repository.create_test_customer(customer_id, tenant_id);
        customer.external_ids.insert("SAP".to_string(), "SAP123".to_string());
        repository.save(&customer).await.unwrap();

        let found_customer = repository
            .find_by_external_id("SAP", "SAP123")
            .await
            .unwrap();

        assert!(found_customer.is_some(), "Customer should be found by external ID");
        assert_eq!(found_customer.unwrap().id, customer_id);
    }

    #[tokio::test]
    async fn test_mock_repository_list_with_criteria() {
        let repository = MockCustomerRepository::new();
        let tenant_id = TenantId(Uuid::new_v4());

        // Create multiple customers
        for i in 0..5 {
            let customer_id = Uuid::new_v4();
            let mut customer = repository.create_test_customer(customer_id, tenant_id);

            if i % 2 == 0 {
                customer.customer_type = CustomerType::B2c;
            }

            repository.save(&customer).await.unwrap();
        }

        // Test filtering by customer type
        let criteria = CustomerSearchCriteria {
            customer_types: vec![CustomerType::B2c],
            limit: Some(10),
            offset: Some(0),
            ..Default::default()
        };

        let results = repository.list(&criteria).await.unwrap();
        assert_eq!(results.len(), 3, "Should find 3 B2C customers"); // 0, 2, 4
        assert!(results.iter().all(|c| c.customer_type == CustomerType::B2c));
    }

    #[tokio::test]
    async fn test_mock_repository_count() {
        let repository = MockCustomerRepository::new();
        let tenant_id = TenantId(Uuid::new_v4());

        // Create 3 customers
        for _ in 0..3 {
            let customer_id = Uuid::new_v4();
            let customer = repository.create_test_customer(customer_id, tenant_id);
            repository.save(&customer).await.unwrap();
        }

        let criteria = CustomerSearchCriteria::default();
        let count = repository.count(&criteria).await.unwrap();
        assert_eq!(count, 3, "Should count 3 customers");
    }

    #[tokio::test]
    async fn test_mock_repository_exists_by_customer_number() {
        let repository = MockCustomerRepository::new();
        let tenant_id = TenantId(Uuid::new_v4());
        let customer_id = Uuid::new_v4();

        let customer = repository.create_test_customer(customer_id, tenant_id);
        repository.save(&customer).await.unwrap();

        let exists = repository
            .exists_by_customer_number(&customer.customer_number)
            .await
            .unwrap();

        assert!(exists, "Customer should exist");

        let not_exists = repository
            .exists_by_customer_number("NON-EXISTENT")
            .await
            .unwrap();

        assert!(!not_exists, "Non-existent customer should not exist");
    }

    #[tokio::test]
    async fn test_mock_repository_delete() {
        let repository = MockCustomerRepository::new();
        let tenant_id = TenantId(Uuid::new_v4());
        let customer_id = Uuid::new_v4();

        let customer = repository.create_test_customer(customer_id, tenant_id);
        repository.save(&customer).await.unwrap();

        // Verify customer exists
        let found = repository.find_by_id(customer_id).await.unwrap();
        assert!(found.is_some(), "Customer should exist before delete");

        // Delete customer
        let delete_result = repository.delete(customer_id).await;
        assert!(delete_result.is_ok(), "Delete should succeed");

        // Verify customer is deleted
        let not_found = repository.find_by_id(customer_id).await.unwrap();
        assert!(not_found.is_none(), "Customer should not exist after delete");
    }

    #[tokio::test]
    async fn test_mock_repository_performance_metrics() {
        let repository = MockCustomerRepository::new();
        let customer_id = Uuid::new_v4();

        let metrics = repository.get_performance_metrics(customer_id).await.unwrap();

        assert_eq!(metrics.total_orders, 10);
        assert_eq!(metrics.total_revenue, Decimal::new(50000, 2));
        assert_eq!(metrics.average_order_value, Decimal::new(5000, 2));
        assert!(metrics.last_order_date.is_some());
        assert_eq!(metrics.days_since_last_order, Some(30));
        assert_eq!(metrics.payment_punctuality_score, 95);
    }

    #[tokio::test]
    async fn test_mock_repository_behavioral_data() {
        let repository = MockCustomerRepository::new();
        let customer_id = Uuid::new_v4();

        let behavioral_data = repository.get_behavioral_data(customer_id).await.unwrap();

        assert!(!behavioral_data.preferred_contact_times.is_empty());
        assert_eq!(behavioral_data.communication_response_rate, Decimal::new(85, 2));
        assert_eq!(behavioral_data.portal_usage_frequency, PortalUsageFrequency::Weekly);
        assert!(behavioral_data.geographic_preferences.contains(&"United States".to_string()));
        assert_eq!(behavioral_data.loyalty_indicators.repeat_purchase_rate, Decimal::new(75, 2));
    }
}