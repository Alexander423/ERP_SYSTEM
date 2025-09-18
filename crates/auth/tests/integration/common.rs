use erp_auth::{AuthService, AuthRepository};
use erp_core::{config::Config, DatabasePool};
use redis::aio::ConnectionManager;
use std::sync::Arc;
use uuid::Uuid;

pub struct TestContext {
    pub auth_service: Arc<AuthService>,
    pub tenant_id: Uuid,
    pub db: DatabasePool,
    pub redis: ConnectionManager,
}

impl TestContext {
    pub async fn new() -> Self {
        // Set test environment variables
        std::env::set_var("ENVIRONMENT", "testing");
        std::env::set_var("DATABASE_URL", "postgresql://erp_admin:erp_secure_password_change_in_production@localhost:5432/erp_main");
        std::env::set_var("REDIS_URL", "redis://:redis_secure_password_change_in_production@localhost:6379/1");
        std::env::set_var("JWT_SECRET", "test-jwt-secret-key-for-testing-only-32-chars-minimum");
        std::env::set_var("AES_ENCRYPTION_KEY", "test-encryption-key-exactly-32-ch");

        let config = Config::load().expect("Failed to load test config");

        // Initialize database
        let db = DatabasePool::new(config.database.clone())
            .await
            .expect("Failed to connect to test database");

        // Initialize Redis
        let redis_client = redis::Client::open(config.redis.url.as_str())
            .expect("Failed to create Redis client");
        let redis = ConnectionManager::new(redis_client)
            .await
            .expect("Failed to connect to Redis");

        // Initialize AuthService
        let auth_service = Arc::new(
            AuthService::new(db.clone(), redis.clone(), config)
                .await
                .expect("Failed to initialize AuthService")
        );

        // Create test tenant
        let repository = AuthRepository::new(db.clone());
        let tenant_name = format!("test_company_{}", Uuid::new_v4());
        let schema_name = format!("test_tenant_{}", Uuid::new_v4().to_string().replace('-', "_"));

        let tenant = repository
            .create_tenant(&tenant_name, &schema_name)
            .await
            .expect("Failed to create test tenant");

        Self {
            auth_service,
            tenant_id: tenant.id,
            db,
            redis,
        }
    }

    pub async fn cleanup(&self) {
        // Clean up test data
        // In a real implementation, you'd clean up the test tenant and all related data
        // For now, we'll just clear Redis keys
        let mut conn = self.redis.clone();
        let _: () = redis::cmd("FLUSHDB").query_async(&mut conn).await.unwrap_or(());
    }
}

#[cfg(test)]
pub fn init_test_logging() {
    use tracing_subscriber::{EnvFilter, FmtSubscriber};
    
    let _ = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env().add_directive("auth=debug".parse().unwrap()))
        .with_test_writer()
        .try_init();
}

pub async fn create_auth_service() -> Arc<AuthService> {
    let ctx = TestContext::new().await;
    ctx.auth_service
}

pub async fn setup_test_db() -> DatabasePool {
    let ctx = TestContext::new().await;
    ctx.db
}