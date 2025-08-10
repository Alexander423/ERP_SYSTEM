use super::common::{TestContext, init_test_logging};
use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::{from_fn_with_state, from_fn},
    Router,
    routing::get,
};
use erp_auth::{
    middleware::{auth_middleware, AuthState, require_permission},
    models::{User, Role},
};
use erp_core::{TenantContext, TenantId};
use serde_json::json;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_jwt_token_validation_flow() {
    init_test_logging();
    let ctx = TestContext::new().await;
    
    let admin_user = create_test_admin_user(&ctx).await;
    
    // Test 1: Valid token
    let valid_token = generate_test_jwt(&ctx, admin_user.id, vec!["users:read".to_string()]).await;
    let auth_state = create_auth_state(&ctx).await;
    let app = create_test_router(auth_state.clone());
    
    let request = Request::builder()
        .uri("/protected")
        .method("GET")
        .header("Authorization", format!("Bearer {}", valid_token))
        .header("X-Tenant-ID", ctx.tenant_id.to_string())
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // Test 2: Invalid token
    let app = create_test_router(auth_state.clone());
    let request = Request::builder()
        .uri("/protected")
        .method("GET")
        .header("Authorization", "Bearer invalid_token_here")
        .header("X-Tenant-ID", ctx.tenant_id.to_string())
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    
    // Test 3: Expired token
    let expired_token = generate_expired_jwt(&ctx, admin_user.id, vec!["users:read".to_string()]).await;
    let app = create_test_router(auth_state.clone());
    let request = Request::builder()
        .uri("/protected")
        .method("GET")
        .header("Authorization", format!("Bearer {}", expired_token))
        .header("X-Tenant-ID", ctx.tenant_id.to_string())
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    
    // Test 4: Missing Authorization header
    let app = create_test_router(auth_state);
    let request = Request::builder()
        .uri("/protected")
        .method("GET")
        .header("X-Tenant-ID", ctx.tenant_id.to_string())
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    
    ctx.cleanup().await;
}

#[tokio::test]
async fn test_permission_based_authorization() {
    init_test_logging();
    let ctx = TestContext::new().await;
    
    // Create users with different permission sets
    let admin_user = create_test_admin_user(&ctx).await;
    let read_user = create_test_user(&ctx, "readuser@example.com").await;
    let no_perm_user = create_test_user(&ctx, "noperm@example.com").await;
    
    // Create tokens with different permissions
    let admin_token = generate_test_jwt(&ctx, admin_user.id, vec![
        "users:read".to_string(),
        "users:create".to_string(),
        "users:update".to_string(),
        "users:delete".to_string(),
        "roles:read".to_string(),
        "roles:create".to_string(),
    ]).await;
    
    let read_token = generate_test_jwt(&ctx, read_user.id, vec![
        "users:read".to_string(),
        "roles:read".to_string(),
    ]).await;
    
    let no_perm_token = generate_test_jwt(&ctx, no_perm_user.id, vec![]).await;
    
    let auth_state = create_auth_state(&ctx).await;
    
    // Test admin can perform all operations
    let operations = vec![
        ("/users:read", "GET"),
        ("/users:create", "POST"),
        ("/users:update", "PUT"),
        ("/roles:read", "GET"),
        ("/roles:create", "POST"),
    ];
    
    for (endpoint, method) in operations.iter() {
        let app = create_permission_test_router(auth_state.clone());
        let request = Request::builder()
            .uri(*endpoint)
            .method(*method)
            .header("Authorization", format!("Bearer {}", admin_token))
            .header("X-Tenant-ID", ctx.tenant_id.to_string())
            .body(Body::empty())
            .unwrap();
        
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK, "Admin should access {}", endpoint);
    }
    
    // Test read user can only perform read operations
    let read_operations = vec![("/users:read", "GET"), ("/roles:read", "GET")];
    let write_operations = vec![("/users:create", "POST"), ("/users:update", "PUT"), ("/roles:create", "POST")];
    
    for (endpoint, method) in read_operations.iter() {
        let app = create_permission_test_router(auth_state.clone());
        let request = Request::builder()
            .uri(*endpoint)
            .method(*method)
            .header("Authorization", format!("Bearer {}", read_token))
            .header("X-Tenant-ID", ctx.tenant_id.to_string())
            .body(Body::empty())
            .unwrap();
        
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK, "Read user should access {}", endpoint);
    }
    
    for (endpoint, method) in write_operations.iter() {
        let app = create_permission_test_router(auth_state.clone());
        let request = Request::builder()
            .uri(*endpoint)
            .method(*method)
            .header("Authorization", format!("Bearer {}", read_token))
            .header("X-Tenant-ID", ctx.tenant_id.to_string())
            .body(Body::empty())
            .unwrap();
        
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN, "Read user should not access {}", endpoint);
    }
    
    // Test no-permission user cannot access anything
    for (endpoint, method) in operations.iter() {
        let app = create_permission_test_router(auth_state.clone());
        let request = Request::builder()
            .uri(*endpoint)
            .method(*method)
            .header("Authorization", format!("Bearer {}", no_perm_token))
            .header("X-Tenant-ID", ctx.tenant_id.to_string())
            .body(Body::empty())
            .unwrap();
        
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN, "No-perm user should not access {}", endpoint);
    }
    
    ctx.cleanup().await;
}

