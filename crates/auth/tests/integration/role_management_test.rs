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
    models::Role,
};
use erp_core::{TenantContext, TenantId};
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_list_roles_with_permissions() {
    init_test_logging();
    let ctx = TestContext::new().await;
    
    // Create a test user with role read permissions
    let admin_user = create_test_admin_user(&ctx).await;
    let token = generate_test_jwt(&ctx, admin_user.id, vec!["roles:read".to_string()]).await;
    
    // Create some test roles
    let _test_role = create_test_role(&ctx, "test_role", "Test Role").await;
    
    // Create test router with auth middleware
    let auth_state = create_auth_state(&ctx).await;
    let app = create_protected_test_router(auth_state);
    
    // Make authenticated request
    let request = Request::builder()
        .uri("/roles/list")
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
async fn test_list_roles_without_permissions() {
    init_test_logging();
    let ctx = TestContext::new().await;
    
    // Create a test user without role read permissions
    let user = create_test_user(&ctx, "testuser@example.com").await;
    let token = generate_test_jwt(&ctx, user.id, vec!["users:read".to_string()]).await;
    
    // Create test router with auth middleware
    let auth_state = create_auth_state(&ctx).await;
    let app = create_protected_test_router(auth_state);
    
    // Make authenticated request without proper permissions
    let request = Request::builder()
        .uri("/roles/list")
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
async fn test_create_role_with_permissions() {
    init_test_logging();
    let ctx = TestContext::new().await;
    
    // Create a test admin user
    let admin_user = create_test_admin_user(&ctx).await;
    let token = generate_test_jwt(&ctx, admin_user.id, vec!["roles:create".to_string()]).await;
    
    // Create test router with auth middleware
    let auth_state = create_auth_state(&ctx).await;
    let app = create_protected_test_router(auth_state);
    
    let create_request = CreateRoleRequest {
        name: "new_role".to_string(),
        description: Some("A new test role".to_string()),
        permission_ids: vec![],
    };
    
    // Make authenticated request
    let request = Request::builder()
        .uri("/roles/create")
        .method("POST")
        .header("Authorization", format!("Bearer {}", token))
        .header("X-Tenant-ID", ctx.tenant_id.to_string())
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&create_request).unwrap()))
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    ctx.cleanup().await;
}

#[tokio::test]
async fn test_create_role_without_permissions() {
    init_test_logging();
    let ctx = TestContext::new().await;
    
    // Create a test user without role create permissions
    let user = create_test_user(&ctx, "testuser@example.com").await;
    let token = generate_test_jwt(&ctx, user.id, vec!["roles:read".to_string()]).await;
    
    // Create test router with auth middleware
    let auth_state = create_auth_state(&ctx).await;
    let app = create_protected_test_router(auth_state);
    
    let create_request = CreateRoleRequest {
        name: "unauthorized_role".to_string(),
        description: Some("This should fail".to_string()),
        permission_ids: vec![],
    };
    
    // Make authenticated request without proper permissions
    let request = Request::builder()
        .uri("/roles/create")
        .method("POST")
        .header("Authorization", format!("Bearer {}", token))
        .header("X-Tenant-ID", ctx.tenant_id.to_string())
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&create_request).unwrap()))
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    
    ctx.cleanup().await;
}

