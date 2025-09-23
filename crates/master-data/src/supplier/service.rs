//! Supplier service implementation
//!
//! This module provides business logic for supplier management,
//! including validation, workflow orchestration, and business rules.

use super::{model::*, repository::SupplierRepository};
use crate::types::{PaginationOptions, PaginationResult, TenantContext};
use async_trait::async_trait;
use chrono::Utc;
use erp_core::error::{Error, ErrorCode, Result};
use std::sync::Arc;
use uuid::Uuid;

#[async_trait]
pub trait SupplierService: Send + Sync {
    // Core supplier management
    async fn create_supplier(&self, request: CreateSupplierRequest) -> Result<Supplier>;
    async fn get_supplier(&self, supplier_id: Uuid) -> Result<Option<Supplier>>;
    async fn get_supplier_by_code(&self, supplier_code: &str) -> Result<Option<Supplier>>;
    async fn update_supplier(&self, supplier_id: Uuid, request: UpdateSupplierRequest) -> Result<Supplier>;
    async fn delete_supplier(&self, supplier_id: Uuid) -> Result<()>;
    async fn activate_supplier(&self, supplier_id: Uuid) -> Result<Supplier>;
    async fn deactivate_supplier(&self, supplier_id: Uuid) -> Result<Supplier>;

    // Search and listing
    async fn search_suppliers(&self, filters: SupplierSearchFilters, pagination: PaginationOptions) -> Result<PaginationResult<SupplierSummary>>;
    async fn list_suppliers(&self, pagination: PaginationOptions) -> Result<PaginationResult<SupplierSummary>>;

    // Contact management
    async fn add_supplier_contact(&self, supplier_id: Uuid, first_name: String, last_name: String, role: String, email: Option<String>, phone: Option<String>) -> Result<SupplierContact>;
    async fn get_supplier_contacts(&self, supplier_id: Uuid) -> Result<Vec<SupplierContact>>;
    async fn update_supplier_contact(&self, contact_id: Uuid, first_name: Option<String>, last_name: Option<String>, email: Option<String>, phone: Option<String>) -> Result<SupplierContact>;
    async fn set_primary_contact(&self, supplier_id: Uuid, contact_id: Uuid) -> Result<()>;
    async fn remove_supplier_contact(&self, contact_id: Uuid) -> Result<()>;

    // Address management
    async fn add_supplier_address(&self, supplier_id: Uuid, address_type: String, street1: String, city: String, country: String) -> Result<SupplierAddress>;
    async fn get_supplier_addresses(&self, supplier_id: Uuid) -> Result<Vec<SupplierAddress>>;
    async fn update_supplier_address(&self, address_id: Uuid, street1: Option<String>, city: Option<String>, country: Option<String>) -> Result<SupplierAddress>;
    async fn set_primary_address(&self, supplier_id: Uuid, address_id: Uuid) -> Result<()>;
    async fn remove_supplier_address(&self, address_id: Uuid) -> Result<()>;

    // Performance tracking
    async fn record_supplier_performance(&self, supplier_id: Uuid, performance_data: SupplierPerformanceData) -> Result<SupplierPerformance>;
    async fn get_supplier_performance(&self, supplier_id: Uuid) -> Result<Vec<SupplierPerformance>>;
    async fn get_supplier_performance_summary(&self, supplier_id: Uuid) -> Result<Option<SupplierPerformance>>;
    async fn update_supplier_rating(&self, supplier_id: Uuid, rating: f64) -> Result<Supplier>;

    // Analytics and reporting
    async fn get_suppliers_by_category(&self) -> Result<Vec<(SupplierCategory, i64)>>;
    async fn get_top_suppliers(&self, limit: i32) -> Result<Vec<SupplierSummary>>;
    async fn get_suppliers_requiring_attention(&self) -> Result<Vec<SupplierSummary>>;
    async fn validate_supplier_code(&self, supplier_code: &str) -> Result<bool>;
}

pub struct DefaultSupplierService {
    repository: Arc<dyn SupplierRepository>,
    tenant_context: TenantContext,
}

