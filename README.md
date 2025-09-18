# Enterprise ERP System

An ERP system built with Rust, currently in development. Features a solid foundation with PostgreSQL database, multi-tenant architecture, and modular design.

## 🚀 Quick Start

```bash
# Clone the repository
git clone <repository-url>
cd ERP

# Setup environment variables
# Set DATABASE_URL, REDIS_URL, JWT_SECRET, etc.

# Run database migrations
DATABASE_URL="your-postgres-url" cargo sqlx migrate run

# Build and run the API server
DATABASE_URL="your-postgres-url" cargo run -p erp-api --bin erp-server
```

**⚠️ Note:** This system is currently in development. See [PROJECT_STATUS.md](PROJECT_STATUS.md) for current implementation status.

## 📚 Documentation

### Core Documentation
- **[Security](docs/SECURITY.md)** - Security policies and guidelines
- **[Contributing](docs/CONTRIBUTING.md)** - How to contribute to the project
- **[Changelog](docs/CHANGELOG.md)** - Version history and changes

### Technical Documentation
- **[Architecture](docs/architecture/ARCHITECTURE.md)** - System architecture overview
- **[Configuration](docs/architecture/CONFIGURATION.md)** - Configuration guide
- **[API Documentation](docs/api/API_DOKUMENTATION.md)** - Complete API reference
- **[Deployment Guide](docs/deployment/DEPLOYMENT_GUIDE.md)** - Production deployment instructions

### Project Information
- **[Project Status](PROJECT_STATUS.md)** - Accurate current development status
- **[Legacy Documentation](docs/project/)** - Historical implementation docs (may contain outdated info)

### Localization
- **[Deutsche Dokumentation](docs/localization/DEUTSCHE_DOKUMENTATION.md)** - German documentation

### Testing
- **[Test Reports](docs/testing/test_reports.md)** - Comprehensive testing results

### GitHub & Development
- **[GitHub Setup](docs/github/GITHUB_SETUP.md)** - Repository setup guide
- **[Issue Templates](docs/github/github_issues_templates.md)** - GitHub issue templates

## 🏗️ System Features

### ✅ Currently Implemented
- **Modular Architecture** - Clean separation of concerns with Rust crates
- **Database Layer** - PostgreSQL with migrations and repository pattern
- **API Framework** - Axum-based HTTP server with middleware
- **Configuration System** - Environment-based configuration management
- **Basic Authentication** - User management and JWT token handling

### 🚧 In Development
- **Customer Management** - Repository layer implemented, API integration ongoing
- **Multi-tenant Support** - Database schema ready, context handling in progress
- **Security Middleware** - Basic security headers, auth middleware being refined

### 📋 Planned Features
- **Analytics Engine** - Customer insights and reporting
- **Event Sourcing** - Audit trail and event replay
- **Advanced Security** - Field-level encryption, compliance features
- **Frontend Interface** - Web UI for system management

## 🛠️ Development

### Prerequisites
- Rust 1.70+
- PostgreSQL 14+
- Docker (optional)

### Project Structure
```
├── crates/
│   ├── core/           # Core business logic and utilities
│   ├── auth/           # Authentication and authorization
│   ├── api/            # REST API endpoints
│   └── master-data/    # Master data management
├── docs/               # Documentation
├── migrations/         # Database migrations
├── scripts/            # Database utility scripts
└── config/             # Configuration files
```

### Running Tests
```bash
# Set up test environment first
# Copy .env.test and configure test database

# Run specific crate tests
DATABASE_URL="your-test-db-url" cargo test -p erp-core
DATABASE_URL="your-test-db-url" cargo test -p erp-auth
DATABASE_URL="your-test-db-url" cargo test -p erp-master-data

# Note: Integration tests require proper database setup
```

## 📊 Current Development Status

**🟡 Alpha Development Stage**

- **Core Infrastructure** - ✅ Complete and functional
- **API Layer** - 🟡 Basic implementation with mock responses
- **Database Layer** - ✅ Repository pattern with proper SQL implementation
- **Testing** - 🟡 Infrastructure working, coverage being expanded
- **Documentation** - 🟡 Realistic status, legacy docs being updated

## 🔒 Security

### ✅ Currently Implemented
- Configuration validation and security checks
- JWT token-based authentication framework
- Password hashing with Argon2
- Basic security headers middleware

### 🚧 In Development
- Role-based access control (RBAC)
- Session management
- Input validation and sanitization

### 📋 Planned
- Field-level encryption (AES-256)
- Comprehensive audit logging
- Multi-factor authentication
- Compliance frameworks (GDPR, SOX)

## 🤝 Contributing

See [Contributing Guide](docs/CONTRIBUTING.md) for details on how to contribute to this project.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🆘 Support

For questions and support:
- Check the [documentation](docs/)
- Review [GitHub issues](docs/github/github_issues_templates.md)
- Read the [troubleshooting guide](docs/deployment/DEPLOYMENT_GUIDE.md)

---

**Built with ❤️ using Rust and PostgreSQL**