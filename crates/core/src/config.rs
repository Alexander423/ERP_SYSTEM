//! # Configuration Management System
//! 
//! This module provides a hierarchical configuration system for the ERP application.
//! It supports loading configuration from multiple sources in order of precedence:
//! 
//! 1. **Environment Variables** (highest precedence)
//! 2. **Environment-specific TOML files** (e.g., `config/production.toml`)
//! 3. **Default TOML file** (`config/default.toml`) (lowest precedence)
//! 
//! ## Usage
//! 
//! ```rust
//! use erp_core::Config;
//! 
//! // Load configuration (automatically detects environment)
//! let config = Config::load().expect("Failed to load configuration");
//! 
//! // Use configuration values
//! let db_url = &config.database.url;
//! let jwt_secret = &config.jwt.secret;
//! ```
//! 
//! ## Environment Selection
//! 
//! The configuration system automatically selects the appropriate environment
//! based on the `ENVIRONMENT` environment variable:
//! 
//! - `development` (default): Uses `config/development.toml`
//! - `testing`: Uses `config/testing.toml`  
//! - `production`: Uses `config/production.toml`
//! 
//! ## Security Considerations
//! 
//! - Sensitive values (passwords, secrets, API keys) should be provided via environment variables
//! - Never commit sensitive data to TOML configuration files
//! - Use strong, randomly generated secrets for production deployments
//! 
//! ## Configuration Categories
//! 
//! The configuration is organized into logical sections:
//! - **Database**: PostgreSQL connection and pool settings
//! - **Redis**: Caching and session store configuration  
//! - **JWT**: Token signing and expiry settings
//! - **Security**: Cryptographic parameters and keys
//! - **Server**: HTTP server and worker configuration
//! - **Email**: Multi-provider email service settings
//! - **CORS**: Cross-Origin Resource Sharing policies
//! - **Metrics**: Prometheus monitoring configuration
//! - **Rate Limiting**: Request throttling and protection

use config::{ConfigError, Environment, File};
use serde::Deserialize;
use std::env;

/// Main configuration structure containing all application settings.
/// 
/// This structure is automatically populated by loading configuration from
/// TOML files and environment variables. All fields are grouped into
/// logical categories for better organization and maintainability.
/// 
/// # Examples
/// 
/// ```rust
/// use erp_core::Config;
/// 
/// let config = Config::load()?;
/// println!("Database URL: {}", config.database.url);
/// println!("JWT secret length: {}", config.jwt.secret.len());
/// ```
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    /// Database connection and pool configuration
    pub database: DatabaseConfig,
    /// Redis caching and session store configuration
    pub redis: RedisConfig,
    /// JWT token signing and expiry configuration
    pub jwt: JwtConfig,
    /// Cryptographic and security parameters
    pub security: SecurityConfig,
    /// HTTP server and worker configuration
    pub server: ServerConfig,
    /// Rate limiting and throttling configuration
    pub rate_limit: RateLimitConfig,
    /// Email service provider configuration
    pub email: EmailConfig,
    /// Application-level settings and feature flags
    pub app: AppConfig,
    /// Prometheus metrics and monitoring configuration
    pub metrics: MetricsConfig,
    /// Cross-Origin Resource Sharing (CORS) policies
    pub cors: CorsConfig,
}

/// PostgreSQL database configuration and connection pool settings.
/// 
/// This configuration manages the database connection parameters and
/// connection pooling behavior for optimal performance and resource usage.
/// 
/// # Connection Pool Tuning
/// 
/// - **Development**: Lower connection limits for resource efficiency
/// - **Production**: Higher limits for concurrent request handling
/// - **Testing**: Minimal connections for isolated test execution
/// 
/// # Example Configuration
/// 
/// ```toml
/// [database]
/// url = "postgresql://user:pass@localhost:5432/erp_main"
/// max_connections = 20
/// min_connections = 5
/// ```
#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    /// PostgreSQL connection URL including credentials and database name.
    /// Format: `postgresql://username:password@host:port/database`
    /// 
    /// **Security Note**: In production, this should be provided via
    /// the `DATABASE_URL` environment variable, never in configuration files.
    pub url: String,
    
    /// Maximum number of database connections in the pool.
    /// 
    /// **Guidelines**:
    /// - Development: 5-10 connections
    /// - Production: 20-50 connections (based on server capacity)
    /// - Testing: 3-5 connections
    pub max_connections: u32,
    
    /// Minimum number of database connections to maintain in the pool.
    /// 
    /// Keeping a minimum number of connections reduces connection
    /// establishment latency during traffic bursts.
    pub min_connections: u32,
}