impl DefaultSupplierService {
    pub fn new(repository: Arc<dyn SupplierRepository>, tenant_context: TenantContext) -> Self {
        Self {
            repository,
            tenant_context,
        }
    }

    fn validate_create_request(&self, request: &CreateSupplierRequest) -> Result<()> {
        // Validate supplier code
        if request.supplier_code.trim().is_empty() {
            return Err(Error::new(ErrorCode::ValidationFailed, "Supplier code cannot be empty"));
        }

        if request.supplier_code.len() > 50 {
            return Err(Error::new(ErrorCode::ValidationFailed, "Supplier code cannot exceed 50 characters"));
        }

        // Validate company name
        if request.company_name.trim().is_empty() {
            return Err(Error::new(ErrorCode::ValidationFailed, "Company name cannot be empty"));
        }

        if request.company_name.len() > 200 {
            return Err(Error::new(ErrorCode::ValidationFailed, "Company name cannot exceed 200 characters"));
        }

        // Validate email format if provided
        if let Some(email) = &request.email {
            if !email.is_empty() && !email.contains('@') {
                return Err(Error::new(ErrorCode::ValidationFailed, "Invalid email format"));
            }
        }

        // Validate credit limit
        if let Some(credit_limit) = request.credit_limit {
            if credit_limit < 0 {
                return Err(Error::new(ErrorCode::ValidationFailed, "Credit limit cannot be negative"));
            }
        }

        // Validate lead time
        if let Some(lead_time) = request.lead_time_days {
            if lead_time < 0 {
                return Err(Error::new(ErrorCode::ValidationFailed, "Lead time cannot be negative"));
            }
        }

        Ok(())
    }

    fn generate_supplier_code(&self, company_name: &str) -> String {
        // Generate a supplier code based on company name and timestamp
        let prefix = company_name
            .chars()
            .filter(|c| c.is_alphanumeric())
            .take(3)
            .collect::<String>()
            .to_uppercase();

        let timestamp = Utc::now().timestamp() % 10000;

        if prefix.len() >= 3 {
            format!("{}{:04}", prefix, timestamp)
        } else {
            format!("SUP{:04}", timestamp)
        }
    }
}

#[async_trait]
impl SupplierService for DefaultSupplierService {
    async fn create_supplier(&self, request: CreateSupplierRequest) -> Result<Supplier> {
        // Validate the request
        self.validate_create_request(&request)?;

        // Check if supplier code already exists
        if let Some(_existing) = self.repository.get_supplier_by_code(self.tenant_context.tenant_id, &request.supplier_code).await? {
            return Err(Error::new(ErrorCode::ConflictError, "Supplier code already exists"));
        }

        // Create the supplier
        let mut supplier = Supplier::new(
            self.tenant_context.tenant_id,
            request.supplier_code,
            request.company_name,
            self.tenant_context.user_id,
        );

        // Set optional fields
        supplier.legal_name = request.legal_name;
        supplier.tax_id = request.tax_id;
        supplier.registration_number = request.registration_number;
        supplier.category = request.category;
        supplier.website = request.website;
        supplier.phone = request.phone;
        supplier.email = request.email;
        supplier.payment_terms = request.payment_terms;
        supplier.currency = request.currency;
        supplier.credit_limit = request.credit_limit;
        supplier.lead_time_days = request.lead_time_days;
        supplier.notes = request.notes;
        supplier.tags = request.tags;

        let created_supplier = self.repository.create_supplier(&supplier).await?;
        Ok(created_supplier)
    }

    async fn get_supplier(&self, supplier_id: Uuid) -> Result<Option<Supplier>> {
        self.repository.get_supplier_by_id(self.tenant_context.tenant_id, supplier_id).await
    }

    async fn get_supplier_by_code(&self, supplier_code: &str) -> Result<Option<Supplier>> {
        self.repository.get_supplier_by_code(self.tenant_context.tenant_id, supplier_code).await
    }

