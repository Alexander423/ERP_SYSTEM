# 🤝 Contributing to ERP System

**Project Status**: ⚠️ **Early Development Stage** - Foundation layer implementation

Thank you for your interest in contributing to the ERP System! This document provides **realistic** guidelines based on the current state of the project.

## 📋 Table of Contents

- [Project Reality Check](#project-reality-check)
- [Getting Started](#getting-started)
- [Current Development Focus](#current-development-focus)
- [Code Standards](#code-standards)
- [Testing Approach](#testing-approach)
- [Submitting Changes](#submitting-changes)
- [Development Priorities](#development-priorities)

## 🔍 Project Reality Check

### What This Project Actually Is

- **Early Alpha**: Foundation infrastructure with basic functionality
- **Learning Project**: Good for learning Rust, PostgreSQL, and API design
- **Not Production Ready**: Mock implementations throughout
- **Active Development**: Core features being actively implemented

### What You Can Contribute To

✅ **Currently Accepting Contributions For:**
- Infrastructure improvements (build system, configuration)
- Test coverage expansion
- Documentation improvements
- Code quality enhancements (error handling, logging)
- API-Repository integration
- Basic authentication implementation

❌ **Not Ready For Contributions:**
- Advanced ERP features (financial modules, complex workflows)
- Frontend development (no UI exists)
- Production deployment (not production-ready)
- Complex multi-tenancy (architecture exists, runtime doesn't)

## 🚀 Getting Started

### Prerequisites

**Required:**
- **Rust** (1.70+): https://rustup.rs/
- **Docker & Docker Compose**: For PostgreSQL/Redis
- **Git**: Version control
- **Basic SQL knowledge**: For database work

**Optional but Helpful:**
- **PostgreSQL client** (psql, pgAdmin)
- **REST API client** (curl, Postman, Insomnia)

### Local Development Setup

1. **Clone and Setup**
   ```bash
   git clone <repository-url>
   cd ERP

   # Check that Rust toolchain works
   cargo --version
   rustc --version
   ```

2. **Start Infrastructure**
   ```bash
   # Start PostgreSQL and Redis via Docker
   docker-compose up -d

   # Verify containers are running
   docker-compose ps
   ```

3. **Configure Environment**
   ```bash
   # Set required environment variables
   export DATABASE_URL="postgresql://erp_admin:erp_secure_password_change_in_production@localhost:5432/erp_main"
   export REDIS_URL="redis://localhost:6379"
   export JWT_SECRET="your-32-char-secret-here-change-me"
   export AES_ENCRYPTION_KEY="your-exactly-32-character-key-here!"
   ```

4. **Build and Run**
   ```bash
   # Build all crates
   cargo build --all

   # Run database migrations
   DATABASE_URL="postgresql://..." cargo sqlx migrate run

   # Start the API server
   DATABASE_URL="postgresql://..." cargo run -p erp-api
   ```

5. **Verify Everything Works**
   ```bash
   # Test health endpoint
   curl http://localhost:3000/health

   # Should return: {"status":"healthy","timestamp":"..."}
   ```

### What Should Work After Setup

✅ **Expected to work:**
- Build system compiles without errors
- HTTP server starts and responds to health checks
- Database connection established
- Basic API endpoints accessible

⚠️ **Known limitations:**
- Most API endpoints return mock data
- Authentication is placeholder implementation
- Tests require specific database setup
- Some features are architectural stubs

## 🎯 Current Development Focus

### Priority Areas (December 2024)

1. **API-Repository Integration** (High Priority)
   - Connect HTTP handlers to repository layer
   - Implement real customer CRUD operations
   - Add proper error handling

2. **Authentication Implementation** (High Priority)
   - Real JWT token validation
   - Password verification against database
   - Session management basics

3. **Testing Infrastructure** (Medium Priority)
   - Expand integration test coverage
   - Mock service implementations for testing
   - Test database setup automation

4. **Code Quality** (Medium Priority)
   - Improve error handling consistency
   - Add structured logging
   - Configuration validation

### Areas to Avoid (For Now)

❌ **Don't work on these yet:**
- Multi-tenant runtime switching
- Advanced ERP business logic
- Performance optimizations
- Complex security features
- Frontend/UI components

## 📝 Code Standards

### Rust Code Guidelines

#### **Basic Formatting**
```bash
# Use standard Rust formatting
cargo fmt --all

# Check for common issues
cargo clippy --all-targets

# Run tests
cargo test --all
```

#### **Error Handling Pattern**
```rust
use erp_core::{Error, Result};

// Always use structured error handling
pub async fn create_customer(data: CreateCustomerRequest) -> Result<Customer> {
    // Validate input
    if data.email.is_empty() {
        return Err(Error::validation("Email is required"));
    }

    // Call repository
    let customer = repository.create_customer(data).await?;

    Ok(customer)
}
```

#### **Documentation Requirements**
```rust
/// Creates a new customer in the system.
///
/// # Arguments
/// * `data` - Customer creation request with required fields
///
/// # Returns
/// * `Ok(Customer)` - Successfully created customer
/// * `Err(Error)` - Validation or database error
///
/// # Examples
/// ```rust
/// let request = CreateCustomerRequest {
///     email: "test@example.com".to_string(),
///     name: "Test Customer".to_string(),
/// };
/// let customer = service.create_customer(request).await?;
/// ```
pub async fn create_customer(&self, data: CreateCustomerRequest) -> Result<Customer> {
    // Implementation
}
```

### Database Guidelines

#### **Migration Files**
- Use descriptive names: `20241216_001_add_customer_table.sql`
- Include both UP and DOWN migrations
- Test migrations against clean database

#### **Repository Pattern**
```rust
#[async_trait]
pub trait CustomerRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Customer>>;
    async fn create(&self, data: CreateCustomerRequest) -> Result<Customer>;
    async fn update(&self, id: Uuid, data: UpdateCustomerRequest) -> Result<Customer>;
    async fn delete(&self, id: Uuid) -> Result<()>;
}
```

## 🧪 Testing Approach

### Current Testing Reality

**What Works:**
- Unit tests for individual functions
- Basic integration tests with TestContext
- Repository-level database tests

**What Needs Work:**
- HTTP endpoint integration tests
- Mock service implementations
- Test database reset between tests

### Writing Tests

#### **Unit Tests**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validate_email() {
        let result = validate_email("test@example.com");
        assert!(result.is_ok());

        let result = validate_email("invalid-email");
        assert!(result.is_err());
    }
}
```

#### **Integration Tests**
```rust
// In tests/ directory
use erp_core::testing::TestContext;

#[tokio::test]
async fn test_customer_crud() {
    let ctx = TestContext::new().await;
    let repo = ctx.customer_repository();

    // Test create
    let customer_data = CreateCustomerRequest {
        email: "test@example.com".to_string(),
        name: "Test Customer".to_string(),
    };

    let created = repo.create(customer_data).await
        .expect("Should create customer");

    // Test read
    let found = repo.find_by_id(created.id).await
        .expect("Should find customer");

    assert_eq!(found.unwrap().email, "test@example.com");
}
```

### Running Tests

```bash
# Unit tests (no database required)
cargo test --lib

# Integration tests (requires database)
DATABASE_URL="postgresql://..." cargo test --test integration

# All tests
DATABASE_URL="postgresql://..." cargo test --all
```

## 📬 Submitting Changes

### Before Submitting

**Required Checks:**
```bash
# 1. Code compiles
cargo build --all

# 2. Tests pass (where applicable)
cargo test --lib

# 3. Code is formatted
cargo fmt --all --check

# 4. No obvious issues
cargo clippy --all-targets
```

### Pull Request Process

1. **Small, Focused Changes**
   - One feature/fix per PR
   - Keep changes small and reviewable
   - Include tests for new functionality

2. **PR Description Template**
   ```markdown
   ## What This Changes
   Brief description of the change and why it's needed.

   ## Testing
   - [ ] Unit tests added/updated
   - [ ] Integration tests pass
   - [ ] Manually tested locally

   ## Checklist
   - [ ] Code compiles without warnings
   - [ ] Tests pass
   - [ ] Documentation updated (if needed)
   - [ ] No secrets or sensitive data added
   ```

3. **Review Process**
   - All PRs require at least one review
   - Focus on code quality and maintainability
   - Security-sensitive changes need extra review

### Commit Message Format

```
type: brief description (50 chars max)

Longer explanation of what this commit does and why.
Include any important details or context.

- Bullet points for multiple changes
- Reference issues with #123
```

**Types**: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`

## 🎯 Development Priorities

### Immediate Needs (Next 1-2 Months)

1. **Connect API to Repository Layer**
   - Customer CRUD endpoints should call repository
   - Proper error handling and validation
   - Remove mock responses

2. **Basic Authentication Flow**
   - JWT token validation in middleware
   - Login endpoint with real password checking
   - Session management basics

3. **Test Coverage**
   - API endpoint integration tests
   - Repository test coverage
   - Test database setup automation

### Medium Term (3-6 Months)

1. **Complete Customer Management**
   - Full customer lifecycle APIs
   - Data validation and business rules
   - Search and filtering

2. **Basic Multi-tenancy**
   - Tenant context in requests
   - Schema-per-tenant implementation
   - Tenant isolation testing

3. **Security Hardening**
   - Input validation throughout
   - Audit logging framework
   - Configuration security validation

### Areas Needing Research

- **API Design Patterns**: How to structure REST APIs effectively
- **Database Performance**: Query optimization and indexing strategies
- **Testing Strategies**: How to test async Rust web applications
- **Error Handling**: Consistent error handling across layers

## 🆘 Getting Help

### Communication

- **GitHub Issues**: For bug reports and feature requests
- **GitHub Discussions**: For questions and design discussions
- **Code Reviews**: For implementation feedback

### Resources

- **Project Status**: See [PROJECT_STATUS.md](../PROJECT_STATUS.md) for current state
- **Architecture**: See [docs/architecture/ARCHITECTURE.md](../architecture/ARCHITECTURE.md)
- **API Docs**: See [docs/api/API_DOKUMENTATION.md](../api/API_DOKUMENTATION.md)

### Common Issues

#### **Build Errors**
```bash
# Clean build
cargo clean && cargo build

# Update dependencies
cargo update
```

#### **Database Connection Issues**
```bash
# Check PostgreSQL is running
docker-compose ps

# Reset database
docker-compose down && docker-compose up -d
DATABASE_URL="..." cargo sqlx migrate run
```

#### **Tests Failing**
```bash
# Run tests with output
cargo test --all -- --nocapture

# Run specific test
cargo test test_name -- --nocapture
```

---

## 🎯 Realistic Expectations

### For New Contributors

**Good First Contributions:**
- Fix clippy warnings
- Add unit tests to existing functions
- Improve error messages
- Add documentation comments
- Simple bug fixes

**Avoid These For Now:**
- Large architectural changes
- New business features
- Complex multi-file refactoring
- Performance optimizations

### For the Project

This is a **learning and development project**. The goal is building a solid foundation and learning Rust/PostgreSQL/API design patterns, not shipping a production ERP system immediately.

**Realistic Timeline:**
- **Q1 2025**: API-Repository integration complete
- **Q2 2025**: Basic customer management working
- **Q3 2025**: Multi-tenant runtime active
- **Q4 2025**: First real business workflows

Contributing to this project means helping build something from the ground up, which is both challenging and educational.

---

**Status**: Updated December 2024 | **Version**: 0.1.0-alpha | **Reality**: Foundation Building Phase