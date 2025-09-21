//! # Advanced Multi-Location Inventory Management System
//!
//! Comprehensive inventory management with support for multiple locations,
//! real-time tracking, demand forecasting, optimization algorithms, and
//! intelligent replenishment strategies.

pub mod model;
pub mod repository;
pub mod service;
pub mod analytics;
pub mod optimization;

#[cfg(feature = "axum")]
pub mod handlers;

// Specific exports to avoid conflicts with product module
pub use model::{
    LocationInventory, InventoryMovement, StockTransfer, ReplenishmentRule,
    InventorySnapshot, LocationType, MovementType, TransferStatus, TransferPriority,
    ABCClassification, MovementVelocity, StorageRequirements,
    ForecastMethod, ForecastAccuracy, UpdateInventoryRequest,
    InventoryOptimization, OptimizationAction, CycleCount, CountStatus,
    StockAgingItem, AgingCategory, InventoryForecast,
    // Add other inventory-specific types
    InventoryReservation, ReservationStatus, ReservationPriority,
    PurchaseOrder, PurchaseOrderLine, OrderStatus,
    InventoryAlert, AlertType, AlertSeverity,
    InventoryValuation, InventoryKPI, InventoryDashboard,
    ReplenishmentSuggestion, InventorySearchCriteria, StockStatusFilter,
    InventoryAnalysisRequest, AnalysisType,
};

pub use repository::{
    InventoryRepository, PostgresInventoryRepository,
    TurnoverAnalysisItem, TurnoverClassification,
};

pub use service::{
    InventoryService, DefaultInventoryService,
    CreateStockTransferRequest, CreateReservationRequest,
    UpdateReplenishmentRuleRequest,
};

pub use analytics::{
    InventoryAnalyticsEngine, DefaultInventoryAnalyticsEngine,
    InventoryAnalyticsMetrics, ABCXYZAnalysis,
    // Other analytics specific to inventory
};

pub use optimization::{
    InventoryOptimizationEngine, PostgresInventoryOptimizationEngine,
    OptimizationResult, DemandForecast, SupplyChainOptimization,
    OptimizationParameters, InventoryOptimizationReport,
    // Other optimization types
};