//! HTTP handlers for supplier endpoints
//!
//! This module provides REST API endpoints for supplier management,
//! including CRUD operations, search, analytics, and reporting.

use super::{model::*, service::SupplierService, analytics::SupplierAnalytics};
use crate::common::repository::PaginationOptions;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{delete, get, post, put},
    Router,
};
use erp_core::{
    error::{Error, ErrorCode},
    tenant::TenantContext,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

/// Query parameters for supplier search
#[derive(Debug, Deserialize)]
pub struct SupplierSearchQuery {
    pub q: Option<String>,
    pub status: Option<SupplierStatus>,
    pub category: Option<SupplierCategory>,
    pub min_rating: Option<f64>,
    pub max_rating: Option<f64>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

/// Response wrapper for API responses
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
    pub errors: Option<Vec<String>>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
            errors: None,
        }
    }

    pub fn error(message: String) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            message: Some(message),
            errors: None,
        }
    }
}

/// Create supplier routes
pub fn supplier_routes<S, A>() -> Router<(S, A)>
where
    S: SupplierService + Clone + 'static,
    A: SupplierAnalytics + Clone + 'static,
{
    Router::new()
        .route("/", post(create_supplier::<S, A>))
        .route("/", get(list_suppliers::<S, A>))
        .route("/search", get(search_suppliers::<S, A>))
        .route("/:id", get(get_supplier::<S, A>))
        .route("/:id", put(update_supplier::<S, A>))
        .route("/:id", delete(delete_supplier::<S, A>))
        .route("/:id/activate", post(activate_supplier::<S, A>))
        .route("/:id/deactivate", post(deactivate_supplier::<S, A>))
        .route("/:id/contacts", get(get_supplier_contacts::<S, A>))
        .route("/:id/contacts", post(add_supplier_contact::<S, A>))
        .route("/:id/addresses", get(get_supplier_addresses::<S, A>))
        .route("/:id/addresses", post(add_supplier_address::<S, A>))
        .route("/:id/performance", get(get_supplier_performance::<S, A>))
        .route("/:id/performance", post(record_supplier_performance::<S, A>))
        .route("/analytics/dashboard", get(get_supplier_dashboard::<S, A>))
        .route("/analytics/top", get(get_top_suppliers::<S, A>))
        .route("/analytics/attention", get(get_suppliers_requiring_attention::<S, A>))
        .route("/analytics/categories", get(get_suppliers_by_category::<S, A>))
}

/// Create a new supplier
async fn create_supplier<S: SupplierService, A: SupplierAnalytics>(
    State((service, _analytics)): State<(S, A)>,
    Json(request): Json<CreateSupplierRequest>,
) -> impl IntoResponse {
    match service.create_supplier(request).await {
        Ok(supplier) => (
            StatusCode::CREATED,
            Json(ApiResponse::success(supplier)),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(e.to_string())),
        ),
    }
}

/// Get a supplier by ID
async fn get_supplier<S: SupplierService, A: SupplierAnalytics>(
    State((service, _analytics)): State<(S, A)>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match service.get_supplier(id).await {
        Ok(Some(supplier)) => (
            StatusCode::OK,
            Json(ApiResponse::success(supplier)),
        ),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("Supplier not found".to_string())),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(e.to_string())),
        ),
    }
}

