use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use validator::Validate;

use crate::types::*;

/// Comprehensive customer entity that exceeds capabilities of SAP/Oracle/Dynamics
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Customer {
    // Core Identity
    pub id: Uuid,

    #[validate(length(min = 1, max = 50))]
    pub customer_number: String, // Auto-generated + custom patterns

    pub external_ids: HashMap<String, String>, // Multi-system integration IDs

    // Business Information
    #[validate(length(min = 1, max = 255))]
    pub legal_name: String,

    pub trade_names: Vec<String>, // Multiple DBA names

    pub customer_type: CustomerType,

    pub industry_classification: IndustryClassification,

    pub business_size: BusinessSize,

    // Hierarchy & Grouping (SAP-level functionality)
    pub parent_customer_id: Option<Uuid>,
    pub corporate_group_id: Option<Uuid>,
    pub customer_hierarchy_level: u8,
    pub consolidation_group: Option<String>,

    // Status & Lifecycle
    pub lifecycle_stage: CustomerLifecycleStage,
    pub status: EntityStatus,
    pub credit_status: CreditStatus,

    // Geographic & Contact Information
    pub primary_address_id: Option<Uuid>,
    pub billing_address_id: Option<Uuid>,
    pub shipping_address_ids: Vec<Uuid>,
    pub addresses: Vec<Address>,

    pub primary_contact_id: Option<Uuid>,
    pub contacts: Vec<ContactInfo>,

    // Tax & Legal
    pub tax_jurisdictions: Vec<TaxJurisdiction>,
    pub tax_numbers: HashMap<String, String>, // VAT, GST, etc.
    pub regulatory_classifications: Vec<RegulatoryClassification>,
    pub compliance_status: ComplianceStatus,
    pub kyc_status: KycStatus,
    pub aml_risk_rating: RiskRating,

    // Commercial & Financial
    pub financial_info: FinancialInfo,
    pub price_group_id: Option<Uuid>,
    pub discount_group_id: Option<Uuid>,

    // Sales & Marketing
    pub sales_representative_id: Option<Uuid>,
    pub account_manager_id: Option<Uuid>,
    pub customer_segments: Vec<CustomerSegment>,
    pub acquisition_channel: Option<AcquisitionChannel>,
    pub customer_lifetime_value: Option<Decimal>,
    pub churn_probability: Option<f64>, // 0.0 to 1.0

    // Analytics & Intelligence
    pub performance_metrics: CustomerPerformanceMetrics,
    pub behavioral_data: CustomerBehavioralData,

    // Integration & Sync
    pub sync_info: SyncInfo,

    // Custom & Extended Data
    pub custom_fields: HashMap<String, serde_json::Value>,
    pub contract_ids: Vec<Uuid>,

    // Audit Trail
    pub audit: AuditFields,
}

/// Customer-specific types and enums

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "customer_type", rename_all = "snake_case")]
pub enum CustomerType {
    B2b,        // Business to Business
    B2c,        // Business to Consumer
    B2g,        // Business to Government
    Business,   // Generic business customer
    Individual, // Individual person
    Government, // Government entity
    Internal,   // Internal company/department
    Reseller,   // Channel partner
    Distributor,
    EndUser,
    Prospect,   // Potential customer
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "customer_lifecycle_stage", rename_all = "snake_case")]
pub enum CustomerLifecycleStage {
    Lead,
    Prospect,
    ProspectCustomer, // Alternative prospect variant
    NewCustomer,
    Active,           // Alternative for ActiveCustomer
    ActiveCustomer,
    VipCustomer,
    AtRiskCustomer,
    InactiveCustomer,
    Churned,          // Churned customer variant
    WonBackCustomer,
    FormerCustomer,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "credit_status", rename_all = "snake_case")]
pub enum CreditStatus {
    Excellent,
    Good,
    Fair,
    Poor,
    OnHold,
    Blocked,
    CashOnly,
    RequiresPrepayment,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "compliance_status", rename_all = "snake_case")]
pub enum ComplianceStatus {
    Compliant,
    NonCompliant,
    UnderReview,
    PendingDocuments,
    Exempt,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "kyc_status", rename_all = "snake_case")]
pub enum KycStatus {
    NotStarted,
    InProgress,
    Completed,
    RequiresUpdate,
    Failed,
    Exempted,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "acquisition_channel", rename_all = "snake_case")]
pub enum AcquisitionChannel {
    DirectSales,
    WebsiteInquiry,
    SocialMedia,
    EmailMarketing,
    SearchEngine,
    Referral,
    PartnerChannel,
    TradeShow,
    ColdCall,
    Advertisement,
    Other,
}

