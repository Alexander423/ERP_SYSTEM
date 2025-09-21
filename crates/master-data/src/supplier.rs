//! # Supplier Management Module
//!
//! This module provides comprehensive supplier management functionality for the ERP system.
//! It includes supplier registration, contact management, performance tracking, and
//! integration with procurement workflows.
//!
//! ## Features
//!
//! - **Supplier Registration**: Complete supplier onboarding with validation
//! - **Contact Management**: Multiple contacts per supplier with roles
//! - **Address Management**: Multiple addresses (billing, shipping, etc.)
//! - **Performance Tracking**: Supplier ratings and KPIs
//! - **Payment Terms**: Flexible payment configurations
//! - **Category Management**: Supplier categorization and classification

pub mod model;
pub mod repository;
pub mod service;
pub mod analytics;

#[cfg(feature = "axum")]
pub mod handlers;

pub use model::*;
pub use repository::*;
pub use service::*;
pub use analytics::*;