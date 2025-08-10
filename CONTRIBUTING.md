# 🤝 Contributing to ERP System

Thank you for your interest in contributing to the ERP System! This document provides guidelines and information for contributors.

## 📋 Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Code Standards](#code-standards)
- [Testing Requirements](#testing-requirements)
- [Documentation Standards](#documentation-standards)
- [Submitting Changes](#submitting-changes)
- [Security](#security)

## Code of Conduct

This project follows a professional code of conduct:

- **Be respectful**: Treat all contributors with respect and professionalism
- **Be inclusive**: Welcome newcomers and help them get started
- **Be constructive**: Provide helpful feedback and suggestions
- **Focus on the code**: Keep discussions technical and objective
- **Security first**: Never commit secrets or introduce security vulnerabilities

## 🚀 Getting Started

### Prerequisites

Ensure you have the following installed:
- **Rust** (stable, 1.70+): https://rustup.rs/
- **Docker & Docker Compose**: https://docker.com/
- **Git**: https://git-scm.com/
- **PostgreSQL client tools** (optional): For database management

### Initial Setup

1. **Fork and Clone**
   ```bash
   git clone https://github.com/your-username/erp-system.git
   cd erp-system
   ```

2. **Environment Setup**
   ```bash
   # Copy environment template
   cp .env.example .env
   
   # Generate secure secrets
   JWT_SECRET=$(openssl rand -base64 32)
   AES_KEY=$(openssl rand -base64 32 | cut -c1-32)
   
   # Update .env with generated secrets
   ```

3. **Infrastructure Setup**
   ```bash
   # Start PostgreSQL and Redis
   docker-compose up -d postgres redis
   
   # Optional: Start pgAdmin for DB management
   docker-compose --profile debug up -d pgadmin
   ```

4. **Build and Test**
   ```bash
   # Install dependencies and build
   cargo build
   
   # Run tests to ensure everything works
   cargo test
   
   # Start the development server
   cargo run --bin erp-api
   ```

5. **Verify Setup**
   ```bash
   # Test health endpoint
   curl http://localhost:3000/health
   
   # Open API documentation
   open http://localhost:3000/swagger-ui
   ```

## 🔄 Development Workflow

### Branching Strategy

We use a feature-branch workflow:

```
main (production-ready)
├── feature/auth-improvements
├── feature/user-management
├── bugfix/password-reset-issue
└── docs/api-documentation
```

### Branch Naming Convention

- `feature/` - New features or enhancements
- `bugfix/` - Bug fixes
- `hotfix/` - Critical production fixes
- `docs/` - Documentation improvements
- `refactor/` - Code refactoring without functional changes

### Creating a Feature

1. **Create Feature Branch**
   ```bash
   git checkout main
   git pull origin main
   git checkout -b feature/your-feature-name
   ```

2. **Implement Changes**
   - Write code following our [Code Standards](#code-standards)
   - Add/update tests for your changes
   - Update documentation as needed
   - Ensure all tests pass

3. **Pre-submission Checks**
   ```bash
   # Format code
   cargo fmt --all
   
   # Check for linting issues
   cargo clippy --all-targets -- -D warnings
   
   # Run all tests
   cargo test --all
   
   # Security audit
   cargo audit
   
   # Check documentation builds
   cargo doc --no-deps
   ```

## 📝 Code Standards

### Rust Code Guidelines

#### **Formatting**
- Use `cargo fmt` for consistent formatting
- Line length: 100 characters maximum
- Use 4 spaces for indentation (no tabs)

#### **Naming Conventions**
```rust
// Modules and files: snake_case
mod user_service;

// Types (structs, enums): PascalCase
struct UserAccount;
enum AccountStatus;

// Functions and variables: snake_case
fn create_user_account() -> Result<UserAccount, Error>;
let user_email = "user@example.com";

// Constants: SCREAMING_SNAKE_CASE
const MAX_LOGIN_ATTEMPTS: u32 = 5;
```

#### **Error Handling**
```rust
// Always use structured error handling
use erp_core::{Error, Result};

// Return Result<T, Error> for fallible operations
pub fn authenticate_user(credentials: &LoginCredentials) -> Result<User> {
    // Implementation
}

// Use ? operator for error propagation
let user = repository.find_user(&email)?;
```

#### **Documentation Requirements**
```rust
/// Authenticates a user with the provided credentials.
/// 
/// This function performs the following steps:
/// 1. Validates the email format and password strength
/// 2. Retrieves the user from the database
/// 3. Verifies the password using Argon2id
/// 4. Checks if 2FA is enabled and required
/// 
/// # Arguments
/// * `credentials` - The login credentials (email and password)
/// * `tenant_id` - The tenant identifier for multi-tenant isolation
/// 
/// # Returns
/// Returns `Ok(AuthResult)` on successful authentication, which may require 2FA.
/// Returns `Err(Error)` for invalid credentials or system errors.
/// 
/// # Examples
/// ```rust
/// use erp_auth::{AuthService, LoginCredentials};
/// 
/// let credentials = LoginCredentials {
///     email: "user@example.com".to_string(),
///     password: "secure_password".to_string(),
/// };
/// let result = auth_service.authenticate(&credentials, tenant_id).await?;
/// ```
/// 
/// # Security Considerations
/// This function implements rate limiting and account lockout protection.
/// Failed attempts are logged for security monitoring.
pub async fn authenticate(
    &self,
    credentials: &LoginCredentials,
    tenant_id: Uuid,
) -> Result<AuthResult> {
    // Implementation
}
```

### Architecture Patterns

#### **Dependency Injection**
```rust
// Services should accept dependencies through constructors
pub struct AuthService {
    repository: Arc<dyn UserRepository>,
    token_manager: Arc<TokenManager>,
    audit_logger: Arc<dyn AuditLogger>,
}

impl AuthService {
    pub fn new(
        repository: Arc<dyn UserRepository>,
        token_manager: Arc<TokenManager>,
        audit_logger: Arc<dyn AuditLogger>,
    ) -> Self {
        Self {
            repository,
            token_manager,
            audit_logger,
        }
    }
}
```

#### **Error Context**
```rust
use erp_core::error::ErrorContext;

// Add context to errors for better debugging
user_repository
    .create_user(user_data)
    .await
    .with_context(|| format!("Failed to create user with email: {}", email))?;
```

## 🧪 Testing Requirements

### Test Categories

#### **Unit Tests**
- Test individual functions and methods
- Mock external dependencies
- Focus on business logic correctness

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;

    mock! {
        UserRepo {}
        
        #[async_trait::async_trait]
        impl UserRepository for UserRepo {
            async fn find_by_email(&self, email: &str) -> Result<Option<User>>;
            async fn create_user(&self, user: CreateUserRequest) -> Result<User>;
        }
    }

    #[tokio::test]
    async fn test_authenticate_valid_user() {
        let mut mock_repo = MockUserRepo::new();
        mock_repo
            .expect_find_by_email()
            .with(eq("user@example.com"))
            .returning(|_| Ok(Some(create_test_user())));

        let auth_service = AuthService::new(Arc::new(mock_repo), /* other deps */);
        let result = auth_service.authenticate(&valid_credentials(), tenant_id).await;

        assert!(result.is_ok());
    }
}
```

#### **Integration Tests**
- Test complete workflows end-to-end
- Use real database (test environment)
- Test API endpoints with actual HTTP requests

```rust
// In tests/integration/auth_test.rs
use erp_auth::testing::TestApp;

#[tokio::test]
async fn test_registration_to_login_workflow() {
    let app = TestApp::new().await;
    
    // Test registration
    let registration_response = app
        .register_tenant(create_registration_request())
        .await
        .expect("Registration should succeed");
    
    // Test login
    let login_response = app
        .login(create_login_request())
        .await
        .expect("Login should succeed");
    
    assert!(login_response.access_token.is_some());
}
```

### Test Coverage Requirements

- **Minimum Coverage**: 80% for new code
- **Critical Paths**: 95%+ coverage for authentication, security, payment flows
- **Documentation**: All test functions must have clear descriptions

```bash
# Run tests with coverage
cargo tarpaulin --out Html
open tarpaulin-report.html
```

### Test Database Setup

```bash
# Integration tests use separate test database
ENVIRONMENT=testing cargo test --test integration
```

## 📚 Documentation Standards

### Rustdoc Requirements

All public APIs must have comprehensive documentation:

```rust
/// Service for managing user authentication and authorization.
/// 
/// The `AuthService` handles all authentication-related operations including:
/// - User login and logout
/// - Token management (JWT access and refresh tokens)
/// - Two-factor authentication (TOTP)
/// - Password reset workflows
/// - Account lockout protection
/// 
/// # Thread Safety
/// This service is designed to be used across multiple async tasks safely.
/// All methods are `async` and use interior mutability where needed.
/// 
/// # Error Handling
/// All methods return `Result<T, Error>` where `Error` provides structured
/// error information with context for debugging and user feedback.
pub struct AuthService {
    // fields...
}
```

### API Documentation

- All API endpoints must be documented with OpenAPI annotations
- Include request/response examples
- Document error responses and status codes
- Specify required permissions for protected endpoints

### Architectural Documentation

When making significant architectural changes:
1. Update relevant documentation in `docs/` directory
2. Include architecture decision records (ADRs) for major decisions
3. Update the main README.md if user-facing changes are made

## 📬 Submitting Changes

### Pull Request Process

1. **Pre-submission Checklist**
   - [ ] All tests pass (`cargo test --all`)
   - [ ] Code is formatted (`cargo fmt --all --check`)
   - [ ] No linting errors (`cargo clippy --all-targets -- -D warnings`)
   - [ ] Documentation is updated
   - [ ] Security audit passes (`cargo audit`)

2. **Pull Request Template**
   ```markdown
   ## Description
   Brief description of changes and motivation.

   ## Type of Change
   - [ ] Bug fix (non-breaking change that fixes an issue)
   - [ ] New feature (non-breaking change that adds functionality)
   - [ ] Breaking change (fix or feature that breaks existing functionality)
   - [ ] Documentation update

   ## Testing
   - [ ] Unit tests added/updated
   - [ ] Integration tests added/updated
   - [ ] Manual testing performed

   ## Security Considerations
   - [ ] No secrets or credentials added
   - [ ] Input validation implemented
   - [ ] Authorization checks in place
   - [ ] Audit logging added where appropriate
   ```

3. **Review Process**
   - All PRs require at least one code review
   - Security-related changes require security team review
   - Large architectural changes require architecture review

### Commit Message Format

Follow conventional commit format:

```
type(scope): brief description

Detailed description of the change, including:
- What was changed
- Why it was changed
- Any breaking changes
- Related issue numbers

Closes #123
```

**Types**: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

**Examples**:
```
feat(auth): implement TOTP-based 2FA

Add two-factor authentication using TOTP (Time-based One-Time Password).
Users can now enable 2FA from their profile settings and will be required
to enter a TOTP code after password authentication.

- Add TOTP generation and verification
- Add 2FA setup and verification endpoints
- Update authentication flow to handle 2FA requirement
- Add comprehensive tests for 2FA workflows

Closes #45
```

## 🛡️ Security

### Security Guidelines

1. **Never commit secrets**
   - Use `.env` files for local secrets (never commit `.env`)
   - Use environment variables in production
   - Use `git-secrets` or similar tools to prevent accidental commits

2. **Input Validation**
   - Validate all user inputs server-side
   - Use type-safe deserialization with `serde`
   - Implement proper error handling for invalid inputs

3. **Authentication & Authorization**
   - Always verify user permissions before performing operations
   - Use structured permission checking
   - Log security-relevant events for auditing

4. **Database Security**
   - Use parameterized queries (SQLx provides this by default)
   - Implement proper tenant isolation
   - Never log sensitive data (passwords, tokens, etc.)

### Reporting Security Issues

**DO NOT open public issues for security vulnerabilities.**

Instead, please email security issues to: [security@yourcompany.com]

Include:
- Description of the vulnerability
- Steps to reproduce
- Potential impact assessment
- Suggested fix (if available)

We will respond within 24 hours and provide updates on our progress.

## ❓ Getting Help

### Communication Channels

- **GitHub Issues**: For bug reports and feature requests
- **Discussions**: For questions and general discussion
- **Code Reviews**: For implementation feedback

### Resources

- **Architecture Documentation**: `docs/ARCHITECTURE.md`
- **API Documentation**: http://localhost:3000/swagger-ui (when running locally)
- **Code Documentation**: `cargo doc --open`

### Common Issues

#### **Build Errors**
```bash
# Clear cache and rebuild
cargo clean
cargo build
```

#### **Database Issues**
```bash
# Reset test database
docker-compose down
docker-compose up -d postgres
cargo test --test integration
```

#### **Docker Issues**
```bash
# Reset all containers and volumes
docker-compose down -v
docker system prune -f
docker-compose up -d
```

---

Thank you for contributing to the ERP System! Your efforts help make this project better for everyone. 🎉