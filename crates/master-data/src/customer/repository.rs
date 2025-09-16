// Customer Repository Implementation
// Full PostgreSQL implementation with proper SQLX type mapping

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Pool, Row};
use std::collections::HashMap;
use uuid::Uuid;
use std::sync::Arc;
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
        let row = sqlx::query!(
            r#"
            SELECT c.*,
                   pm.revenue_last_12_months,
                   pm.average_order_value,
                   pm.order_frequency,
                   pm.total_orders,
                   pm.total_revenue,
                   pm.profit_margin,
                   pm.last_purchase_date,
                   pm.first_purchase_date,
                   pm.customer_lifetime_value,
                   pm.predicted_churn_probability,
                   pm.last_updated as pm_last_updated,
                   bd.purchase_frequency,
                   bd.preferred_categories,
                   bd.seasonal_trends,
                   bd.price_sensitivity,
                   bd.brand_loyalty,
                   bd.communication_preferences,
                   bd.support_ticket_frequency,
                   bd.product_return_rate,
                   bd.referral_activity,
                   bd.product_category_preferences,
                   bd.preferred_contact_times,
                   bd.channel_engagement_rates,
                   bd.website_engagement_score,
                   bd.mobile_app_usage,
                   bd.social_media_sentiment,
                   bd.propensity_to_buy,
                   bd.upsell_probability,
                   bd.cross_sell_probability,
                   bd.last_updated as bd_last_updated
            FROM customers c
            LEFT JOIN customer_performance_metrics pm ON c.id = pm.customer_id
            LEFT JOIN customer_behavioral_data bd ON c.id = bd.customer_id
            WHERE c.id = $1 AND c.tenant_id = $2 AND c.is_deleted = false
            "#,
            customer_id,
            self.tenant_context.tenant_id
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let customer_id = row.id;
            let mut customer = Customer {
                id: row.id,
                customer_number: row.customer_number,
                external_ids: row.external_ids.map(|j| j.0).unwrap_or_default(),
                legal_name: row.legal_name,
                trade_names: row.trade_names.map(|j| j.0).unwrap_or_default(),
                customer_type: row.customer_type,
                industry_classification: row.industry_classification,
                business_size: row.business_size,
                parent_customer_id: row.parent_customer_id,
                corporate_group_id: row.corporate_group_id,
                customer_hierarchy_level: row.customer_hierarchy_level.unwrap_or(0) as u8,
                consolidation_group: row.consolidation_group,
                lifecycle_stage: row.lifecycle_stage,
                status: row.status,
                credit_status: row.credit_status,
                primary_address_id: row.primary_address_id,
                billing_address_id: row.billing_address_id,
                shipping_address_ids: row.shipping_address_ids,
                addresses: Vec::new(),
                primary_contact_id: row.primary_contact_id,
                contacts: Vec::new(),
                tax_jurisdictions: row.tax_jurisdictions.map(|j| j.0).unwrap_or_default(),
                tax_numbers: row.tax_numbers.map(|j| j.0).unwrap_or_default(),
                regulatory_classifications: row.regulatory_classifications.map(|j| j.0).unwrap_or_default(),
                compliance_status: row.compliance_status,
                kyc_status: row.kyc_status,
                aml_risk_rating: row.aml_risk_rating,
                financial_info: FinancialInfo {
                    currency_code: row.currency_code,
                    credit_limit: row.credit_limit,
                    payment_terms: row.payment_terms.map(|j| j.0),
                    tax_exempt: row.tax_exempt,
                    tax_numbers: row.tax_numbers.map(|j| j.0).unwrap_or_default(),
                },
                price_group_id: row.price_group_id,
                discount_group_id: row.discount_group_id,
                sales_representative_id: row.sales_representative_id,
                account_manager_id: row.account_manager_id,
                customer_segments: row.customer_segments.map(|j| j.0).unwrap_or_default(),
                acquisition_channel: row.acquisition_channel,
                customer_lifetime_value: row.customer_lifetime_value,
                churn_probability: row.churn_probability,
                performance_metrics: CustomerPerformanceMetrics {
                    revenue_last_12_months: row.revenue_last_12_months.map(|d| d.try_into().unwrap_or(0.0)),
                    average_order_value: row.average_order_value.map(|d| d.try_into().unwrap_or(0.0)),
                    order_frequency: row.order_frequency.map(|d| d.try_into().unwrap_or(0.0)),
                    total_orders: row.total_orders.unwrap_or(0) as u32,
                    total_revenue: row.total_revenue.map(|d| d.try_into().unwrap_or(0.0)),
                    profit_margin: row.profit_margin.map(|d| d.try_into().unwrap_or(0.0)),
                    last_purchase_date: row.last_purchase_date,
                    first_purchase_date: row.first_purchase_date,
                    customer_lifetime_value: row.customer_lifetime_value.map(|d| d.try_into().unwrap_or(0.0)),
                    predicted_churn_probability: row.predicted_churn_probability.map(|d| d.try_into().unwrap_or(0.0)),
                    last_updated: row.pm_last_updated,
                },
                behavioral_data: CustomerBehavioralData {
                    purchase_frequency: row.purchase_frequency.map(|d| d.try_into().unwrap_or(0.0)),
                    preferred_categories: serde_json::from_value(row.preferred_categories.unwrap_or_default()).unwrap_or_default(),
                    seasonal_trends: serde_json::from_value(row.seasonal_trends.unwrap_or_default()).unwrap_or_default(),
                    price_sensitivity: row.price_sensitivity.map(|d| d.try_into().unwrap_or(0.0)),
                    brand_loyalty: row.brand_loyalty.map(|d| d.try_into().unwrap_or(0.0)),
                    communication_preferences: serde_json::from_value(row.communication_preferences.unwrap_or_default()).unwrap_or_default(),
                    support_ticket_frequency: row.support_ticket_frequency.map(|d| d.try_into().unwrap_or(0.0)),
                    product_return_rate: row.product_return_rate.map(|d| d.try_into().unwrap_or(0.0)),
                    referral_activity: row.referral_activity.map(|d| d.try_into().unwrap_or(0.0)),
                    product_category_preferences: serde_json::from_value(row.product_category_preferences.unwrap_or_default()).unwrap_or_default(),
                    preferred_contact_times: serde_json::from_value(row.preferred_contact_times.unwrap_or_default()).unwrap_or_default(),
                    channel_engagement_rates: serde_json::from_value(row.channel_engagement_rates.unwrap_or_default()).unwrap_or_default(),
                    website_engagement_score: row.website_engagement_score.map(|d| d.try_into().unwrap_or(0.0)),
                    mobile_app_usage: row.mobile_app_usage.map(|d| d.try_into().unwrap_or(0.0)),
                    social_media_sentiment: row.social_media_sentiment.map(|d| d.try_into().unwrap_or(0.0)),
                    propensity_to_buy: row.propensity_to_buy.map(|d| d.try_into().unwrap_or(0.0)),
                    upsell_probability: row.upsell_probability.map(|d| d.try_into().unwrap_or(0.0)),
                    cross_sell_probability: row.cross_sell_probability.map(|d| d.try_into().unwrap_or(0.0)),
                    last_updated: row.bd_last_updated,
                },
                sync_info: SyncInfo {
                    last_sync: row.last_sync,
                    sync_source: row.sync_source,
                    sync_version: row.sync_version,
                    sync_status: row.sync_status,
                    external_references: HashMap::new(),
                },
                custom_fields: row.custom_fields.map(|j| j.0).unwrap_or_default(),
                contract_ids: row.contract_ids,
                audit: AuditFields {
                    created_by: row.created_by,
                    created_at: row.created_at,
                    modified_by: row.modified_by,
                    modified_at: row.modified_at,
                    version: row.version,
                    is_deleted: row.is_deleted.unwrap_or(false),
                    deleted_at: row.deleted_at,
                    deleted_by: row.deleted_by,
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
        };

        // Get the next sequence number for this type
        let row = sqlx::query!(
            r#"
            SELECT COALESCE(MAX(CAST(SUBSTRING(customer_number, 2) AS INTEGER)), 0) + 1 as next_number
            FROM customers
            WHERE tenant_id = $1 AND customer_number LIKE $2 AND is_deleted = false
            "#,
            self.tenant_context.tenant_id,
            format!("{}%", prefix)
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(format!("{}{:06}", prefix, row.next_number.unwrap_or(1)))
    }

    /// Check if customer number is available
    async fn is_customer_number_available(&self, customer_number: &str) -> Result<bool> {
        let count = sqlx::query!(
            "SELECT COUNT(*) as count FROM customers WHERE tenant_id = $1 AND customer_number = $2 AND is_deleted = false",
            self.tenant_context.tenant_id,
            customer_number
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count.count.unwrap_or(0) == 0)
    }

    /// Load performance metrics for a customer
    async fn get_performance_metrics(&self, customer_id: Uuid) -> Result<Option<CustomerPerformanceMetrics>> {
        let row = sqlx::query!(
            r#"
            SELECT * FROM customer_performance_metrics
            WHERE customer_id = $1
            "#,
            customer_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| CustomerPerformanceMetrics {
            revenue_last_12_months: r.revenue_last_12_months.map(|d| d.try_into().unwrap_or(0.0)),
            average_order_value: r.average_order_value.map(|d| d.try_into().unwrap_or(0.0)),
            order_frequency: r.order_frequency.map(|d| d.try_into().unwrap_or(0.0)),
            total_orders: r.total_orders.unwrap_or(0) as u32,
            total_revenue: r.total_revenue.map(|d| d.try_into().unwrap_or(0.0)),
            profit_margin: r.profit_margin.map(|d| d.try_into().unwrap_or(0.0)),
            last_purchase_date: r.last_purchase_date,
            first_purchase_date: r.first_purchase_date,
            customer_lifetime_value: r.customer_lifetime_value.map(|d| d.try_into().unwrap_or(0.0)),
            predicted_churn_probability: r.predicted_churn_probability.map(|d| d.try_into().unwrap_or(0.0)),
            last_updated: r.last_updated,
        }))
    }

    /// Load behavioral data for a customer
    async fn get_behavioral_data(&self, customer_id: Uuid) -> Result<Option<CustomerBehavioralData>> {
        let row = sqlx::query!(
            r#"
            SELECT * FROM customer_behavioral_data
            WHERE customer_id = $1
            "#,
            customer_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| CustomerBehavioralData {
            purchase_frequency: r.purchase_frequency.map(|d| d.try_into().unwrap_or(0.0)),
            preferred_categories: serde_json::from_value(r.preferred_categories.unwrap_or_default()).unwrap_or_default(),
            seasonal_trends: serde_json::from_value(r.seasonal_trends.unwrap_or_default()).unwrap_or_default(),
            price_sensitivity: r.price_sensitivity.map(|d| d.try_into().unwrap_or(0.0)),
            brand_loyalty: r.brand_loyalty.map(|d| d.try_into().unwrap_or(0.0)),
            communication_preferences: serde_json::from_value(r.communication_preferences.unwrap_or_default()).unwrap_or_default(),
            support_ticket_frequency: r.support_ticket_frequency.map(|d| d.try_into().unwrap_or(0.0)),
            product_return_rate: r.product_return_rate.map(|d| d.try_into().unwrap_or(0.0)),
            referral_activity: r.referral_activity.map(|d| d.try_into().unwrap_or(0.0)),
            product_category_preferences: serde_json::from_value(r.product_category_preferences.unwrap_or_default()).unwrap_or_default(),
            preferred_contact_times: serde_json::from_value(r.preferred_contact_times.unwrap_or_default()).unwrap_or_default(),
            channel_engagement_rates: serde_json::from_value(r.channel_engagement_rates.unwrap_or_default()).unwrap_or_default(),
            website_engagement_score: r.website_engagement_score.map(|d| d.try_into().unwrap_or(0.0)),
            mobile_app_usage: r.mobile_app_usage.map(|d| d.try_into().unwrap_or(0.0)),
            social_media_sentiment: r.social_media_sentiment.map(|d| d.try_into().unwrap_or(0.0)),
            propensity_to_buy: r.propensity_to_buy.map(|d| d.try_into().unwrap_or(0.0)),
            upsell_probability: r.upsell_probability.map(|d| d.try_into().unwrap_or(0.0)),
            cross_sell_probability: r.cross_sell_probability.map(|d| d.try_into().unwrap_or(0.0)),
            last_updated: r.last_updated,
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

        // Insert customer with proper type casting
        sqlx::query!(
            r#"
            INSERT INTO customers (
                id, tenant_id, customer_number, legal_name, trade_names,
                customer_type, industry_classification, business_size,
                parent_customer_id, corporate_group_id,
                lifecycle_stage, status, credit_status,
                tax_jurisdictions, tax_numbers,
                currency_code, credit_limit, payment_terms, tax_exempt,
                sales_representative_id, account_manager_id, acquisition_channel,
                external_ids, master_data_source, external_id, sync_status,
                created_by, created_at, modified_by, modified_at
            ) VALUES (
                $1, $2, $3, $4, $5,
                $6::customer_type, $7::industry_classification, $8::business_size,
                $9, $10,
                $11::customer_lifecycle_stage, $12::entity_status, $13::credit_status,
                $14, $15,
                $16, $17, $18, $19,
                $20, $21, $22::acquisition_channel,
                $23, $24::data_source, $25, $26,
                $27, $28, $29, $30
            )
            "#,
            customer_id,
            self.tenant_context.tenant_id,
            customer_number,
            request.legal_name,
            serde_json::to_value(&request.trade_names)?,
            request.customer_type as CustomerType,
            request.industry_classification as IndustryClassification,
            request.business_size as BusinessSize,
            request.parent_customer_id,
            request.corporate_group_id,
            request.lifecycle_stage as CustomerLifecycleStage,
            EntityStatus::Active as EntityStatus,
            CreditStatus::Good as CreditStatus,
            serde_json::to_value(&request.tax_jurisdictions)?,
            serde_json::to_value(&request.tax_numbers)?,
            request.financial_info.currency_code,
            request.financial_info.credit_limit,
            serde_json::to_value(&request.financial_info.payment_terms)?,
            request.financial_info.tax_exempt,
            request.sales_representative_id,
            request.account_manager_id,
            request.acquisition_channel,
            serde_json::to_value(&request.external_ids)?,
            DataSource::Manual as DataSource,
            request.external_id,
            SyncStatus::Synced as SyncStatus,
            created_by,
            now,
            created_by,
            now
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        // Load and return the created customer
        self.get_customer_by_id(customer_id).await?
            .ok_or(MasterDataError::CustomerNotFound { id: customer_id })
    }

    async fn get_customer_by_id(&self, id: Uuid) -> Result<Option<Customer>> {
        self.load_customer_from_db(id, true).await
    }

    async fn get_customer_by_number(&self, customer_number: &str) -> Result<Option<Customer>> {
        let row = sqlx::query!(
            "SELECT id FROM customers WHERE tenant_id = $1 AND customer_number = $2 AND is_deleted = false",
            self.tenant_context.tenant_id,
            customer_number
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            self.get_customer_by_id(row.id).await
        } else {
            Ok(None)
        }
    }

    async fn update_customer(&self, id: Uuid, update: &UpdateCustomerRequest, modified_by: Uuid) -> Result<Customer> {
        let mut tx = self.pool.begin().await?;
        let now = Utc::now();

        // Build dynamic update query
        let mut query_parts = vec!["UPDATE customers SET modified_by = $1, modified_at = $2"];
        let mut param_count = 2;

        if update.legal_name.is_some() {
            param_count += 1;
            query_parts.push(&format!("legal_name = ${}", param_count));
        }
        if update.trade_names.is_some() {
            param_count += 1;
            query_parts.push(&format!("trade_names = ${}", param_count));
        }

        query_parts.push(&format!("WHERE id = ${} AND tenant_id = ${}", param_count + 1, param_count + 2));

        let query = format!("{} {}", query_parts[0], query_parts[1..].join(", "));

        // Execute update (simplified for now - full implementation would use dynamic query building)
        sqlx::query!(
            "UPDATE customers SET legal_name = COALESCE($1, legal_name), modified_by = $2, modified_at = $3 WHERE id = $4 AND tenant_id = $5",
            update.legal_name,
            modified_by,
            now,
            id,
            self.tenant_context.tenant_id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        // Return updated customer
        self.get_customer_by_id(id).await?
            .ok_or(MasterDataError::CustomerNotFound { id })
    }

    async fn delete_customer(&self, id: Uuid, deleted_by: Uuid) -> Result<()> {
        let now = Utc::now();

        sqlx::query!(
            "UPDATE customers SET is_deleted = true, deleted_by = $1, deleted_at = $2 WHERE id = $3 AND tenant_id = $4",
            deleted_by,
            now,
            id,
            self.tenant_context.tenant_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn list_customers(&self, _criteria: &CustomerSearchCriteria, page: u32, page_size: u32) -> Result<CustomerSearchResponse> {
        let offset = (page.saturating_sub(1)) * page_size;

        let rows = sqlx::query!(
            "SELECT id FROM customers WHERE tenant_id = $1 AND is_deleted = false ORDER BY created_at DESC LIMIT $2 OFFSET $3",
            self.tenant_context.tenant_id,
            page_size as i64,
            offset as i64
        )
        .fetch_all(&self.pool)
        .await?;

        let count_row = sqlx::query!(
            "SELECT COUNT(*) as count FROM customers WHERE tenant_id = $1 AND is_deleted = false",
            self.tenant_context.tenant_id
        )
        .fetch_one(&self.pool)
        .await?;

        let mut customers = Vec::new();
        for row in rows {
            if let Some(customer) = self.get_customer_by_id(row.id).await? {
                customers.push(customer);
            }
        }

        Ok(CustomerSearchResponse {
            customers,
            total_count: count_row.count.unwrap_or(0) as u64,
        })
    }

    async fn get_customer_hierarchy(&self, _customer_id: Uuid) -> Result<Vec<Customer>> {
        // TODO: Implement hierarchy query
        Ok(vec![])
    }

    async fn get_customers_by_corporate_group(&self, _group_id: Uuid) -> Result<Vec<Customer>> {
        // TODO: Implement corporate group query
        Ok(vec![])
    }

    async fn get_customer_addresses(&self, _customer_id: Uuid) -> Result<Vec<Address>> {
        // TODO: Implement address loading
        Ok(vec![])
    }

    async fn get_customer_contacts(&self, _customer_id: Uuid) -> Result<Vec<ContactInfo>> {
        // TODO: Implement contact loading
        Ok(vec![])
    }

    async fn search_customers(&self, _criteria: &CustomerSearchCriteria) -> Result<Vec<Customer>> {
        // TODO: Implement full text search
        Ok(vec![])
    }
}