    async fn update_supplier(&self, supplier_id: Uuid, request: UpdateSupplierRequest) -> Result<Supplier> {
        // Get the existing supplier
        let mut supplier = self.repository.get_supplier_by_id(self.tenant_context.tenant_id, supplier_id).await?
            .ok_or_else(|| Error::new(ErrorCode::NotFound, "Supplier not found"))?;

        // Update fields if provided
        if let Some(company_name) = request.company_name {
            if company_name.trim().is_empty() {
                return Err(Error::new(ErrorCode::ValidationFailed, "Company name cannot be empty"));
            }
            supplier.company_name = company_name;
        }

        if let Some(legal_name) = request.legal_name {
            supplier.legal_name = Some(legal_name);
        }

        if let Some(tax_id) = request.tax_id {
            supplier.tax_id = Some(tax_id);
        }

        if let Some(registration_number) = request.registration_number {
            supplier.registration_number = Some(registration_number);
        }

        if let Some(category) = request.category {
            supplier.category = category;
        }

        if let Some(status) = request.status {
            supplier.status = status;
        }

        if let Some(website) = request.website {
            supplier.website = Some(website);
        }

        if let Some(phone) = request.phone {
            supplier.phone = Some(phone);
        }

        if let Some(email) = request.email {
            if !email.is_empty() && !email.contains('@') {
                return Err(Error::new(ErrorCode::ValidationFailed, "Invalid email format"));
            }
            supplier.email = Some(email);
        }

        if let Some(payment_terms) = request.payment_terms {
            supplier.payment_terms = payment_terms;
        }

        if let Some(currency) = request.currency {
            supplier.currency = currency;
        }

        if let Some(credit_limit) = request.credit_limit {
            if credit_limit < 0 {
                return Err(Error::new(ErrorCode::ValidationFailed, "Credit limit cannot be negative"));
            }
            supplier.credit_limit = Some(credit_limit);
        }

        if let Some(lead_time_days) = request.lead_time_days {
            if lead_time_days < 0 {
                return Err(Error::new(ErrorCode::ValidationFailed, "Lead time cannot be negative"));
            }
            supplier.lead_time_days = Some(lead_time_days);
        }

        if let Some(rating) = request.rating {
            if !(1.0..=5.0).contains(&rating) {
                return Err(Error::new(ErrorCode::ValidationFailed, "Rating must be between 1.0 and 5.0"));
            }
            supplier.rating = Some(rating);
        }

        if let Some(notes) = request.notes {
            supplier.notes = Some(notes);
        }

        if let Some(tags) = request.tags {
            supplier.tags = Some(tags);
        }

        // Update metadata
        supplier.updated_at = Utc::now();
        supplier.updated_by = self.tenant_context.user_id;

        let updated_supplier = self.repository.update_supplier(&supplier).await?;
        Ok(updated_supplier)
    }

    async fn delete_supplier(&self, supplier_id: Uuid) -> Result<()> {
        // Check if supplier exists
        let supplier = self.repository.get_supplier_by_id(self.tenant_context.tenant_id, supplier_id).await?
            .ok_or_else(|| Error::new(ErrorCode::NotFound, "Supplier not found"))?;

        // Check if supplier can be deleted (business rules)
        if supplier.status == SupplierStatus::Active {
            return Err(Error::new(ErrorCode::BusinessRuleViolation, "Cannot delete active supplier. Please deactivate first."));
        }

        self.repository.delete_supplier(self.tenant_context.tenant_id, supplier_id).await
    }

    async fn activate_supplier(&self, supplier_id: Uuid) -> Result<Supplier> {
        let request = UpdateSupplierRequest {
            status: Some(SupplierStatus::Active),
            ..Default::default()
        };
        self.update_supplier(supplier_id, request).await
    }

    async fn deactivate_supplier(&self, supplier_id: Uuid) -> Result<Supplier> {
        let request = UpdateSupplierRequest {
            status: Some(SupplierStatus::Inactive),
            ..Default::default()
        };
        self.update_supplier(supplier_id, request).await
    }

