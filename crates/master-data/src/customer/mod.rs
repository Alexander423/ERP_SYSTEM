pub mod model;
pub mod repository;
pub mod service;
pub mod search;
pub mod validation;
pub mod analytics;
pub mod analytics_engine;
pub mod events;
pub mod event_store;
pub mod aggregate;

#[cfg(feature = "axum")]
pub mod handlers;

#[cfg(test)]
pub mod tests;

// Re-exports for public API
pub use model::{
    Customer, CustomerType, CustomerLifecycleStage, CreditStatus,
    CreateCustomerRequest, UpdateCustomerRequest, CustomerSearchCriteria, CustomerSearchResponse,
    CustomerPerformanceMetrics, CustomerBehavioralData,
    TaxJurisdiction, RegulatoryClassification, CustomerSegment,
    AcquisitionChannel, ComplianceStatus, KycStatus,
};

pub use repository::{CustomerRepository, PostgresCustomerRepository};
pub use service::{CustomerService, DefaultCustomerService};
pub use events::{CustomerEvent, CustomerEventWithMetadata, EventMetadata};
pub use event_store::{CustomerEventStore, PostgresCustomerEventStore, EventStatistics};
pub use aggregate::CustomerAggregate;
pub use analytics_engine::{CustomerAnalyticsEngine, InMemoryAnalyticsEngine, CustomerInsights};
pub use search::{CustomerSearchEngine, AdvancedSearchEngine, SearchOptions, SearchResults, AdvancedSearchFilters};
pub use validation::CustomerValidator;

#[cfg(feature = "axum")]
pub use handlers::{
    CustomerHandlers, CustomerResponse, CustomersResponse,
    UpdateLifecycleStageRequest, ValidateCreditLimitRequest,
    CreditLimitValidationResponse, CustomerNumberResponse, PerformanceMetricsResponse,
    CustomerSearchQueryParams,
};