//! # Product Management Module
//!
//! This module provides comprehensive product information management for the ERP system.
//! It includes product catalog management, categories, pricing, inventory tracking,
//! and supplier relationships.
//!
//! ## Features
//!
//! - **Product Catalog**: Complete product information management
//! - **Category Management**: Hierarchical product categorization
//! - **Pricing Management**: Multiple price levels and currency support
//! - **Inventory Integration**: Stock levels and movement tracking
//! - **Supplier Relations**: Product-supplier mappings and sourcing
//! - **Variant Support**: Product variations and configurations

pub mod model;
pub mod repository;
pub mod service;
pub mod analytics;

#[cfg(feature = "axum")]
pub mod handlers;

// Specific exports to avoid conflicts
pub use model::{
    Product, ProductType, ProductStatus, UnitOfMeasure,
    ProductCategory, ProductPrice, ProductVariant, ProductSupplier,
    CreateProductRequest, UpdateProductRequest, ProductSearchFilters,
    ProductSummary, CarbonFootprint, ProductInventory, ProductAttributes,
    ProductBatch, QualityStatus, ProductLifecycle, LifecycleStage, DynamicPrice,
    ProductAnalytics, AdvancedProductSearch, BulkPriceUpdateRequest,
    PriceUpdateType, PriceField, StockAdjustmentRequest, StockAdjustmentType,
};

pub use repository::{
    ProductRepository, PostgresProductRepository,
    // Avoid conflicts - don't export pagination types here
};

pub use service::{
    ProductService, DefaultProductService,
    // Add specific service types
    ProductAnalyticsReport, CategoryOptimizationSuggestion,
    ReorderRecommendation, StockOptimization,
};

pub use analytics::{
    ProductAnalyticsEngine, DefaultProductAnalyticsEngine,
    ProductPerformanceMetrics, MarketIntelligence,
    ProductRecommendation, RecommendationType,
    // Add other analytics types but avoid conflicts
};