    async fn search_suppliers(&self, filters: SupplierSearchFilters, pagination: PaginationOptions) -> Result<PaginationResult<SupplierSummary>> {
        self.repository.search_suppliers(self.tenant_context.tenant_id, &filters, &pagination).await
    }

    async fn list_suppliers(&self, pagination: PaginationOptions) -> Result<PaginationResult<SupplierSummary>> {
        self.repository.list_suppliers(self.tenant_context.tenant_id, &pagination).await
    }

    async fn add_supplier_contact(&self, supplier_id: Uuid, first_name: String, last_name: String, role: String, email: Option<String>, phone: Option<String>) -> Result<SupplierContact> {
        // Verify supplier exists
        let _supplier = self.repository.get_supplier_by_id(self.tenant_context.tenant_id, supplier_id).await?
            .ok_or_else(|| Error::new(ErrorCode::NotFound, "Supplier not found"))?;

        // Validate contact data
        if first_name.trim().is_empty() {
            return Err(Error::new(ErrorCode::ValidationFailed, "First name cannot be empty"));
        }
        if last_name.trim().is_empty() {
            return Err(Error::new(ErrorCode::ValidationFailed, "Last name cannot be empty"));
        }
        if role.trim().is_empty() {
            return Err(Error::new(ErrorCode::ValidationFailed, "Role cannot be empty"));
        }

        // Validate email if provided
        if let Some(ref email_addr) = email {
            if !email_addr.is_empty() && !email_addr.contains('@') {
                return Err(Error::new(ErrorCode::ValidationFailed, "Invalid email format"));
            }
        }

        let mut contact = SupplierContact::new(
            supplier_id,
            self.tenant_context.tenant_id,
            first_name,
            last_name,
            role,
            self.tenant_context.user_id,
        );

        contact.email = email;
        contact.phone = phone;

        self.repository.create_supplier_contact(&contact).await
    }

    async fn get_supplier_contacts(&self, supplier_id: Uuid) -> Result<Vec<SupplierContact>> {
        self.repository.get_supplier_contacts(self.tenant_context.tenant_id, supplier_id).await
    }

    async fn update_supplier_contact(&self, contact_id: Uuid, first_name: Option<String>, last_name: Option<String>, email: Option<String>, phone: Option<String>) -> Result<SupplierContact> {
        // Get existing contact to verify tenant and get current data
        let contacts = self.repository.get_supplier_contacts(self.tenant_context.tenant_id, Uuid::new_v4()).await?; // This is a simplified approach
        let mut contact = contacts.into_iter()
            .find(|c| c.id == contact_id)
            .ok_or_else(|| Error::new(ErrorCode::NotFound, "Supplier contact not found"))?;

        // Update fields if provided
        if let Some(fname) = first_name {
            if fname.trim().is_empty() {
                return Err(Error::new(ErrorCode::ValidationFailed, "First name cannot be empty"));
            }
            contact.first_name = fname;
        }

        if let Some(lname) = last_name {
            if lname.trim().is_empty() {
                return Err(Error::new(ErrorCode::ValidationFailed, "Last name cannot be empty"));
            }
            contact.last_name = lname;
        }

        if let Some(email_addr) = email {
            if !email_addr.is_empty() && !email_addr.contains('@') {
                return Err(Error::new(ErrorCode::ValidationFailed, "Invalid email format"));
            }
            contact.email = Some(email_addr);
        }

        if let Some(phone_num) = phone {
            contact.phone = Some(phone_num);
        }

        // Update metadata
        contact.updated_at = Utc::now();
        contact.updated_by = self.tenant_context.user_id;

        self.repository.update_supplier_contact(&contact).await
    }

