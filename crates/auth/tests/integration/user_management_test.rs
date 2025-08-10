use super::common::{TestContext, init_test_logging};
use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::{from_fn_with_state, from_fn},
    Router,
    routing::get,
};
use erp_auth::{
    dto::*,
    middleware::{auth_middleware, AuthState, require_permission},
};
use erp_core::{TenantContext, TenantId};
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_list_users_with_valid_permissions() {
    init_test_logging();
    let ctx = TestContext::new().await;
    
    // Create a test user with admin role
    let admin_user = create_test_admin_user(&ctx).await;
    let token = generate_test_jwt(&ctx, admin_user.id, vec!["users:read".to_string()]).await;
    
    // Create test router with auth middleware
    let auth_state = create_auth_state(&ctx).await;
    let app = create_protected_test_router(auth_state);
    
    // Make authenticated request
    let request = Request::builder()
        .uri("/users/list")
        .method("GET")
        .header("Authorization", format!("Bearer {}", token))
        .header("X-Tenant-ID", ctx.tenant_id.to_string())
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    ctx.cleanup().await;
}

#[tokio::test]
async fn test_list_users_without_permissions() {
    init_test_logging();
    let ctx = TestContext::new().await;
    
    // Create a test user without user read permissions
    let user = create_test_user(&ctx, "testuser@example.com").await;
    let token = generate_test_jwt(&ctx, user.id, vec!["some:other".to_string()]).await;
    
    // Create test router with auth middleware
    let auth_state = create_auth_state(&ctx).await;
    let app = create_protected_test_router(auth_state);
    
    // Make authenticated request without proper permissions
    let request = Request::builder()
        .uri("/users/list")
        .method("GET")
        .header("Authorization", format!("Bearer {}", token))
        .header("X-Tenant-ID", ctx.tenant_id.to_string())
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    
    ctx.cleanup().await;
}

#[tokio::test]
async fn test_list_users_without_authentication() {
    init_test_logging();
    let ctx = TestContext::new().await;
    
    // Create test router with auth middleware
    let auth_state = create_auth_state(&ctx).await;
    let app = create_protected_test_router(auth_state);
    
    // Make unauthenticated request
    let request = Request::builder()
        .uri("/users/list")
        .method("GET")
        .header("X-Tenant-ID", ctx.tenant_id.to_string())
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    
    ctx.cleanup().await;
}

#[tokio::test]
async fn test_create_user_with_permissions() {
    init_test_logging();
    let ctx = TestContext::new().await;
    
    // Create a test admin user
    let admin_user = create_test_admin_user(&ctx).await;
    let token = generate_test_jwt(&ctx, admin_user.id, vec!["users:create".to_string()]).await;
    
    // Create test router with auth middleware
    let auth_state = create_auth_state(&ctx).await;
    let app = create_protected_test_router(auth_state);
    
    let invite_request = InviteUserRequest {
        email: "newuser@example.com".to_string(),
        first_name: Some("New".to_string()),
        last_name: Some("User".to_string()),
        role_ids: vec![],
    };
    
    // Make authenticated request
    let request = Request::builder()
        .uri("/users/invite")
        .method("POST")
        .header("Authorization", format!("Bearer {}", token))
        .header("X-Tenant-ID", ctx.tenant_id.to_string())
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&invite_request).unwrap()))
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    ctx.cleanup().await;
}

#[tokio::test]
async fn test_create_user_without_permissions() {
    init_test_logging();
    let ctx = TestContext::new().await;
    
    // Create a test user without create permissions
    let user = create_test_user(&ctx, "testuser@example.com").await;
    let token = generate_test_jwt(&ctx, user.id, vec!["users:read".to_string()]).await;
    
    // Create test router with auth middleware
    let auth_state = create_auth_state(&ctx).await;
    let app = create_protected_test_router(auth_state);
    
    let invite_request = InviteUserRequest {
        email: "newuser@example.com".to_string(),
        first_name: Some("New".to_string()),
        last_name: Some("User".to_string()),
        role_ids: vec![],
    };
    
    // Make authenticated request without proper permissions
    let request = Request::builder()
        .uri("/users/invite")
        .method("POST")
        .header("Authorization", format!("Bearer {}", token))
        .header("X-Tenant-ID", ctx.tenant_id.to_string())
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&invite_request).unwrap()))
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    
    ctx.cleanup().await;
}

#[tokio::test]
async fn test_update_user_with_permissions() {
    init_test_logging();
    let ctx = TestContext::new().await;
    
    // Create test users
    let admin_user = create_test_admin_user(&ctx).await;
    let target_user = create_test_user(&ctx, "targetuser@example.com").await;
    let token = generate_test_jwt(&ctx, admin_user.id, vec!["users:update".to_string()]).await;
    
    // Create test router with auth middleware
    let auth_state = create_auth_state(&ctx).await;
    let app = create_protected_test_router(auth_state);
    
    let update_request = UpdateUserRequest {
        first_name: Some("Updated".to_string()),
        last_name: Some("Name".to_string()),
        is_active: Some(false),
    };
    
    // Make authenticated request
    let request = Request::builder()
        .uri(&format!("/users/{}/update", target_user.id))
        .method("PUT")
        .header("Authorization", format!("Bearer {}", token))
        .header("X-Tenant-ID", ctx.tenant_id.to_string())
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&update_request).unwrap()))
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    ctx.cleanup().await;
}