#[tokio::test]
async fn test_update_role_with_permissions() {
    init_test_logging();
    let ctx = TestContext::new().await;
    
    // Create test role and admin user
    let test_role = create_test_role(&ctx, "updatable_role", "Original description").await;
    let admin_user = create_test_admin_user(&ctx).await;
    let token = generate_test_jwt(&ctx, admin_user.id, vec!["roles:update".to_string()]).await;
    
    // Create test router with auth middleware
    let auth_state = create_auth_state(&ctx).await;
    let app = create_protected_test_router(auth_state);
    
    let update_request = UpdateRoleRequest {
        name: Some("updated_role".to_string()),
        description: Some("Updated description".to_string()),
        permission_ids: Some(vec![]),
    };
    
    // Make authenticated request
    let request = Request::builder()
        .uri(&format!("/roles/{}/update", test_role.id))
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
async fn test_delete_role_with_permissions() {
    init_test_logging();
    let ctx = TestContext::new().await;
    
    // Create test role and admin user
    let test_role = create_test_role(&ctx, "deletable_role", "Role to be deleted").await;
    let admin_user = create_test_admin_user(&ctx).await;
    let token = generate_test_jwt(&ctx, admin_user.id, vec!["roles:delete".to_string()]).await;
    
    // Create test router with auth middleware
    let auth_state = create_auth_state(&ctx).await;
    let app = create_protected_test_router(auth_state);
    
    // Make authenticated request
    let request = Request::builder()
        .uri(&format!("/roles/{}/delete", test_role.id))
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
async fn test_get_role_details_with_permissions() {
    init_test_logging();
    let ctx = TestContext::new().await;
    
    // Create test role and admin user
    let test_role = create_test_role(&ctx, "detailed_role", "Role with details").await;
    let admin_user = create_test_admin_user(&ctx).await;
    let token = generate_test_jwt(&ctx, admin_user.id, vec!["roles:read".to_string()]).await;
    
    // Create test router with auth middleware
    let auth_state = create_auth_state(&ctx).await;
    let app = create_protected_test_router(auth_state);
    
    // Make authenticated request
    let request = Request::builder()
        .uri(&format!("/roles/{}/get", test_role.id))
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
async fn test_role_tenant_isolation() {
    init_test_logging();
    let ctx1 = TestContext::new().await;
    let ctx2 = TestContext::new().await;
    
    // Create roles in different tenants
    let role1 = create_test_role(&ctx1, "tenant1_role", "Role in tenant 1").await;
    let admin2 = create_test_admin_user(&ctx2).await;
    
    // Generate token for admin2 trying to access role1 from different tenant
    let token2 = generate_test_jwt(&ctx2, admin2.id, vec!["roles:read".to_string()]).await;
    
    // Create test router with ctx1's auth state
    let auth_state1 = create_auth_state(&ctx1).await;
    let app = create_protected_test_router(auth_state1);
    
    // Try to access role1 with token from tenant2 (should fail)
    let request = Request::builder()
        .uri(&format!("/roles/{}/get", role1.id))
        .method("GET")
        .header("Authorization", format!("Bearer {}", token2))
        .header("X-Tenant-ID", ctx1.tenant_id.to_string())
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    // Should be unauthorized due to tenant mismatch
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    
    ctx1.cleanup().await;
    ctx2.cleanup().await;
}

#[tokio::test]
async fn test_role_permissions_hierarchy() {
    init_test_logging();
    let ctx = TestContext::new().await;
    
    // Create users with different permission levels
    let read_user = create_test_user(&ctx, "readonly@example.com").await;
    let create_user = create_test_user(&ctx, "creator@example.com").await;
    
    let read_token = generate_test_jwt(&ctx, read_user.id, vec!["roles:read".to_string()]).await;
    let create_token = generate_test_jwt(&ctx, create_user.id, vec!["roles:create".to_string()]).await;
    
    // Create test router
    let auth_state = create_auth_state(&ctx).await;
    
    // Test that read user can list roles
    let app1 = create_protected_test_router(auth_state.clone());
    let request = Request::builder()
        .uri("/roles/list")
        .method("GET")
        .header("Authorization", format!("Bearer {}", read_token))
        .header("X-Tenant-ID", ctx.tenant_id.to_string())
        .body(Body::empty())
        .unwrap();
    
    let response = app1.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // Test that read user cannot create roles
    let app2 = create_protected_test_router(auth_state.clone());
    let create_request = CreateRoleRequest {
        name: "unauthorized_role".to_string(),
        description: Some("Should fail".to_string()),
        permission_ids: vec![],
    };
    
    let request = Request::builder()
        .uri("/roles/create")
        .method("POST")
        .header("Authorization", format!("Bearer {}", read_token))
        .header("X-Tenant-ID", ctx.tenant_id.to_string())
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&create_request).unwrap()))
        .unwrap();
    
    let response = app2.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    
    // Test that create user can create roles
    let app3 = create_protected_test_router(auth_state);
    let request = Request::builder()
        .uri("/roles/create")
        .method("POST")
        .header("Authorization", format!("Bearer {}", create_token))
        .header("X-Tenant-ID", ctx.tenant_id.to_string())
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&create_request).unwrap()))
        .unwrap();
    
    let response = app3.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    ctx.cleanup().await;
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

async fn create_test_role(ctx: &TestContext, name: &str, description: &str) -> Role {
    let tenant_context = TenantContext {
        tenant_id: TenantId(ctx.tenant_id),
        schema_name: format!("test_tenant_{}", ctx.tenant_id),
    };
    
    ctx.auth_service
        .repository()
        .create_role(&tenant_context, name, Some(description), true)
        .await
        .expect("Failed to create test role")
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
    // Create simple test handlers
    async fn test_list_roles() -> &'static str { "roles_list" }
    async fn test_create_role() -> &'static str { "role_created" }
    async fn test_get_role() -> &'static str { "role_details" }
    async fn test_update_role() -> &'static str { "role_updated" }
    async fn test_delete_role() -> &'static str { "role_deleted" }
    
    Router::new()
        .route("/roles/list", get(test_list_roles)
            .layer(from_fn(require_permission("roles:read"))))
        .route("/roles/create", axum::routing::post(test_create_role)
            .layer(from_fn(require_permission("roles:create"))))
        .route("/roles/:id/get", get(test_get_role)
            .layer(from_fn(require_permission("roles:read"))))
        .route("/roles/:id/update", axum::routing::put(test_update_role)
            .layer(from_fn(require_permission("roles:update"))))
        .route("/roles/:id/delete", axum::routing::delete(test_delete_role)
            .layer(from_fn(require_permission("roles:delete"))))
        .layer(from_fn_with_state(auth_state.clone(), auth_middleware))
        .with_state(auth_state)
}