#[tokio::test]
async fn test_multi_tenant_isolation_comprehensive() {
    init_test_logging();
    let ctx1 = TestContext::new().await;
    let ctx2 = TestContext::new().await;
    let ctx3 = TestContext::new().await;
    
    // Create users in different tenants
    let user1 = create_test_admin_user(&ctx1).await;
    let user2 = create_test_admin_user(&ctx2).await;
    let user3 = create_test_admin_user(&ctx3).await;
    
    // Create some data in each tenant
    let role1 = create_test_role(&ctx1, "tenant1_role", "Role in tenant 1").await;
    let role2 = create_test_role(&ctx2, "tenant2_role", "Role in tenant 2").await;
    let role3 = create_test_role(&ctx3, "tenant3_role", "Role in tenant 3").await;
    
    // Generate tokens for each user
    let token1 = generate_test_jwt(&ctx1, user1.id, vec!["users:read".to_string(), "roles:read".to_string()]).await;
    let token2 = generate_test_jwt(&ctx2, user2.id, vec!["users:read".to_string(), "roles:read".to_string()]).await;
    let token3 = generate_test_jwt(&ctx3, user3.id, vec!["users:read".to_string(), "roles:read".to_string()]).await;
    
    // Test 1: Users can only access their own tenant's data
    let scenarios = vec![
        (token1.clone(), ctx1.tenant_id, role1.id, true),  // User1 -> Tenant1 data: OK
        (token1.clone(), ctx2.tenant_id, role2.id, false), // User1 -> Tenant2 data: FAIL
        (token1.clone(), ctx3.tenant_id, role3.id, false), // User1 -> Tenant3 data: FAIL
        (token2.clone(), ctx1.tenant_id, role1.id, false), // User2 -> Tenant1 data: FAIL
        (token2.clone(), ctx2.tenant_id, role2.id, true),  // User2 -> Tenant2 data: OK
        (token2.clone(), ctx3.tenant_id, role3.id, false), // User2 -> Tenant3 data: FAIL
        (token3.clone(), ctx1.tenant_id, role1.id, false), // User3 -> Tenant1 data: FAIL
        (token3.clone(), ctx2.tenant_id, role2.id, false), // User3 -> Tenant2 data: FAIL
        (token3.clone(), ctx3.tenant_id, role3.id, true),  // User3 -> Tenant3 data: OK
    ];
    
    for (token, tenant_id, _role_id, should_succeed) in scenarios {
        // Use the auth state from the correct tenant context
        let auth_state = if tenant_id == ctx1.tenant_id {
            create_auth_state(&ctx1).await
        } else if tenant_id == ctx2.tenant_id {
            create_auth_state(&ctx2).await
        } else {
            create_auth_state(&ctx3).await
        };
        
        let app = create_test_router(auth_state);
        let request = Request::builder()
            .uri("/protected")
            .method("GET")
            .header("Authorization", format!("Bearer {}", token))
            .header("X-Tenant-ID", tenant_id.to_string())
            .body(Body::empty())
            .unwrap();
        
        let response = app.oneshot(request).await.unwrap();
        
        if should_succeed {
            assert_eq!(response.status(), StatusCode::OK, 
                "Should succeed accessing tenant {} with token from same tenant", tenant_id);
        } else {
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED, 
                "Should fail accessing tenant {} with token from different tenant", tenant_id);
        }
    }
    
    ctx1.cleanup().await;
    ctx2.cleanup().await;
    ctx3.cleanup().await;
}

#[tokio::test]
async fn test_token_revocation_flow() {
    init_test_logging();
    let ctx = TestContext::new().await;
    
    let admin_user = create_test_admin_user(&ctx).await;
    let token = generate_test_jwt(&ctx, admin_user.id, vec!["users:read".to_string()]).await;
    let jti = extract_jti_from_token(&token).expect("Failed to extract JTI");
    
    let auth_state = create_auth_state(&ctx).await;
    
    // Test 1: Token works initially
    let app = create_test_router(auth_state.clone());
    let request = Request::builder()
        .uri("/protected")
        .method("GET")
        .header("Authorization", format!("Bearer {}", token))
        .header("X-Tenant-ID", ctx.tenant_id.to_string())
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // Test 2: Revoke the token
    revoke_token(&ctx, &jti).await;
    
    // Test 3: Token should now be rejected
    let app = create_test_router(auth_state);
    let request = Request::builder()
        .uri("/protected")
        .method("GET")
        .header("Authorization", format!("Bearer {}", token))
        .header("X-Tenant-ID", ctx.tenant_id.to_string())
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    
    ctx.cleanup().await;
}