#[tokio::test]
async fn test_delete_user_with_permissions() {
    init_test_logging();
    let ctx = TestContext::new().await;
    
    // Create test users
    let admin_user = create_test_admin_user(&ctx).await;
    let target_user = create_test_user(&ctx, "deleteuser@example.com").await;
    let token = generate_test_jwt(&ctx, admin_user.id, vec!["users:delete".to_string()]).await;
    
    // Create test router with auth middleware
    let auth_state = create_auth_state(&ctx).await;
    let app = create_protected_test_router(auth_state);
    
    // Make authenticated request
    let request = Request::builder()
        .uri(&format!("/users/{}/delete", target_user.id))
        .method("DELETE")
        .header("Authorization", format!("Bearer {}", token))
        .header("X-Tenant-ID", ctx.tenant_id.to_string())
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    ctx.cleanup().await;
}

#[tokio::test]
async fn test_multi_tenant_isolation() {
    init_test_logging();
    let ctx1 = TestContext::new().await;
    let ctx2 = TestContext::new().await;
    
    // Create users in different tenants
    let user1 = create_test_admin_user(&ctx1).await;
    let _user2 = create_test_admin_user(&ctx2).await;
    
    // Generate token for user1 with ctx1 tenant
    let token1 = generate_test_jwt(&ctx1, user1.id, vec!["users:read".to_string()]).await;
    
    // Create test router with ctx2's auth state
    let auth_state2 = create_auth_state(&ctx2).await;
    let app = create_protected_test_router(auth_state2);
    
    // Try to access ctx2's users with ctx1's token (should fail due to tenant mismatch)
    let request = Request::builder()
        .uri("/users/list")
        .method("GET")
        .header("Authorization", format!("Bearer {}", token1))
        .header("X-Tenant-ID", ctx2.tenant_id.to_string())
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    // Should be unauthorized due to tenant mismatch in JWT vs header
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    
    ctx1.cleanup().await;
    ctx2.cleanup().await;
}

// Helper functions

async fn create_test_user(ctx: &TestContext, email: &str) -> erp_auth::models::User {
    let tenant_context = TenantContext {
        tenant_id: TenantId(ctx.tenant_id),
        schema_name: format!("test_tenant_{}", ctx.tenant_id),
    };
    
    ctx.auth_service
        .repository()
        .create_user(
            &tenant_context,
            email,
            Some("$argon2id$v=19$m=65536,t=3,p=4$test$test"), // dummy hash
            "Test",
            "User"
        )
        .await
        .expect("Failed to create test user")
}

async fn create_test_admin_user(ctx: &TestContext) -> erp_auth::models::User {
    let tenant_context = TenantContext {
        tenant_id: TenantId(ctx.tenant_id),
        schema_name: format!("test_tenant_{}", ctx.tenant_id),
    };
    
    let user = ctx.auth_service
        .repository()
        .create_user(
            &tenant_context,
            "admin@example.com",
            Some("$argon2id$v=19$m=65536,t=3,p=4$test$test"), // dummy hash
            "Admin",
            "User"
        )
        .await
        .expect("Failed to create admin user");
    
    // Create admin role and assign it
    let admin_role = ctx.auth_service
        .repository()
        .create_role(&tenant_context, "admin", Some("Administrator role"), true)
        .await
        .expect("Failed to create admin role");
    
    ctx.auth_service
        .repository()
        .assign_role_to_user(&tenant_context, user.id, admin_role.id)
        .await
        .expect("Failed to assign admin role");
    
    user
}

async fn generate_test_jwt(ctx: &TestContext, user_id: Uuid, permissions: Vec<String>) -> String {
    use erp_core::types::JwtClaims;
    use jsonwebtoken::{encode, Header, EncodingKey};
    
    let claims = JwtClaims {
        sub: user_id.to_string(),
        tenant_id: ctx.tenant_id.to_string(),
        roles: vec!["test_role".to_string()],
        permissions,
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp(),
        iat: chrono::Utc::now().timestamp(),
        jti: Uuid::new_v4().to_string(),
        impersonator_id: None,
    };
    
    // Use a test JWT secret - in production this would come from config
    let secret = b"test_jwt_secret_key_for_testing_only";
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret))
        .expect("Failed to generate test JWT")
}

async fn create_auth_state(ctx: &TestContext) -> AuthState {
    AuthState {
        jwt_service: ctx.auth_service.jwt_service().clone(),
        db: Arc::new(ctx.db.clone()),
        redis: ctx.redis.clone(),
    }
}

fn create_protected_test_router(auth_state: AuthState) -> Router {
    // Create a simple test handler
    async fn test_list_users() -> &'static str { "users_list" }
    async fn test_invite_user() -> &'static str { "user_invited" }
    async fn test_get_user() -> &'static str { "user_details" }
    async fn test_update_user() -> &'static str { "user_updated" }
    async fn test_delete_user() -> &'static str { "user_deleted" }
    
    Router::new()
        .route("/users/list", get(test_list_users)
            .layer(from_fn(require_permission("users:read"))))
        .route("/users/invite", axum::routing::post(test_invite_user)
            .layer(from_fn(require_permission("users:create"))))
        .route("/users/:id/get", get(test_get_user)
            .layer(from_fn(require_permission("users:read"))))
        .route("/users/:id/update", axum::routing::put(test_update_user)
            .layer(from_fn(require_permission("users:update"))))
        .route("/users/:id/delete", axum::routing::delete(test_delete_user)
            .layer(from_fn(require_permission("users:delete"))))
        .layer(from_fn_with_state(auth_state.clone(), auth_middleware))
        .with_state(auth_state)
}