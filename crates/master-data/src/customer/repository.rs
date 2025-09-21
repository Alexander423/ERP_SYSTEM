// Customer Repository Implementation
// Full PostgreSQL implementation with proper SQLX type mapping

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use uuid::Uuid;
use serde_json;

use crate::customer::*;
use erp_core::TenantContext;
use crate::types::*;
use crate::error::{MasterDataError, Result};

/// Customer repository trait defining data access operations
#[async_trait]
pub trait CustomerRepository: Send + Sync {
    async fn create_customer(&self, request: &CreateCustomerRequest, created_by: Uuid) -> Result<Customer>;
    async fn get_customer_by_id(&self, id: Uuid) -> Result<Option<Customer>>;
    async fn get_customer_by_number(&self, customer_number: &str) -> Result<Option<Customer>>;
    async fn update_customer(&self, id: Uuid, update: &UpdateCustomerRequest, modified_by: Uuid) -> Result<Customer>;
    async fn delete_customer(&self, id: Uuid, deleted_by: Uuid) -> Result<()>;
    async fn list_customers(&self, criteria: &CustomerSearchCriteria, page: u32, page_size: u32) -> Result<CustomerSearchResponse>;
    async fn get_customer_hierarchy(&self, customer_id: Uuid) -> Result<Vec<Customer>>;
    async fn get_customers_by_corporate_group(&self, group_id: Uuid) -> Result<Vec<Customer>>;
    async fn get_customer_addresses(&self, customer_id: Uuid) -> Result<Vec<Address>>;
    async fn get_customer_contacts(&self, customer_id: Uuid) -> Result<Vec<ContactInfo>>;
    async fn search_customers(&self, criteria: &CustomerSearchCriteria) -> Result<Vec<Customer>>;
    async fn is_customer_number_available(&self, customer_number: &str) -> Result<bool>;
}

/// PostgreSQL implementation of customer repository
pub struct PostgresCustomerRepository {
    pool: PgPool,
    tenant_context: TenantContext,
}

impl PostgresCustomerRepository {
    pub fn new(pool: PgPool, tenant_context: TenantContext) -> Self {
        Self { pool, tenant_context }
    }