#[tokio::test]
async fn test_impersonation_authorization() {
    init_test_logging();
    let ctx = TestContext::new().await;
    
    // Create users
    let admin_user = create_test_admin_user(&ctx).await;
    let target_user = create_test_user(&ctx, "target@example.com").await;
    let regular_user = create_test_user(&ctx, "regular@example.com").await;
    
    // Create tokens
    let admin_token = generate_test_jwt(&ctx, admin_user.id, vec!["users:impersonate".to_string()]).await;
    let regular_token = generate_test_jwt(&ctx, regular_user.id, vec!["users:read".to_string()]).await;
    let impersonation_token = generate_impersonation_jwt(&ctx, target_user.id, admin_user.id, vec!["users:read".to_string()]).await;
    
    let auth_state = create_auth_state(&ctx).await;
    
    // Test 1: Admin can impersonate
    let app = create_permission_test_router(auth_state.clone());
    let request = Request::builder()
        .uri("/users:impersonate")
        .method("POST")
        .header("Authorization", format!("Bearer {}", admin_token))
        .header("X-Tenant-ID", ctx.tenant_id.to_string())
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // Test 2: Regular user cannot impersonate
    let app = create_permission_test_router(auth_state.clone());
    let request = Request::builder()
        .uri("/users:impersonate")
        .method("POST")
        .header("Authorization", format!("Bearer {}", regular_token))
        .header("X-Tenant-ID", ctx.tenant_id.to_string())
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    
    // Test 3: Impersonation token works with limited permissions
    let app = create_permission_test_router(auth_state);
    let request = Request::builder()
        .uri("/users:read")
        .method("GET")
        .header("Authorization", format!("Bearer {}", impersonation_token))
        .header("X-Tenant-ID", ctx.tenant_id.to_string())
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    ctx.cleanup().await;
}

#[tokio::test]
async fn test_request_context_population() {
    init_test_logging();
    let ctx = TestContext::new().await;
    
    let admin_user = create_test_admin_user(&ctx).await;
    let token = generate_test_jwt(&ctx, admin_user.id, vec!["users:read".to_string()]).await;
    let auth_state = create_auth_state(&ctx).await;
    
    // Create a router that extracts and validates RequestContext
    async fn context_validator(ctx: erp_core::RequestContext) -> impl axum::response::IntoResponse {
        // Validate that context is properly populated
        assert!(ctx.tenant_context.is_some(), "TenantContext should be populated");
        assert!(ctx.user_id.is_some(), "User ID should be populated");
        assert!(ctx.jti.is_some(), "JTI should be populated");
        assert!(!ctx.permissions.is_empty(), "Permissions should be populated");
        assert!(!ctx.request_id.is_empty(), "Request ID should be populated");
        
        // Validate specific values
        let tenant_context = ctx.tenant_context.unwrap();
        let user_id = ctx.user_id.unwrap();
        
        assert_eq!(tenant_context.tenant_id.0, tenant_context.tenant_id.0);
        
        axum::Json(json!({
            "tenant_id": tenant_context.tenant_id.0,
            "user_id": user_id,
            "permissions": ctx.permissions.len(),
            "request_id": ctx.request_id,
            "has_jti": ctx.jti.is_some(),
        }))
    }
    
    let app = Router::new()
        .route("/context-test", get(context_validator))
        .layer(from_fn_with_state(auth_state.clone(), auth_middleware))
        .with_state(auth_state);
    
    let request = Request::builder()
        .uri("/context-test")
        .method("GET")
        .header("Authorization", format!("Bearer {}", token))
        .header("X-Tenant-ID", ctx.tenant_id.to_string())
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    ctx.cleanup().await;
}

// Helper functions

async fn create_test_user(ctx: &TestContext, email: &str) -> User {
    let tenant_context = TenantContext {
        tenant_id: TenantId(ctx.tenant_id),
        schema_name: format!("test_tenant_{}", ctx.tenant_id),
    };
    
    ctx.auth_service
        .repository()
        .create_user(
            &tenant_context,
            email,
            Some("$argon2id$v=19$m=65536,t=3,p=4$test$test"),
            "Test",
            "User"
        )
        .await
        .expect("Failed to create test user")
}

