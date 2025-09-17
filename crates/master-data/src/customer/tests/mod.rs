pub mod simple;
pub mod unit;
pub mod integration;
pub mod security;
pub mod performance;
pub mod comprehensive_test;

use sqlx::{PgPool, Pool, Postgres};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::customer::*;
use crate::types::*;

pub struct TestContext {
    pub pool: PgPool,
    pub tenant_id: TenantId,
    pub test_user_id: Uuid,
}

impl TestContext {
    pub async fn new(pool: PgPool) -> Self {
        let tenant_id = TenantId(Uuid::new_v4());
        let test_user_id = Uuid::new_v4();

        sqlx::query(
            "INSERT INTO tenants (id, name, settings, created_by, modified_by)
             VALUES ($1, $2, '{}', $3, $3)",
        )
        .bind(tenant_id.0)
        .bind(format!("Test Tenant {}", tenant_id.0))
        .bind(test_user_id)
        .execute(&pool)
        .await
        .expect("Failed to create test tenant");

        Self {
            pool,
            tenant_id,
            test_user_id,
        }
    }

    pub async fn cleanup(&self) {
        sqlx::query("DELETE FROM tenants WHERE id = $1")
            .bind(self.tenant_id.0)
            .execute(&self.pool)
            .await
            .expect("Failed to cleanup test tenant");
    }

    pub fn create_test_customer_request(&self) -> CreateCustomerRequest {
        CreateCustomerRequest {
            customer_number: Some(format!("TEST-{}", Uuid::new_v4().to_string()[..8].to_uppercase())),
            legal_name: "Test Customer Ltd.".to_string(),
            display_name: Some("Test Customer".to_string()),
            customer_type: CustomerType::Business,
            lifecycle_stage: CustomerLifecycleStage::Lead,
            addresses: vec![Address {
                address_type: AddressType::Billing,
                street1: "123 Test Street".to_string(),
                street2: None,
                city: "Test City".to_string(),
                state_province: Some("Test State".to_string()),
                postal_code: "12345".to_string(),
                country: "US".to_string(),
                is_primary: true,
            }],
            contacts: vec![Contact {
                contact_type: ContactType::Primary,
                first_name: "John".to_string(),
                last_name: "Doe".to_string(),
                email: Some("john.doe@testcustomer.com".to_string()),
                phone: Some("+1-555-123-4567".to_string()),
                position: Some("CEO".to_string()),
                department: None,
                is_primary: true,
            }],
            tax_numbers: vec![],
            notes: Some("Test customer for automated testing".to_string()),
            tags: vec!["test".to_string(), "automation".to_string()],
            custom_fields: std::collections::HashMap::new(),
            financial_info: None,
            metadata: std::collections::HashMap::new(),
        }
    }
}

pub fn assert_customer_equals(actual: &Customer, expected: &Customer) {
    assert_eq!(actual.id, expected.id);
    assert_eq!(actual.customer_number, expected.customer_number);
    assert_eq!(actual.legal_name, expected.legal_name);
    assert_eq!(actual.customer_type, expected.customer_type);
    assert_eq!(actual.lifecycle_stage, expected.lifecycle_stage);
    assert_eq!(actual.addresses.len(), expected.addresses.len());
    assert_eq!(actual.contacts.len(), expected.contacts.len());
}

pub async fn create_test_pool() -> PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://erp_admin:erp_secure_password_change_in_production@localhost:5432/erp_main".to_string());

    Pool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}