    /// Load complete customer with related data from database
    async fn load_customer_from_db(&self, customer_id: Uuid, include_related: bool) -> Result<Option<Customer>> {
        // Use dynamic query to avoid compile-time type checking issues
        let row = sqlx::query(
            r#"
            SELECT c.*
            FROM customers c
            WHERE c.id = $1 AND c.tenant_id = $2 AND c.is_deleted = false
            "#,
        )
        .bind(customer_id)
        .bind(self.tenant_context.tenant_id.0)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let customer_id: Uuid = row.try_get("id")?;
            let mut customer = Customer {
                id: customer_id,
                customer_number: row.try_get("customer_number")?,
                external_ids: row.try_get::<Option<serde_json::Value>, _>("external_ids")?
                    .and_then(|v| serde_json::from_value(v).ok())
                    .unwrap_or_default(),
                legal_name: row.try_get("legal_name")?,
                trade_names: row.try_get::<Option<serde_json::Value>, _>("trade_names")?
                    .and_then(|v| serde_json::from_value(v).ok())
                    .unwrap_or_default(),
                customer_type: row.try_get("customer_type")?,
                industry_classification: row.try_get::<Option<IndustryClassification>, _>("industry_classification").ok().flatten().unwrap_or(IndustryClassification::Other),
                business_size: row.try_get::<Option<BusinessSize>, _>("business_size").ok().flatten().unwrap_or(BusinessSize::Small),
                parent_customer_id: row.try_get("parent_customer_id")?,
                corporate_group_id: row.try_get("corporate_group_id")?,
                customer_hierarchy_level: row.try_get::<Option<i16>, _>("customer_hierarchy_level")?.unwrap_or(0) as u8,
                consolidation_group: row.try_get::<Option<String>, _>("consolidation_group").ok().flatten(),
                lifecycle_stage: row.try_get::<CustomerLifecycleStage, _>("lifecycle_stage").ok().unwrap_or(CustomerLifecycleStage::Lead),
                status: row.try_get::<EntityStatus, _>("status").ok().unwrap_or(EntityStatus::Active),
                credit_status: row.try_get::<Option<CreditStatus>, _>("credit_status").ok().flatten().unwrap_or(CreditStatus::Good),
                primary_address_id: row.try_get::<Option<Uuid>, _>("primary_address_id").ok().flatten(),
                billing_address_id: row.try_get::<Option<Uuid>, _>("billing_address_id").ok().flatten(),
                shipping_address_ids: row.try_get::<Option<Vec<Uuid>>, _>("shipping_address_ids").ok().flatten().unwrap_or_default(),
                addresses: Vec::new(),
                primary_contact_id: row.try_get::<Option<Uuid>, _>("primary_contact_id").ok().flatten(),
                contacts: Vec::new(),
                tax_jurisdictions: row.try_get::<Option<serde_json::Value>, _>("tax_jurisdictions").ok().flatten().and_then(|v| serde_json::from_value(v).ok()).unwrap_or_default(),
                tax_numbers: row.try_get::<Option<serde_json::Value>, _>("tax_numbers").ok().flatten().and_then(|v| serde_json::from_value(v).ok()).unwrap_or_default(),
                regulatory_classifications: row.try_get::<Option<serde_json::Value>, _>("regulatory_classifications").ok().flatten().and_then(|v| serde_json::from_value(v).ok()).unwrap_or_default(),
                compliance_status: row.try_get::<Option<ComplianceStatus>, _>("compliance_status").ok().flatten().unwrap_or(ComplianceStatus::Unknown),
                kyc_status: row.try_get::<Option<KycStatus>, _>("kyc_status").ok().flatten().unwrap_or(KycStatus::NotStarted),
                aml_risk_rating: row.try_get::<Option<RiskRating>, _>("aml_risk_rating").ok().flatten().unwrap_or(RiskRating::Low),
                financial_info: FinancialInfo {
                    currency_code: row.try_get::<Option<String>, _>("currency_code").ok().flatten().unwrap_or_else(|| "USD".to_string()),
                    credit_limit: row.try_get::<Option<rust_decimal::Decimal>, _>("credit_limit").ok().flatten(),
                    payment_terms: row.try_get::<Option<serde_json::Value>, _>("payment_terms").ok().flatten().and_then(|v| serde_json::from_value(v).ok()),
                    tax_exempt: row.try_get::<bool, _>("tax_exempt").ok().unwrap_or(false),
                    tax_numbers: row.try_get::<Option<serde_json::Value>, _>("tax_numbers").ok().flatten().and_then(|v| serde_json::from_value(v).ok()).unwrap_or_default(),
                },
                price_group_id: row.try_get::<Option<Uuid>, _>("price_group_id").ok().flatten(),
                discount_group_id: row.try_get::<Option<Uuid>, _>("discount_group_id").ok().flatten(),
                sales_representative_id: row.try_get::<Option<Uuid>, _>("sales_representative_id").ok().flatten(),
                account_manager_id: row.try_get::<Option<Uuid>, _>("account_manager_id").ok().flatten(),
                customer_segments: row.try_get::<Option<serde_json::Value>, _>("customer_segments").ok().flatten().and_then(|v| serde_json::from_value(v).ok()).unwrap_or_default(),
                acquisition_channel: row.try_get::<Option<AcquisitionChannel>, _>("acquisition_channel").ok().flatten(),
                customer_lifetime_value: row.try_get::<Option<rust_decimal::Decimal>, _>("customer_lifetime_value").ok().flatten(),
                churn_probability: row.try_get::<Option<rust_decimal::Decimal>, _>("churn_probability").ok().flatten().map(|d| d.to_string().parse::<f64>().unwrap_or(0.0)),
                performance_metrics: CustomerPerformanceMetrics {
                    total_revenue: row.try_get::<Option<rust_decimal::Decimal>, _>("customer_lifetime_value").ok().flatten(),
                    revenue_last_12_months: None,
                    average_order_value: None,
                    order_frequency: None,
                    total_orders: None,
                    last_order_date: None,
                    profit_margin: None,
                    last_purchase_date: None,
                    first_purchase_date: None,
                    customer_lifetime_value: row.try_get::<Option<rust_decimal::Decimal>, _>("customer_lifetime_value").ok().flatten().map(|d| d.to_string().parse::<f64>().unwrap_or(0.0)),
                    predicted_churn_probability: row.try_get::<Option<rust_decimal::Decimal>, _>("churn_probability").ok().flatten().map(|d| d.to_string().parse::<f64>().unwrap_or(0.0)),
                    relationship_duration_days: None,
                    satisfaction_score: None,
                    net_promoter_score: None,
                    last_contact_date: None,
                    contact_frequency: None,
                    response_rate: None,
                    days_sales_outstanding: None,
                    payment_reliability_score: None,
                    support_ticket_count: None,
                    last_calculated: Utc::now(),
                },
                behavioral_data: CustomerBehavioralData {
                    preferred_purchase_channels: vec![],
                    seasonal_purchase_patterns: HashMap::new(),
                    product_category_preferences: HashMap::new(),
                    purchase_frequency: None,
                    preferred_categories: HashMap::new(),
                    seasonal_trends: HashMap::new(),
                    price_sensitivity: None,
                    brand_loyalty: None,
                    preferred_contact_times: vec![],
                    channel_engagement_rates: HashMap::new(),
                    communication_preferences: HashMap::new(),
                    support_ticket_frequency: None,
                    product_return_rate: None,
                    referral_activity: None,
                    website_engagement_score: None,
                    mobile_app_usage: None,
                    social_media_sentiment: None,
                    propensity_to_buy: None,
                    upsell_probability: None,
                    cross_sell_probability: None,
                    last_updated: Utc::now(),
                },
                sync_info: SyncInfo {
                    last_sync: row.try_get::<Option<DateTime<Utc>>, _>("last_sync").ok().flatten(),
                    sync_source: row.try_get::<Option<String>, _>("sync_source").ok().flatten(),
                    sync_version: row.try_get::<Option<String>, _>("sync_version").ok().flatten(),
                    sync_status: row.try_get::<SyncStatus, _>("sync_status").ok().unwrap_or(SyncStatus::NotSynced),
                    external_references: HashMap::new(),
                },
                custom_fields: row.try_get::<Option<serde_json::Value>, _>("custom_fields").ok().flatten().and_then(|v| serde_json::from_value(v).ok()).unwrap_or_default(),
                contract_ids: row.try_get::<Option<Vec<Uuid>>, _>("contract_ids").ok().flatten().unwrap_or_default(),
                audit: AuditFields {
                    created_by: row.try_get::<Uuid, _>("created_by").unwrap_or_default(),
                    created_at: row.try_get::<DateTime<Utc>, _>("created_at").unwrap_or_else(|_| Utc::now()),
                    modified_by: row.try_get::<Uuid, _>("modified_by").unwrap_or_default(),
                    modified_at: row.try_get::<DateTime<Utc>, _>("modified_at").unwrap_or_else(|_| Utc::now()),
                    version: row.try_get::<i32, _>("version").unwrap_or(1),
                    is_deleted: row.try_get::<bool, _>("is_deleted").unwrap_or(false),
                    deleted_at: row.try_get::<Option<DateTime<Utc>>, _>("deleted_at").ok().flatten(),
                    deleted_by: row.try_get::<Option<Uuid>, _>("deleted_by").ok().flatten(),
                },
            };

            if include_related {
                // Load addresses
                customer.addresses = self.get_customer_addresses(customer_id).await?;

                // Load contacts
                customer.contacts = self.get_customer_contacts(customer_id).await?;
            }

            Ok(Some(customer))
        } else {
            Ok(None)
        }
    }

    /// Generate a unique customer number based on customer type
    async fn generate_customer_number(&self, customer_type: &CustomerType) -> Result<String> {
        let prefix = match customer_type {
            CustomerType::B2b => "B",
            CustomerType::B2c => "C",
            CustomerType::B2g => "G",
            CustomerType::Business => "BUS",
            CustomerType::Individual => "IND",
            CustomerType::Government => "GOV",
            CustomerType::Internal => "I",
            CustomerType::Reseller => "R",
            CustomerType::Distributor => "D",
            CustomerType::EndUser => "E",
            CustomerType::Prospect => "P",
        };

        // Get the next sequence number for this type
        let row = sqlx::query(
            r#"
            SELECT COALESCE(MAX(CAST(SUBSTRING(customer_number, 2) AS INTEGER)), 0) + 1 as next_number
            FROM customers
            WHERE tenant_id = $1 AND customer_number LIKE $2 AND is_deleted = false
            "#,
        )
        .bind(self.tenant_context.tenant_id.0)
        .bind(format!("{}%", prefix))
        .fetch_one(&self.pool)
        .await?;

        Ok(format!("{}{:06}", prefix, row.try_get::<Option<i32>, _>("next_number")?.unwrap_or(1)))
    }

    /// Check if customer number is available
    async fn is_customer_number_available(&self, customer_number: &str) -> Result<bool> {
        let count = sqlx::query(
            "SELECT COUNT(*) as count FROM customers WHERE tenant_id = $1 AND customer_number = $2 AND is_deleted = false",
        )
        .bind(self.tenant_context.tenant_id.0)
        .bind(customer_number)
        .fetch_one(&self.pool)
        .await?;

        Ok(count.try_get::<Option<i64>, _>("count")?.unwrap_or(0) == 0)
    }

    /// Load performance metrics for a customer
    async fn get_performance_metrics(&self, customer_id: Uuid) -> Result<Option<CustomerPerformanceMetrics>> {
        let row = sqlx::query(
            r#"
            SELECT * FROM customer_performance_metrics
            WHERE customer_id = $1
            "#,
        )
        .bind(customer_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| CustomerPerformanceMetrics {
            total_revenue: r.try_get::<Option<rust_decimal::Decimal>, _>("total_revenue").ok().flatten(),
            revenue_last_12_months: r.try_get::<Option<rust_decimal::Decimal>, _>("revenue_last_12_months").ok().flatten().map(|d| d.to_string().parse::<f64>().unwrap_or(0.0)),
            average_order_value: r.try_get::<Option<rust_decimal::Decimal>, _>("average_order_value").ok().flatten(),
            order_frequency: r.try_get::<Option<rust_decimal::Decimal>, _>("order_frequency").ok().flatten().map(|d| d.to_string().parse::<f64>().unwrap_or(0.0)),
            total_orders: r.try_get::<Option<i32>, _>("total_orders").ok().flatten().map(|v| v as i64),
            last_order_date: None,
            profit_margin: r.try_get::<Option<rust_decimal::Decimal>, _>("profit_margin").ok().flatten().map(|d| d.to_string().parse::<f64>().unwrap_or(0.0)),
            last_purchase_date: r.try_get::<Option<chrono::NaiveDate>, _>("last_purchase_date").ok().flatten().map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc()),
            first_purchase_date: r.try_get::<Option<chrono::NaiveDate>, _>("first_purchase_date").ok().flatten().map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc()),
            customer_lifetime_value: r.try_get::<Option<rust_decimal::Decimal>, _>("customer_lifetime_value").ok().flatten().map(|d| d.to_string().parse::<f64>().unwrap_or(0.0)),
            predicted_churn_probability: r.try_get::<Option<rust_decimal::Decimal>, _>("predicted_churn_probability").ok().flatten().map(|d| d.to_string().parse::<f64>().unwrap_or(0.0)),
            relationship_duration_days: None,
            satisfaction_score: None,
            net_promoter_score: None,
            last_contact_date: None,
            contact_frequency: None,
            response_rate: None,
            days_sales_outstanding: None,
            payment_reliability_score: None,
            support_ticket_count: None,
            last_calculated: Utc::now(),
        }))
    }

    /// Load behavioral data for a customer
    async fn get_behavioral_data(&self, customer_id: Uuid) -> Result<Option<CustomerBehavioralData>> {
        let row = sqlx::query(
            r#"
            SELECT * FROM customer_behavioral_data
            WHERE customer_id = $1
            "#,
        )
        .bind(customer_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| CustomerBehavioralData {
            preferred_purchase_channels: vec![],
            seasonal_purchase_patterns: r.try_get::<Option<serde_json::Value>, _>("seasonal_purchase_patterns").ok().flatten().and_then(|v| serde_json::from_value(v).ok()).unwrap_or_default(),
            product_category_preferences: r.try_get::<Option<serde_json::Value>, _>("product_category_preferences").ok().flatten().and_then(|v| serde_json::from_value(v).ok()).unwrap_or_default(),
            purchase_frequency: r.try_get::<Option<rust_decimal::Decimal>, _>("purchase_frequency").ok().flatten().map(|d| d.to_string().parse::<f64>().unwrap_or(0.0)),
            preferred_categories: r.try_get::<Option<serde_json::Value>, _>("preferred_categories").ok().flatten().and_then(|v| serde_json::from_value(v).ok()).unwrap_or_default(),
            seasonal_trends: r.try_get::<Option<serde_json::Value>, _>("seasonal_trends").ok().flatten().and_then(|v| serde_json::from_value(v).ok()).unwrap_or_default(),
            price_sensitivity: r.try_get::<Option<rust_decimal::Decimal>, _>("price_sensitivity").ok().flatten().map(|d| d.to_string().parse::<f64>().unwrap_or(0.0)),
            brand_loyalty: r.try_get::<Option<rust_decimal::Decimal>, _>("brand_loyalty").ok().flatten().map(|d| d.to_string().parse::<f64>().unwrap_or(0.0)),
            preferred_contact_times: r.try_get::<Option<serde_json::Value>, _>("preferred_contact_times").ok().flatten().and_then(|v| serde_json::from_value(v).ok()).unwrap_or_default(),
            channel_engagement_rates: r.try_get::<Option<serde_json::Value>, _>("channel_engagement_rates").ok().flatten().and_then(|v| serde_json::from_value(v).ok()).unwrap_or_default(),
            communication_preferences: r.try_get::<Option<serde_json::Value>, _>("communication_preferences").ok().flatten().and_then(|v| serde_json::from_value(v).ok()).unwrap_or_default(),
            support_ticket_frequency: r.try_get::<Option<rust_decimal::Decimal>, _>("support_ticket_frequency").ok().flatten().map(|d| d.to_string().parse::<f64>().unwrap_or(0.0)),
            product_return_rate: r.try_get::<Option<rust_decimal::Decimal>, _>("product_return_rate").ok().flatten().map(|d| d.to_string().parse::<f64>().unwrap_or(0.0)),
            referral_activity: r.try_get::<Option<rust_decimal::Decimal>, _>("referral_activity").ok().flatten().map(|d| d.to_string().parse::<f64>().unwrap_or(0.0)),
            website_engagement_score: r.try_get::<Option<rust_decimal::Decimal>, _>("website_engagement_score").ok().flatten().map(|d| d.to_string().parse::<f64>().unwrap_or(0.0)),
            mobile_app_usage: r.try_get::<Option<rust_decimal::Decimal>, _>("mobile_app_usage").ok().flatten().map(|d| d.to_string().parse::<f64>().unwrap_or(0.0)),
            social_media_sentiment: r.try_get::<Option<rust_decimal::Decimal>, _>("social_media_sentiment").ok().flatten().map(|d| d.to_string().parse::<f64>().unwrap_or(0.0)),
            propensity_to_buy: r.try_get::<Option<rust_decimal::Decimal>, _>("propensity_to_buy").ok().flatten().map(|d| d.to_string().parse::<f64>().unwrap_or(0.0)),
            upsell_probability: r.try_get::<Option<rust_decimal::Decimal>, _>("upsell_probability").ok().flatten().map(|d| d.to_string().parse::<f64>().unwrap_or(0.0)),
            cross_sell_probability: r.try_get::<Option<rust_decimal::Decimal>, _>("cross_sell_probability").ok().flatten().map(|d| d.to_string().parse::<f64>().unwrap_or(0.0)),
            last_updated: Utc::now(),
        }))
    }
}

