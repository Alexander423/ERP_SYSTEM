//! # Inventory Data Models
//!
//! Core data structures for multi-location inventory management with
//! advanced features for optimization, forecasting, and real-time tracking.

use chrono::{DateTime, Utc, NaiveDate};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use std::collections::HashMap;
use crate::types::{ValuationMethod, ReservationType};
use rust_decimal::Decimal;

use serde_json::Value;

// These types are defined directly in this inventory module
// (removed dependency on product module)

// Multi-location inventory types
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LocationInventory {
    pub id: Uuid,
    pub product_id: Uuid,
    pub location_id: Uuid,
    pub location_name: String,
    pub location_type: LocationType,
    pub quantity_available: i32,
    pub quantity_reserved: i32,
    pub quantity_on_order: i32,
    pub quantity_in_transit: i32,
    pub reorder_point: i32,
    pub max_stock_level: i32,
    pub min_stock_level: i32,
    pub safety_stock: i32,
    pub economic_order_quantity: i32,
    pub lead_time_days: i32,
    pub storage_cost_per_unit: f64,
    pub handling_cost_per_unit: f64,
    pub last_counted_at: Option<DateTime<Utc>>,
    pub cycle_count_frequency_days: Option<i32>,
    pub abc_classification: ABCClassification,
    pub movement_velocity: MovementVelocity,
    pub seasonal_factors: HashMap<String, f64>,
    pub storage_requirements: StorageRequirements,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "location_type", rename_all = "snake_case")]
pub enum LocationType {
    Warehouse,
    Store,
    DistributionCenter,
    ManufacturingPlant,
    Supplier,
    Customer,
    Transit,
    Virtual,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "abc_classification", rename_all = "snake_case")]
