use erp_auth::AuthService;
use erp_core::{Config, DatabasePool, TenantContext};
use erp_master_data::customer::repository::{CustomerRepository, PostgresCustomerRepository};
use redis::aio::ConnectionManager;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db: DatabasePool,
    pub redis: ConnectionManager,
    pub auth_service: Arc<AuthService>,
}

impl AppState {
    /// Create a CustomerRepository for a specific tenant context
    pub fn customer_repository(&self, tenant_context: TenantContext) -> Box<dyn CustomerRepository> {
        Box::new(PostgresCustomerRepository::new(self.db.main_pool.clone(), tenant_context))
    }
}