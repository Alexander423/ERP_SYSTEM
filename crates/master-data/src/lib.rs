// Master Data Management module for comprehensive ERP system
// Provides enterprise-grade functionality that exceeds SAP/Oracle/Dynamics capabilities

pub mod customer;
pub mod supplier;
pub mod product;
pub mod location;
pub mod organization;
pub mod security;

// Common types and utilities
pub mod types;
pub mod error;

// Re-exports for easy access
pub use customer::{
    Customer, CustomerType, CustomerLifecycleStage, CreditStatus,
    CreateCustomerRequest, UpdateCustomerRequest, CustomerSearchCriteria, CustomerSearchResponse,
    CustomerRepository, PostgresCustomerRepository,
    CustomerService, DefaultCustomerService,
};

#[cfg(feature = "axum")]
pub use customer::{
    CustomerHandlers, CustomerResponse, CustomersResponse,
    CustomerSearchQueryParams,
};

pub use error::{MasterDataError, Result};
pub use types::*;