    async fn set_primary_contact(&self, supplier_id: Uuid, contact_id: Uuid) -> Result<()> {
        // Get all contacts for the supplier
        let contacts = self.repository.get_supplier_contacts(self.tenant_context.tenant_id, supplier_id).await?;

        // Find the contact to set as primary
        let mut target_contact = contacts.into_iter()
            .find(|c| c.id == contact_id)
            .ok_or_else(|| Error::new(ErrorCode::NotFound, "Supplier contact not found"))?;

        // Set as primary
        target_contact.is_primary = true;
        target_contact.updated_at = Utc::now();
        target_contact.updated_by = self.tenant_context.user_id;

        self.repository.update_supplier_contact(&target_contact).await?;

        // TODO: Set all other contacts for this supplier as non-primary
        // This would require a batch update operation

        Ok(())
    }

    async fn remove_supplier_contact(&self, contact_id: Uuid) -> Result<()> {
        self.repository.delete_supplier_contact(self.tenant_context.tenant_id, contact_id).await
    }

    async fn add_supplier_address(&self, supplier_id: Uuid, address_type: String, street1: String, city: String, country: String) -> Result<SupplierAddress> {
        // Verify supplier exists
        let _supplier = self.repository.get_supplier_by_id(self.tenant_context.tenant_id, supplier_id).await?
            .ok_or_else(|| Error::new(ErrorCode::NotFound, "Supplier not found"))?;

        // Validate address data
        if address_type.trim().is_empty() {
            return Err(Error::new(ErrorCode::ValidationFailed, "Address type cannot be empty"));
        }
        if street1.trim().is_empty() {
            return Err(Error::new(ErrorCode::ValidationFailed, "Street address cannot be empty"));
        }
        if city.trim().is_empty() {
            return Err(Error::new(ErrorCode::ValidationFailed, "City cannot be empty"));
        }
        if country.trim().is_empty() {
            return Err(Error::new(ErrorCode::ValidationFailed, "Country cannot be empty"));
        }

        let address = SupplierAddress::new(
            supplier_id,
            self.tenant_context.tenant_id,
            address_type,
            street1,
            city,
            country,
            self.tenant_context.user_id,
        );

        self.repository.create_supplier_address(&address).await
    }

    async fn get_supplier_addresses(&self, supplier_id: Uuid) -> Result<Vec<SupplierAddress>> {
        self.repository.get_supplier_addresses(self.tenant_context.tenant_id, supplier_id).await
    }

    async fn update_supplier_address(&self, address_id: Uuid, street1: Option<String>, city: Option<String>, country: Option<String>) -> Result<SupplierAddress> {
        // Get existing address to verify tenant and get current data
        let addresses = self.repository.get_supplier_addresses(self.tenant_context.tenant_id, Uuid::new_v4()).await?; // This is a simplified approach
        let mut address = addresses.into_iter()
            .find(|a| a.id == address_id)
            .ok_or_else(|| Error::new(ErrorCode::NotFound, "Supplier address not found"))?;

        // Update fields if provided
        if let Some(street) = street1 {
            if street.trim().is_empty() {
                return Err(Error::new(ErrorCode::ValidationFailed, "Street address cannot be empty"));
            }
            address.street1 = street;
        }

        if let Some(city_name) = city {
            if city_name.trim().is_empty() {
                return Err(Error::new(ErrorCode::ValidationFailed, "City cannot be empty"));
            }
            address.city = city_name;
        }

        if let Some(country_name) = country {
            if country_name.trim().is_empty() {
                return Err(Error::new(ErrorCode::ValidationFailed, "Country cannot be empty"));
            }
            address.country = country_name;
        }

        // Update metadata
        address.updated_at = Utc::now();
        address.updated_by = self.tenant_context.user_id;

        self.repository.update_supplier_address(&address).await
    }

    async fn set_primary_address(&self, supplier_id: Uuid, address_id: Uuid) -> Result<()> {
        // Get all addresses for the supplier
        let addresses = self.repository.get_supplier_addresses(self.tenant_context.tenant_id, supplier_id).await?;

        // Find the address to set as primary
        let mut target_address = addresses.into_iter()
            .find(|a| a.id == address_id)
            .ok_or_else(|| Error::new(ErrorCode::NotFound, "Supplier address not found"))?;

        // Set as primary
        target_address.is_primary = true;
        target_address.updated_at = Utc::now();
        target_address.updated_by = self.tenant_context.user_id;

        self.repository.update_supplier_address(&target_address).await?;

        // TODO: Set all other addresses for this supplier as non-primary
        // This would require a batch update operation

        Ok(())
    }

