// Master Data Management module for comprehensive ERP system
// Provides enterprise-grade functionality that exceeds SAP/Oracle/Dynamics capabilities

pub mod customer;
pub mod supplier;
pub mod product;
pub mod inventory;
pub mod location;
pub mod organization;
pub mod security;

// Common types and utilities
pub mod types;
pub mod error;
pub mod utils;

// Re-exports for easy access
pub use customer::{
    Customer, CustomerType, CustomerLifecycleStage, CreditStatus,
    CreateCustomerRequest, UpdateCustomerRequest, CustomerSearchCriteria, CustomerSearchResponse,
    CustomerRepository, PostgresCustomerRepository,
    CustomerService, DefaultCustomerService,
};

pub use inventory::{
    LocationInventory, InventoryMovement, StockTransfer, ReplenishmentRule,
    InventoryAnalyticsMetrics, ABCXYZAnalysis,
    InventoryService, DefaultInventoryService,
    InventoryAnalyticsEngine, DefaultInventoryAnalyticsEngine,
    InventoryOptimizationEngine, PostgresInventoryOptimizationEngine,
    OptimizationResult, SupplyChainOptimization,
};

pub use product::{
    Product, ProductType, ProductStatus, UnitOfMeasure,
    ProductCategory, ProductPrice, ProductVariant, ProductSupplier,
    ProductSummary, CreateProductRequest, UpdateProductRequest, ProductSearchFilters,
};

#[cfg(feature = "axum")]
pub use customer::{
    CustomerHandlers, CustomerResponse, CustomersResponse,
    CustomerSearchQueryParams,
};

pub use error::{MasterDataError, Result};
pub use types::*;
pub use utils::*;