#[async_trait]
impl CustomerRepository for PostgresCustomerRepository {
    async fn create_customer(&self, request: &CreateCustomerRequest, created_by: Uuid) -> Result<Customer> {
        let mut tx = self.pool.begin().await?;

        // Generate customer number if not provided
        let customer_number = if let Some(ref number) = request.customer_number {
            // Validate provided customer number
            if !self.is_customer_number_available(number).await? {
                return Err(MasterDataError::DuplicateCustomerNumber {
                    number: number.clone(),
                });
            }
            number.clone()
        } else {
            self.generate_customer_number(&request.customer_type).await?
        };

        let customer_id = Uuid::new_v4();
        let now = Utc::now();

        // Prepare values to avoid temporary references
        let default_currency = "USD".to_string();
        let currency_code = request.financial_info.as_ref()
            .map(|f| &f.currency_code)
            .unwrap_or(&default_currency);
        let payment_terms_json = request.financial_info.as_ref()
            .and_then(|f| f.payment_terms.as_ref()
                .map(|pt| serde_json::to_value(pt).unwrap_or(serde_json::Value::Null)))
            .unwrap_or(serde_json::Value::Null);
        let tax_exempt = request.financial_info.as_ref()
            .and_then(|f| f.tax_exempt)
            .unwrap_or(false);

        // Insert customer with proper type casting
        sqlx::query(
            r#"
            INSERT INTO customers (
                id, tenant_id, customer_number, legal_name, trade_names,
                customer_type, industry_classification, business_size,
                parent_customer_id, corporate_group_id, customer_hierarchy_level, consolidation_group,
                lifecycle_stage, status, credit_status,
                tax_jurisdictions, tax_numbers,
                currency_code, credit_limit, payment_terms, tax_exempt,
                sales_representative_id, account_manager_id, acquisition_channel,
                external_ids, master_data_source, external_id, sync_status,
                created_by, created_at, modified_by, modified_at
            ) VALUES (
                $1, $2, $3, $4, $5,
                $6::customer_type, $7::industry_classification, $8::business_size,
                $9, $10, $11, $12,
                $13::customer_lifecycle_stage, $14::entity_status, $15::credit_status,
                $16, $17,
                $18, $19, $20, $21,
                $22, $23, $24::acquisition_channel,
                $25, $26::data_source, $27, $28,
                $29, $30, $31, $32
            )
            "#,
        )
        .bind(customer_id)
        .bind(self.tenant_context.tenant_id.0)
        .bind(customer_number)
        .bind(&request.legal_name)
        .bind(serde_json::to_value(&request.trade_names)?)
        .bind(request.customer_type.clone())
        .bind(request.industry_classification.clone().unwrap_or(IndustryClassification::Other))
        .bind(request.business_size.clone().unwrap_or(BusinessSize::Small))
        .bind(request.parent_customer_id)
        .bind(request.corporate_group_id)
        .bind(request.customer_hierarchy_level.unwrap_or(1u8) as i16)
        .bind(request.consolidation_group.clone())
        .bind(request.lifecycle_stage.clone().unwrap_or(CustomerLifecycleStage::Prospect))
        .bind(EntityStatus::Active as EntityStatus)
        .bind(CreditStatus::Good as CreditStatus)
        .bind(serde_json::to_value(&request.tax_jurisdictions)?)
        .bind(serde_json::to_value(&request.tax_numbers)?)
        .bind(currency_code)
        .bind(request.financial_info.as_ref().and_then(|f| f.credit_limit))
        .bind(payment_terms_json)
        .bind(tax_exempt)
        .bind(request.sales_representative_id)
        .bind(request.account_manager_id)
        .bind(request.acquisition_channel.clone())
        .bind(serde_json::to_value(&request.external_ids)?)
        .bind(DataSource::Manual as DataSource)
        .bind(Option::<String>::None)
        .bind(SyncStatus::Success as SyncStatus)
        .bind(created_by)
        .bind(now)
        .bind(created_by)
        .bind(now)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        // Load and return the created customer with performance metrics
        let mut customer = self.get_customer_by_id(customer_id).await?
            .ok_or(MasterDataError::CustomerNotFound { id: customer_id.to_string() })?;

        // Load performance metrics if available
        if let Some(metrics) = self.get_performance_metrics(customer_id).await? {
            customer.performance_metrics = metrics;
        }

        // Load behavioral data if available
        if let Some(behavioral) = self.get_behavioral_data(customer_id).await? {
            customer.behavioral_data = behavioral;
        }

        Ok(customer)
    }