pub enum ABCClassification {
    A, // High value, high frequency
    B, // Medium value, medium frequency
    C, // Low value, low frequency
    X, // Special handling required
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "movement_velocity", rename_all = "snake_case")]
pub enum MovementVelocity {
    Fast,     // High turnover
    Medium,   // Medium turnover
    Slow,     // Low turnover
    Dead,     // No movement
    Seasonal, // Seasonal patterns
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StorageRequirements {
    pub temperature_min: Option<f64>,
    pub temperature_max: Option<f64>,
    pub humidity_min: Option<f64>,
    pub humidity_max: Option<f64>,
    pub requires_refrigeration: bool,
    pub requires_freezing: bool,
    pub hazardous_material: bool,
    pub fragile: bool,
    pub stackable: bool,
    pub special_handling_instructions: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InventoryMovement {
    pub id: Option<Uuid>,
    pub product_id: Option<Uuid>,
    pub location_id: Option<Uuid>,
    pub movement_type: Option<String>,
    pub quantity: Option<i32>,
    pub unit_cost: Option<Decimal>,
    pub reference_document: Option<String>,
    pub reference_number: Option<String>,
    pub reason: Option<String>,
    pub batch_number: Option<String>,
    pub serial_numbers: Option<Vec<String>>,
    pub expiry_date: Option<NaiveDate>,
    pub operator_id: Option<Uuid>,
    pub operator_name: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub effective_date: Option<DateTime<Utc>>,
    pub audit_trail: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "movement_type", rename_all = "snake_case")]
pub enum MovementType {
    Receipt,
    Shipment,
    Transfer,
    Adjustment,
    Return,
    Damage,
    Loss,
    Found,
    Production,
    Consumption,
    CycleCount,
    PhysicalCount,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InventoryForecast {
    pub id: Uuid,
    pub product_id: Uuid,
    pub location_id: Uuid,
    pub forecast_date: DateTime<Utc>,
    pub forecast_horizon_days: i32,
    pub predicted_demand: f64,
    pub predicted_supply: f64,
    pub predicted_stock_level: f64,
    pub confidence_level: f64,
    pub confidence_lower: f64,
    pub confidence_upper: f64,
    pub forecast_method: ForecastMethod,
    pub seasonal_index: f64,
    pub seasonal_component: f64,
    pub trend_factor: f64,
    pub trend_component: f64,
    pub external_factors: HashMap<String, f64>,
    pub accuracy_metrics: ForecastAccuracy,
    pub accuracy_score: f64,
    pub created_at: DateTime<Utc>,
    pub model_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "forecast_method", rename_all = "snake_case")]
pub enum ForecastMethod {
    MovingAverage,
    ExponentialSmoothing,
    LinearRegression,
    SeasonalDecomposition,
    Arima,
    MachineLearning,
    HybridModel,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ForecastAccuracy {
    pub mean_absolute_error: f64,
    pub mean_squared_error: f64,
    pub mean_absolute_percentage_error: f64,
    pub forecast_bias: f64,
    pub tracking_signal: f64,
    pub accuracy_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StockTransfer {
    pub id: Uuid,
    pub product_id: Uuid,
    pub from_location_id: Uuid,
    pub to_location_id: Uuid,
    pub quantity: i32,
    pub quantity_shipped: Option<i32>,
    pub quantity_received: Option<i32>,
    pub status: TransferStatus,
    pub priority: TransferPriority,
    pub reason: String,
    pub requested_by: Uuid,
    pub approved_by: Option<Uuid>,
    pub shipped_by: Option<Uuid>,
    pub received_by: Option<Uuid>,
    pub requested_date: DateTime<Utc>,
    pub approved_date: Option<DateTime<Utc>>,
    pub shipped_date: Option<DateTime<Utc>>,
    pub received_date: Option<DateTime<Utc>>,
    pub actual_delivery_date: Option<DateTime<Utc>>,
    pub tracking_number: Option<String>,
    pub carrier: Option<String>,
    pub shipping_cost: Option<f64>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "transfer_status", rename_all = "snake_case")]
pub enum TransferStatus {
    Requested,
    Approved,
    Rejected,
    InTransit,
    PartiallyReceived,
    Completed,
    Cancelled,
    Pending,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "transfer_priority", rename_all = "snake_case")]
pub enum TransferPriority {
    Low,
    Normal,
    High,
    Urgent,
    Emergency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInventoryRequest {
    pub location_id: Uuid,
    pub quantity_change: i32,
    pub movement_type: MovementType,
    pub reason: Option<String>,
    pub reference_document: Option<String>,
    pub batch_number: Option<String>,
    pub unit_cost: Option<f64>,
    pub effective_date: Option<DateTime<Utc>>,
    pub operator_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryOptimization {
    pub product_id: Uuid,
    pub location_id: Uuid,
    pub current_stock_level: i32,
    pub optimal_stock_level: i32,
    pub recommended_action: OptimizationAction,
    pub potential_savings: f64,
    pub service_level_impact: f64,
    pub implementation_priority: i32,
    pub rationale: String,
    pub expected_roi: f64,
    pub implementation_timeline: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "optimization_action", rename_all = "snake_case")]
pub enum OptimizationAction {
    Increase,
    Decrease,
    Maintain,
    Discontinue,
    Transfer,
    Liquidate,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CycleCount {
    pub id: Uuid,
    pub location_id: Uuid,
    pub count_date: NaiveDate,
    pub status: CountStatus,
    pub total_items: i32,
    pub counted_items: i32,
    pub variance_items: i32,
    pub variance: i32,
    pub adjustment_required: bool,
    pub adjustment_date: Option<DateTime<Utc>>,
    pub adjustment_by: Option<Uuid>,
    pub approval_required: bool,
    pub approved_by: Option<Uuid>,
    pub approved_date: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,

    // Additional fields needed by service layer
    pub counter_name: String,
    pub book_quantity: i32,
    pub counted_quantity: i32,
    pub variance_percentage: f64,
    pub variance_value: f64,
    pub count_status: CountStatus,
    pub adjustment_applied: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "count_status", rename_all = "snake_case")]
pub enum CountStatus {
    Scheduled,
    InProgress,
    Completed,
    Reviewed,
    Approved,
    Rejected,
    Adjusted,
}

// Additional missing types for inventory management
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InventoryReservation {
    pub id: Uuid,
    pub product_id: Uuid,
    pub location_id: Uuid,
    pub quantity_reserved: i32,
    pub reservation_status: ReservationStatus,
    pub priority: ReservationPriority,
    pub reference_id: Uuid,
    pub reference_type: String,
    pub expiry_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub notes: Option<String>,

    // Additional audit fields needed by the service layer
    pub created_by: Uuid,
    pub released_at: Option<DateTime<Utc>>,
    pub released_by: Option<Uuid>,

    // Additional fields expected by repository layer
    pub quantity: i32,  // Alias for quantity_reserved for repository compatibility
    pub reservation_type: String,  // Additional type field expected by repository
    pub status: ReservationStatus,  // Alias for reservation_status for repository compatibility
    pub reserved_until: Option<DateTime<Utc>>,  // Alias for expiry_date for repository compatibility
    pub fulfilled_at: Option<DateTime<Utc>>,
    pub fulfilled_quantity: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "reservation_status", rename_all = "snake_case")]
pub enum ReservationStatus {
    Active,
    Fulfilled,
    Expired,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "reservation_priority", rename_all = "snake_case")]
