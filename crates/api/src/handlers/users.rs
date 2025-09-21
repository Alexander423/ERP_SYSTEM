//! User management handlers
//!
//! HTTP handlers for user CRUD operations and management

use axum::{
    extract::{State, Path, Query, Extension},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete, Router},
};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::state::AppState;
use erp_core::TenantContext;
use erp_auth::dto::{InviteUserRequest as AuthInviteUserRequest, UpdateUserRequest as AuthUpdateUserRequest};

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_page() -> u32 { 1 }
fn default_limit() -> u32 { 20 }

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub role_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct InviteUserRequest {
    pub email: String,
    pub role_ids: Vec<Uuid>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

/// Create user management routes
pub fn user_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_users))
        .route("/", post(create_user))
        .route("/:id", get(get_user))
        .route("/:id", put(update_user))
        .route("/:id", delete(delete_user))
        .route("/invite", post(invite_user))
}

/// List all users
async fn list_users(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
    Extension(tenant_context): Extension<TenantContext>,
) -> Result<Json<Value>, StatusCode> {
    match state.auth_service.list_users(&tenant_context, params.limit, (params.page - 1) * params.limit).await {
        Ok(users) => {
            Ok(Json(json!({
                "success": true,
                "users": users,
                "pagination": {
                    "page": params.page,
                    "limit": params.limit,
                    "total": users.len(),
                    "total_pages": (users.len() as f64 / params.limit as f64).ceil() as u32
                }
            })))
        }
        Err(e) => {
            tracing::error!("Failed to list users: {}", e);
            Ok(Json(json!({
                "success": false,
                "error": "Failed to retrieve users",
                "message": e.to_string()
            })))
        }
    }
}

/// Create a new user
async fn create_user(
    State(_state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<Value>, StatusCode> {
    // For now, return a meaningful error since direct user creation should use invite flow
    tracing::info!("Create user request for email: {}, first_name: {}, last_name: {}, role_ids: {:?}, password_provided: {}",
        payload.email, payload.first_name, payload.last_name, payload.role_ids, !payload.password.is_empty());

    Ok(Json(json!({
        "success": false,
        "message": "Direct user creation not implemented. Use invite_user endpoint instead.",
        "requested_email": payload.email
    })))
}

/// Get user by ID
async fn get_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Extension(tenant_context): Extension<TenantContext>,
) -> Result<Json<Value>, StatusCode> {
    match state.auth_service.get_user(&tenant_context, user_id).await {
        Ok(user) => {
            Ok(Json(json!({
                "success": true,
                "user": user
            })))
        }
        Err(e) => {
            tracing::error!("Failed to get user {}: {}", user_id, e);
            Ok(Json(json!({
                "success": false,
                "error": "Failed to retrieve user",
                "message": e.to_string()
            })))
        }
    }
}

/// Update user
async fn update_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Extension(tenant_context): Extension<TenantContext>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<Value>, StatusCode> {
    // Convert to auth service request
    let auth_request = AuthUpdateUserRequest {
        first_name: payload.first_name,
        last_name: payload.last_name,
        is_active: payload.is_active,
    };

    match state.auth_service.update_user(&tenant_context, user_id, auth_request).await {
        Ok(user) => {
            Ok(Json(json!({
                "success": true,
                "user": user,
                "message": "User updated successfully"
            })))
        }
        Err(e) => {
            tracing::error!("Failed to update user {}: {}", user_id, e);
            Ok(Json(json!({
                "success": false,
                "error": "Failed to update user",
                "message": e.to_string()
            })))
        }
    }
}

/// Delete user
async fn delete_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Extension(tenant_context): Extension<TenantContext>,
) -> Result<Json<Value>, StatusCode> {
    match state.auth_service.delete_user(&tenant_context, user_id).await {
        Ok(()) => {
            Ok(Json(json!({
                "success": true,
                "message": format!("User {} deleted successfully", user_id)
            })))
        }
        Err(e) => {
            tracing::error!("Failed to delete user {}: {}", user_id, e);
            Ok(Json(json!({
                "success": false,
                "error": "Failed to delete user",
                "message": e.to_string()
            })))
        }
    }
}

/// Invite a new user
async fn invite_user(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Json(payload): Json<InviteUserRequest>,
) -> Result<Json<Value>, StatusCode> {
    // Basic validation
    if payload.email.trim().is_empty() {
        return Ok(Json(json!({
            "success": false,
            "error": "Email is required"
        })));
    }

    // Convert to auth service request
    let auth_request = AuthInviteUserRequest {
        email: payload.email,
        first_name: payload.first_name,
        last_name: payload.last_name,
        role_ids: payload.role_ids,
    };

    match state.auth_service.invite_user(&tenant_context, auth_request).await {
        Ok(user) => {
            Ok(Json(json!({
                "success": true,
                "user": user,
                "message": "User invitation sent successfully"
            })))
        }
        Err(e) => {
            tracing::error!("Failed to invite user: {}", e);
            Ok(Json(json!({
                "success": false,
                "error": "Failed to invite user",
                "message": e.to_string()
            })))
        }
    }
}