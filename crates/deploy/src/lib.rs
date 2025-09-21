//! ERP System Deployment CLI Library
//!
//! This library provides the core functionality for the ERP deployment CLI tool.

pub mod commands;
pub mod config;
pub mod database;
pub mod docker;
pub mod tenant;
pub mod utils;

// Re-export commonly used types
pub use config::Config;

// Error types
pub type Result<T> = anyhow::Result<T>;