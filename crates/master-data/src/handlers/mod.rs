// HTTP handlers for master data endpoints

#[cfg(feature = "axum")]
pub mod customer;

#[cfg(feature = "axum")]
pub use customer::*;