//! Supplier data models and types
//!
//! This module defines the core data structures for supplier management,
//! including supplier profiles, contacts, addresses, and related entities.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Supplier status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "supplier_status", rename_all = "snake_case")]
pub enum SupplierStatus {
    /// Supplier is active and can be used for procurement
    Active,
    /// Supplier is temporarily inactive
    Inactive,
    /// Supplier is under evaluation/approval process
    Pending,
    /// Supplier has been suspended due to performance issues
    Suspended,
    /// Supplier relationship has been terminated
    Terminated,
}

impl Default for SupplierStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// Supplier category for classification
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "supplier_category", rename_all = "snake_case")]
pub enum SupplierCategory {
    /// Raw materials supplier
    RawMaterials,
    /// Manufacturing and production
    Manufacturing,
    /// Technology and software
    Technology,
    /// Professional services
    Services,
    /// Logistics and transportation
    Logistics,
    /// Office supplies and equipment
    OfficeSupplies,
    /// Marketing and advertising
    Marketing,
    /// Utilities and facilities
    Utilities,
    /// Other category
    Other,
}

impl Default for SupplierCategory {
    fn default() -> Self {
        Self::Other
    }
}

/// Payment terms enumeration
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "payment_terms", rename_all = "snake_case")]
pub enum PaymentTerms {
    /// Net 15 days
    Net15,
    /// Net 30 days
    Net30,
    /// Net 45 days
    Net45,
    /// Net 60 days
    Net60,
    /// Net 90 days
    Net90,
    /// 2% discount if paid within 10 days, otherwise net 30
    TwoTenNet30,
    /// Cash on delivery
    Cod,
    /// Prepaid
    Prepaid,
}

impl Default for PaymentTerms {
    fn default() -> Self {
        Self::Net30
    }
}

/// Main supplier entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Supplier {
    pub id: Uuid,
    pub tenant_id: Uuid,

    // Basic Information
    pub supplier_code: String,
    pub company_name: String,
    pub legal_name: Option<String>,
    pub tax_id: Option<String>,
    pub registration_number: Option<String>,

    // Classification
    pub category: SupplierCategory,
    pub status: SupplierStatus,
    pub tags: Option<Vec<String>>,

    // Contact Information
    pub website: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,

    // Business Terms
    pub payment_terms: PaymentTerms,
    pub currency: String,
    pub credit_limit: Option<i64>, // in cents
    pub lead_time_days: Option<i32>,

    // Performance Metrics
    pub rating: Option<f64>,
    pub on_time_delivery_rate: Option<f64>,
    pub quality_rating: Option<f64>,

    // Metadata
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub updated_by: Uuid,
}

impl Supplier {
    /// Create a new supplier with default values
    pub fn new(
        tenant_id: Uuid,
        supplier_code: String,
        company_name: String,
        created_by: Uuid,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            supplier_code,
            company_name,
            legal_name: None,
            tax_id: None,
            registration_number: None,
            category: SupplierCategory::default(),
            status: SupplierStatus::default(),
            tags: None,
            website: None,
            phone: None,
            email: None,
            payment_terms: PaymentTerms::default(),
            currency: "USD".to_string(),
            credit_limit: None,
            lead_time_days: None,
            rating: None,
            on_time_delivery_rate: None,
            quality_rating: None,
            notes: None,
            created_at: now,
            updated_at: now,
            created_by,
            updated_by: created_by,
        }
    }

    /// Check if supplier is active
    pub fn is_active(&self) -> bool {
        matches!(self.status, SupplierStatus::Active)
    }

    /// Check if supplier can be used for ordering
    pub fn can_order(&self) -> bool {
        matches!(self.status, SupplierStatus::Active | SupplierStatus::Pending)
    }

    /// Get display name (company name or legal name)
    pub fn display_name(&self) -> &str {
        self.legal_name.as_ref().unwrap_or(&self.company_name)
    }
}

/// Supplier contact person
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SupplierContact {
    pub id: Uuid,
    pub supplier_id: Uuid,
    pub tenant_id: Uuid,

    // Personal Information
    pub first_name: String,
    pub last_name: String,
    pub title: Option<String>,
    pub department: Option<String>,

    // Contact Information
    pub email: Option<String>,
    pub phone: Option<String>,
    pub mobile: Option<String>,

    // Role and Status
    pub role: String,
    pub is_primary: bool,
    pub is_active: bool,

    // Metadata
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub updated_by: Uuid,
}

impl SupplierContact {
    /// Create a new supplier contact
    pub fn new(
        supplier_id: Uuid,
        tenant_id: Uuid,
        first_name: String,
        last_name: String,
        role: String,
        created_by: Uuid,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            supplier_id,
            tenant_id,
            first_name,
            last_name,
            title: None,
            department: None,
            email: None,
            phone: None,
            mobile: None,
            role,
            is_primary: false,
            is_active: true,
            notes: None,
            created_at: now,
            updated_at: now,
            created_by,
            updated_by: created_by,
        }
    }

    /// Get full name
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }
}