/// Tax jurisdiction information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxJurisdiction {
    pub jurisdiction_code: String,
    pub jurisdiction_name: String,
    pub tax_rate: Option<Decimal>,
    pub effective_date: DateTime<Utc>,
    pub expiry_date: Option<DateTime<Utc>>,
}

/// Regulatory classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegulatoryClassification {
    pub classification_type: String, // e.g., "EXPORT_CONTROL", "SANCTIONS", "INDUSTRY_SPECIFIC"
    pub classification_code: String,
    pub description: String,
    pub effective_date: DateTime<Utc>,
    pub expiry_date: Option<DateTime<Utc>>,
    pub issuing_authority: String,
}

/// Customer segmentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerSegment {
    pub segment_type: String, // e.g., "GEOGRAPHIC", "DEMOGRAPHIC", "BEHAVIORAL", "PSYCHOGRAPHIC"
    pub segment_value: String,
    pub confidence_score: Option<f64>, // AI-driven confidence in classification
    pub effective_date: DateTime<Utc>,
    pub expiry_date: Option<DateTime<Utc>>,
}

/// Customer performance metrics for analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerPerformanceMetrics {
    // Financial Metrics
    pub total_revenue: Option<Decimal>,
    pub revenue_last_12_months: Option<f64>,
    pub average_order_value: Option<Decimal>,
    pub order_frequency: Option<f64>,
    pub total_orders: Option<i64>,
    pub last_order_date: Option<DateTime<Utc>>,
    pub profit_margin: Option<f64>,
    pub last_purchase_date: Option<DateTime<Utc>>,
    pub first_purchase_date: Option<DateTime<Utc>>,
    pub customer_lifetime_value: Option<f64>,
    pub predicted_churn_probability: Option<f64>,

    // Relationship Metrics
    pub relationship_duration_days: Option<i32>,
    pub satisfaction_score: Option<f64>, // 1.0 to 5.0
    pub net_promoter_score: Option<i32>, // -100 to 100

    // Engagement Metrics
    pub last_contact_date: Option<DateTime<Utc>>,
    pub contact_frequency: Option<f64>, // Contacts per month
    pub response_rate: Option<f64>, // Percentage 0.0 to 1.0

    // Risk Metrics
    pub days_sales_outstanding: Option<f64>,
    pub payment_reliability_score: Option<f64>, // 0.0 to 1.0
    pub support_ticket_count: Option<i32>,

    // Updated timestamp
    pub last_calculated: DateTime<Utc>,
}

/// Customer behavioral data for advanced analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerBehavioralData {
    // Purchase Behavior
    pub preferred_purchase_channels: Vec<String>,
    pub seasonal_purchase_patterns: HashMap<String, f64>,
    pub product_category_preferences: HashMap<String, f64>,
    pub purchase_frequency: Option<f64>,
    pub preferred_categories: HashMap<String, f64>,
    pub seasonal_trends: HashMap<String, f64>,
    pub price_sensitivity: Option<f64>,
    pub brand_loyalty: Option<f64>,

    // Communication Behavior
    pub preferred_contact_times: Vec<String>, // e.g., ["morning", "weekday"]
    pub channel_engagement_rates: HashMap<String, f64>, // email, phone, etc.
    pub communication_preferences: HashMap<String, String>,

    // Service & Support
    pub support_ticket_frequency: Option<f64>,
    pub product_return_rate: Option<f64>,
    pub referral_activity: Option<f64>,

    // Digital Behavior
    pub website_engagement_score: Option<f64>,
    pub mobile_app_usage: Option<f64>,
    pub social_media_sentiment: Option<f64>, // -1.0 to 1.0

    // Predictive Scores (AI/ML generated)
    pub propensity_to_buy: Option<f64>, // 0.0 to 1.0
    pub upsell_probability: Option<f64>,
    pub cross_sell_probability: Option<f64>,

    // Updated timestamp
    pub last_updated: DateTime<Utc>,
}