    async fn remove_supplier_address(&self, address_id: Uuid) -> Result<()> {
        self.repository.delete_supplier_address(self.tenant_context.tenant_id, address_id).await
    }

    async fn record_supplier_performance(&self, supplier_id: Uuid, performance_data: SupplierPerformanceData) -> Result<SupplierPerformance> {
        // Verify supplier exists
        let _supplier = self.repository.get_supplier_by_id(self.tenant_context.tenant_id, supplier_id).await?
            .ok_or_else(|| Error::new(ErrorCode::NotFound, "Supplier not found"))?;

        let performance = SupplierPerformance {
            id: Uuid::new_v4(),
            supplier_id,
            tenant_id: self.tenant_context.tenant_id,
            period_start: performance_data.period_start,
            period_end: performance_data.period_end,
            total_orders: performance_data.total_orders,
            on_time_deliveries: performance_data.on_time_deliveries,
            late_deliveries: performance_data.late_deliveries,
            early_deliveries: performance_data.early_deliveries,
            average_lead_time_days: performance_data.average_lead_time_days,
            quality_rating: performance_data.quality_rating,
            defect_rate: performance_data.defect_rate,
            return_rate: performance_data.return_rate,
            total_spend: performance_data.total_spend,
            average_order_value: performance_data.average_order_value,
            payment_compliance_rate: performance_data.payment_compliance_rate,
            overall_rating: performance_data.overall_rating,
            notes: performance_data.notes,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: self.tenant_context.user_id,
        };

        self.repository.create_supplier_performance(&performance).await
    }

    async fn get_supplier_performance(&self, supplier_id: Uuid) -> Result<Vec<SupplierPerformance>> {
        self.repository.get_supplier_performance(self.tenant_context.tenant_id, supplier_id).await
    }

    async fn get_supplier_performance_summary(&self, supplier_id: Uuid) -> Result<Option<SupplierPerformance>> {
        self.repository.get_supplier_performance_summary(self.tenant_context.tenant_id, supplier_id).await
    }

    async fn update_supplier_rating(&self, supplier_id: Uuid, rating: f64) -> Result<Supplier> {
        if !(1.0..=5.0).contains(&rating) {
            return Err(Error::new(ErrorCode::ValidationFailed, "Rating must be between 1.0 and 5.0"));
        }

        let request = UpdateSupplierRequest {
            rating: Some(rating),
            ..Default::default()
        };
        self.update_supplier(supplier_id, request).await
    }

    async fn get_suppliers_by_category(&self) -> Result<Vec<(SupplierCategory, i64)>> {
        self.repository.get_suppliers_by_category(self.tenant_context.tenant_id).await
    }

    async fn get_top_suppliers(&self, limit: i32) -> Result<Vec<SupplierSummary>> {
        self.repository.get_top_suppliers_by_rating(self.tenant_context.tenant_id, limit).await
    }

    async fn get_suppliers_requiring_attention(&self) -> Result<Vec<SupplierSummary>> {
        self.repository.get_suppliers_requiring_attention(self.tenant_context.tenant_id).await
    }

    async fn validate_supplier_code(&self, supplier_code: &str) -> Result<bool> {
        if supplier_code.trim().is_empty() {
            return Ok(false);
        }

        let existing = self.repository.get_supplier_by_code(self.tenant_context.tenant_id, supplier_code).await?;
        Ok(existing.is_none())
    }
}

// Mock implementation for testing
pub struct MockSupplierService {
    suppliers: std::sync::Arc<std::sync::Mutex<Vec<Supplier>>>,
    tenant_context: TenantContext,
}

impl MockSupplierService {
    pub fn new(tenant_context: TenantContext) -> Self {
        Self {
            suppliers: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            tenant_context,
        }
    }
}