/// Redis configuration for caching and session storage.
/// 
/// Redis is used for:
/// - JWT refresh token storage and revocation
/// - Session management and user state
/// - Rate limiting counters
/// - Background job queue (planned)
/// 
/// # Example Configuration
/// 
/// ```toml
/// [redis]
/// url = "redis://:password@localhost:6379"
/// max_connections = 10
/// ```
#[derive(Debug, Deserialize, Clone)]
pub struct RedisConfig {
    /// Redis connection URL with optional authentication.
    /// Format: `redis://[:password@]host:port[/db]`
    /// 
    /// **Security Note**: Use strong passwords and consider TLS encryption
    /// for production deployments.
    pub url: String,
    
    /// Maximum number of Redis connections in the pool.
    /// 
    /// Redis connections are generally lightweight, but limiting
    /// the pool size prevents resource exhaustion under load.
    pub max_connections: u32,
}

/// JWT (JSON Web Token) configuration for authentication.
/// 
/// This configuration controls JWT token generation, validation,
/// and expiry behavior. The system uses both access tokens (short-lived)
/// and refresh tokens (longer-lived) for enhanced security.
/// 
/// # Security Considerations
/// 
/// - Use strong, randomly generated secrets (minimum 32 characters)
/// - Rotate secrets regularly in production
/// - Set appropriate expiry times for your security requirements
/// 
/// # Example Configuration
/// 
/// ```toml
/// [jwt]
/// secret = "your-super-secret-jwt-signing-key-min-32-chars"
/// access_token_expiry = 1800   # 30 minutes
/// refresh_token_expiry = 604800 # 7 days
/// ```
#[derive(Debug, Deserialize, Clone)]
pub struct JwtConfig {
    /// Secret key used for signing and verifying JWT tokens.
    /// 
    /// **Critical Security Requirement**: 
    /// - Must be at least 32 characters long
    /// - Should be cryptographically random
    /// - Must be provided via `JWT_SECRET` environment variable in production
    /// 
    /// Generate with: `openssl rand -base64 32`
    pub secret: String,
    
    /// Access token expiry time in seconds.
    /// 
    /// Access tokens are used for API authentication and should be short-lived
    /// for security. Typical values:
    /// - Development: 3600 (1 hour)
    /// - Production: 900-1800 (15-30 minutes)
    pub access_token_expiry: i64,
    
    /// Refresh token expiry time in seconds.
    /// 
    /// Refresh tokens are used to obtain new access tokens and can be longer-lived.
    /// They are stored securely and can be revoked. Typical values:
    /// - Development: 2592000 (30 days)
    /// - Production: 604800 (7 days)
    pub refresh_token_expiry: i64,
}

/// Security and cryptographic configuration.
/// 
/// This configuration controls password hashing parameters and encryption
/// settings. These values directly impact security strength and performance,
/// so they should be tuned based on your environment and security requirements.
/// 
/// # Argon2id Parameters
/// 
/// Argon2id is a memory-hard password hashing function that provides
/// excellent protection against both GPU and ASIC attacks.
/// 
/// # Example Configuration
/// 
/// ```toml
/// [security]
/// argon2_memory_cost = 65536  # 64 MB
/// argon2_time_cost = 3
/// argon2_parallelism = 2
/// aes_encryption_key = "your-32-char-encryption-key-here!"
/// ```
#[derive(Debug, Deserialize, Clone)]
pub struct SecurityConfig {
    /// Argon2id memory cost parameter (in KiB).
    /// 
    /// Controls the amount of memory used during password hashing.
    /// Higher values provide better security but require more resources.
    /// 
    /// **Recommended values**:
    /// - Development: 32768 (32 MB)
    /// - Production: 65536-131072 (64-128 MB)
    /// - Testing: 16384 (16 MB)
    pub argon2_memory_cost: u32,
    
