use thiserror::Error;

/// Master Data specific errors
#[derive(Error, Debug)]
pub enum MasterDataError {
    #[error("Customer not found: {id}")]
    CustomerNotFound { id: String },

    #[error("Supplier not found: {id}")]
    SupplierNotFound { id: String },

    #[error("Product not found: {id}")]
    ProductNotFound { id: String },

    #[error("Location not found: {id}")]
    LocationNotFound { id: String },

    #[error("Organization unit not found: {id}")]
    OrganizationUnitNotFound { id: String },

    #[error("Entity not found")]
    NotFound,

    #[error("Invalid customer number format: {number}")]
    InvalidCustomerNumber { number: String },

    #[error("Invalid supplier number format: {number}")]
    InvalidSupplierNumber { number: String },

    #[error("Invalid product number format: {number}")]
    InvalidProductNumber { number: String },

    #[error("Duplicate customer number: {number}")]
    DuplicateCustomerNumber { number: String },

    #[error("Duplicate supplier number: {number}")]
    DuplicateSupplierNumber { number: String },

    #[error("Duplicate product number: {number}")]
    DuplicateProductNumber { number: String },

    #[error("Customer has active orders and cannot be deleted")]
    CustomerHasActiveOrders,

    #[error("Supplier has active purchase orders and cannot be deleted")]
    SupplierHasActivePurchaseOrders,

    #[error("Product has active inventory and cannot be deleted")]
    ProductHasActiveInventory,

    #[error("Invalid business relationship: customer cannot be both {existing} and {new}")]
    InvalidBusinessRelationship { existing: String, new: String },

    #[error("Credit limit exceeded: {requested} > {limit}")]
    CreditLimitExceeded { requested: String, limit: String },

    #[error("Validation error: {field}: {message}")]
    ValidationError { field: String, message: String },

    #[error("Data quality issue: {entity_type}: {entity_id}: {issue}")]
    DataQualityIssue {
        entity_type: String,
        entity_id: String,
        issue: String,
    },

    #[error("Synchronization conflict: {entity_type}: {entity_id}: local version {local_version} conflicts with remote version {remote_version}")]
    SynchronizationConflict {
        entity_type: String,
        entity_id: String,
        local_version: i32,
        remote_version: i32,
    },

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Core system error: {0}")]
    Core(#[from] erp_core::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Internal error: {message}")]
    Internal { message: String },

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Not found: {0}")]
    NotFoundError(String),

    #[error("Anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, MasterDataError>;

// Implement conversion to HTTP status codes for web handlers
#[cfg(feature = "axum")]
impl axum::response::IntoResponse for MasterDataError {
    fn into_response(self) -> axum::response::Response {
        use axum::http::StatusCode;
        use axum::Json;
        use serde_json::json;

        let (status, error_message) = match &self {
            MasterDataError::CustomerNotFound { .. }
            | MasterDataError::SupplierNotFound { .. }
            | MasterDataError::ProductNotFound { .. }
            | MasterDataError::LocationNotFound { .. }
            | MasterDataError::OrganizationUnitNotFound { .. }
            | MasterDataError::NotFound => {
                (StatusCode::NOT_FOUND, self.to_string())
            }

            MasterDataError::InvalidCustomerNumber { .. }
            | MasterDataError::InvalidSupplierNumber { .. }
            | MasterDataError::InvalidProductNumber { .. }
            | MasterDataError::ValidationError { .. }
            | MasterDataError::InvalidBusinessRelationship { .. }
            | MasterDataError::CreditLimitExceeded { .. } => {
                (StatusCode::BAD_REQUEST, self.to_string())
            }

            MasterDataError::DuplicateCustomerNumber { .. }
            | MasterDataError::DuplicateSupplierNumber { .. }
            | MasterDataError::DuplicateProductNumber { .. } => {
                (StatusCode::CONFLICT, self.to_string())
            }

            MasterDataError::CustomerHasActiveOrders
            | MasterDataError::SupplierHasActivePurchaseOrders
            | MasterDataError::ProductHasActiveInventory => {
                (StatusCode::CONFLICT, self.to_string())
            }

            MasterDataError::SynchronizationConflict { .. } => {
                (StatusCode::CONFLICT, self.to_string())
            }

            MasterDataError::Database(_) | MasterDataError::Internal { .. } => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }

            MasterDataError::Core(core_err) => {
                // Delegate to core error handling
                return core_err.into_response();
            }

            MasterDataError::Serialization(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Serialization error".to_string())
            }

            MasterDataError::DataQualityIssue { .. } => {
                (StatusCode::UNPROCESSABLE_ENTITY, self.to_string())
            }

            MasterDataError::DatabaseError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
            }


            MasterDataError::NotFoundError(_) => {
                (StatusCode::NOT_FOUND, self.to_string())
            }
        };

        let body = Json(json!({
            "error": {
                "message": error_message,
                "type": "master_data_error"
            }
        }));

        (status, body).into_response()
    }
}