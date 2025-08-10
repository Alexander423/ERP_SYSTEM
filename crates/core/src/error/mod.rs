//! # Error Handling Framework
//! 
//! This module provides a comprehensive error handling system for the ERP application.
//! It implements structured error management with:
//! 
//! ## Core Features
//! 
//! - **Structured Error Types**: Categorized error codes for consistent handling
//! - **Rich Context**: Request context and error metadata for debugging
//! - **Severity Classification**: Error categorization for appropriate responses
//! - **Metrics Integration**: Error tracking and monitoring capabilities
//! - **User-Friendly Messages**: Localized error messages for end users
//! 
//! ## Error Categories
//! 
//! - **Validation**: Input validation and business rule violations
//! - **Authentication**: Identity verification and authorization failures
//! - **Database**: Persistence layer errors and constraint violations
//! - **Network**: External service communication failures
//! - **Configuration**: System setup and configuration issues
//! - **Internal**: Unexpected system errors requiring investigation
//! 
//! ## Context Tracking
//! 
//! - **Request Context**: HTTP request tracking with correlation IDs
//! - **Error Context**: Structured error metadata and stack traces
//! - **Tenant Context**: Multi-tenant error isolation and tracking
//! - **User Context**: User-specific error handling and permissions
//! 
//! ## Monitoring Integration
//! 
//! - **Error Metrics**: Prometheus metrics for error rates and patterns
//! - **Structured Logging**: Consistent error logging with correlation
//! - **Alert Integration**: Critical error notification and escalation
//! - **Dashboards**: Error tracking and trend analysis
//! 
//! ## Usage Examples
//! 
//! ```rust
//! use erp_core::error::{Error, ErrorCode, Result};
//! 
//! // Creating structured errors
//! fn validate_email(email: &str) -> Result<()> {
//!     if !email.contains('@') {
//!         return Err(Error::validation("Invalid email format")
//!             .with_code(ErrorCode::InvalidEmailFormat)
//!             .with_field("email"));
//!     }
//!     Ok(())
//! }
//! 
//! // Error handling with context
//! async fn process_request() -> Result<Response> {
//!     validate_input()
//!         .map_err(|e| e.with_context("Request validation failed"))?;
//!     
//!     database_operation()
//!         .await
//!         .map_err(|e| e.with_context("Database operation failed"))?;
//!     
//!     Ok(success_response())
//! }
//! ```

pub mod codes;
pub mod context;
pub mod framework;
pub mod metrics;

pub use codes::ErrorCode;
pub use context::{ErrorContext, RequestContext};
pub use framework::{Error, ErrorCategory, ErrorSeverity, Result};
pub use metrics::ErrorMetrics;