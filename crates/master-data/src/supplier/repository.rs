//! Supplier repository implementation
//!
//! This module provides database operations for supplier management,
//! including CRUD operations, search functionality, and performance tracking.

use super::model::*;
use async_trait::async_trait;
use erp_core::{
    database::DatabasePool,
    error::{Error, ErrorCode, Result},
};
use sqlx::{Row, Postgres, Encode};
use uuid::Uuid;
use crate::types::{PaginationOptions, PaginationResult};

#[async_trait]
pub trait SupplierRepository: Send + Sync {
    // Basic CRUD operations
    async fn create_supplier(&self, supplier: &Supplier) -> Result<Supplier>;
    async fn get_supplier_by_id(&self, tenant_id: Uuid, supplier_id: Uuid) -> Result<Option<Supplier>>;
    async fn get_supplier_by_code(&self, tenant_id: Uuid, supplier_code: &str) -> Result<Option<Supplier>>;
    async fn update_supplier(&self, supplier: &Supplier) -> Result<Supplier>;
    async fn delete_supplier(&self, tenant_id: Uuid, supplier_id: Uuid) -> Result<()>;

    // Search and listing
    async fn search_suppliers(
        &self,
        tenant_id: Uuid,
        filters: &SupplierSearchFilters,
        pagination: &PaginationOptions,
    ) -> Result<PaginationResult<SupplierSummary>>;
    async fn list_suppliers(&self, tenant_id: Uuid, pagination: &PaginationOptions) -> Result<PaginationResult<SupplierSummary>>;

    // Contact management
    async fn create_supplier_contact(&self, contact: &SupplierContact) -> Result<SupplierContact>;
    async fn get_supplier_contacts(&self, tenant_id: Uuid, supplier_id: Uuid) -> Result<Vec<SupplierContact>>;
    async fn update_supplier_contact(&self, contact: &SupplierContact) -> Result<SupplierContact>;
    async fn delete_supplier_contact(&self, tenant_id: Uuid, contact_id: Uuid) -> Result<()>;

    // Address management
    async fn create_supplier_address(&self, address: &SupplierAddress) -> Result<SupplierAddress>;
    async fn get_supplier_addresses(&self, tenant_id: Uuid, supplier_id: Uuid) -> Result<Vec<SupplierAddress>>;
    async fn update_supplier_address(&self, address: &SupplierAddress) -> Result<SupplierAddress>;
    async fn delete_supplier_address(&self, tenant_id: Uuid, address_id: Uuid) -> Result<()>;

    // Performance tracking
    async fn create_supplier_performance(&self, performance: &SupplierPerformance) -> Result<SupplierPerformance>;
    async fn get_supplier_performance(&self, tenant_id: Uuid, supplier_id: Uuid) -> Result<Vec<SupplierPerformance>>;
    async fn get_supplier_performance_summary(&self, tenant_id: Uuid, supplier_id: Uuid) -> Result<Option<SupplierPerformance>>;

    // Analytics
    async fn get_suppliers_by_category(&self, tenant_id: Uuid) -> Result<Vec<(SupplierCategory, i64)>>;
    async fn get_top_suppliers_by_rating(&self, tenant_id: Uuid, limit: i32) -> Result<Vec<SupplierSummary>>;
    async fn get_suppliers_requiring_attention(&self, tenant_id: Uuid) -> Result<Vec<SupplierSummary>>;
}

pub struct PostgresSupplierRepository {
    db: DatabasePool,
}

impl PostgresSupplierRepository {
    pub fn new(db: DatabasePool) -> Self {
        Self { db }
    }

    fn get_pool(&self) -> &sqlx::PgPool {
        &self.db.main_pool
    }
}

