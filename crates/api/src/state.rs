use erp_auth::AuthService;
use erp_core::{Config, DatabasePool};
use redis::aio::ConnectionManager;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db: DatabasePool,
    pub redis: ConnectionManager,
    pub auth_service: Arc<AuthService>,
}