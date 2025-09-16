use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use validator::Validate;

/// Common audit fields for all master data entities
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuditFields {
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub modified_by: Uuid,
    pub modified_at: DateTime<Utc>,
    pub version: i32,
    pub is_deleted: bool,
    pub deleted_at: Option<DateTime<Utc>>,
    pub deleted_by: Option<Uuid>,
}

/// Geographic coordinates
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GeoCoordinates {
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy: Option<f32>, // Accuracy in meters
}

/// Address information
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Address {
    pub id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
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
    pub country_code: String, // ISO 3166-1 alpha-2 or alpha-3
    pub coordinates: Option<GeoCoordinates>,
    pub is_primary: bool,
    pub is_active: bool,
    pub audit: AuditFields,
}

/// Contact information
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ContactInfo {
    pub id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
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
    #[validate(length(max = 50))]
    pub fax: Option<String>,
    pub preferred_language: Option<String>, // ISO 639-1
    pub communication_preferences: Option<CommunicationPreferences>,
    pub is_primary: bool,
    pub is_active: bool,
    pub audit: AuditFields,
}

/// Communication preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationPreferences {
    pub email_notifications: bool,
    pub sms_notifications: bool,
    pub phone_calls: bool,
    pub postal_mail: bool,
    pub preferred_time_start: Option<String>, // HH:MM format
    pub preferred_time_end: Option<String>,   // HH:MM format
    pub preferred_timezone: Option<String>,   // IANA timezone
}

/// Financial information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialInfo {
    pub currency_code: String, // ISO 4217
    pub credit_limit: Option<rust_decimal::Decimal>,
    pub payment_terms: Option<PaymentTerms>,
    pub tax_exempt: bool,
    pub tax_numbers: HashMap<String, String>, // Tax type -> Tax number
}

/// Payment terms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentTerms {
    pub payment_method: PaymentMethod,
    pub net_days: Option<i32>,
    pub discount_percentage: Option<rust_decimal::Decimal>,
    pub discount_days: Option<i32>,
    pub late_fee_percentage: Option<rust_decimal::Decimal>,
}

/// Data synchronization information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncInfo {
    pub last_sync: Option<DateTime<Utc>>,
    pub sync_source: Option<String>,
    pub sync_version: Option<String>,
    pub sync_status: SyncStatus,
    pub external_references: HashMap<String, String>,
}

// Enumerations

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "address_type", rename_all = "snake_case")]
pub enum AddressType {
    Billing,
    Shipping,
    Mailing,
    Physical,
    Headquarters,
    Branch,
    Warehouse,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "contact_type", rename_all = "snake_case")]
pub enum ContactType {
    Primary,
    Billing,
    Technical,
    Sales,
    Purchasing,
    Support,
    Executive,
    DecisionMaker,
    Influencer,
    User,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "payment_method", rename_all = "snake_case")]
pub enum PaymentMethod {
    Cash,
    Check,
    BankTransfer,
    CreditCard,
    DebitCard,
    DigitalWallet,
    Cryptocurrency,
    TradeCredit,
    LetterOfCredit,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "data_source", rename_all = "snake_case")]
pub enum DataSource {
    Manual,
    Import,
    Api,
    Integration,
    Migration,
    Synchronization,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "sync_status", rename_all = "snake_case")]
pub enum SyncStatus {
    NotSynced,
    Pending,
    InProgress,
    Success,
    Failed,
    Conflict,
    Skipped,
}

/// Generic status for entities
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "entity_status", rename_all = "snake_case")]
pub enum EntityStatus {
    Active,
    Inactive,
    Pending,
    Suspended,
    Blocked,
    Archived,
    Deleted,
}

/// Risk rating levels
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "risk_rating", rename_all = "snake_case")]
pub enum RiskRating {
    Low,
    Medium,
    High,
    Critical,
}

/// Business size classifications
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "business_size", rename_all = "snake_case")]
pub enum BusinessSize {
    Micro,     // < 10 employees
    Small,     // 10-49 employees
    Medium,    // 50-249 employees
    Large,     // 250-999 employees
    Enterprise, // 1000+ employees
}

/// Industry classification systems
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "industry_classification", rename_all = "snake_case")]
pub enum IndustryClassification {
    Technology,
    Manufacturing,
    Healthcare,
    Finance,
    Retail,
    Education,
    Government,
    Energy,
    Transportation,
    RealEstate,
    Agriculture,
    Construction,
    Entertainment,
    Telecommunications,
    Other,
}

/// Default implementations
impl Default for CommunicationPreferences {
    fn default() -> Self {
        Self {
            email_notifications: true,
            sms_notifications: false,
            phone_calls: true,
            postal_mail: false,
            preferred_time_start: None,
            preferred_time_end: None,
            preferred_timezone: None,
        }
    }
}

impl Default for PaymentTerms {
    fn default() -> Self {
        Self {
            payment_method: PaymentMethod::BankTransfer,
            net_days: Some(30),
            discount_percentage: None,
            discount_days: None,
            late_fee_percentage: None,
        }
    }
}

impl Default for FinancialInfo {
    fn default() -> Self {
        Self {
            currency_code: "USD".to_string(),
            credit_limit: None,
            payment_terms: Some(PaymentTerms::default()),
            tax_exempt: false,
            tax_numbers: HashMap::new(),
        }
    }
}

impl Default for SyncInfo {
    fn default() -> Self {
        Self {
            last_sync: None,
            sync_source: None,
            sync_version: None,
            sync_status: SyncStatus::NotSynced,
            external_references: HashMap::new(),
        }
    }
}