pub enum ReservationPriority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "alert_status", rename_all = "snake_case")]
pub enum AlertStatus {
    Active,
    Acknowledged,
    Resolved,
    Dismissed,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PurchaseOrder {
    pub id: Uuid,
    pub order_number: String,
    pub supplier_id: Uuid,
    pub supplier_name: String,
    pub location_id: Uuid,
    pub status: OrderStatus,
    pub order_date: DateTime<Utc>,
    pub expected_delivery_date: Option<DateTime<Utc>>,
    pub actual_delivery_date: Option<DateTime<Utc>>,
    pub total_amount: f64,
    pub currency: String,
    pub payment_terms: Option<String>,
    pub shipping_terms: Option<String>,
    pub priority: Option<String>,
    pub approved_by: Option<Uuid>,
    pub tracking_number: Option<String>,
    pub notes: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // Additional fields expected by repository layer
    pub shipping_address: Option<String>,
    pub billing_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PurchaseOrderLine {
    pub id: Uuid,
    pub purchase_order_id: Uuid,
    pub product_id: Uuid,
    pub quantity_ordered: i32,
    pub quantity_received: i32,
    pub unit_price: f64,
    pub line_total: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "order_status", rename_all = "snake_case")]
pub enum OrderStatus {
    Draft,
    Submitted,
    Approved,
    Ordered,
    PartiallyReceived,
    Received,
    Cancelled,
    Pending,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InventoryAlert {
    pub id: Uuid,
    pub product_id: Uuid,
    pub location_id: Uuid,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub title: String,
    pub description: Option<String>,
    pub current_quantity: i32,
    pub threshold_value: Decimal,
    pub recommended_action: Option<String>,
    pub alert_status: AlertStatus,
    pub created_at: DateTime<Utc>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub acknowledged_by: Option<Uuid>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolved_by: Option<Uuid>,
    pub resolution_notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "alert_type", rename_all = "snake_case")]
pub enum AlertType {
    LowStock,
    OverStock,
    StockOut,
    ExpiryWarning,
    QualityIssue,
    ReorderPoint,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq, Hash)]