#[async_trait]
impl SupplierService for MockSupplierService {
    async fn create_supplier(&self, request: CreateSupplierRequest) -> Result<Supplier> {
        let supplier = Supplier::new(
            self.tenant_context.tenant_id,
            request.supplier_code,
            request.company_name,
            self.tenant_context.user_id,
        );

        let mut suppliers = self.suppliers.lock().unwrap();
        suppliers.push(supplier.clone());

        Ok(supplier)
    }

    async fn get_supplier(&self, supplier_id: Uuid) -> Result<Option<Supplier>> {
        let suppliers = self.suppliers.lock().unwrap();
        Ok(suppliers.iter().find(|s| s.id == supplier_id).cloned())
    }

    async fn get_supplier_by_code(&self, supplier_code: &str) -> Result<Option<Supplier>> {
        let suppliers = self.suppliers.lock().unwrap();
        Ok(suppliers.iter().find(|s| s.supplier_code == supplier_code).cloned())
    }

    async fn update_supplier(&self, supplier_id: Uuid, _request: UpdateSupplierRequest) -> Result<Supplier> {
        let suppliers = self.suppliers.lock().unwrap();
        suppliers.iter().find(|s| s.id == supplier_id)
            .cloned()
            .ok_or_else(|| Error::new(ErrorCode::NotFound, "Supplier not found"))
    }

    async fn delete_supplier(&self, supplier_id: Uuid) -> Result<()> {
        let mut suppliers = self.suppliers.lock().unwrap();
        if let Some(pos) = suppliers.iter().position(|s| s.id == supplier_id) {
            suppliers.remove(pos);
            Ok(())
        } else {
            Err(Error::new(ErrorCode::NotFound, "Supplier not found"))
        }
    }

    async fn activate_supplier(&self, supplier_id: Uuid) -> Result<Supplier> {
        self.update_supplier(supplier_id, UpdateSupplierRequest::default()).await
    }

    async fn deactivate_supplier(&self, supplier_id: Uuid) -> Result<Supplier> {
        self.update_supplier(supplier_id, UpdateSupplierRequest::default()).await
    }

    async fn search_suppliers(&self, _filters: SupplierSearchFilters, pagination: PaginationOptions) -> Result<PaginationResult<SupplierSummary>> {
        let suppliers = self.suppliers.lock().unwrap();
        let total = suppliers.len() as i64;
        let items: Vec<SupplierSummary> = suppliers.iter().map(|s| SupplierSummary {
            id: s.id,
            supplier_code: s.supplier_code.clone(),
            company_name: s.company_name.clone(),
            category: s.category.clone(),
            status: s.status.clone(),
            rating: s.rating,
            on_time_delivery_rate: s.on_time_delivery_rate,
            total_orders: None,
            created_at: s.created_at,
        }).collect();

        Ok(PaginationResult {
            items,
            total,
            page: pagination.page() as i64,
            limit: pagination.limit() as i64,
            total_pages: 1,
        })
    }

    async fn list_suppliers(&self, pagination: PaginationOptions) -> Result<PaginationResult<SupplierSummary>> {
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
        self.search_suppliers(filters, pagination).await
    }

    // Mock implementations for other methods
    async fn add_supplier_contact(&self, _supplier_id: Uuid, first_name: String, last_name: String, role: String, _email: Option<String>, _phone: Option<String>) -> Result<SupplierContact> {
        Ok(SupplierContact::new(
            Uuid::new_v4(),
            self.tenant_context.tenant_id,
            first_name,
            last_name,
            role,
            self.tenant_context.user_id,
        ))
    }

    async fn get_supplier_contacts(&self, _supplier_id: Uuid) -> Result<Vec<SupplierContact>> {
        Ok(Vec::new())
    }

    async fn update_supplier_contact(&self, _contact_id: Uuid, _first_name: Option<String>, _last_name: Option<String>, _email: Option<String>, _phone: Option<String>) -> Result<SupplierContact> {
        Err(Error::new(ErrorCode::NotImplemented, "Mock implementation"))
    }

