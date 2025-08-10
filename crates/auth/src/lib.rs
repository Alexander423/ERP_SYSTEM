pub mod models;
pub mod repository;
pub mod service;
pub mod handlers;
pub mod middleware;
pub mod dto;
pub mod openapi;
pub mod email;
pub mod tokens;
pub mod workflows;
pub mod validation;

pub use models::*;
pub use repository::{AuthRepository, UserRepository};
pub use service::{AuthService, LoginOrTwoFactorResponse};
pub use handlers::SharedAuthService;
pub use middleware::{auth_middleware, require_permission, AuthState};
pub use openapi::AuthApiDoc;
pub use email::{EmailService, EmailTemplate};
pub use tokens::{TokenManager, TokenPurpose, TokenData};
pub use workflows::{PasswordResetWorkflow, EmailVerificationWorkflow, PasswordResetConfig, EmailVerificationConfig};

#[cfg(test)]
mod tests;

#[cfg(test)]
mod unit_tests;