    /// Argon2id time cost parameter (iterations).
    /// 
    /// Controls the number of iterations performed during hashing.
    /// Higher values increase computation time and security.
    /// 
    /// **Recommended values**:
    /// - Development: 2-3
    /// - Production: 3-4
    /// - Testing: 1
    pub argon2_time_cost: u32,
    
    /// Argon2id parallelism parameter (number of threads).
    /// 
    /// Controls how many threads are used for parallel computation.
    /// Should match server CPU capabilities.
    /// 
    /// **Recommended values**:
    /// - Development: 1-2
    /// - Production: 2-4
    /// - Testing: 1
    pub argon2_parallelism: u32,
    
    /// AES-GCM encryption key for sensitive data at rest.
    /// 
    /// Used for encrypting sensitive data before storing in the database.
    /// **Must be exactly 32 characters long**.
    /// 
    /// **Security Requirements**:
    /// - Must be provided via `AES_ENCRYPTION_KEY` environment variable
    /// - Should be cryptographically random
    /// - Must be rotated periodically
    /// 
    /// Generate with: `openssl rand -base64 32 | cut -c1-32`
    pub aes_encryption_key: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst_size: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EmailConfig {
    pub provider: String, // "mock", "smtp", "sendgrid", "aws_ses"
    pub smtp_host: Option<String>,
    pub smtp_port: Option<u16>,
    pub smtp_username: Option<String>,
    pub smtp_password: Option<String>,
    pub smtp_from_email: String,
    pub smtp_from_name: String,
    pub use_tls: bool,
    pub use_starttls: bool,
    pub timeout_seconds: u64,
    pub max_retries: u32,
    // Provider-specific configs
    pub sendgrid_api_key: Option<String>,
    pub aws_region: Option<String>,
    pub aws_access_key_id: Option<String>,
    pub aws_secret_access_key: Option<String>,
}

impl Default for EmailConfig {
    fn default() -> Self {
        Self {
            provider: "mock".to_string(),
            smtp_host: None,
            smtp_port: Some(587),
            smtp_username: None,
            smtp_password: None,
            smtp_from_email: "noreply@example.com".to_string(),
            smtp_from_name: "ERP System".to_string(),
            use_tls: true,
            use_starttls: false,
            timeout_seconds: 30,
            max_retries: 3,
            sendgrid_api_key: None,
            aws_region: None,
            aws_access_key_id: None,
            aws_secret_access_key: None,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub company_name: String,
    pub base_url: String,
    pub environment: String,
    pub log_level: String,
    pub enable_registration: bool,
    pub enable_2fa: bool,
    pub enable_email_verification: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub port: u16,
    pub path: String,
    pub namespace: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub expose_headers: Vec<String>,
    pub max_age: Option<u64>,
    pub allow_credentials: bool,
}

impl Config {
    /// Loads configuration from multiple sources in hierarchical order.
    /// 
    /// This method implements a layered configuration approach where values
    /// are loaded in order of increasing precedence:
    /// 
    /// 1. **Default configuration** (`config/default.toml`) - Base values
    /// 2. **Environment-specific configuration** (e.g., `config/production.toml`) - Environment overrides
    /// 3. **Environment variables** - Runtime overrides (highest precedence)
    /// 
    /// # Environment Detection
    /// 
    /// The environment is determined by the `ENVIRONMENT` environment variable:
    /// - If not set, defaults to "development"
    /// - Valid values: "development", "testing", "production"
    /// 
    /// # Configuration Sources
    /// 
    /// ## TOML Files
    /// Configuration files are loaded from the `config/` directory:
    /// - `config/default.toml` - Always loaded if present
    /// - `config/{environment}.toml` - Environment-specific overrides
    /// 
    /// ## Environment Variables
    /// Environment variables override TOML settings using underscore separation:
    /// - `DATABASE_URL` maps to `database.url`
    /// - `JWT_SECRET` maps to `jwt.secret`
    /// - `ARGON2_MEMORY_COST` maps to `security.argon2_memory_cost`
    /// 
    /// # Error Handling
    /// 
    /// Returns `ConfigError` if:
    /// - Configuration files contain invalid TOML syntax
    /// - Required environment variables are missing
    /// - Configuration values fail validation (e.g., invalid types)
    /// - Deserialization fails due to schema mismatches
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use erp_core::Config;
    /// 
    /// // Load configuration with default environment detection
    /// let config = Config::load()?;
    /// 
    /// // Environment variable overrides TOML values
    /// std::env::set_var("DATABASE_MAX_CONNECTIONS", "50");
    /// let config = Config::load()?;
    /// assert_eq!(config.database.max_connections, 50);
    /// ```
    /// 
    /// # Security Considerations
    /// 
    /// - Sensitive values should always be provided via environment variables
    /// - Never commit secrets to TOML configuration files
    /// - Use strong, randomly generated values for cryptographic keys
    /// - Validate that required secrets are present before starting the application
    /// 
    /// # Performance Notes
    /// 
    /// Configuration loading is designed to be called once at application startup.
    /// The resulting `Config` struct should be cloned and shared across the application.
    pub fn load() -> Result<Self, ConfigError> {
        let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
        
        let builder = config::Config::builder()
            // Load default configuration (lowest precedence)
            .add_source(File::with_name("config/default").required(false))
            // Load environment-specific configuration (medium precedence)
            .add_source(File::with_name(&format!("config/{}", environment)).required(false))
            // Load environment variables (highest precedence)
            .add_source(Environment::with_prefix("").separator("_"));

        let config = builder.build()?;
        let mut loaded_config: Config = config.try_deserialize()?;
        
        // Validate configuration and fail fast if critical values are missing
        loaded_config.validate(&environment)?;
        
        Ok(loaded_config)
    }
    
    /// Validates the loaded configuration and ensures critical security requirements are met.
    /// 
    /// This method performs comprehensive validation of configuration values, with special
    /// emphasis on security-critical settings. It implements environment-specific validation
    /// rules to prevent common misconfigurations.
    /// 
    /// # Validation Rules
    /// 
    /// ## Production Environment
    /// - JWT secret must be at least 32 characters and not contain error messages
    /// - AES encryption key must be exactly 32 characters and not contain error messages
    /// - Database and Redis URLs must not contain error messages
    /// - SMTP configuration must be properly set for email functionality
    /// 
    /// ## All Environments
    /// - JWT secret minimum length validation
    /// - AES key length validation
    /// - Database connection string format validation
    /// - Token expiry time sanity checks
    /// 
    /// # Security Validation
    /// 
    /// The method specifically checks for:
    /// - Default/insecure passwords and keys
    /// - Missing environment variable indicators
    /// - Weak cryptographic parameters
    /// - Misconfigured external service endpoints
    /// 
    /// # Error Handling
    /// 
    /// Returns `ConfigError::Message` with descriptive error messages for:
    /// - Missing required environment variables
    /// - Insecure default values in production
    /// - Invalid configuration value formats
    /// - Security policy violations
    fn validate(&mut self, environment: &str) -> Result<(), ConfigError> {
        use config::ConfigError;
        
        // Validate JWT secret
        if self.jwt.secret.len() < 32 {
            return Err(ConfigError::Message(format!(
                "JWT secret must be at least 32 characters long (current: {})", 
                self.jwt.secret.len()
            )));
        }
        
        // Validate AES encryption key
        if self.security.aes_encryption_key.len() != 32 {
            return Err(ConfigError::Message(format!(
                "AES encryption key must be exactly 32 characters long (current: {})", 
                self.security.aes_encryption_key.len()
            )));
        }
        
        // Environment-specific validation
        if environment == "production" {
            self.validate_production_security()?;
        }
        
        // Validate database URL format
        if !self.database.url.starts_with("postgresql://") {
            return Err(ConfigError::Message(
                "Database URL must be a PostgreSQL connection string starting with 'postgresql://'".to_string()
            ));
        }
        
        // Validate Redis URL format
        if !self.redis.url.starts_with("redis://") {
            return Err(ConfigError::Message(
                "Redis URL must be a Redis connection string starting with 'redis://'".to_string()
            ));
        }
        
        // Validate token expiry times
        if self.jwt.access_token_expiry <= 0 || self.jwt.access_token_expiry > 86400 {
            return Err(ConfigError::Message(
                "Access token expiry must be between 1 second and 24 hours".to_string()
            ));
        }
        
        if self.jwt.refresh_token_expiry <= self.jwt.access_token_expiry {
            return Err(ConfigError::Message(
                "Refresh token expiry must be longer than access token expiry".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Validates production-specific security requirements.
    /// 
    /// This method enforces strict security policies for production deployments,
    /// ensuring that no default or insecure values are used in production environments.
    /// 
    /// # Production Security Checks
    /// 
    /// - All critical secrets must be provided via environment variables
    /// - No default/placeholder values are allowed
    /// - Strong cryptographic parameters are enforced
    /// - External service configurations must be complete
    /// 
    /// # Fail-Fast Approach
    /// 
    /// This method implements a fail-fast approach where any security violation
    /// causes the application to refuse to start, preventing insecure deployments.
    fn validate_production_security(&self) -> Result<(), ConfigError> {
        use config::ConfigError;
        
        // Check for error messages indicating missing environment variables
        let error_indicators = [
            "ERROR_", "INSECURE_DEFAULT", "CHANGE_THIS", "NOT_SET", 
            "CHECK_ENVIRONMENT", "PLACEHOLDER"
        ];
        
        // Validate JWT secret
        for indicator in &error_indicators {
            if self.jwt.secret.contains(indicator) {
                return Err(ConfigError::Message(format!(
                    "Production deployment detected insecure JWT secret. Set JWT_SECRET environment variable. Current value contains: {}", 
                    indicator
                )));
            }
        }
        
        // Validate AES encryption key  
        for indicator in &error_indicators {
            if self.security.aes_encryption_key.contains(indicator) {
                return Err(ConfigError::Message(format!(
                    "Production deployment detected insecure AES encryption key. Set AES_ENCRYPTION_KEY environment variable. Current value contains: {}", 
                    indicator
                )));
            }
        }
        
        // Validate database URL
        for indicator in &error_indicators {
            if self.database.url.contains(indicator) {
                return Err(ConfigError::Message(format!(
                    "Production deployment detected missing database configuration. Set DATABASE_URL environment variable. Current value contains: {}", 
                    indicator
                )));
            }
        }
        
        // Validate Redis URL
        for indicator in &error_indicators {
            if self.redis.url.contains(indicator) {
                return Err(ConfigError::Message(format!(
                    "Production deployment detected missing Redis configuration. Set REDIS_URL environment variable. Current value contains: {}", 
                    indicator
                )));
            }
        }
        
        // Validate base URL
        for indicator in &error_indicators {
            if self.app.base_url.contains(indicator) {
                return Err(ConfigError::Message(format!(
                    "Production deployment detected missing base URL. Set BASE_URL environment variable. Current value contains: {}", 
                    indicator
                )));
            }
        }
        
        // Validate SMTP configuration if using SMTP provider
        if self.email.provider == "smtp" {
            if let Some(ref smtp_host) = self.email.smtp_host {
                for indicator in &error_indicators {
                    if smtp_host.contains(indicator) {
                        return Err(ConfigError::Message(format!(
                            "Production deployment detected missing SMTP host. Set SMTP_HOST environment variable. Current value contains: {}", 
                            indicator
                        )));
                    }
                }
            }
        }
        
        // Production-specific security parameter validation
        if self.security.argon2_memory_cost < 65536 {
            return Err(ConfigError::Message(
                "Production deployment requires Argon2 memory cost of at least 65536 (64 MB)".to_string()
            ));
        }
        
        if self.security.argon2_time_cost < 3 {
            return Err(ConfigError::Message(
                "Production deployment requires Argon2 time cost of at least 3".to_string()
            ));
        }
        
        // Validate that public registration is disabled in production
        if self.app.enable_registration {
            return Err(ConfigError::Message(
                "Production deployment should not allow public registration. Set enable_registration to false or set ENABLE_REGISTRATION=false".to_string()
            ));
        }
        
        // Validate CORS configuration for production
        for origin in &self.cors.allowed_origins {
            for indicator in &error_indicators {
                if origin.contains(indicator) {
                    return Err(ConfigError::Message(format!(
                        "Production deployment detected missing CORS origin. Set FRONTEND_URL environment variable. Current value contains: {}", 
                        indicator
                    )));
                }
            }
            
            // Check for wildcards in production CORS
            if origin == "*" {
                return Err(ConfigError::Message(
                    "Production deployment must not use wildcard (*) CORS origins. Set specific frontend URL via FRONTEND_URL environment variable".to_string()
                ));
            }
        }
        
        Ok(())
    }
}