#[sqlx(type_name = "alert_severity", rename_all = "snake_case")]
pub enum AlertSeverity {
    Info,
    Low,
    Medium,
    High,
    Warning,
    Critical,
    Emergency,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InventoryValuation {
    pub id: Uuid,
    pub product_id: Uuid,
    pub location_id: Uuid,
    pub valuation_date: DateTime<Utc>,
    pub valuation_method: ValuationMethod,
    pub quantity: i32,
    pub unit_cost: f64,
    pub total_value: f64,
    pub average_cost: f64,
    pub fifo_cost: f64,
    pub lifo_cost: f64,
    pub standard_cost: f64,
    pub market_value: f64,
    pub replacement_cost: f64,
    pub net_realizable_value: f64,
    pub obsolescence_reserve: f64,
    pub shrinkage_reserve: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryKPI {
    pub location_id: Option<Uuid>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_value: f64,
    pub turnover_ratio: f64,
    pub stockout_rate: f64,
    pub fill_rate: f64,
    pub carrying_cost: f64,
    pub accuracy_percentage: f64,

    // Additional fields needed by repository layer
    pub id: Uuid,
    pub inventory_turnover: f64,
    pub inventory_turnover_days: f64,
    pub carrying_cost_rate: f64,
    pub gross_margin_rate: f64,
    pub inventory_accuracy: f64,
    pub obsolete_inventory_rate: f64,
    pub dead_stock_rate: f64,
    pub average_inventory_level: f64,
    pub total_inventory_value: f64,
    pub calculated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryDashboard {
    pub location_id: Option<Uuid>,
    pub total_products: i32,
    pub low_stock_alerts: i32,
    pub stockout_alerts: i32,
    pub pending_transfers: i32,
    pub total_inventory_value: f64,
    pub top_moving_products: Vec<String>,
    pub recent_alerts: Vec<InventoryAlert>,

    // Additional fields needed by repository layer
    pub id: Uuid,
    pub snapshot_date: DateTime<Utc>,
    pub total_sku_count: i32,
    pub stockout_count: i32,
    pub low_stock_count: i32,
    pub excess_stock_count: i32,
    pub slow_moving_count: i32,
    pub inventory_turnover: f64,
    pub fill_rate: f64,
    pub carrying_cost_percentage: f64,
    pub abc_analysis: HashMap<String, i32>,
    pub top_movers: Vec<String>,
    pub pending_orders: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InventorySnapshot {
    pub id: Uuid,
    pub product_id: Uuid,
    pub location_id: Uuid,
    pub snapshot_date: DateTime<Utc>,
    pub quantity_on_hand: i32,
    pub quantity_available: i32,
    pub quantity_reserved: i32,
    pub quantity_on_order: i32,
    pub quantity_in_transit: i32,
    pub unit_cost: f64,
    pub total_value: f64,
    pub turnover_rate: f64,
    pub days_on_hand: f64,
    pub stockout_risk: f64,
    pub excess_risk: f64,
    pub service_level: f64,
    pub fill_rate: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReplenishmentRule {
    pub id: Uuid,
    pub product_id: Uuid,
    pub location_id: Uuid,
    pub rule_type: ReplenishmentType,
    pub reorder_point: i32,
    pub reorder_quantity: i32,
    pub max_stock_level: i32,
    pub min_stock_level: i32,
    pub safety_stock: i32,
    pub lead_time_days: i32,
    pub review_period_days: i32,
    pub service_level_target: f64,
    pub cost_per_order: f64,
    pub carrying_cost_rate: f64,
    pub automatic_ordering: bool,
    pub supplier_id: Option<Uuid>,
    pub preferred_vendor: Option<String>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // Additional fields expected by repository layer
    pub economic_order_quantity: f64,
    pub preferred_supplier_id: Option<Uuid>,
    pub is_active: bool,  // Alias for active for repository compatibility
    pub last_triggered: Option<DateTime<Utc>>,
    pub created_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "replenishment_type", rename_all = "snake_case")]
pub enum ReplenishmentType {
    ReorderPoint,       // (s, Q) policy
    PeriodicReview,     // (R, S) policy
    MinMax,             // Min-Max policy
    EconomicOrderQuantity, // EOQ-based
    JustInTime,         // JIT replenishment
    DemandDriven,       // DDMRP
    KanbanSystem,       // Kanban-based
    ManualOnly,         // Manual ordering only
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryTrendPoint {
    pub date: DateTime<Utc>,
    pub total_value: f64,
    pub quantity_on_hand: i32,
    pub turnover_rate: f64,
    pub service_level: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopMoverItem {
    pub product_id: Uuid,
    pub product_name: String,
    pub sku: String,
    pub movement_quantity: i32,
    pub turnover_rate: f64,
    pub revenue_impact: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BottomPerformerItem {
    pub product_id: Uuid,
    pub product_name: String,
    pub sku: String,
    pub days_without_movement: i32,
    pub quantity_on_hand: i32,
    pub tied_up_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentMovement {
    pub movement_id: Uuid,
    pub product_name: String,
    pub location_name: String,
    pub movement_type: MovementType,
    pub quantity: i32,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingOrder {
    pub order_id: Uuid,
    pub order_number: String,
    pub supplier_name: String,
    pub total_amount: f64,
    pub expected_delivery: DateTime<Utc>,
    pub status: OrderStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpcomingDelivery {
    pub delivery_id: Uuid,
    pub order_number: String,
    pub supplier_name: String,
    pub product_count: i32,
    pub expected_delivery: DateTime<Utc>,
    pub tracking_number: Option<String>,
}

// Request/Response Types for API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReplenishmentRuleRequest {
    pub product_id: Uuid,
    pub location_id: Uuid,
    pub rule_type: ReplenishmentType,
    pub reorder_point: i32,
    pub reorder_quantity: i32,
    pub max_stock_level: i32,
    pub min_stock_level: i32,
    pub safety_stock: i32,
    pub lead_time_days: i32,
    pub service_level_target: f64,
    pub automatic_ordering: bool,
    pub supplier_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateReplenishmentRuleRequest {
    pub reorder_point: Option<i32>,
    pub reorder_quantity: Option<i32>,
    pub max_stock_level: Option<i32>,
    pub min_stock_level: Option<i32>,
    pub safety_stock: Option<i32>,
    pub lead_time_days: Option<i32>,
    pub service_level_target: Option<f64>,
    pub automatic_ordering: Option<bool>,
    pub supplier_id: Option<Uuid>,
    pub active: Option<bool>,

    // Additional fields needed by repository layer
    pub product_id: Option<Uuid>,
    pub location_id: Option<Uuid>,
    pub economic_order_quantity: Option<f64>,
    pub preferred_supplier_id: Option<Uuid>,
    pub is_active: Option<bool>,  // Alias for active for repository compatibility
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateStockTransferRequest {
    pub product_id: Uuid,
    pub from_location_id: Uuid,
    pub to_location_id: Uuid,
    pub quantity_requested: i32,
    pub priority: TransferPriority,
    pub reason: String,
    pub expected_delivery_date: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReservationRequest {
    pub product_id: Uuid,
    pub location_id: Uuid,
    pub reservation_type: ReservationType,
    pub quantity_reserved: i32,
    pub reserved_for: String,
    pub reference_id: Uuid,
    pub reference_type: String,
    pub expiry_date: Option<DateTime<Utc>>,
    pub priority: ReservationPriority,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleCountRequest {
    pub product_id: Uuid,
    pub location_id: Uuid,
    pub counted_quantity: i32,
    pub counter_id: Uuid,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryAdjustmentRequest {
    pub product_id: Uuid,
    pub location_id: Uuid,
    pub adjustment_quantity: i32,
    pub reason: String,
    pub reference_document: Option<String>,
    pub cost_adjustment: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplenishmentSuggestion {
    pub product_id: Uuid,
    pub product_name: String,
    pub location_id: Uuid,
    pub location_name: String,
    pub current_stock: i32,
    pub suggested_order_quantity: i32,
    pub reorder_point: i32,
    pub lead_time_days: i32,
    pub supplier_id: Option<Uuid>,
    pub supplier_name: Option<String>,
    pub estimated_cost: f64,
    pub urgency_score: f64,
    pub stockout_risk: f64,
    pub expected_delivery_date: DateTime<Utc>,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventorySearchCriteria {
    pub product_ids: Option<Vec<Uuid>>,
    pub location_ids: Option<Vec<Uuid>>,
    pub abc_classification: Option<ABCClassification>,
    pub movement_velocity: Option<MovementVelocity>,
    pub stock_status: Option<StockStatusFilter>,
    pub value_range: Option<(f64, f64)>,
    pub quantity_range: Option<(i32, i32)>,
    pub turnover_range: Option<(f64, f64)>,
    pub include_inactive: Option<bool>,
    pub alert_types: Option<Vec<AlertType>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StockStatusFilter {
    InStock,
    LowStock,
    OutOfStock,
    ExcessStock,
    DeadStock,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryAnalysisRequest {
    pub analysis_type: AnalysisType,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub location_ids: Option<Vec<Uuid>>,
    pub product_ids: Option<Vec<Uuid>>,
    pub category_ids: Option<Vec<Uuid>>,
    pub include_forecasts: bool,
    pub include_optimization: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisType {
    TurnoverAnalysis,
    ABCAnalysis,
    SlowMovingAnalysis,
    ExcessStockAnalysis,
    ServiceLevelAnalysis,
    CarryingCostAnalysis,
    VarianceAnalysis,
    SeasonalityAnalysis,
    ComprehensiveAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StockAgingItem {
    pub product_id: Uuid,
    pub product_name: String,
    pub location_id: Uuid,
    pub location_name: String,
    pub current_stock: i32,
    pub unit_cost: f64,
    pub total_value: f64,
    pub last_movement_date: Option<DateTime<Utc>>,
    pub days_since_last_movement: Option<i32>,
    pub aging_category: AgingCategory,
    pub turnover_rate: Option<f64>,
    pub suggested_action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "aging_category", rename_all = "snake_case")]
pub enum AgingCategory {
    Current,      // 0-30 days
    Slow,         // 31-90 days
    Dead,         // 90+ days
    VeryDead,     // 180+ days
}