    async fn set_primary_contact(&self, _supplier_id: Uuid, _contact_id: Uuid) -> Result<()> {
        Ok(())
    }

    async fn remove_supplier_contact(&self, _contact_id: Uuid) -> Result<()> {
        Ok(())
    }

    async fn add_supplier_address(&self, _supplier_id: Uuid, address_type: String, street1: String, city: String, country: String) -> Result<SupplierAddress> {
        Ok(SupplierAddress::new(
            Uuid::new_v4(),
            self.tenant_context.tenant_id,
            address_type,
            street1,
            city,
            country,
            self.tenant_context.user_id,
        ))
    }

    async fn get_supplier_addresses(&self, _supplier_id: Uuid) -> Result<Vec<SupplierAddress>> {
        Ok(Vec::new())
    }

    async fn update_supplier_address(&self, _address_id: Uuid, _street1: Option<String>, _city: Option<String>, _country: Option<String>) -> Result<SupplierAddress> {
        Err(Error::new(ErrorCode::NotImplemented, "Mock implementation"))
    }

    async fn set_primary_address(&self, _supplier_id: Uuid, _address_id: Uuid) -> Result<()> {
        Ok(())
    }

    async fn remove_supplier_address(&self, _address_id: Uuid) -> Result<()> {
        Ok(())
    }

    async fn record_supplier_performance(&self, _supplier_id: Uuid, _performance_data: SupplierPerformanceData) -> Result<SupplierPerformance> {
        Err(Error::new(ErrorCode::NotImplemented, "Mock implementation"))
    }

    async fn get_supplier_performance(&self, _supplier_id: Uuid) -> Result<Vec<SupplierPerformance>> {
        Ok(Vec::new())
    }

    async fn get_supplier_performance_summary(&self, _supplier_id: Uuid) -> Result<Option<SupplierPerformance>> {
        Ok(None)
    }

    async fn update_supplier_rating(&self, supplier_id: Uuid, _rating: f64) -> Result<Supplier> {
        self.get_supplier(supplier_id).await?.ok_or_else(|| Error::new(ErrorCode::NotFound, "Supplier not found"))
    }

    async fn get_suppliers_by_category(&self) -> Result<Vec<(SupplierCategory, i64)>> {
        Ok(vec![(SupplierCategory::Technology, 5), (SupplierCategory::Manufacturing, 3)])
    }

    async fn get_top_suppliers(&self, _limit: i32) -> Result<Vec<SupplierSummary>> {
        Ok(Vec::new())
    }

    async fn get_suppliers_requiring_attention(&self) -> Result<Vec<SupplierSummary>> {
        Ok(Vec::new())
    }

    async fn validate_supplier_code(&self, _supplier_code: &str) -> Result<bool> {
        Ok(true)
    }
}

impl Default for UpdateSupplierRequest {
    fn default() -> Self {
        Self {
            company_name: None,
            legal_name: None,
            tax_id: None,
            registration_number: None,
            category: None,
            status: None,
            website: None,
            phone: None,
            email: None,
            payment_terms: None,
            currency: None,
            credit_limit: None,
            lead_time_days: None,
            rating: None,
            notes: None,
            tags: None,
        }
    }
}

/// Data structure for recording supplier performance
#[derive(Debug, Clone)]
pub struct SupplierPerformanceData {
    pub period_start: chrono::DateTime<Utc>,
    pub period_end: chrono::DateTime<Utc>,
    pub total_orders: i32,
    pub on_time_deliveries: i32,
    pub late_deliveries: i32,
    pub early_deliveries: i32,
    pub average_lead_time_days: Option<f64>,
    pub quality_rating: Option<f64>,
    pub defect_rate: Option<f64>,
    pub return_rate: Option<f64>,
    pub total_spend: i64,
    pub average_order_value: i64,
    pub payment_compliance_rate: Option<f64>,
    pub overall_rating: Option<f64>,
    pub notes: Option<String>,
}