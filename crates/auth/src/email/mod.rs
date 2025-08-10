pub mod jobs;
pub mod service;
pub mod templates;

pub use jobs::{EmailJob, EmailJobData};
pub use service::EmailService;
pub use erp_core::config::EmailConfig;
pub use templates::{EmailTemplate, VerificationEmailTemplate, PasswordResetEmailTemplate, WelcomeEmailTemplate};