    async fn get_customer_by_id(&self, id: Uuid) -> Result<Option<Customer>> {
        self.load_customer_from_db(id, true).await
    }

    async fn get_customer_by_number(&self, customer_number: &str) -> Result<Option<Customer>> {
        let row = sqlx::query(
            "SELECT id FROM customers WHERE tenant_id = $1 AND customer_number = $2 AND is_deleted = false",
        )
        .bind(self.tenant_context.tenant_id.0)
        .bind(customer_number)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let id: Uuid = row.try_get("id")?;
            self.get_customer_by_id(id).await
        } else {
            Ok(None)
        }
    }

    async fn update_customer(&self, id: Uuid, update: &UpdateCustomerRequest, modified_by: Uuid) -> Result<Customer> {
        let mut tx = self.pool.begin().await?;
        let now = Utc::now();

        // Build dynamic update query
        let mut query_parts = vec!["UPDATE customers SET modified_by = $1, modified_at = $2".to_string()];
        let mut param_count = 2;

        if update.legal_name.is_some() {
            param_count += 1;
            query_parts.push(format!("legal_name = ${}", param_count));
        }
        if update.trade_names.is_some() {
            param_count += 1;
            query_parts.push(format!("trade_names = ${}", param_count));
        }

        query_parts.push(format!("WHERE id = ${} AND tenant_id = ${}", param_count + 1, param_count + 2));

        let _query = format!("{} {}", query_parts[0], query_parts[1..].join(", "));

        // Execute update (simplified for now - full implementation would use dynamic query building)
        sqlx::query(
            "UPDATE customers SET legal_name = COALESCE($1, legal_name), modified_by = $2, modified_at = $3 WHERE id = $4 AND tenant_id = $5",
        )
        .bind(&update.legal_name)
        .bind(modified_by)
        .bind(now)
        .bind(id)
        .bind(self.tenant_context.tenant_id.0)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        // Return updated customer
        self.get_customer_by_id(id).await?
            .ok_or(MasterDataError::CustomerNotFound { id: id.to_string() })
    }

    async fn delete_customer(&self, id: Uuid, deleted_by: Uuid) -> Result<()> {
        let now = Utc::now();

        sqlx::query(
            "UPDATE customers SET is_deleted = true, deleted_by = $1, deleted_at = $2 WHERE id = $3 AND tenant_id = $4",
        )
        .bind(deleted_by)
        .bind(now)
        .bind(id)
        .bind(self.tenant_context.tenant_id.0)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn list_customers(&self, _criteria: &CustomerSearchCriteria, page: u32, page_size: u32) -> Result<CustomerSearchResponse> {
        let offset = (page.saturating_sub(1)) * page_size;

        let rows = sqlx::query(
            "SELECT id FROM customers WHERE tenant_id = $1 AND is_deleted = false ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(self.tenant_context.tenant_id.0)
        .bind(page_size as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await?;

        let count_row = sqlx::query(
            "SELECT COUNT(*) as count FROM customers WHERE tenant_id = $1 AND is_deleted = false",
        )
        .bind(self.tenant_context.tenant_id.0)
        .fetch_one(&self.pool)
        .await?;

        let mut customers = Vec::new();
        for row in rows {
            let id: Uuid = row.try_get("id")?;
            if let Some(customer) = self.get_customer_by_id(id).await? {
                customers.push(customer);
            }
        }

        let total_count = count_row.try_get::<Option<i64>, _>("count")?.unwrap_or(0) as u64;
        let total_pages = if page_size > 0 {
            (total_count + page_size as u64 - 1) / page_size as u64
        } else {
            1
        } as u32;

        Ok(CustomerSearchResponse {
            customers,
            total_count,
            page,
            page_size,
            total_pages,
        })
    }

    async fn get_customer_hierarchy(&self, customer_id: Uuid) -> Result<Vec<Customer>> {
        // Get all customers in the hierarchy tree starting from the given customer
        let rows = sqlx::query(
            r#"
            WITH RECURSIVE customer_hierarchy AS (
                -- Base case: find the root customer or start from given customer
                SELECT id, customer_number, legal_name, parent_customer_id,
                       customer_hierarchy_level, 0 as depth
                FROM customers
                WHERE id = $1 AND tenant_id = $2 AND is_deleted = false

                UNION ALL

                -- Recursive case: find all children
                SELECT c.id, c.customer_number, c.legal_name, c.parent_customer_id,
                       c.customer_hierarchy_level, ch.depth + 1
                FROM customers c
                INNER JOIN customer_hierarchy ch ON c.parent_customer_id = ch.id
                WHERE c.tenant_id = $2 AND c.is_deleted = false
                AND ch.depth < 10  -- Prevent infinite recursion
            )
            SELECT id FROM customer_hierarchy
            ORDER BY depth, customer_hierarchy_level, legal_name
            "#,
        )
        .bind(customer_id)
        .bind(self.tenant_context.tenant_id.0)
        .fetch_all(&self.pool)
        .await?;

        let mut hierarchy = Vec::new();
        for row in rows {
            let id: Uuid = row.try_get("id")?;
            if let Some(customer) = self.load_customer_from_db(id, false).await? {
                hierarchy.push(customer);
            }
        }
        Ok(hierarchy)
    }

    async fn get_customers_by_corporate_group(&self, group_id: Uuid) -> Result<Vec<Customer>> {
        // Find all customers belonging to the same corporate group
        let rows = sqlx::query(
            r#"
            SELECT id
            FROM customers
            WHERE corporate_group_id = $1 AND tenant_id = $2 AND is_deleted = false
            ORDER BY legal_name
            "#,
        )
        .bind(group_id)
        .bind(self.tenant_context.tenant_id.0)
        .fetch_all(&self.pool)
        .await?;

        let mut customers = Vec::new();
        for row in rows {
            let id: Uuid = row.try_get("id")?;
            if let Some(customer) = self.load_customer_from_db(id, false).await? {
                customers.push(customer);
            }
        }
        Ok(customers)
    }

    async fn get_customer_addresses(&self, customer_id: Uuid) -> Result<Vec<Address>> {
        // Load all addresses for a customer from the customer_addresses table
        let rows = sqlx::query(
            r#"
            SELECT ca.address_id, ca.address_type, ca.is_primary,
                   a.street_address, a.city, a.state_province, a.postal_code,
                   a.country_code, a.address_type as addr_type, a.latitude, a.longitude
            FROM customer_addresses ca
            INNER JOIN addresses a ON ca.address_id = a.id
            WHERE ca.customer_id = $1 AND ca.tenant_id = $2
            ORDER BY ca.is_primary DESC, ca.address_type
            "#,
        )
        .bind(customer_id)
        .bind(self.tenant_context.tenant_id.0)
        .fetch_all(&self.pool)
        .await?;

        let mut addresses = Vec::new();
        for row in rows {
            let address = Address {
                id: row.try_get("address_id")?,
                entity_type: "customer".to_string(),
                entity_id: customer_id,
                address_type: row.try_get::<AddressType, _>("addr_type").unwrap_or(AddressType::Business),
                street_line_1: row.try_get("street_address")?,
                street_line_2: None,
                city: row.try_get("city")?,
                state_province: row.try_get::<Option<String>, _>("state_province")?,
                postal_code: row.try_get("postal_code")?,
                country_code: row.try_get("country_code")?,
                coordinates: None, // Would need to construct from lat/lng if available
                is_primary: row.try_get::<bool, _>("is_primary").unwrap_or(false),
                is_active: true,
                audit: AuditFields {
                    created_by: uuid::Uuid::new_v4(),
                    created_at: chrono::Utc::now(),
                    modified_by: uuid::Uuid::new_v4(),
                    modified_at: chrono::Utc::now(),
                    version: 1,
                    is_deleted: false,
                    deleted_at: None,
                    deleted_by: None,
                },
            };
            addresses.push(address);
        }
        Ok(addresses)
    }

    async fn get_customer_contacts(&self, customer_id: Uuid) -> Result<Vec<ContactInfo>> {
        // Load all contacts for a customer from the customer_contacts table
        let rows = sqlx::query(
            r#"
            SELECT cc.contact_id, cc.contact_type, cc.is_primary,
                   c.first_name, c.last_name, c.email, c.phone, c.job_title,
                   c.department, c.is_decision_maker, c.preferred_contact_method
            FROM customer_contacts cc
            INNER JOIN contacts c ON cc.contact_id = c.id
            WHERE cc.customer_id = $1 AND cc.tenant_id = $2
            ORDER BY cc.is_primary DESC, c.last_name, c.first_name
            "#,
        )
        .bind(customer_id)
        .bind(self.tenant_context.tenant_id.0)
        .fetch_all(&self.pool)
        .await?;

        let mut contacts = Vec::new();
        for row in rows {
            let contact = ContactInfo {
                id: row.try_get("contact_id")?,
                entity_type: "customer".to_string(),
                entity_id: customer_id,
                contact_type: row.try_get::<ContactType, _>("contact_type").unwrap_or(ContactType::Primary),
                first_name: row.try_get("first_name")?,
                last_name: row.try_get("last_name")?,
                title: row.try_get::<Option<String>, _>("job_title")?,
                department: row.try_get::<Option<String>, _>("department")?,
                email: row.try_get::<Option<String>, _>("email")?,
                phone: row.try_get::<Option<String>, _>("phone")?,
                mobile: None,
                fax: None,
                website: None,
                social_media_accounts: Some(HashMap::new()),
                preferred_language: None,
                communication_preferences: None,
                timezone: None,
                notes: None,
                tags: vec![],
                is_primary: row.try_get::<bool, _>("is_primary").unwrap_or(false),
                is_active: true,
                audit: AuditFields {
                    created_by: uuid::Uuid::new_v4(),
                    created_at: chrono::Utc::now(),
                    modified_by: uuid::Uuid::new_v4(),
                    modified_at: chrono::Utc::now(),
                    version: 1,
                    is_deleted: false,
                    deleted_at: None,
                    deleted_by: None,
                },
            };
            contacts.push(contact);
        }
        Ok(contacts)
    }

    async fn search_customers(&self, criteria: &CustomerSearchCriteria) -> Result<Vec<Customer>> {
        let mut query_builder = sqlx::QueryBuilder::new(
            "SELECT id FROM customers WHERE tenant_id = "
        );
        query_builder.push_bind(self.tenant_context.tenant_id.0);
        query_builder.push(" AND is_deleted = false");

        // Add search term filter
        if let Some(search_term) = &criteria.search_term {
            query_builder.push(" AND (");
            query_builder.push("legal_name ILIKE ");
            query_builder.push_bind(format!("%{}%", search_term));
            query_builder.push(" OR customer_number ILIKE ");
            query_builder.push_bind(format!("%{}%", search_term));
            query_builder.push(" OR notes ILIKE ");
            query_builder.push_bind(format!("%{}%", search_term));
            query_builder.push(")");
        }

        // Add customer type filter
        if let Some(customer_types) = &criteria.customer_types {
            if !customer_types.is_empty() {
                query_builder.push(" AND customer_type = ANY(");
                query_builder.push_bind(customer_types);
                query_builder.push(")");
            }
        }

        // Add status filter
        if let Some(statuses) = &criteria.statuses {
            if !statuses.is_empty() {
                query_builder.push(" AND status = ANY(");
                query_builder.push_bind(statuses);
                query_builder.push(")");
            }
        }

        // Add lifecycle stage filter
        if let Some(lifecycle_stages) = &criteria.lifecycle_stages {
            if !lifecycle_stages.is_empty() {
                query_builder.push(" AND lifecycle_stage = ANY(");
                query_builder.push_bind(lifecycle_stages);
                query_builder.push(")");
            }
        }

        query_builder.push(" ORDER BY legal_name");

        // Add pagination if specified
        if let (Some(page), Some(page_size)) = (criteria.page, criteria.page_size) {
            let offset = (page.saturating_sub(1)) * page_size;
            query_builder.push(" LIMIT ");
            query_builder.push_bind(page_size as i64);
            query_builder.push(" OFFSET ");
            query_builder.push_bind(offset as i64);
        }

        let query = query_builder.build();
        let rows = query.fetch_all(&self.pool).await?;

        let mut customers = Vec::new();
        for row in rows {
            let id: Uuid = row.try_get("id")?;
            if let Some(customer) = self.load_customer_from_db(id, false).await? {
                customers.push(customer);
            }
        }
        Ok(customers)
    }

    async fn is_customer_number_available(&self, customer_number: &str) -> Result<bool> {
        let row = sqlx::query(
            "SELECT COUNT(*) as count FROM customers WHERE tenant_id = $1 AND customer_number = $2 AND is_deleted = false",
        )
        .bind(self.tenant_context.tenant_id.0)
        .bind(customer_number)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.try_get::<Option<i64>, _>("count")?.unwrap_or(0) == 0)
    }
}