/// Supplier address (billing, shipping, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SupplierAddress {
    pub id: Uuid,
    pub supplier_id: Uuid,
    pub tenant_id: Uuid,

    // Address Type
    pub address_type: String, // "billing", "shipping", "headquarters", etc.
    pub is_primary: bool,

    // Address Fields
    pub street1: String,
    pub street2: Option<String>,
    pub city: String,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: String,

    // Metadata
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub updated_by: Uuid,
}

impl SupplierAddress {
    /// Create a new supplier address
    pub fn new(
        supplier_id: Uuid,
        tenant_id: Uuid,
        address_type: String,
        street1: String,
        city: String,
        country: String,
        created_by: Uuid,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            supplier_id,
            tenant_id,
            address_type,
            is_primary: false,
            street1,
            street2: None,
            city,
            state: None,
            postal_code: None,
            country,
            is_active: true,
            created_at: now,
            updated_at: now,
            created_by,
            updated_by: created_by,
        }
    }

    /// Format address as a single string
    pub fn format_address(&self) -> String {
        let mut parts = vec![self.street1.clone()];

        if let Some(street2) = &self.street2 {
            if !street2.is_empty() {
                parts.push(street2.clone());
            }
        }

        let mut city_line = self.city.clone();
        if let Some(state) = &self.state {
            city_line.push_str(&format!(", {}", state));
        }
        if let Some(postal) = &self.postal_code {
            city_line.push_str(&format!(" {}", postal));
        }
        parts.push(city_line);
        parts.push(self.country.clone());

        parts.join("\n")
    }
}

/// Supplier performance metrics
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SupplierPerformance {
    pub id: Uuid,
    pub supplier_id: Uuid,
    pub tenant_id: Uuid,

    // Performance Period
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,

    // Delivery Metrics
    pub total_orders: i32,
    pub on_time_deliveries: i32,
    pub late_deliveries: i32,
    pub early_deliveries: i32,
    pub average_lead_time_days: Option<f64>,

    // Quality Metrics
    pub quality_rating: Option<f64>,
    pub defect_rate: Option<f64>,
    pub return_rate: Option<f64>,

    // Financial Metrics
    pub total_spend: i64, // in cents
    pub average_order_value: i64, // in cents
    pub payment_compliance_rate: Option<f64>,

    // Overall Rating
    pub overall_rating: Option<f64>,

    // Metadata
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
}

impl SupplierPerformance {
    /// Calculate on-time delivery rate
    pub fn on_time_delivery_rate(&self) -> f64 {
        if self.total_orders == 0 {
            0.0
        } else {
            self.on_time_deliveries as f64 / self.total_orders as f64
        }
    }

    /// Calculate late delivery rate
    pub fn late_delivery_rate(&self) -> f64 {
        if self.total_orders == 0 {
            0.0
        } else {
            self.late_deliveries as f64 / self.total_orders as f64
        }
    }
}

/// Request/Response DTOs for API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSupplierRequest {
    pub supplier_code: String,
    pub company_name: String,
    pub legal_name: Option<String>,
    pub tax_id: Option<String>,
    pub registration_number: Option<String>,
    pub category: SupplierCategory,
    pub website: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub payment_terms: PaymentTerms,
    pub currency: String,
    pub credit_limit: Option<i64>,
    pub lead_time_days: Option<i32>,
    pub notes: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSupplierRequest {
    pub company_name: Option<String>,
    pub legal_name: Option<String>,
    pub tax_id: Option<String>,
    pub registration_number: Option<String>,
    pub category: Option<SupplierCategory>,
    pub status: Option<SupplierStatus>,
    pub website: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub payment_terms: Option<PaymentTerms>,
    pub currency: Option<String>,
    pub credit_limit: Option<i64>,
    pub lead_time_days: Option<i32>,
    pub rating: Option<f64>,
    pub notes: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplierSearchFilters {
    pub query: Option<String>,
    pub status: Option<SupplierStatus>,
    pub category: Option<SupplierCategory>,
    pub tags: Option<Vec<String>>,
    pub min_rating: Option<f64>,
    pub max_rating: Option<f64>,
    pub payment_terms: Option<PaymentTerms>,
    pub country: Option<String>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplierSummary {
    pub id: Uuid,
    pub supplier_code: String,
    pub company_name: String,
    pub category: SupplierCategory,
    pub status: SupplierStatus,
    pub rating: Option<f64>,
    pub on_time_delivery_rate: Option<f64>,
    pub total_orders: Option<i32>,
    pub created_at: DateTime<Utc>,
}