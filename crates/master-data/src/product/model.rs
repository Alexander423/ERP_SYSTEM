//! Product data models and types
//!
//! This module defines the core data structures for product management,
//! including products, categories, pricing, and variants.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use std::collections::HashMap;

/// Product status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "product_status", rename_all = "snake_case")]
pub enum ProductStatus {
    /// Product is active and available for sale
    Active,
    /// Product is temporarily inactive
    Inactive,
    /// Product is under development
    Development,
    /// Product has been discontinued
    Discontinued,
    /// Product is planned but not yet developed
    Planned,
}

impl Default for ProductStatus {
    fn default() -> Self {
        Self::Development
    }
}

/// Product type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "product_type", rename_all = "snake_case")]
pub enum ProductType {
    /// Physical product that can be stocked
    Physical,
    /// Digital/virtual product
    Digital,
    /// Service offering
    Service,
    /// Bundle of multiple products
    Bundle,
    /// Subscription-based product
    Subscription,
}

impl Default for ProductType {
    fn default() -> Self {
        Self::Physical
    }
}

/// Unit of measure for products
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "unit_of_measure", rename_all = "snake_case")]
pub enum UnitOfMeasure {
    /// Piece/each
    Piece,
    /// Kilogram
    Kg,
    /// Gram
    Gram,
    /// Liter
    Liter,
    /// Milliliter
    Ml,
    /// Meter
    Meter,
    /// Centimeter
    Cm,
    /// Square meter
    SquareMeter,
    /// Cubic meter
    CubicMeter,
    /// Hour (for services)
    Hour,
    /// Box/carton
    Box,
    /// Pallet
    Pallet,
}

impl Default for UnitOfMeasure {
    fn default() -> Self {
        Self::Piece
    }
}

/// Main product entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Product {
    pub id: Uuid,
    pub tenant_id: Uuid,

    // Basic Information
    pub sku: String,
    pub name: String,
    pub description: Option<String>,
    pub short_description: Option<String>,

    // Classification
    pub category_id: Option<Uuid>,
    pub product_type: ProductType,
    pub status: ProductStatus,
    pub tags: Option<Vec<String>>,

    // Physical Properties
    pub unit_of_measure: UnitOfMeasure,
    pub weight: Option<f64>, // in kg
    pub dimensions_length: Option<f64>, // in cm
    pub dimensions_width: Option<f64>, // in cm
    pub dimensions_height: Option<f64>, // in cm

    // Pricing
    pub base_price: i64, // in cents
    pub currency: String,
    pub cost_price: Option<i64>, // in cents
    pub list_price: Option<i64>, // in cents

    // Inventory
    pub is_tracked: bool,
    pub current_stock: Option<i32>,
    pub min_stock_level: Option<i32>,
    pub max_stock_level: Option<i32>,
    pub reorder_point: Option<i32>,

    // Supplier Information
    pub primary_supplier_id: Option<Uuid>,
    pub lead_time_days: Option<i32>,

    // Additional Properties
    pub barcode: Option<String>,
    pub brand: Option<String>,
    pub manufacturer: Option<String>,
    pub model_number: Option<String>,
    pub warranty_months: Option<i32>,

    // SEO and Marketing
    pub slug: Option<String>,
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
    pub is_featured: bool,
    pub is_digital_download: bool,

    // Metadata
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub updated_by: Uuid,
}

