//! Role management handlers
//!
//! HTTP handlers for role and permission management

use axum::{
    extract::{State, Path, Extension},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete, Router},
};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::state::AppState;
use erp_core::TenantContext;
use erp_auth::dto::{CreateRoleRequest as AuthCreateRoleRequest, UpdateRoleRequest as AuthUpdateRoleRequest};

#[derive(Debug, Deserialize)]
pub struct CreateRoleRequest {
    pub name: String,
    pub description: Option<String>,
    pub permission_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRoleRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub permission_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Deserialize)]
pub struct AssignPermissionsRequest {
    pub permission_ids: Vec<Uuid>,
}

/// Create role management routes
pub fn role_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_roles))
        .route("/", post(create_role))
        .route("/:id", get(get_role))
        .route("/:id", put(update_role))
        .route("/:id", delete(delete_role))
        .route("/:id/permissions", get(get_role_permissions))
        .route("/:id/permissions", post(assign_permissions))
}

/// List all roles
async fn list_roles(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
) -> Result<Json<Value>, StatusCode> {
    match state.auth_service.list_roles(&tenant_context).await {
        Ok(roles) => {
            Ok(Json(json!({
                "success": true,
                "roles": roles
            })))
        }
        Err(e) => {
            tracing::error!("Failed to list roles: {}", e);
            Ok(Json(json!({
                "success": false,
                "error": "Failed to retrieve roles",
                "message": e.to_string()
            })))
        }
    }
}

/// Create a new role
async fn create_role(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Json(payload): Json<CreateRoleRequest>,
) -> Result<Json<Value>, StatusCode> {
    // Basic validation
    if payload.name.trim().is_empty() {
        return Ok(Json(json!({
            "success": false,
            "error": "Role name is required"
        })));
    }

    // Convert to auth service request
    let auth_request = AuthCreateRoleRequest {
        name: payload.name,
        description: payload.description,
        permission_ids: payload.permission_ids.unwrap_or_default(),
    };

    match state.auth_service.create_role(&tenant_context, auth_request).await {
        Ok(role) => {
            Ok(Json(json!({
                "success": true,
                "role": role,
                "message": "Role created successfully"
            })))
        }
        Err(e) => {
            tracing::error!("Failed to create role: {}", e);
            Ok(Json(json!({
                "success": false,
                "error": "Failed to create role",
                "message": e.to_string()
            })))
        }
    }
}

/// Get role by ID
async fn get_role(
    State(state): State<AppState>,
    Path(role_id): Path<Uuid>,
    Extension(tenant_context): Extension<TenantContext>,
) -> Result<Json<Value>, StatusCode> {
    match state.auth_service.get_role(&tenant_context, role_id).await {
        Ok(role) => {
            Ok(Json(json!({
                "success": true,
                "role": role
            })))
        }
        Err(e) => {
            tracing::error!("Failed to get role {}: {}", role_id, e);
            Ok(Json(json!({
                "success": false,
                "error": "Failed to retrieve role",
                "message": e.to_string()
            })))
        }
    }
}

/// Update role
async fn update_role(
    State(state): State<AppState>,
    Path(role_id): Path<Uuid>,
    Extension(tenant_context): Extension<TenantContext>,
    Json(payload): Json<UpdateRoleRequest>,
) -> Result<Json<Value>, StatusCode> {
    // Convert to auth service request
    let auth_request = AuthUpdateRoleRequest {
        name: payload.name,
        description: payload.description,
        permission_ids: payload.permission_ids,
    };

    match state.auth_service.update_role(&tenant_context, role_id, auth_request).await {
        Ok(role) => {
            Ok(Json(json!({
                "success": true,
                "role": role,
                "message": "Role updated successfully"
            })))
        }
        Err(e) => {
            tracing::error!("Failed to update role {}: {}", role_id, e);
            Ok(Json(json!({
                "success": false,
                "error": "Failed to update role",
                "message": e.to_string()
            })))
        }
    }
}

/// Delete role
async fn delete_role(
    State(state): State<AppState>,
    Path(role_id): Path<Uuid>,
    Extension(tenant_context): Extension<TenantContext>,
) -> Result<Json<Value>, StatusCode> {
    match state.auth_service.delete_role(&tenant_context, role_id).await {
        Ok(()) => {
            Ok(Json(json!({
                "success": true,
                "message": format!("Role {} deleted successfully", role_id)
            })))
        }
        Err(e) => {
            tracing::error!("Failed to delete role {}: {}", role_id, e);
            Ok(Json(json!({
                "success": false,
                "error": "Failed to delete role",
                "message": e.to_string()
            })))
        }
    }
}

/// Get role permissions
async fn get_role_permissions(
    State(state): State<AppState>,
    Path(role_id): Path<Uuid>,
    Extension(tenant_context): Extension<TenantContext>,
) -> Result<Json<Value>, StatusCode> {
    // Get role details which include permission information
    match state.auth_service.get_role(&tenant_context, role_id).await {
        Ok(role) => {
            Ok(Json(json!({
                "success": true,
                "role_id": role_id,
                "role_name": role.name,
                "message": "Role permissions retrieved successfully"
            })))
        }
        Err(e) => {
            tracing::error!("Failed to get role permissions for {}: {}", role_id, e);
            Ok(Json(json!({
                "success": false,
                "error": "Failed to retrieve role permissions",
                "message": e.to_string()
            })))
        }
    }
}

/// Assign permissions to role
async fn assign_permissions(
    State(state): State<AppState>,
    Path(role_id): Path<Uuid>,
    Extension(tenant_context): Extension<TenantContext>,
    Json(payload): Json<AssignPermissionsRequest>,
) -> Result<Json<Value>, StatusCode> {
    // Use the update_role method to assign new permissions
    let auth_request = AuthUpdateRoleRequest {
        name: None,
        description: None,
        permission_ids: Some(payload.permission_ids),
    };

    match state.auth_service.update_role(&tenant_context, role_id, auth_request).await {
        Ok(role) => {
            Ok(Json(json!({
                "success": true,
                "role": role,
                "message": "Permissions assigned successfully"
            })))
        }
        Err(e) => {
            tracing::error!("Failed to assign permissions to role {}: {}", role_id, e);
            Ok(Json(json!({
                "success": false,
                "error": "Failed to assign permissions",
                "message": e.to_string()
            })))
        }
    }
}