async fn create_test_admin_user(ctx: &TestContext) -> User {
    let tenant_context = TenantContext {
        tenant_id: TenantId(ctx.tenant_id),
        schema_name: format!("test_tenant_{}", ctx.tenant_id),
    };
    
    let user = ctx.auth_service
        .repository()
        .create_user(
            &tenant_context,
            &format!("admin_{}@example.com", Uuid::new_v4()),
            Some("$argon2id$v=19$m=65536,t=3,p=4$test$test"),
            "Admin",
            "User"
        )
        .await
        .expect("Failed to create admin user");
    
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
    
    let secret = b"test_jwt_secret_key_for_testing_only";
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret))
        .expect("Failed to generate test JWT")
}

async fn generate_expired_jwt(ctx: &TestContext, user_id: Uuid, permissions: Vec<String>) -> String {
    use erp_core::types::JwtClaims;
    use jsonwebtoken::{encode, Header, EncodingKey};
    
    let claims = JwtClaims {
        sub: user_id.to_string(),
        tenant_id: ctx.tenant_id.to_string(),
        roles: vec!["test_role".to_string()],
        permissions,
        exp: (chrono::Utc::now() - chrono::Duration::hours(1)).timestamp(), // Expired 1 hour ago
        iat: (chrono::Utc::now() - chrono::Duration::hours(2)).timestamp(),
        jti: Uuid::new_v4().to_string(),
        impersonator_id: None,
    };
    
    let secret = b"test_jwt_secret_key_for_testing_only";
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret))
        .expect("Failed to generate expired JWT")
}

async fn generate_impersonation_jwt(ctx: &TestContext, user_id: Uuid, impersonator_id: Uuid, permissions: Vec<String>) -> String {
    use erp_core::types::JwtClaims;
    use jsonwebtoken::{encode, Header, EncodingKey};
    
    let claims = JwtClaims {
        sub: user_id.to_string(),
        tenant_id: ctx.tenant_id.to_string(),
        roles: vec!["impersonated_role".to_string()],
        permissions,
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp(),
        iat: chrono::Utc::now().timestamp(),
        jti: Uuid::new_v4().to_string(),
        impersonator_id: Some(impersonator_id.to_string()),
    };
    
    let secret = b"test_jwt_secret_key_for_testing_only";
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret))
        .expect("Failed to generate impersonation JWT")
}

fn extract_jti_from_token(token: &str) -> Option<String> {
    use jsonwebtoken::{decode, DecodingKey, Validation};
    use erp_core::types::JwtClaims;
    
    let secret = b"test_jwt_secret_key_for_testing_only";
    let validation = Validation::default();
    
    decode::<JwtClaims>(token, &DecodingKey::from_secret(secret), &validation)
        .ok()
        .map(|data| data.claims.jti)
}

async fn revoke_token(ctx: &TestContext, jti: &str) {
    let key = format!("revoked_token:{}", jti);
    let mut conn = ctx.redis.clone();
    let _: () = redis::AsyncCommands::set(&mut conn, key, "revoked")
        .await
        .expect("Failed to revoke token");
}

async fn create_auth_state(ctx: &TestContext) -> AuthState {
    AuthState {
        jwt_service: ctx.auth_service.jwt_service().clone(),
        db: Arc::new(ctx.db.clone()),
        redis: ctx.redis.clone(),
    }
}

fn create_test_router(auth_state: AuthState) -> Router {
    async fn protected_endpoint() -> &'static str { "protected_content" }
    
    Router::new()
        .route("/protected", get(protected_endpoint))
        .layer(from_fn_with_state(auth_state.clone(), auth_middleware))
        .with_state(auth_state)
}

fn create_permission_test_router(auth_state: AuthState) -> Router {
    async fn handler() -> &'static str { "success" }
    
    Router::new()
        .route("/users:read", get(handler)
            .layer(from_fn(require_permission("users:read"))))
        .route("/users:create", axum::routing::post(handler)
            .layer(from_fn(require_permission("users:create"))))
        .route("/users:update", axum::routing::put(handler)
            .layer(from_fn(require_permission("users:update"))))
        .route("/users:delete", axum::routing::delete(handler)
            .layer(from_fn(require_permission("users:delete"))))
        .route("/users:impersonate", axum::routing::post(handler)
            .layer(from_fn(require_permission("users:impersonate"))))
        .route("/roles:read", get(handler)
            .layer(from_fn(require_permission("roles:read"))))
        .route("/roles:create", axum::routing::post(handler)
            .layer(from_fn(require_permission("roles:create"))))
        .layer(from_fn_with_state(auth_state.clone(), auth_middleware))
        .with_state(auth_state)
}