/// Customer creation request DTO
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateCustomerRequest {
    #[validate(length(max = 50))]
    pub customer_number: Option<String>, // If not provided, will be auto-generated

    #[validate(length(min = 1, max = 255))]
    pub legal_name: String,

    pub trade_names: Option<Vec<String>>,

    pub customer_type: CustomerType,

    pub industry_classification: Option<IndustryClassification>,

    pub business_size: Option<BusinessSize>,

    // Hierarchy
    pub parent_customer_id: Option<Uuid>,
    pub corporate_group_id: Option<Uuid>,
    pub customer_hierarchy_level: Option<u8>,
    pub consolidation_group: Option<String>,

    // Status
    pub lifecycle_stage: Option<CustomerLifecycleStage>,
    pub status: Option<EntityStatus>,
    pub credit_status: Option<CreditStatus>,

    // Contact Information
    #[validate(nested)]
    pub addresses: Option<Vec<CreateAddressRequest>>,

    #[validate(nested)]
    pub contacts: Option<Vec<CreateContactRequest>>,

    // Tax & Legal
    pub tax_jurisdictions: Option<Vec<TaxJurisdiction>>,
    pub tax_numbers: Option<HashMap<String, String>>,

    // Commercial
    pub financial_info: Option<CreateFinancialInfoRequest>,

    // Sales & Marketing
    pub sales_representative_id: Option<Uuid>,
    pub account_manager_id: Option<Uuid>,
    pub acquisition_channel: Option<AcquisitionChannel>,

    // Integration
    pub external_ids: Option<HashMap<String, String>>,
    pub sync_info: Option<SyncInfo>,
}

/// Customer update request DTO
#[derive(Debug, Clone, Serialize, Deserialize, Validate, Default)]
pub struct UpdateCustomerRequest {
    #[validate(length(max = 50))]
    pub customer_number: Option<String>,
    #[validate(length(min = 1, max = 255))]
    pub legal_name: Option<String>,

    pub trade_names: Option<Vec<String>>,

    pub customer_type: Option<CustomerType>,

    pub industry_classification: Option<IndustryClassification>,

    pub business_size: Option<BusinessSize>,

    // Hierarchy
    pub parent_customer_id: Option<Option<Uuid>>, // Option<Option<>> allows setting to null
    pub corporate_group_id: Option<Option<Uuid>>,

    // Status
    pub lifecycle_stage: Option<CustomerLifecycleStage>,
    pub status: Option<EntityStatus>,
    pub credit_status: Option<CreditStatus>,

    // Tax & Legal
    pub tax_numbers: Option<HashMap<String, String>>,

    // Commercial
    pub financial_info: Option<UpdateFinancialInfoRequest>,

    // Sales & Marketing
    pub sales_representative_id: Option<Option<Uuid>>,
    pub account_manager_id: Option<Option<Uuid>>,

    // Integration
    pub external_ids: Option<HashMap<String, String>>,
    pub sync_info: Option<SyncInfo>,

    // Version for optimistic locking
    pub version: i32,
}