impl Product {
    /// Create a new product with default values
    pub fn new(
        tenant_id: Uuid,
        sku: String,
        name: String,
        created_by: Uuid,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            sku,
            name,
            description: None,
            short_description: None,
            category_id: None,
            product_type: ProductType::default(),
            status: ProductStatus::default(),
            tags: None,
            unit_of_measure: UnitOfMeasure::default(),
            weight: None,
            dimensions_length: None,
            dimensions_width: None,
            dimensions_height: None,
            base_price: 0,
            currency: "USD".to_string(),
            cost_price: None,
            list_price: None,
            is_tracked: true,
            current_stock: Some(0),
            min_stock_level: None,
            max_stock_level: None,
            reorder_point: None,
            primary_supplier_id: None,
            lead_time_days: None,
            barcode: None,
            brand: None,
            manufacturer: None,
            model_number: None,
            warranty_months: None,
            slug: None,
            meta_title: None,
            meta_description: None,
            is_featured: false,
            is_digital_download: false,
            notes: None,
            created_at: now,
            updated_at: now,
            created_by,
            updated_by: created_by,
        }
    }

    /// Check if product is active
    pub fn is_active(&self) -> bool {
        matches!(self.status, ProductStatus::Active)
    }

    /// Check if product is in stock
    pub fn is_in_stock(&self) -> bool {
        if !self.is_tracked {
            return true; // Non-tracked products are always "in stock"
        }
        self.current_stock.unwrap_or(0) > 0
    }

    /// Check if product needs reordering
    pub fn needs_reorder(&self) -> bool {
        if let (Some(current), Some(reorder_point)) = (self.current_stock, self.reorder_point) {
            current <= reorder_point
        } else {
            false
        }
    }

    /// Calculate volume in cubic centimeters
    pub fn volume_cm3(&self) -> Option<f64> {
        match (self.dimensions_length, self.dimensions_width, self.dimensions_height) {
            (Some(l), Some(w), Some(h)) => Some(l * w * h),
            _ => None,
        }
    }

    /// Get display price (base price or list price)
    pub fn display_price(&self) -> i64 {
        self.list_price.unwrap_or(self.base_price)
    }

    /// Calculate profit margin if cost price is available
    pub fn profit_margin(&self) -> Option<f64> {
        if let Some(cost) = self.cost_price {
            if cost > 0 {
                let selling_price = self.display_price();
                let profit = selling_price - cost;
                Some((profit as f64 / selling_price as f64) * 100.0)
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// Product category for hierarchical organization
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProductCategory {
    pub id: Uuid,
    pub tenant_id: Uuid,

    pub name: String,
    pub description: Option<String>,
    pub slug: String,

    // Hierarchy
    pub parent_id: Option<Uuid>,
    pub level: i32,
    pub sort_order: i32,

    // SEO
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,

    // Status
    pub is_active: bool,

    // Metadata
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub updated_by: Uuid,
}

impl ProductCategory {
    /// Create a new product category
    pub fn new(
        tenant_id: Uuid,
        name: String,
        slug: String,
        parent_id: Option<Uuid>,
        created_by: Uuid,
    ) -> Self {
        let now = Utc::now();
        let level = if parent_id.is_some() { 1 } else { 0 }; // Simplified level calculation

        Self {
            id: Uuid::new_v4(),
            tenant_id,
            name,
            description: None,
            slug,
            parent_id,
            level,
            sort_order: 0,
            meta_title: None,
            meta_description: None,
            is_active: true,
            created_at: now,
            updated_at: now,
            created_by,
            updated_by: created_by,
        }
    }

    /// Check if this is a root category
    pub fn is_root(&self) -> bool {
        self.parent_id.is_none()
    }
}

/// Product pricing information
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProductPrice {
    pub id: Uuid,
    pub product_id: Uuid,
    pub tenant_id: Uuid,

    // Price Information
    pub price_type: String, // "base", "retail", "wholesale", "member", etc.
    pub price: i64, // in cents
    pub currency: String,

    // Validity
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_until: Option<DateTime<Utc>>,

    // Conditions
    pub min_quantity: Option<i32>,
    pub max_quantity: Option<i32>,

    // Status
    pub is_active: bool,

    // Metadata
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub updated_by: Uuid,
}

/// Product variant (for products with variations)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProductVariant {
    pub id: Uuid,
    pub product_id: Uuid,
    pub tenant_id: Uuid,

    // Variant Information
    pub variant_name: String,
    pub sku: Option<String>,
    pub attributes: Option<serde_json::Value>, // JSON object for variant attributes

    // Pricing
    pub price_adjustment: i64, // in cents, can be negative
    pub weight_adjustment: Option<f64>, // in kg, can be negative

    // Inventory
    pub current_stock: Option<i32>,
    pub barcode: Option<String>,

    // Status
    pub is_active: bool,

    // Metadata
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub updated_by: Uuid,
}

/// Product-Supplier relationship
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProductSupplier {
    pub id: Uuid,
    pub product_id: Uuid,
    pub supplier_id: Uuid,
    pub tenant_id: Uuid,

    // Supplier-specific information
    pub supplier_sku: Option<String>,
    pub supplier_name: Option<String>, // Supplier's name for this product
    pub cost_price: i64, // in cents
    pub currency: String,

    // Order Information
    pub min_order_quantity: Option<i32>,
    pub lead_time_days: i32,

    // Status
    pub is_primary: bool,
    pub is_active: bool,

    // Metadata
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub updated_by: Uuid,
}

/// Request/Response DTOs for API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProductRequest {
    pub sku: String,
    pub name: String,
    pub description: Option<String>,
    pub category_id: Option<Uuid>,
    pub product_type: ProductType,
    pub unit_of_measure: UnitOfMeasure,
    pub base_price: i64,
    pub currency: String,
    pub cost_price: Option<i64>,
    pub is_tracked: bool,
    pub current_stock: Option<i32>,
    pub min_stock_level: Option<i32>,
    pub reorder_point: Option<i32>,
    pub primary_supplier_id: Option<Uuid>,
    pub weight: Option<f64>,
    pub barcode: Option<String>,
    pub brand: Option<String>,
    pub manufacturer: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProductRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub category_id: Option<Uuid>,
    pub product_type: Option<ProductType>,
    pub status: Option<ProductStatus>,
    pub unit_of_measure: Option<UnitOfMeasure>,
    pub base_price: Option<i64>,
    pub cost_price: Option<i64>,
    pub list_price: Option<i64>,
    pub is_tracked: Option<bool>,
    pub current_stock: Option<i32>,
    pub min_stock_level: Option<i32>,
    pub max_stock_level: Option<i32>,
    pub reorder_point: Option<i32>,
    pub primary_supplier_id: Option<Uuid>,
    pub weight: Option<f64>,
    pub dimensions_length: Option<f64>,
    pub dimensions_width: Option<f64>,
    pub dimensions_height: Option<f64>,
    pub barcode: Option<String>,
    pub brand: Option<String>,
    pub manufacturer: Option<String>,
    pub model_number: Option<String>,
    pub warranty_months: Option<i32>,
    pub is_featured: Option<bool>,
    pub tags: Option<Vec<String>>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductSearchFilters {
    pub query: Option<String>,
    pub category_id: Option<Uuid>,
    pub status: Option<ProductStatus>,
    pub product_type: Option<ProductType>,
    pub tags: Option<Vec<String>>,
    pub min_price: Option<i64>,
    pub max_price: Option<i64>,
    pub in_stock_only: Option<bool>,
    pub featured_only: Option<bool>,
    pub supplier_id: Option<Uuid>,
    pub brand: Option<String>,
    pub manufacturer: Option<String>,
    pub needs_reorder: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductSummary {
    pub id: Uuid,
    pub sku: String,
    pub name: String,
    pub status: ProductStatus,
    pub product_type: ProductType,
    pub base_price: i64,
    pub currency: String,
    pub current_stock: Option<i32>,
    pub is_in_stock: bool,
    pub needs_reorder: bool,
    pub category_name: Option<String>,
    pub supplier_name: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<Uuid>,
    pub slug: Option<String>,
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockAdjustmentRequest {
    pub product_id: Uuid,
    pub adjustment_type: StockAdjustmentType,
    pub quantity: i32,
    pub reason: String,
    pub reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StockAdjustmentType {
    /// Increase stock
    Increase,
    /// Decrease stock
    Decrease,
    /// Set absolute stock level
    Set,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkPriceUpdateRequest {
    pub product_ids: Vec<Uuid>,
    pub update_type: PriceUpdateType,
    pub value: f64, // Percentage or absolute amount
    pub price_field: PriceField,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PriceUpdateType {
    /// Increase by percentage
    IncreasePercent,
    /// Decrease by percentage
    DecreasePercent,
    /// Increase by absolute amount
    IncreaseAmount,
    /// Decrease by absolute amount
    DecreaseAmount,
    /// Set to specific value
    SetValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PriceField {
    BasePrice,
    CostPrice,
    ListPrice,
}

/// Advanced product attributes for comprehensive management
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProductAttributes {
    pub id: Uuid,
    pub product_id: Uuid,
    pub tenant_id: Uuid,

    // Digital Twin & IoT
    pub digital_twin_id: Option<String>,
    pub iot_device_ids: Option<Vec<String>>,
    pub three_d_model_url: Option<String>,
    pub ar_model_url: Option<String>,

    // Sustainability & ESG
    pub carbon_footprint: Option<f64>, // kg CO2 equivalent
    pub recyclable_percentage: Option<f64>,
    pub sustainability_rating: Option<String>,
    pub eco_certifications: Option<Vec<String>>,

    // Compliance & Quality
    pub compliance_standards: Option<Vec<String>>, // ISO, FDA, CE, etc.
    pub quality_grade: Option<String>,
    pub certification_expiry: Option<DateTime<Utc>>,
    pub batch_tracking_required: bool,

    // Advanced Properties
    pub shelf_life_days: Option<i32>,
    pub storage_temperature_min: Option<f64>,
    pub storage_temperature_max: Option<f64>,
    pub storage_humidity_max: Option<f64>,
    pub hazardous_material: bool,
    pub fragile: bool,

    // Market Intelligence
    pub market_position: Option<String>, // "premium", "economy", "luxury"
    pub competitor_products: Option<Vec<String>>,
    pub market_share_percentage: Option<f64>,

    // AI Generated Content
    pub ai_generated_description: Option<String>,
    pub ai_tags: Option<Vec<String>>,
    pub seo_keywords: Option<Vec<String>>,

    // Blockchain & Authentication
    pub blockchain_hash: Option<String>,
    pub authenticity_token: Option<String>,
    pub provenance_data: Option<serde_json::Value>,

    // Metadata
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub updated_by: Uuid,
}

/// Multi-location inventory tracking
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProductInventory {
    pub id: Uuid,
    pub product_id: Uuid,
    pub location_id: Uuid,
    pub tenant_id: Uuid,

    // Stock Information
    pub current_stock: i32,
    pub available_stock: i32, // Current - Reserved
    pub reserved_stock: i32,
    pub incoming_stock: i32,
    pub outgoing_stock: i32,

    // Location-specific Settings
    pub min_stock_level: Option<i32>,
    pub max_stock_level: Option<i32>,
    pub reorder_point: Option<i32>,
    pub safety_stock: Option<i32>,

    // Physical Storage
    pub zone: Option<String>,
    pub aisle: Option<String>,
    pub shelf: Option<String>,
    pub bin: Option<String>,

    // Tracking
    pub last_count_date: Option<DateTime<Utc>>,
    pub last_movement_date: Option<DateTime<Utc>>,

    // Metadata
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub updated_by: Uuid,
}

/// Advanced pricing with dynamic rules
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DynamicPrice {
    pub id: Uuid,
    pub product_id: Uuid,
    pub tenant_id: Uuid,

    // Price Configuration
    pub price_type: String, // "base", "tier1", "volume", "seasonal", "promotional"
    pub price: i64, // in cents
    pub currency: String,

    // Dynamic Rules
    pub customer_tier: Option<String>,
    pub min_quantity: Option<i32>,
    pub max_quantity: Option<i32>,
    pub geographic_region: Option<String>,
    pub seasonal_factor: Option<f64>,

    // Time-based Rules
    pub valid_from: DateTime<Utc>,
    pub valid_until: Option<DateTime<Utc>>,
    pub time_of_day_start: Option<String>, // "09:00"
    pub time_of_day_end: Option<String>, // "17:00"
    pub days_of_week: Option<Vec<String>>, // ["monday", "tuesday"]

    // Conditions
    pub conditions: Option<serde_json::Value>, // JSON rules engine
    pub priority: i32, // Higher number = higher priority

    // Status
    pub is_active: bool,

    // Metadata
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub updated_by: Uuid,
}

/// Quality control and batch tracking
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProductBatch {
    pub id: Uuid,
    pub product_id: Uuid,
    pub tenant_id: Uuid,

    // Batch Information
    pub batch_number: String,
    pub lot_number: Option<String>,
    pub serial_numbers: Option<Vec<String>>,

    // Production Information
    pub manufactured_date: Option<DateTime<Utc>>,
    pub expiry_date: Option<DateTime<Utc>>,
    pub supplier_id: Option<Uuid>,
    pub production_line: Option<String>,

    // Quality Control
    pub quality_status: QualityStatus,
    pub quality_score: Option<f64>,
    pub quality_tests: Option<serde_json::Value>,
    pub inspector_id: Option<Uuid>,
    pub inspection_date: Option<DateTime<Utc>>,

    // Quantities
    pub initial_quantity: i32,
    pub current_quantity: i32,
    pub allocated_quantity: i32,

    // Traceability
    pub source_batches: Option<Vec<String>>,
    pub destination_batches: Option<Vec<String>>,
    pub recall_status: Option<String>,

    // Metadata
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub updated_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "quality_status", rename_all = "snake_case")]
pub enum QualityStatus {
    Pending,
    Passed,
    Failed,
    Quarantined,
    Recalled,
}

/// Product analytics and performance metrics
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProductAnalytics {
    pub id: Uuid,
    pub product_id: Uuid,
    pub tenant_id: Uuid,

    // Time Period
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub period_type: String, // "daily", "weekly", "monthly", "quarterly"

    // Sales Metrics
    pub units_sold: i32,
    pub revenue: i64, // in cents
    pub gross_profit: i64, // in cents
    pub profit_margin: f64,

    // Inventory Metrics
    pub stock_turns: f64,
    pub days_of_inventory: f64,
    pub stockout_days: i32,
    pub excess_stock_value: i64,

    // Quality Metrics
    pub return_rate: f64,
    pub defect_rate: f64,
    pub customer_satisfaction: Option<f64>,
    pub quality_incidents: i32,

    // Market Metrics
    pub market_share: Option<f64>,
    pub price_competitiveness: Option<f64>,
    pub demand_forecast: Option<i32>,
    pub trend_score: Option<f64>, // -1 to +1

    // AI Insights
    pub anomaly_score: Option<f64>,
    pub predicted_demand: Option<i32>,
    pub optimization_suggestions: Option<Vec<String>>,

    // Metadata
    pub calculated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Product lifecycle management
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProductLifecycle {
    pub id: Uuid,
    pub product_id: Uuid,
    pub tenant_id: Uuid,

    // Lifecycle Stages
    pub stage: LifecycleStage,
    pub stage_changed_at: DateTime<Utc>,
    pub previous_stage: Option<LifecycleStage>,

    // Lifecycle Metrics
    pub time_to_market: Option<i32>, // days from concept to launch
    pub development_cost: Option<i64>, // in cents
    pub total_revenue: i64, // lifetime revenue in cents
    pub units_produced: i32,
    pub units_sold: i32,

    // Future Planning
    pub planned_eol_date: Option<DateTime<Utc>>,
    pub replacement_product_id: Option<Uuid>,
    pub sunset_strategy: Option<String>,

    // Environmental Impact
    pub total_carbon_footprint: Option<f64>,
    pub materials_recycled: Option<f64>,
    pub waste_generated: Option<f64>,

    // Metadata
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub updated_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "lifecycle_stage", rename_all = "snake_case")]
pub enum LifecycleStage {
    Concept,
    Development,
    Testing,
    Launch,
    Growth,
    Maturity,
    Decline,
    EndOfLife,
    Retired,
}

/// Advanced search and filtering capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedProductSearch {
    // Basic Search
    pub query: Option<String>,
    pub fuzzy_search: Option<bool>,
    pub search_fields: Option<Vec<String>>, // ["name", "description", "sku", "tags"]

    // Category and Classification
    pub category_ids: Option<Vec<Uuid>>,
    pub product_types: Option<Vec<ProductType>>,
    pub statuses: Option<Vec<ProductStatus>>,
    pub tags: Option<Vec<String>>,

    // Pricing
    pub min_price: Option<i64>,
    pub max_price: Option<i64>,
    pub currency: Option<String>,
    pub price_type: Option<String>,

    // Inventory
    pub in_stock_only: Option<bool>,
    pub needs_reorder: Option<bool>,
    pub location_ids: Option<Vec<Uuid>>,
    pub min_stock: Option<i32>,
    pub max_stock: Option<i32>,

    // Suppliers and Sourcing
    pub supplier_ids: Option<Vec<Uuid>>,
    pub min_lead_time: Option<i32>,
    pub max_lead_time: Option<i32>,

    // Quality and Compliance
    pub quality_statuses: Option<Vec<QualityStatus>>,
    pub compliance_standards: Option<Vec<String>>,
    pub certification_required: Option<bool>,

    // Lifecycle and Performance
    pub lifecycle_stages: Option<Vec<LifecycleStage>>,
    pub min_profit_margin: Option<f64>,
    pub max_profit_margin: Option<f64>,
    pub performance_score_min: Option<f64>,

    // Sustainability
    pub eco_friendly_only: Option<bool>,
    pub max_carbon_footprint: Option<f64>,
    pub recyclable_only: Option<bool>,

    // Physical Properties
    pub min_weight: Option<f64>,
    pub max_weight: Option<f64>,
    pub min_volume: Option<f64>,
    pub max_volume: Option<f64>,

    // Time-based Filters
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub updated_after: Option<DateTime<Utc>>,
    pub updated_before: Option<DateTime<Utc>>,

    // Advanced Features
    pub has_digital_twin: Option<bool>,
    pub has_iot_integration: Option<bool>,
    pub blockchain_verified: Option<bool>,
    pub ai_optimized: Option<bool>,

    // Sorting and Pagination
    pub sort_by: Option<String>,
    pub sort_order: Option<String>, // "asc" or "desc"
    pub include_analytics: Option<bool>,
    pub include_predictions: Option<bool>,
}

/// Carbon footprint tracking for sustainability
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CarbonFootprint {
    pub id: Uuid,
    pub product_id: Uuid,
    pub tenant_id: Uuid,

    // Carbon Emissions (kg CO2 equivalent)
    pub raw_materials_emissions: f64,
    pub manufacturing_emissions: f64,
    pub packaging_emissions: f64,
    pub transportation_emissions: f64,
    pub usage_emissions: f64,
    pub end_of_life_emissions: f64,
    pub total_emissions: f64,

    // Calculation Details
    pub calculation_method: String,
    pub data_sources: Vec<String>,
    pub uncertainty_percentage: Option<f64>,
    pub verification_status: String,
    pub verified_by: Option<String>,
    pub verification_date: Option<DateTime<Utc>>,

    // Reduction Targets
    pub reduction_target_percentage: Option<f64>,
    pub target_deadline: Option<DateTime<Utc>>,
    pub current_reduction_percentage: f64,

    // Offset Information
    pub offset_credits_purchased: f64,
    pub offset_credits_cost: Option<i64>,
    pub offset_projects: Option<Vec<String>>,

    // Metadata
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub updated_by: Uuid,
}

impl CarbonFootprint {
    pub fn new(product_id: Uuid, tenant_id: Uuid, created_by: Uuid) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            product_id,
            tenant_id,
            raw_materials_emissions: 0.0,
            manufacturing_emissions: 0.0,
            packaging_emissions: 0.0,
            transportation_emissions: 0.0,
            usage_emissions: 0.0,
            end_of_life_emissions: 0.0,
            total_emissions: 0.0,
            calculation_method: "default".to_string(),
            data_sources: vec!["manual_entry".to_string()],
            uncertainty_percentage: Some(10.0),
            verification_status: "unverified".to_string(),
            verified_by: None,
            verification_date: None,
            reduction_target_percentage: None,
            target_deadline: None,
            current_reduction_percentage: 0.0,
            offset_credits_purchased: 0.0,
            offset_credits_cost: None,
            offset_projects: None,
            created_at: now,
            updated_at: now,
            created_by,
            updated_by: created_by,
        }
    }

    pub fn calculate_total(&mut self) {
        self.total_emissions = self.raw_materials_emissions
            + self.manufacturing_emissions
            + self.packaging_emissions
            + self.transportation_emissions
            + self.usage_emissions
            + self.end_of_life_emissions;
    }

    pub fn net_emissions(&self) -> f64 {
        self.total_emissions - self.offset_credits_purchased
    }

    pub fn is_carbon_neutral(&self) -> bool {
        self.net_emissions() <= 0.0
    }
}

/// AI-powered product recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductRecommendation {
    pub product_id: Uuid,
    pub recommendation_type: RecommendationType,
    pub confidence_score: f64,
    pub reason: String,
    pub expected_impact: Option<String>,
    pub priority: i32,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationType {
    PriceOptimization,
    InventoryRebalancing,
    QualityImprovement,
    SupplierChange,
    ProductBundling,
    LifecycleTransition,
    MarketExpansion,
    SustainabilityImprovement,
    CrossSelling,
    Discontinuation,
}