/// Update a supplier
async fn update_supplier<S: SupplierService, A: SupplierAnalytics>(
    State((service, _analytics)): State<(S, A)>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateSupplierRequest>,
) -> impl IntoResponse {
    match service.update_supplier(id, request).await {
        Ok(supplier) => (
            StatusCode::OK,
            Json(ApiResponse::success(supplier)),
        ),
        Err(e) => {
            let status = match e.code {
                ErrorCode::NotFound => StatusCode::NOT_FOUND,
                ErrorCode::ValidationFailed => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// Delete a supplier
async fn delete_supplier<S: SupplierService, A: SupplierAnalytics>(
    State((service, _analytics)): State<(S, A)>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match service.delete_supplier(id).await {
        Ok(()) => (
            StatusCode::NO_CONTENT,
            Json(json!({"success": true, "message": "Supplier deleted successfully"})),
        ),
        Err(e) => {
            let status = match e.code {
                ErrorCode::NotFound => StatusCode::NOT_FOUND,
                ErrorCode::BusinessRuleViolation => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// Activate a supplier
async fn activate_supplier<S: SupplierService, A: SupplierAnalytics>(
    State((service, _analytics)): State<(S, A)>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match service.activate_supplier(id).await {
        Ok(supplier) => (
            StatusCode::OK,
            Json(ApiResponse::success(supplier)),
        ),
        Err(e) => {
            let status = match e.code {
                ErrorCode::NotFound => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// Deactivate a supplier
async fn deactivate_supplier<S: SupplierService, A: SupplierAnalytics>(
    State((service, _analytics)): State<(S, A)>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match service.deactivate_supplier(id).await {
        Ok(supplier) => (
            StatusCode::OK,
            Json(ApiResponse::success(supplier)),
        ),
        Err(e) => {
            let status = match e.code {
                ErrorCode::NotFound => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// List suppliers with pagination
async fn list_suppliers<S: SupplierService, A: SupplierAnalytics>(
    State((service, _analytics)): State<(S, A)>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let page = params.get("page")
        .and_then(|p| p.parse().ok())
        .unwrap_or(1);
    let limit = params.get("limit")
        .and_then(|l| l.parse().ok())
        .unwrap_or(20);

    let pagination = PaginationOptions { page, limit };

    match service.list_suppliers(pagination).await {
        Ok(result) => (
            StatusCode::OK,
            Json(ApiResponse::success(result)),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(e.to_string())),
        ),
    }
}

/// Search suppliers
async fn search_suppliers<S: SupplierService, A: SupplierAnalytics>(
    State((service, _analytics)): State<(S, A)>,
    Query(query): Query<SupplierSearchQuery>,
) -> impl IntoResponse {
    let filters = SupplierSearchFilters {
        query: query.q,
        status: query.status,
        category: query.category,
        tags: None,
        min_rating: query.min_rating,
        max_rating: query.max_rating,
        payment_terms: None,
        country: None,
        created_after: None,
        created_before: None,
    };

    let pagination = PaginationOptions {
        page: query.page.unwrap_or(1),
        limit: query.limit.unwrap_or(20),
    };

    match service.search_suppliers(filters, pagination).await {
        Ok(result) => (
            StatusCode::OK,
            Json(ApiResponse::success(result)),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(e.to_string())),
        ),
    }
}

/// Get supplier contacts
async fn get_supplier_contacts<S: SupplierService, A: SupplierAnalytics>(
    State((service, _analytics)): State<(S, A)>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match service.get_supplier_contacts(id).await {
        Ok(contacts) => (
            StatusCode::OK,
            Json(ApiResponse::success(contacts)),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(e.to_string())),
        ),
    }
}

/// Add supplier contact request
#[derive(Debug, Deserialize)]
pub struct AddContactRequest {
    pub first_name: String,
    pub last_name: String,
    pub role: String,
    pub email: Option<String>,
    pub phone: Option<String>,
}

/// Add a supplier contact
async fn add_supplier_contact<S: SupplierService, A: SupplierAnalytics>(
    State((service, _analytics)): State<(S, A)>,
    Path(id): Path<Uuid>,
    Json(request): Json<AddContactRequest>,
) -> impl IntoResponse {
    match service.add_supplier_contact(
        id,
        request.first_name,
        request.last_name,
        request.role,
        request.email,
        request.phone,
    ).await {
        Ok(contact) => (
            StatusCode::CREATED,
            Json(ApiResponse::success(contact)),
        ),
        Err(e) => {
            let status = match e.code {
                ErrorCode::NotFound => StatusCode::NOT_FOUND,
                ErrorCode::ValidationFailed => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// Get supplier addresses
async fn get_supplier_addresses<S: SupplierService, A: SupplierAnalytics>(
    State((service, _analytics)): State<(S, A)>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match service.get_supplier_addresses(id).await {
        Ok(addresses) => (
            StatusCode::OK,
            Json(ApiResponse::success(addresses)),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(e.to_string())),
        ),
    }
}

/// Add supplier address request
#[derive(Debug, Deserialize)]
pub struct AddAddressRequest {
    pub address_type: String,
    pub street1: String,
    pub city: String,
    pub country: String,
}

/// Add a supplier address
async fn add_supplier_address<S: SupplierService, A: SupplierAnalytics>(
    State((service, _analytics)): State<(S, A)>,
    Path(id): Path<Uuid>,
    Json(request): Json<AddAddressRequest>,
) -> impl IntoResponse {
    match service.add_supplier_address(
        id,
        request.address_type,
        request.street1,
        request.city,
        request.country,
    ).await {
        Ok(address) => (
            StatusCode::CREATED,
            Json(ApiResponse::success(address)),
        ),
        Err(e) => {
            let status = match e.code {
                ErrorCode::NotFound => StatusCode::NOT_FOUND,
                ErrorCode::ValidationFailed => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// Get supplier performance data
async fn get_supplier_performance<S: SupplierService, A: SupplierAnalytics>(
    State((service, _analytics)): State<(S, A)>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match service.get_supplier_performance(id).await {
        Ok(performance) => (
            StatusCode::OK,
            Json(ApiResponse::success(performance)),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(e.to_string())),
        ),
    }
}

/// Record supplier performance request
#[derive(Debug, Deserialize)]
pub struct RecordPerformanceRequest {
    pub period_start: chrono::DateTime<chrono::Utc>,
    pub period_end: chrono::DateTime<chrono::Utc>,
    pub total_orders: i32,
    pub on_time_deliveries: i32,
    pub late_deliveries: i32,
    pub early_deliveries: i32,
    pub quality_rating: Option<f64>,
    pub total_spend: i64,
    pub notes: Option<String>,
}

/// Record supplier performance
async fn record_supplier_performance<S: SupplierService, A: SupplierAnalytics>(
    State((service, _analytics)): State<(S, A)>,
    Path(id): Path<Uuid>,
    Json(request): Json<RecordPerformanceRequest>,
) -> impl IntoResponse {
    let performance_data = super::service::SupplierPerformanceData {
        period_start: request.period_start,
        period_end: request.period_end,
        total_orders: request.total_orders,
        on_time_deliveries: request.on_time_deliveries,
        late_deliveries: request.late_deliveries,
        early_deliveries: request.early_deliveries,
        average_lead_time_days: None,
        quality_rating: request.quality_rating,
        defect_rate: None,
        return_rate: None,
        total_spend: request.total_spend,
        average_order_value: if request.total_orders > 0 {
            request.total_spend / request.total_orders as i64
        } else {
            0
        },
        payment_compliance_rate: None,
        overall_rating: request.quality_rating,
        notes: request.notes,
    };

    match service.record_supplier_performance(id, performance_data).await {
        Ok(performance) => (
            StatusCode::CREATED,
            Json(ApiResponse::success(performance)),
        ),
        Err(e) => {
            let status = match e.code {
                ErrorCode::NotFound => StatusCode::NOT_FOUND,
                ErrorCode::ValidationFailed => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// Get supplier dashboard analytics
async fn get_supplier_dashboard<S: SupplierService, A: SupplierAnalytics>(
    State((_service, analytics)): State<(S, A)>,
) -> impl IntoResponse {
    // For now, use a dummy tenant ID - in production this would come from auth context
    let tenant_id = Uuid::new_v4();

    let dashboard = analytics.generate_dashboard(tenant_id).await;
    (
        StatusCode::OK,
        Json(ApiResponse::success(dashboard)),
    )
}

/// Get top suppliers
async fn get_top_suppliers<S: SupplierService, A: SupplierAnalytics>(
    State((service, _analytics)): State<(S, A)>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let limit = params.get("limit")
        .and_then(|l| l.parse().ok())
        .unwrap_or(10);

    match service.get_top_suppliers(limit).await {
        Ok(suppliers) => (
            StatusCode::OK,
            Json(ApiResponse::success(suppliers)),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(e.to_string())),
        ),
    }
}

/// Get suppliers requiring attention
async fn get_suppliers_requiring_attention<S: SupplierService, A: SupplierAnalytics>(
    State((service, _analytics)): State<(S, A)>,
) -> impl IntoResponse {
    match service.get_suppliers_requiring_attention().await {
        Ok(suppliers) => (
            StatusCode::OK,
            Json(ApiResponse::success(suppliers)),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(e.to_string())),
        ),
    }
}

/// Get suppliers by category
async fn get_suppliers_by_category<S: SupplierService, A: SupplierAnalytics>(
    State((service, _analytics)): State<(S, A)>,
) -> impl IntoResponse {
    match service.get_suppliers_by_category().await {
        Ok(categories) => (
            StatusCode::OK,
            Json(ApiResponse::success(categories)),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(e.to_string())),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_supplier_routes_compile() {
        // This test just ensures the routes compile correctly
        // In a real test, we would set up a mock service and test the endpoints
        let _router = supplier_routes::<super::super::service::MockSupplierService, super::super::analytics::DefaultSupplierAnalytics>();
    }
}