#[async_trait]
impl SupplierRepository for PostgresSupplierRepository {
    async fn create_supplier(&self, supplier: &Supplier) -> Result<Supplier> {
        let query = r#"
            INSERT INTO suppliers (
                id, tenant_id, supplier_code, company_name, legal_name, tax_id, registration_number,
                category, status, tags, website, phone, email, payment_terms, currency,
                credit_limit, lead_time_days, rating, on_time_delivery_rate, quality_rating,
                notes, created_at, updated_at, created_by, updated_by
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15,
                $16, $17, $18, $19, $20, $21, $22, $23, $24, $25
            ) RETURNING *
        "#;

        let row = sqlx::query(query)
            .bind(&supplier.id)
            .bind(&supplier.tenant_id)
            .bind(&supplier.supplier_code)
            .bind(&supplier.company_name)
            .bind(&supplier.legal_name)
            .bind(&supplier.tax_id)
            .bind(&supplier.registration_number)
            .bind(&supplier.category)
            .bind(&supplier.status)
            .bind(&supplier.tags)
            .bind(&supplier.website)
            .bind(&supplier.phone)
            .bind(&supplier.email)
            .bind(&supplier.payment_terms)
            .bind(&supplier.currency)
            .bind(&supplier.credit_limit)
            .bind(&supplier.lead_time_days)
            .bind(&supplier.rating)
            .bind(&supplier.on_time_delivery_rate)
            .bind(&supplier.quality_rating)
            .bind(&supplier.notes)
            .bind(&supplier.created_at)
            .bind(&supplier.updated_at)
            .bind(&supplier.created_by)
            .bind(&supplier.updated_by)
            .fetch_one(self.get_pool())
            .await
            .map_err(|e| Error::new(ErrorCode::DatabaseError, format!("Failed to create supplier: {}", e)))?;

        Ok(Supplier {
            id: row.get("id"),
            tenant_id: row.get("tenant_id"),
            supplier_code: row.get("supplier_code"),
            company_name: row.get("company_name"),
            legal_name: row.get("legal_name"),
            tax_id: row.get("tax_id"),
            registration_number: row.get("registration_number"),
            category: row.get("category"),
            status: row.get("status"),
            tags: row.get("tags"),
            website: row.get("website"),
            phone: row.get("phone"),
            email: row.get("email"),
            payment_terms: row.get("payment_terms"),
            currency: row.get("currency"),
            credit_limit: row.get("credit_limit"),
            lead_time_days: row.get("lead_time_days"),
            rating: row.get("rating"),
            on_time_delivery_rate: row.get("on_time_delivery_rate"),
            quality_rating: row.get("quality_rating"),
            notes: row.get("notes"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            created_by: row.get("created_by"),
            updated_by: row.get("updated_by"),
        })
    }

    async fn get_supplier_by_id(&self, tenant_id: Uuid, supplier_id: Uuid) -> Result<Option<Supplier>> {
        let query = r#"
            SELECT * FROM suppliers
            WHERE id = $1 AND tenant_id = $2
        "#;

        let row = sqlx::query(query)
            .bind(supplier_id)
            .bind(tenant_id)
            .fetch_optional(self.get_pool())
            .await
            .map_err(|e| Error::new(ErrorCode::DatabaseError, format!("Failed to get supplier: {}", e)))?;

        Ok(row.map(|r| Supplier {
            id: r.get("id"),
            tenant_id: r.get("tenant_id"),
            supplier_code: r.get("supplier_code"),
            company_name: r.get("company_name"),
            legal_name: r.get("legal_name"),
            tax_id: r.get("tax_id"),
            registration_number: r.get("registration_number"),
            category: r.get("category"),
            status: r.get("status"),
            tags: r.get("tags"),
            website: r.get("website"),
            phone: r.get("phone"),
            email: r.get("email"),
            payment_terms: r.get("payment_terms"),
            currency: r.get("currency"),
            credit_limit: r.get("credit_limit"),
            lead_time_days: r.get("lead_time_days"),
            rating: r.get("rating"),
            on_time_delivery_rate: r.get("on_time_delivery_rate"),
            quality_rating: r.get("quality_rating"),
            notes: r.get("notes"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
            created_by: r.get("created_by"),
            updated_by: r.get("updated_by"),
        }))
    }

    async fn get_supplier_by_code(&self, tenant_id: Uuid, supplier_code: &str) -> Result<Option<Supplier>> {
        let query = r#"
            SELECT * FROM suppliers
            WHERE supplier_code = $1 AND tenant_id = $2
        "#;

        let row = sqlx::query(query)
            .bind(supplier_code)
            .bind(tenant_id)
            .fetch_optional(self.get_pool())
            .await
            .map_err(|e| Error::new(ErrorCode::DatabaseError, format!("Failed to get supplier by code: {}", e)))?;

        Ok(row.map(|r| Supplier {
            id: r.get("id"),
            tenant_id: r.get("tenant_id"),
            supplier_code: r.get("supplier_code"),
            company_name: r.get("company_name"),
            legal_name: r.get("legal_name"),
            tax_id: r.get("tax_id"),
            registration_number: r.get("registration_number"),
            category: r.get("category"),
            status: r.get("status"),
            tags: r.get("tags"),
            website: r.get("website"),
            phone: r.get("phone"),
            email: r.get("email"),
            payment_terms: r.get("payment_terms"),
            currency: r.get("currency"),
            credit_limit: r.get("credit_limit"),
            lead_time_days: r.get("lead_time_days"),
            rating: r.get("rating"),
            on_time_delivery_rate: r.get("on_time_delivery_rate"),
            quality_rating: r.get("quality_rating"),
            notes: r.get("notes"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
            created_by: r.get("created_by"),
            updated_by: r.get("updated_by"),
        }))
    }

    async fn update_supplier(&self, supplier: &Supplier) -> Result<Supplier> {
        let query = r#"
            UPDATE suppliers SET
                company_name = $3, legal_name = $4, tax_id = $5, registration_number = $6,
                category = $7, status = $8, tags = $9, website = $10, phone = $11, email = $12,
                payment_terms = $13, currency = $14, credit_limit = $15, lead_time_days = $16,
                rating = $17, on_time_delivery_rate = $18, quality_rating = $19, notes = $20,
                updated_at = $21, updated_by = $22
            WHERE id = $1 AND tenant_id = $2
            RETURNING *
        "#;

        let row = sqlx::query(query)
            .bind(&supplier.id)
            .bind(&supplier.tenant_id)
            .bind(&supplier.company_name)
            .bind(&supplier.legal_name)
            .bind(&supplier.tax_id)
            .bind(&supplier.registration_number)
            .bind(&supplier.category)
            .bind(&supplier.status)
            .bind(&supplier.tags)
            .bind(&supplier.website)
            .bind(&supplier.phone)
            .bind(&supplier.email)
            .bind(&supplier.payment_terms)
            .bind(&supplier.currency)
            .bind(&supplier.credit_limit)
            .bind(&supplier.lead_time_days)
            .bind(&supplier.rating)
            .bind(&supplier.on_time_delivery_rate)
            .bind(&supplier.quality_rating)
            .bind(&supplier.notes)
            .bind(&supplier.updated_at)
            .bind(&supplier.updated_by)
            .fetch_one(self.get_pool())
            .await
            .map_err(|e| Error::new(ErrorCode::DatabaseError, format!("Failed to update supplier: {}", e)))?;

        Ok(Supplier {
            id: row.get("id"),
            tenant_id: row.get("tenant_id"),
            supplier_code: row.get("supplier_code"),
            company_name: row.get("company_name"),
            legal_name: row.get("legal_name"),
            tax_id: row.get("tax_id"),
            registration_number: row.get("registration_number"),
            category: row.get("category"),
            status: row.get("status"),
            tags: row.get("tags"),
            website: row.get("website"),
            phone: row.get("phone"),
            email: row.get("email"),
            payment_terms: row.get("payment_terms"),
            currency: row.get("currency"),
            credit_limit: row.get("credit_limit"),
            lead_time_days: row.get("lead_time_days"),
            rating: row.get("rating"),
            on_time_delivery_rate: row.get("on_time_delivery_rate"),
            quality_rating: row.get("quality_rating"),
            notes: row.get("notes"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            created_by: row.get("created_by"),
            updated_by: row.get("updated_by"),
        })
    }

    async fn delete_supplier(&self, tenant_id: Uuid, supplier_id: Uuid) -> Result<()> {
        let query = r#"
            DELETE FROM suppliers
            WHERE id = $1 AND tenant_id = $2
        "#;

        let result = sqlx::query(query)
            .bind(supplier_id)
            .bind(tenant_id)
            .execute(self.get_pool())
            .await
            .map_err(|e| Error::new(ErrorCode::DatabaseError, format!("Failed to delete supplier: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(Error::new(ErrorCode::NotFound, "Supplier not found"));
        }

        Ok(())
    }

    async fn search_suppliers(
        &self,
        tenant_id: Uuid,
        filters: &SupplierSearchFilters,
        pagination: &PaginationOptions,
    ) -> Result<PaginationResult<SupplierSummary>> {
        let mut where_conditions = vec!["s.tenant_id = $1".to_string()];
        let mut params: Vec<Box<dyn sqlx::Encode<Postgres> + Send + Sync>> = vec![Box::new(tenant_id)];
        let mut param_count = 1;

        if let Some(query) = &filters.query {
            param_count += 1;
            where_conditions.push(format!("(s.company_name ILIKE ${} OR s.legal_name ILIKE ${} OR s.supplier_code ILIKE ${})", param_count, param_count, param_count));
            let search_term = format!("%{}%", query);
            params.push(Box::new(search_term));
        }

        if let Some(status) = &filters.status {
            param_count += 1;
            where_conditions.push(format!("s.status = ${}", param_count));
            params.push(Box::new(status.clone()));
        }

        if let Some(category) = &filters.category {
            param_count += 1;
            where_conditions.push(format!("s.category = ${}", param_count));
            params.push(Box::new(category.clone()));
        }

        if let Some(min_rating) = filters.min_rating {
            param_count += 1;
            where_conditions.push(format!("s.rating >= ${}", param_count));
            params.push(Box::new(min_rating));
        }

        let where_clause = where_conditions.join(" AND ");
        let offset = (pagination.page - 1) * pagination.limit;

        let count_query = format!("SELECT COUNT(*) FROM suppliers s WHERE {}", where_clause);
        let data_query = format!(
            r#"
            SELECT s.id, s.supplier_code, s.company_name, s.category, s.status,
                   s.rating, s.on_time_delivery_rate, s.created_at,
                   COALESCE(perf.total_orders, 0) as total_orders
            FROM suppliers s
            LEFT JOIN (
                SELECT supplier_id, SUM(total_orders) as total_orders
                FROM supplier_performance
                GROUP BY supplier_id
            ) perf ON s.id = perf.supplier_id
            WHERE {}
            ORDER BY s.created_at DESC
            LIMIT {} OFFSET {}
            "#,
            where_clause, pagination.limit, offset
        );

        // Execute count query
        let mut count_query_builder = sqlx::query_scalar::<_, i64>(&count_query);
        for param in &params {
            count_query_builder = count_query_builder.bind(param.as_ref());
        }
        let total = count_query_builder
            .fetch_one(self.get_pool())
            .await
            .map_err(|e| Error::new(ErrorCode::DatabaseError, format!("Failed to count suppliers: {}", e)))?;

        // Execute data query
        let mut data_query_builder = sqlx::query(&data_query);
        for param in &params {
            data_query_builder = data_query_builder.bind(param.as_ref());
        }
        let rows = data_query_builder
            .fetch_all(self.get_pool())
            .await
            .map_err(|e| Error::new(ErrorCode::DatabaseError, format!("Failed to search suppliers: {}", e)))?;

        let items: Vec<SupplierSummary> = rows
            .into_iter()
            .map(|row| SupplierSummary {
                id: row.get("id"),
                supplier_code: row.get("supplier_code"),
                company_name: row.get("company_name"),
                category: row.get("category"),
                status: row.get("status"),
                rating: row.get("rating"),
                on_time_delivery_rate: row.get("on_time_delivery_rate"),
                total_orders: row.get("total_orders"),
                created_at: row.get("created_at"),
            })
            .collect();

        Ok(PaginationResult {
            items,
            total,
            page: pagination.page,
            limit: pagination.limit,
            total_pages: (total as f64 / pagination.limit as f64).ceil() as i64,
        })
    }

    async fn list_suppliers(&self, tenant_id: Uuid, pagination: &PaginationOptions) -> Result<PaginationResult<SupplierSummary>> {
        let filters = SupplierSearchFilters {
            query: None,
            status: None,
            category: None,
            tags: None,
            min_rating: None,
            max_rating: None,
            payment_terms: None,
            country: None,
            created_after: None,
            created_before: None,
        };
        self.search_suppliers(tenant_id, &filters, pagination).await
    }

    async fn create_supplier_contact(&self, contact: &SupplierContact) -> Result<SupplierContact> {
        let query = r#"
            INSERT INTO supplier_contacts (
                id, supplier_id, tenant_id, first_name, last_name, title, department,
                email, phone, mobile, role, is_primary, is_active, notes,
                created_at, updated_at, created_by, updated_by
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18
            ) RETURNING *
        "#;

        let row = sqlx::query(query)
            .bind(&contact.id)
            .bind(&contact.supplier_id)
            .bind(&contact.tenant_id)
            .bind(&contact.first_name)
            .bind(&contact.last_name)
            .bind(&contact.title)
            .bind(&contact.department)
            .bind(&contact.email)
            .bind(&contact.phone)
            .bind(&contact.mobile)
            .bind(&contact.role)
            .bind(contact.is_primary)
            .bind(contact.is_active)
            .bind(&contact.notes)
            .bind(&contact.created_at)
            .bind(&contact.updated_at)
            .bind(&contact.created_by)
            .bind(&contact.updated_by)
            .fetch_one(self.get_pool())
            .await
            .map_err(|e| Error::new(ErrorCode::DatabaseError, format!("Failed to create supplier contact: {}", e)))?;

        Ok(SupplierContact {
            id: row.get("id"),
            supplier_id: row.get("supplier_id"),
            tenant_id: row.get("tenant_id"),
            first_name: row.get("first_name"),
            last_name: row.get("last_name"),
            title: row.get("title"),
            department: row.get("department"),
            email: row.get("email"),
            phone: row.get("phone"),
            mobile: row.get("mobile"),
            role: row.get("role"),
            is_primary: row.get("is_primary"),
            is_active: row.get("is_active"),
            notes: row.get("notes"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            created_by: row.get("created_by"),
            updated_by: row.get("updated_by"),
        })
    }

    async fn get_supplier_contacts(&self, tenant_id: Uuid, supplier_id: Uuid) -> Result<Vec<SupplierContact>> {
        let query = r#"
            SELECT * FROM supplier_contacts
            WHERE supplier_id = $1 AND tenant_id = $2 AND is_active = true
            ORDER BY is_primary DESC, created_at ASC
        "#;

        let rows = sqlx::query(query)
            .bind(supplier_id)
            .bind(tenant_id)
            .fetch_all(self.get_pool())
            .await
            .map_err(|e| Error::new(ErrorCode::DatabaseError, format!("Failed to get supplier contacts: {}", e)))?;

        Ok(rows
            .into_iter()
            .map(|row| SupplierContact {
                id: row.get("id"),
                supplier_id: row.get("supplier_id"),
                tenant_id: row.get("tenant_id"),
                first_name: row.get("first_name"),
                last_name: row.get("last_name"),
                title: row.get("title"),
                department: row.get("department"),
                email: row.get("email"),
                phone: row.get("phone"),
                mobile: row.get("mobile"),
                role: row.get("role"),
                is_primary: row.get("is_primary"),
                is_active: row.get("is_active"),
                notes: row.get("notes"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                created_by: row.get("created_by"),
                updated_by: row.get("updated_by"),
            })
            .collect())
    }

    async fn update_supplier_contact(&self, contact: &SupplierContact) -> Result<SupplierContact> {
        let query = r#"
            UPDATE supplier_contacts SET
                first_name = $4, last_name = $5, title = $6, department = $7,
                email = $8, phone = $9, mobile = $10, role = $11, is_primary = $12,
                is_active = $13, notes = $14, updated_at = $15, updated_by = $16
            WHERE id = $1 AND supplier_id = $2 AND tenant_id = $3
            RETURNING *
        "#;

        let row = sqlx::query(query)
            .bind(&contact.id)
            .bind(&contact.supplier_id)
            .bind(&contact.tenant_id)
            .bind(&contact.first_name)
            .bind(&contact.last_name)
            .bind(&contact.title)
            .bind(&contact.department)
            .bind(&contact.email)
            .bind(&contact.phone)
            .bind(&contact.mobile)
            .bind(&contact.role)
            .bind(contact.is_primary)
            .bind(contact.is_active)
            .bind(&contact.notes)
            .bind(&contact.updated_at)
            .bind(&contact.updated_by)
            .fetch_one(self.get_pool())
            .await
            .map_err(|e| Error::new(ErrorCode::DatabaseError, format!("Failed to update supplier contact: {}", e)))?;

        Ok(SupplierContact {
            id: row.get("id"),
            supplier_id: row.get("supplier_id"),
            tenant_id: row.get("tenant_id"),
            first_name: row.get("first_name"),
            last_name: row.get("last_name"),
            title: row.get("title"),
            department: row.get("department"),
            email: row.get("email"),
            phone: row.get("phone"),
            mobile: row.get("mobile"),
            role: row.get("role"),
            is_primary: row.get("is_primary"),
            is_active: row.get("is_active"),
            notes: row.get("notes"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            created_by: row.get("created_by"),
            updated_by: row.get("updated_by"),
        })
    }

    async fn delete_supplier_contact(&self, tenant_id: Uuid, contact_id: Uuid) -> Result<()> {
        let query = r#"
            DELETE FROM supplier_contacts
            WHERE id = $1 AND tenant_id = $2
        "#;

        let result = sqlx::query(query)
            .bind(contact_id)
            .bind(tenant_id)
            .execute(self.get_pool())
            .await
            .map_err(|e| Error::new(ErrorCode::DatabaseError, format!("Failed to delete supplier contact: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(Error::new(ErrorCode::NotFound, "Supplier contact not found"));
        }

        Ok(())
    }

    async fn create_supplier_address(&self, address: &SupplierAddress) -> Result<SupplierAddress> {
        let query = r#"
            INSERT INTO supplier_addresses (
                id, supplier_id, tenant_id, address_type, is_primary, street1, street2,
                city, state, postal_code, country, is_active, created_at, updated_at,
                created_by, updated_by
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16
            ) RETURNING *
        "#;

        let row = sqlx::query(query)
            .bind(&address.id)
            .bind(&address.supplier_id)
            .bind(&address.tenant_id)
            .bind(&address.address_type)
            .bind(address.is_primary)
            .bind(&address.street1)
            .bind(&address.street2)
            .bind(&address.city)
            .bind(&address.state)
            .bind(&address.postal_code)
            .bind(&address.country)
            .bind(address.is_active)
            .bind(&address.created_at)
            .bind(&address.updated_at)
            .bind(&address.created_by)
            .bind(&address.updated_by)
            .fetch_one(self.get_pool())
            .await
            .map_err(|e| Error::new(ErrorCode::DatabaseError, format!("Failed to create supplier address: {}", e)))?;

        Ok(SupplierAddress {
            id: row.get("id"),
            supplier_id: row.get("supplier_id"),
            tenant_id: row.get("tenant_id"),
            address_type: row.get("address_type"),
            is_primary: row.get("is_primary"),
            street1: row.get("street1"),
            street2: row.get("street2"),
            city: row.get("city"),
            state: row.get("state"),
            postal_code: row.get("postal_code"),
            country: row.get("country"),
            is_active: row.get("is_active"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            created_by: row.get("created_by"),
            updated_by: row.get("updated_by"),
        })
    }

    async fn get_supplier_addresses(&self, tenant_id: Uuid, supplier_id: Uuid) -> Result<Vec<SupplierAddress>> {
        let query = r#"
            SELECT * FROM supplier_addresses
            WHERE supplier_id = $1 AND tenant_id = $2 AND is_active = true
            ORDER BY is_primary DESC, created_at ASC
        "#;

        let rows = sqlx::query(query)
            .bind(supplier_id)
            .bind(tenant_id)
            .fetch_all(self.get_pool())
            .await
            .map_err(|e| Error::new(ErrorCode::DatabaseError, format!("Failed to get supplier addresses: {}", e)))?;

        Ok(rows
            .into_iter()
            .map(|row| SupplierAddress {
                id: row.get("id"),
                supplier_id: row.get("supplier_id"),
                tenant_id: row.get("tenant_id"),
                address_type: row.get("address_type"),
                is_primary: row.get("is_primary"),
                street1: row.get("street1"),
                street2: row.get("street2"),
                city: row.get("city"),
                state: row.get("state"),
                postal_code: row.get("postal_code"),
                country: row.get("country"),
                is_active: row.get("is_active"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                created_by: row.get("created_by"),
                updated_by: row.get("updated_by"),
            })
            .collect())
    }

    async fn update_supplier_address(&self, address: &SupplierAddress) -> Result<SupplierAddress> {
        let query = r#"
            UPDATE supplier_addresses SET
                address_type = $4, is_primary = $5, street1 = $6, street2 = $7,
                city = $8, state = $9, postal_code = $10, country = $11,
                is_active = $12, updated_at = $13, updated_by = $14
            WHERE id = $1 AND supplier_id = $2 AND tenant_id = $3
            RETURNING *
        "#;

        let row = sqlx::query(query)
            .bind(&address.id)
            .bind(&address.supplier_id)
            .bind(&address.tenant_id)
            .bind(&address.address_type)
            .bind(address.is_primary)
            .bind(&address.street1)
            .bind(&address.street2)
            .bind(&address.city)
            .bind(&address.state)
            .bind(&address.postal_code)
            .bind(&address.country)
            .bind(address.is_active)
            .bind(&address.updated_at)
            .bind(&address.updated_by)
            .fetch_one(self.get_pool())
            .await
            .map_err(|e| Error::new(ErrorCode::DatabaseError, format!("Failed to update supplier address: {}", e)))?;

        Ok(SupplierAddress {
            id: row.get("id"),
            supplier_id: row.get("supplier_id"),
            tenant_id: row.get("tenant_id"),
            address_type: row.get("address_type"),
            is_primary: row.get("is_primary"),
            street1: row.get("street1"),
            street2: row.get("street2"),
            city: row.get("city"),
            state: row.get("state"),
            postal_code: row.get("postal_code"),
            country: row.get("country"),
            is_active: row.get("is_active"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            created_by: row.get("created_by"),
            updated_by: row.get("updated_by"),
        })
    }

    async fn delete_supplier_address(&self, tenant_id: Uuid, address_id: Uuid) -> Result<()> {
        let query = r#"
            DELETE FROM supplier_addresses
            WHERE id = $1 AND tenant_id = $2
        "#;

        let result = sqlx::query(query)
            .bind(address_id)
            .bind(tenant_id)
            .execute(self.get_pool())
            .await
            .map_err(|e| Error::new(ErrorCode::DatabaseError, format!("Failed to delete supplier address: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(Error::new(ErrorCode::NotFound, "Supplier address not found"));
        }

        Ok(())
    }

    async fn create_supplier_performance(&self, performance: &SupplierPerformance) -> Result<SupplierPerformance> {
        let query = r#"
            INSERT INTO supplier_performance (
                id, supplier_id, tenant_id, period_start, period_end, total_orders,
                on_time_deliveries, late_deliveries, early_deliveries, average_lead_time_days,
                quality_rating, defect_rate, return_rate, total_spend, average_order_value,
                payment_compliance_rate, overall_rating, notes, created_at, updated_at, created_by
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21
            ) RETURNING *
        "#;

        let row = sqlx::query(query)
            .bind(&performance.id)
            .bind(&performance.supplier_id)
            .bind(&performance.tenant_id)
            .bind(&performance.period_start)
            .bind(&performance.period_end)
            .bind(&performance.total_orders)
            .bind(&performance.on_time_deliveries)
            .bind(&performance.late_deliveries)
            .bind(&performance.early_deliveries)
            .bind(&performance.average_lead_time_days)
            .bind(&performance.quality_rating)
            .bind(&performance.defect_rate)
            .bind(&performance.return_rate)
            .bind(&performance.total_spend)
            .bind(&performance.average_order_value)
            .bind(&performance.payment_compliance_rate)
            .bind(&performance.overall_rating)
            .bind(&performance.notes)
            .bind(&performance.created_at)
            .bind(&performance.updated_at)
            .bind(&performance.created_by)
            .fetch_one(self.get_pool())
            .await
            .map_err(|e| Error::new(ErrorCode::DatabaseError, format!("Failed to create supplier performance: {}", e)))?;

        // Map row to SupplierPerformance struct
        Ok(SupplierPerformance {
            id: row.get("id"),
            supplier_id: row.get("supplier_id"),
            tenant_id: row.get("tenant_id"),
            period_start: row.get("period_start"),
            period_end: row.get("period_end"),
            total_orders: row.get("total_orders"),
            on_time_deliveries: row.get("on_time_deliveries"),
            late_deliveries: row.get("late_deliveries"),
            early_deliveries: row.get("early_deliveries"),
            average_lead_time_days: row.get("average_lead_time_days"),
            quality_rating: row.get("quality_rating"),
            defect_rate: row.get("defect_rate"),
            return_rate: row.get("return_rate"),
            total_spend: row.get("total_spend"),
            average_order_value: row.get("average_order_value"),
            payment_compliance_rate: row.get("payment_compliance_rate"),
            overall_rating: row.get("overall_rating"),
            notes: row.get("notes"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            created_by: row.get("created_by"),
        })
    }

    async fn get_supplier_performance(&self, tenant_id: Uuid, supplier_id: Uuid) -> Result<Vec<SupplierPerformance>> {
        let query = r#"
            SELECT * FROM supplier_performance
            WHERE supplier_id = $1 AND tenant_id = $2
            ORDER BY period_start DESC
        "#;

        let rows = sqlx::query(query)
            .bind(supplier_id)
            .bind(tenant_id)
            .fetch_all(self.get_pool())
            .await
            .map_err(|e| Error::new(ErrorCode::DatabaseError, format!("Failed to get supplier performance: {}", e)))?;

        Ok(rows
            .into_iter()
            .map(|row| SupplierPerformance {
                id: row.get("id"),
                supplier_id: row.get("supplier_id"),
                tenant_id: row.get("tenant_id"),
                period_start: row.get("period_start"),
                period_end: row.get("period_end"),
                total_orders: row.get("total_orders"),
                on_time_deliveries: row.get("on_time_deliveries"),
                late_deliveries: row.get("late_deliveries"),
                early_deliveries: row.get("early_deliveries"),
                average_lead_time_days: row.get("average_lead_time_days"),
                quality_rating: row.get("quality_rating"),
                defect_rate: row.get("defect_rate"),
                return_rate: row.get("return_rate"),
                total_spend: row.get("total_spend"),
                average_order_value: row.get("average_order_value"),
                payment_compliance_rate: row.get("payment_compliance_rate"),
                overall_rating: row.get("overall_rating"),
                notes: row.get("notes"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                created_by: row.get("created_by"),
            })
            .collect())
    }

    async fn get_supplier_performance_summary(&self, tenant_id: Uuid, supplier_id: Uuid) -> Result<Option<SupplierPerformance>> {
        let query = r#"
            SELECT * FROM supplier_performance
            WHERE supplier_id = $1 AND tenant_id = $2
            ORDER BY period_end DESC
            LIMIT 1
        "#;

        let row = sqlx::query(query)
            .bind(supplier_id)
            .bind(tenant_id)
            .fetch_optional(self.get_pool())
            .await
            .map_err(|e| Error::new(ErrorCode::DatabaseError, format!("Failed to get supplier performance summary: {}", e)))?;

        Ok(row.map(|r| SupplierPerformance {
            id: r.get("id"),
            supplier_id: r.get("supplier_id"),
            tenant_id: r.get("tenant_id"),
            period_start: r.get("period_start"),
            period_end: r.get("period_end"),
            total_orders: r.get("total_orders"),
            on_time_deliveries: r.get("on_time_deliveries"),
            late_deliveries: r.get("late_deliveries"),
            early_deliveries: r.get("early_deliveries"),
            average_lead_time_days: r.get("average_lead_time_days"),
            quality_rating: r.get("quality_rating"),
            defect_rate: r.get("defect_rate"),
            return_rate: r.get("return_rate"),
            total_spend: r.get("total_spend"),
            average_order_value: r.get("average_order_value"),
            payment_compliance_rate: r.get("payment_compliance_rate"),
            overall_rating: r.get("overall_rating"),
            notes: r.get("notes"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
            created_by: r.get("created_by"),
        }))
    }

    async fn get_suppliers_by_category(&self, tenant_id: Uuid) -> Result<Vec<(SupplierCategory, i64)>> {
        let query = r#"
            SELECT category, COUNT(*) as count
            FROM suppliers
            WHERE tenant_id = $1 AND status != 'terminated'
            GROUP BY category
            ORDER BY count DESC
        "#;

        let rows = sqlx::query(query)
            .bind(tenant_id)
            .fetch_all(self.get_pool())
            .await
            .map_err(|e| Error::new(ErrorCode::DatabaseError, format!("Failed to get suppliers by category: {}", e)))?;

        Ok(rows
            .into_iter()
            .map(|row| (row.get("category"), row.get("count")))
            .collect())
    }

    async fn get_top_suppliers_by_rating(&self, tenant_id: Uuid, limit: i32) -> Result<Vec<SupplierSummary>> {
        let query = r#"
            SELECT s.id, s.supplier_code, s.company_name, s.category, s.status,
                   s.rating, s.on_time_delivery_rate, s.created_at,
                   COALESCE(perf.total_orders, 0) as total_orders
            FROM suppliers s
            LEFT JOIN (
                SELECT supplier_id, SUM(total_orders) as total_orders
                FROM supplier_performance
                GROUP BY supplier_id
            ) perf ON s.id = perf.supplier_id
            WHERE s.tenant_id = $1 AND s.status = 'active' AND s.rating IS NOT NULL
            ORDER BY s.rating DESC, s.on_time_delivery_rate DESC
            LIMIT $2
        "#;

        let rows = sqlx::query(query)
            .bind(tenant_id)
            .bind(limit)
            .fetch_all(self.get_pool())
            .await
            .map_err(|e| Error::new(ErrorCode::DatabaseError, format!("Failed to get top suppliers: {}", e)))?;

        Ok(rows
            .into_iter()
            .map(|row| SupplierSummary {
                id: row.get("id"),
                supplier_code: row.get("supplier_code"),
                company_name: row.get("company_name"),
                category: row.get("category"),
                status: row.get("status"),
                rating: row.get("rating"),
                on_time_delivery_rate: row.get("on_time_delivery_rate"),
                total_orders: row.get("total_orders"),
                created_at: row.get("created_at"),
            })
            .collect())
    }

    async fn get_suppliers_requiring_attention(&self, tenant_id: Uuid) -> Result<Vec<SupplierSummary>> {
        let query = r#"
            SELECT s.id, s.supplier_code, s.company_name, s.category, s.status,
                   s.rating, s.on_time_delivery_rate, s.created_at,
                   COALESCE(perf.total_orders, 0) as total_orders
            FROM suppliers s
            LEFT JOIN (
                SELECT supplier_id, SUM(total_orders) as total_orders
                FROM supplier_performance
                GROUP BY supplier_id
            ) perf ON s.id = perf.supplier_id
            WHERE s.tenant_id = $1 AND s.status = 'active'
            AND (
                s.rating < 3.0
                OR s.on_time_delivery_rate < 0.85
                OR s.status = 'suspended'
            )
            ORDER BY s.rating ASC, s.on_time_delivery_rate ASC
        "#;

        let rows = sqlx::query(query)
            .bind(tenant_id)
            .fetch_all(self.get_pool())
            .await
            .map_err(|e| Error::new(ErrorCode::DatabaseError, format!("Failed to get suppliers requiring attention: {}", e)))?;

        Ok(rows
            .into_iter()
            .map(|row| SupplierSummary {
                id: row.get("id"),
                supplier_code: row.get("supplier_code"),
                company_name: row.get("company_name"),
                category: row.get("category"),
                status: row.get("status"),
                rating: row.get("rating"),
                on_time_delivery_rate: row.get("on_time_delivery_rate"),
                total_orders: row.get("total_orders"),
                created_at: row.get("created_at"),
            })
            .collect())
    }
}