/// Supporting DTOs for nested structures
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateAddressRequest {
    pub address_type: AddressType,
    #[validate(length(min = 1, max = 255))]
    pub street_line_1: String,
    #[validate(length(max = 255))]
    pub street_line_2: Option<String>,
    #[validate(length(min = 1, max = 100))]
    pub city: String,
    #[validate(length(max = 100))]
    pub state_province: Option<String>,
    #[validate(length(min = 1, max = 20))]
    pub postal_code: String,
    #[validate(length(min = 2, max = 3))]
    pub country_code: String,
    pub coordinates: Option<GeoCoordinates>,
    pub is_primary: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateContactRequest {
    pub contact_type: ContactType,
    #[validate(length(min = 1, max = 100))]
    pub first_name: String,
    #[validate(length(min = 1, max = 100))]
    pub last_name: String,
    #[validate(length(max = 100))]
    pub title: Option<String>,
    #[validate(length(max = 100))]
    pub department: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    #[validate(length(max = 50))]
    pub phone: Option<String>,
    #[validate(length(max = 50))]
    pub mobile: Option<String>,
    pub preferred_language: Option<String>,
    pub communication_preferences: Option<CommunicationPreferences>,
    pub is_primary: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateFinancialInfoRequest {
    #[validate(length(min = 3, max = 3))]
    pub currency_code: String,
    pub credit_limit: Option<Decimal>,
    pub payment_terms: Option<PaymentTerms>,
    pub tax_exempt: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateFinancialInfoRequest {
    #[validate(length(min = 3, max = 3))]
    pub currency_code: Option<String>,
    pub credit_limit: Option<Option<Decimal>>,
    pub payment_terms: Option<PaymentTerms>,
    pub tax_exempt: Option<bool>,
}

/// Customer search and filtering
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CustomerSearchCriteria {
    // Text search
    pub search_term: Option<String>, // Searches across name, number, email, etc.

    // Basic filters
    pub customer_numbers: Option<Vec<String>>,
    pub customer_types: Option<Vec<CustomerType>>,
    pub statuses: Option<Vec<EntityStatus>>,
    pub lifecycle_stages: Option<Vec<CustomerLifecycleStage>>,

    // Hierarchy filters
    pub parent_customer_id: Option<Uuid>,
    pub corporate_group_id: Option<Uuid>,
    pub hierarchy_level: Option<u8>,

    // Geographic filters
    pub country_codes: Option<Vec<String>>,
    pub state_provinces: Option<Vec<String>>,
    pub cities: Option<Vec<String>>,

    // Commercial filters
    pub sales_representative_ids: Option<Vec<Uuid>>,
    pub account_manager_ids: Option<Vec<Uuid>>,
    pub credit_statuses: Option<Vec<CreditStatus>>,
    pub customer_segments: Option<Vec<String>>,

    // Financial filters
    pub min_credit_limit: Option<Decimal>,
    pub max_credit_limit: Option<Decimal>,
    pub min_customer_lifetime_value: Option<Decimal>,
    pub max_customer_lifetime_value: Option<Decimal>,

    // Date filters
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub modified_after: Option<DateTime<Utc>>,
    pub modified_before: Option<DateTime<Utc>>,
    pub last_order_after: Option<DateTime<Utc>>,
    pub last_order_before: Option<DateTime<Utc>>,

    // Risk and analytics filters
    pub min_churn_probability: Option<f64>,
    pub max_churn_probability: Option<f64>,
    pub risk_ratings: Option<Vec<RiskRating>>,

    // Pagination
    pub page: Option<u32>,
    pub page_size: Option<u32>,

    // Sorting
    pub sort_by: Option<CustomerSortField>,
    pub sort_order: Option<SortOrder>,

    // Include related data
    pub include_addresses: Option<bool>,
    pub include_contacts: Option<bool>,
    pub include_performance_metrics: Option<bool>,
    pub include_behavioral_data: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CustomerSortField {
    CustomerNumber,
    LegalName,
    CreatedAt,
    ModifiedAt,
    LastOrderDate,
    CustomerLifetimeValue,
    ChurnProbability,
    TotalRevenue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    Asc,
    Desc,
}

/// Customer response with pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerSearchResponse {
    pub customers: Vec<Customer>,
    pub total_count: u64,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
}

// Default implementations
impl Default for CustomerType {
    fn default() -> Self {
        CustomerType::B2b
    }
}

impl Default for CustomerLifecycleStage {
    fn default() -> Self {
        CustomerLifecycleStage::Prospect
    }
}

impl Default for CreditStatus {
    fn default() -> Self {
        CreditStatus::Good
    }
}

impl Default for CustomerPerformanceMetrics {
    fn default() -> Self {
        Self {
            total_revenue: None,
            revenue_last_12_months: None,
            average_order_value: None,
            order_frequency: None,
            total_orders: None,
            last_order_date: None,
            profit_margin: None,
            last_purchase_date: None,
            first_purchase_date: None,
            customer_lifetime_value: None,
            predicted_churn_probability: None,
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
        }
    }
}

impl Default for CustomerBehavioralData {
    fn default() -> Self {
        Self {
            preferred_purchase_channels: Vec::new(),
            seasonal_purchase_patterns: HashMap::new(),
            product_category_preferences: HashMap::new(),
            purchase_frequency: None,
            preferred_categories: HashMap::new(),
            seasonal_trends: HashMap::new(),
            price_sensitivity: None,
            brand_loyalty: None,
            preferred_contact_times: Vec::new(),
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
        }
    }
}

// Additional type definitions for full functionality

/// Customer list result with pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerListResult {
    pub customers: Vec<Customer>,
    pub total_count: u64,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
}

/// Request to update an address
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateAddressRequest {
    #[validate(length(max = 255))]
    pub street_line_1: Option<String>,
    #[validate(length(max = 255))]
    pub street_line_2: Option<String>,
    #[validate(length(max = 100))]
    pub city: Option<String>,
    #[validate(length(max = 100))]
    pub state_province: Option<String>,
    #[validate(length(max = 20))]
    pub postal_code: Option<String>,
    #[validate(length(min = 2, max = 3))]
    pub country_code: Option<String>,
    pub is_primary: Option<bool>,
    pub is_active: Option<bool>,
}

/// Request to update a contact
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateContactRequest {
    #[validate(length(max = 100))]
    pub first_name: Option<String>,
    #[validate(length(max = 100))]
    pub last_name: Option<String>,
    #[validate(length(max = 100))]
    pub title: Option<String>,
    #[validate(length(max = 100))]
    pub department: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    #[validate(length(max = 50))]
    pub phone: Option<String>,
    #[validate(length(max = 50))]
    pub mobile: Option<String>,
    pub is_primary: Option<bool>,
    pub is_active: Option<bool>,
}