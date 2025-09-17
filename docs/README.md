# 🚀 ERP System - Enterprise-Grade Modular Monolith

> **Ein hochmodernes, sicheres und skalierbares ERP-System, entwickelt mit Rust**  
> Implementierung von **Modul 1: Benutzer & Authentifizierung** als Fundament für ein umfassendes Enterprise Resource Planning System.

[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://rustup.rs/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![API Docs](https://img.shields.io/badge/API-Swagger_UI-green.svg)](#api-dokumentation)

---

## 🎯 Projektübersicht

Dieses ERP-System ist darauf ausgelegt, bestehende Lösungen wie SAP, Oracle oder Microsoft Dynamics zu übertreffen durch:

- **🏗️ Modularer Monolith**: Klare Trennung zwischen Infrastruktur und Business Logic
- **🔒 Enterprise Security**: Multi-Tenant-Architektur mit höchsten Sicherheitsstandards  
- **⚡ High Performance**: Rust-basierte Performance mit async/await
- **🎯 API-First**: RESTful API mit vollständiger OpenAPI 3.0 Dokumentation
- **📊 Production-Ready**: Monitoring, Metrics, Health Checks, Audit Logging

### Aktueller Stand: Modul 1 ✅
**Benutzer & Authentifizierung** vollständig implementiert mit:
- Multi-Tenant-Registrierung und -Verwaltung
- Sichere JWT-basierte Authentifizierung  
- Two-Factor Authentication (TOTP)
- Rollen-basierte Zugriffskontrolle (RBAC)
- Email-Verifikation und Passwort-Reset
- Enterprise-grade Sicherheitsfeatures

---

## 🏗️ Architektur-Überblick

### Modularer Monolith Design
```
📦 ERP System (Cargo Workspace)
├── 🧠 crates/core/           # 🔧 Infrastruktur & Shared Services
│   ├── config/               # Konfigurationssystem (TOML + ENV)
│   ├── database/             # Multi-Tenant PostgreSQL Pool
│   ├── security/             # Crypto, JWT, Password Hashing
│   ├── audit/                # Event Logging & Compliance
│   ├── jobs/                 # Background Job System (Redis)
│   ├── metrics/              # Prometheus Metrics
│   └── error/                # Zentralisiertes Error Framework
├── 👤 crates/auth/           # 🎯 Business Logic - Authentication
│   ├── service/              # Domain Services
│   ├── repository/           # Data Access Layer
│   ├── workflows/            # Email Verification, Password Reset
│   ├── tokens/               # Token Management
│   └── email/                # Multi-Provider Email Service
└── 🌐 crates/api/            # 🚀 HTTP Server & API Layer
    ├── handlers/             # HTTP Request Handlers
    ├── middleware/           # Security Headers, CORS, Request ID
    └── routes/               # Route Definitions
```

### Architektur-Prinzipien
- **🔄 Dependency Inversion**: Business Logic abhängig von Abstractions, nicht von Details
- **📦 Single Responsibility**: Jedes Crate hat eine klar definierte Verantwortung
- **🎯 API-First Design**: OpenAPI-Spezifikation als Single Source of Truth
- **🏛️ Clean Architecture**: Trennung von Infrastructure, Domain und Application Layer

---

## 🛠️ Technologie-Stack

### **Backend-Core**
- **[Rust](https://rust-lang.org/)** (stable) - Systems Programming Language
- **[Axum](https://github.com/tokio-rs/axum)** - High-performance Web Framework  
- **[Tokio](https://tokio.rs/)** - Async Runtime
- **[Tower](https://github.com/tower-rs/tower)** - Service-oriented Middleware

### **Persistence & Caching**
- **[PostgreSQL](https://postgresql.org/)** 16+ - Primary Database
- **[SQLx](https://github.com/launchbadge/sqlx)** - Compile-time checked SQL queries
- **[Redis](https://redis.io/)** 7+ - Caching & Session Store

### **Security & Crypto**
- **[Argon2id](https://github.com/RustCrypto/password-hashes)** - Password Hashing
- **[AES-GCM](https://github.com/RustCrypto/AEADs)** - Symmetric Encryption  
- **[JWT](https://github.com/Keats/jsonwebtoken)** - Token-based Authentication
- **[TOTP](https://github.com/constantoine/totp-rs)** - Two-Factor Authentication

### **Observability & Monitoring**  
- **[Prometheus](https://prometheus.io/)** - Metrics Collection
- **[Tracing](https://github.com/tokio-rs/tracing)** - Structured Logging
- **Health Checks** - Application Health Monitoring

### **Documentation & API**
- **[OpenAPI 3.0](https://swagger.io/specification/)** - API Specification
- **[Swagger UI](https://swagger.io/tools/swagger-ui/)** - Interactive API Documentation
- **[Rustdoc](https://doc.rust-lang.org/rustdoc/)** - Code Documentation

---

## ⚡ Schnellstart & Setup

### Voraussetzungen
- **[Rust](https://rustup.rs/)** (stable - 1.70+)
- **[Docker](https://docker.com/)** & Docker Compose
- **[Git](https://git-scm.com/)**

### 1️⃣ Repository Setup
```bash
git clone <repository-url>
cd erp-system

# Environment konfigurieren
cp .env.example .env
# ⚠️  .env-Datei mit Ihren spezifischen Werten bearbeiten
```

### 2️⃣ Infrastruktur starten
```bash
# PostgreSQL & Redis via Docker
docker-compose up -d postgres redis

# Optional: pgAdmin für DB-Management  
docker-compose --profile debug up -d pgadmin
# Zugriff: http://localhost:5050 (admin@erp.local / siehe docker-compose.yml)
```

### 3️⃣ Datenbank-Setup
```bash
# Dependencies installieren
cargo build

# Datenbank-Migrationen ausführen
# Automatisch beim Server-Start oder manuell:
# sqlx migrate run
```

### 4️⃣ Server starten
```bash
# Development Server  
cargo run --bin erp-api

# Mit spezifischem Environment
ENVIRONMENT=development cargo run --bin erp-api
```

### 5️⃣ API testen
- **🏥 Health Check**: http://localhost:3000/health
- **📖 API Dokumentation**: http://localhost:3000/swagger-ui  
- **🔍 Readiness Check**: http://localhost:3000/ready

```bash
# Schneller Health Check
curl http://localhost:3000/health
# Response: {"status":"healthy","version":"0.1.0","environment":"development"}
```

---

## 🔧 Konfigurationssystem

Das System verwendet hierarchische Konfiguration: **TOML-Dateien + Environment Variables**

### Konfigurationsdateien
```
config/
├── default.toml      # 🔧 Basis-Konfiguration
├── development.toml  # 👨‍💻 Entwicklung (localhost, Debug-Logs)
├── testing.toml      # 🧪 Tests (In-Memory, Mock-Services)  
└── production.toml   # 🚀 Produktion (Sicherheit, Performance)
```

### Environment-Spezifische Einstellungen
```bash
# Environment auswählen
export ENVIRONMENT=production  # default: development

# Sensitive Daten über ENV-Variablen (Production)
export DATABASE_URL="postgresql://user:pass@host:5432/db"
export JWT_SECRET="your-super-secret-jwt-key-min-32-chars"
export REDIS_URL="redis://:password@host:6379"
```

### Wichtige Konfigurationsbereiche

#### 🔒 **Security Configuration**
```toml
[security]
argon2_memory_cost = 65536    # 64 MB (Development: 32MB)
argon2_time_cost = 3          # Iterations
argon2_parallelism = 2        # Threads
aes_encryption_key = "32-char-key-for-data-encryption"

[jwt]  
secret = "jwt-signing-secret-min-32-characters"
access_token_expiry = 1800    # 30 minutes (Production)
refresh_token_expiry = 604800 # 7 days (Production)
```

#### 🌐 **CORS Configuration**
```toml
[cors]
allowed_origins = ["https://yourdomain.com"]  # Production
allowed_methods = ["GET", "POST", "PUT", "DELETE", "OPTIONS"]
allowed_headers = ["authorization", "content-type", "x-request-id"]
allow_credentials = true
max_age = 7200  # 2 hours
```

#### 📊 **Monitoring Configuration**
```toml
[metrics]
enabled = true
port = 9090
path = "/metrics"  
namespace = "erp_production"
```

---

## 🧪 Tests & Qualitätssicherung

### Test-Ausführung
```bash
# 🧪 Unit Tests
cargo test

# 🔗 Integration Tests  
cargo test --test integration

# 📊 Code Coverage
cargo tarpaulin --out Html
open tarpaulin-report.html

# 🔍 Code Quality
cargo fmt      # Code-Formatierung
cargo clippy   # Linting & Best Practices
cargo audit    # Security Vulnerability Check
```

### Test-Kategorien

#### **Unit Tests** 
- **Core Crate**: Crypto-Funktionen, Database-Pool, Error-Handling
- **Auth Crate**: Service-Logic, Repository-Layer, Token-Management
- **API Crate**: Handler-Logic, Middleware-Funktionen

#### **Integration Tests**
```bash
# Vollständige API-Workflows testen
cargo test --test integration -- --test-threads=1

# Spezifische Workflows
cargo test integration::auth::registration_flow
cargo test integration::auth::password_reset_flow  
cargo test integration::auth::email_verification_flow
```

### Test-Datenbank
```bash
# Separates Test-Environment
ENVIRONMENT=testing cargo test

# Verwendet testing.toml Konfiguration:
# - Separate Test-DB (Port 5433)
# - In-Memory Redis (Port 6380)
# - Mock Email-Provider
```

---

## 🚀 Deployment & Production  

### Docker Production Build
```bash
# Multi-stage Docker Build
docker build -t erp-system:latest .
docker build -t erp-system:v1.0.0 .

# Production Container starten
docker run -d \
  --name erp-prod \
  --env-file .env.production \
  -p 8080:3000 \
  erp-system:latest
```

### Production Deployment Checklist
- [ ] **Environment Variables**: Alle Secrets über ENV, nicht in Config-Dateien
- [ ] **TLS/HTTPS**: Reverse Proxy (nginx/Caddy) mit TLS-Terminierung
- [ ] **Database**: Managed PostgreSQL mit Connection Pooling  
- [ ] **Redis**: Managed Redis mit Persistence
- [ ] **Monitoring**: Prometheus + Grafana Dashboard Setup
- [ ] **Logging**: Centralized Logging (ELK Stack / Cloud Logging)
- [ ] **Backups**: Automated DB Backups with Point-in-Time Recovery
- [ ] **CORS**: Production-spezifische Origins (keine Wildcards!)

### Container Orchestration
```yaml
# docker-compose.production.yml
version: '3.9'
services:
  erp-api:
    image: erp-system:latest
    env_file: .env.production
    depends_on: [postgres, redis]
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3
```

---

## 📊 Monitoring & Health Checks

### Health Check Endpoints
```bash
# 🏥 Basic Health Check
curl http://localhost:3000/health
# {"status":"healthy","version":"0.1.0","environment":"development"}

# 🔍 Readiness Check (DB + Redis Connectivity)  
curl http://localhost:3000/ready
# {"status":"ready","checks":{"database":"ok","redis":"ok"}}

# 📊 Prometheus Metrics
curl http://localhost:9090/metrics
# Umfassende Application & Business Metrics
```

### Verfügbare Metriken
- **🔐 Authentication Metrics**: Login-Erfolg/Fehler, 2FA-Verifikationen
- **👤 User Metrics**: Registrierungen, Account-Lockouts, Session-Dauer
- **📧 Email Metrics**: Versendete E-Mails, Delivery-Failures  
- **🔍 Security Metrics**: Rate-Limit-Violations, Invalid-Token-Attempts
- **⚡ Performance Metrics**: Request-Duration, Database-Query-Times
- **💾 System Metrics**: Memory-Usage, Connection-Pool-Status

### Monitoring Dashboard Setup
```bash
# Prometheus Config Beispiel
scrape_configs:
  - job_name: 'erp-system'
    static_configs:
      - targets: ['localhost:9090']
    scrape_interval: 15s
    metrics_path: /metrics
```

---

## 🔒 Sicherheit & Best Practices

### 🛡️ Implementierte Sicherheitsfeatures
- **🔐 Argon2id Password Hashing**: Brute-Force resistent
- **🎫 JWT Token Management**: Secure + HttpOnly Refresh Tokens  
- **🔢 Two-Factor Authentication**: TOTP-basiert mit QR-Code Setup
- **🚪 Account Lockout**: Automatischer Schutz vor Brute-Force-Angriffen
- **📧 Email Verification**: Verhindert fake Account-Registrierungen
- **🌐 CORS Configuration**: Environment-spezifische Origin-Kontrolle
- **🔍 Security Headers**: HSTS, CSP, X-Frame-Options, etc.
- **📋 Audit Logging**: Vollständige Nachverfolgung sicherheitsrelevanter Events

### 🔧 Sicherheits-Konfiguration
```bash
# 🔑 Strong Secrets (Production)
JWT_SECRET=$(openssl rand -base64 32)
AES_ENCRYPTION_KEY=$(openssl rand -base64 32)  

# 🗄️ Database Security
DATABASE_URL="postgresql://user:$(openssl rand -base64 20)@host:5432/db"
REDIS_URL="redis://:$(openssl rand -base64 20)@host:6379"

# ⚡ Performance Security
ARGON2_MEMORY_COST=131072  # 128 MB Production
ARGON2_TIME_COST=4         # 4 Iterations Production
```

### 🚨 Security Best Practices
1. **Never commit secrets**: Alle sensiblen Daten über Environment Variables
2. **Rotate keys regularly**: JWT-Secrets und Encryption-Keys regelmäßig rotieren  
3. **Monitor audit logs**: Automatische Alerts bei verdächtigen Aktivitäten
4. **Regular security audits**: `cargo audit` in CI/CD Pipeline integrieren
5. **HTTPS everywhere**: Niemals unverschlüsselte Verbindungen in Production
6. **Rate limiting**: API-Endpoints gegen Abuse schützen
7. **Input validation**: Alle User-Inputs server-seitig validieren

---

## 🤝 Entwicklung & Contribution

### Development Workflow
```bash
# 🔧 Development Setup
git clone <repo>
cd erp-system
cp .env.example .env

# 🏗️ Build & Development Tools
cargo build
cargo install sqlx-cli
cargo install cargo-tarpaulin  # Coverage
cargo install cargo-audit      # Security

# 🎯 Pre-commit Checks
cargo fmt --all --check        # Code formatting
cargo clippy --all-targets     # Linting  
cargo test --all              # All tests
cargo audit                   # Security audit
```

### 📁 Projekt-Struktur Verstehen
```rust
// 🧠 Core Crate - Infrastructure Services
use erp_core::{
    Config,              // Hierarchical configuration
    DatabasePool,        // Multi-tenant DB pool
    security::jwt,       // JWT token operations
    audit::AuditLogger,  // Security event logging
};

// 👤 Auth Crate - Business Logic  
use erp_auth::{
    AuthService,         // Main authentication service
    workflows::*,        // Email verification, password reset
    middleware::*,       // Authentication & authorization
};
```

### 🎨 Code Style & Standards
- **📝 Documentation**: Jede public Function mit `///` Rustdoc
- **🧪 Test Coverage**: Minimum 80% Coverage für neue Features
- **🏗️ Architecture**: Clean Architecture Prinzipien befolgen
- **🔍 Error Handling**: Structured Error-Handling mit Context
- **📊 Logging**: Structured Logging mit tracing crate

### 🚀 Feature Development
```bash
# 1️⃣ Feature Branch erstellen
git checkout -b feature/new-awesome-feature

# 2️⃣ Implementation mit Tests
cargo test --all
cargo clippy --all-targets

# 3️⃣ Documentation aktualisieren  
cargo doc --open

# 4️⃣ Pull Request mit vollständiger Beschreibung
```

---

## 📖 API-Dokumentation

### 🌐 Interactive Swagger UI
**URL**: http://localhost:3000/swagger-ui  
**Features**: 
- Vollständige OpenAPI 3.0 Spezifikation
- Interactive Request/Response Testing
- Authentication mit Bearer Token
- Realistische Beispiel-Payloads

### 🔗 Haupt-Endpoints

#### **🔓 Public Endpoints** (No Authentication)
| Endpoint | Method | Beschreibung |
|----------|--------|--------------|
| `POST /api/v1/auth/register` | POST | Tenant & Admin-User registrieren |
| `POST /api/v1/auth/login` | POST | Benutzer-Login mit Email/Password |
| `POST /api/v1/auth/verify-2fa` | POST | Two-Factor Authentication |
| `POST /api/v1/auth/refresh-token` | POST | Access Token erneuern |
| `POST /api/v1/auth/forgot-password` | POST | Passwort-Reset anfordern |
| `POST /api/v1/auth/reset-password` | POST | Passwort mit Token zurücksetzen |
| `POST /api/v1/auth/verify-email` | POST | Email-Adresse verifizieren |

#### **🔒 Protected Endpoints** (Bearer Token Required)
| Endpoint | Method | Permission | Beschreibung |
|----------|--------|------------|--------------|
| `GET /api/v1/users` | GET | `user:read` | Benutzer auflisten |
| `POST /api/v1/users/invite` | POST | `user:create` | Neuen Benutzer einladen |
| `GET /api/v1/users/{id}` | GET | `user:read` | Benutzer-Details abrufen |
| `PUT /api/v1/users/{id}` | PUT | `user:update` | Benutzer-Daten aktualisieren |
| `DELETE /api/v1/users/{id}` | DELETE | `user:delete` | Benutzer löschen |
| `GET /api/v1/roles` | GET | `role:manage` | Rollen auflisten |
| `POST /api/v1/roles` | POST | `role:manage` | Neue Rolle erstellen |
| `POST /api/v1/auth/impersonate` | POST | `user:impersonate` | Benutzer impersonation |

### 📋 Beispiel-Requests

#### **Tenant Registrierung**
```bash
curl -X POST http://localhost:3000/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "companyName": "Awesome Corp GmbH",
    "email": "admin@awesomecorp.com",  
    "password": "SecurePassword123!",
    "firstName": "Max",
    "lastName": "Mustermann"
  }'

# Response: 201 Created
{
  "success": true,
  "tenantId": "123e4567-e89b-12d3-a456-426614174000",
  "userId": "987fcdeb-51a2-43d1-9c4e-123456789abc",
  "message": "Registration successful. Please verify your email address."
}
```

#### **Benutzer Login**
```bash
curl -X POST http://localhost:3000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -H "X-Tenant-Id: 123e4567-e89b-12d3-a456-426614174000" \
  -d '{
    "email": "admin@awesomecorp.com",
    "password": "SecurePassword123!"
  }'

# Response: 200 OK (Success) or 200 OK (2FA Required)
{
  "success": true,
  "accessToken": "eyJ0eXAiOiJKV1QiLCJhbGc...",
  "refreshToken": "def456...", 
  "expiresIn": 1800,
  "user": {
    "id": "987fcdeb-51a2-43d1-9c4e-123456789abc",
    "email": "admin@awesomecorp.com",
    "firstName": "Max",
    "lastName": "Mustermann", 
    "roles": ["admin"]
  }
}
```

#### **Protected API Call**
```bash
curl -X GET http://localhost:3000/api/v1/users \
  -H "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGc..." \
  -H "X-Tenant-Id: 123e4567-e89b-12d3-a456-426614174000"

# Response: 200 OK
{
  "users": [
    {
      "id": "987fcdeb-51a2-43d1-9c4e-123456789abc",
      "email": "admin@awesomecorp.com",
      "firstName": "Max", 
      "lastName": "Mustermann",
      "roles": ["admin"],
      "isActive": true,
      "emailVerified": true,
      "lastLoginAt": "2024-01-15T10:30:00Z",
      "createdAt": "2024-01-10T09:15:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "limit": 20,
    "total": 1,
    "totalPages": 1
  }
}
```

---

## 🛠️ Development Tools & Commands

### 📦 Dependency Management
```bash
# Workspace dependencies aktualisieren
cargo update

# Dependency vulnerabilities checken  
cargo audit

# Unused dependencies finden
cargo +nightly udeps
```

### 🗄️ Database Operations
```bash
# New migration erstellen
sqlx migrate add create_new_feature_table

# Migrations anwenden
sqlx migrate run

# Migration rollback (letzten N rückgängig)
sqlx migrate revert

# Schema für sqlx vorbereiten (Offline-Modus)
cargo sqlx prepare
```

### 🧹 Code Quality
```bash
# Vollständiger Quality Check
./scripts/quality-check.sh

# Einzelne Checks
cargo fmt --all                    # Code formatting
cargo clippy --all-targets        # Linting
cargo test --all                  # All tests  
cargo doc --document-private-items # Documentation build
```

---

## 🗺️ Roadmap & Nächste Module

### ✅ **Modul 1: Benutzer & Authentifizierung** (Completed)
- Multi-Tenant-Registrierung ✅
- JWT-basierte Authentifizierung ✅  
- Two-Factor Authentication ✅
- Rollen-basierte Zugriffskontrolle ✅
- Email-Verifikation & Password-Reset ✅
- Enterprise Security Features ✅

### 🔄 **Modul 2: Stammdatenverwaltung** (Planning)
- Organisationsstrukturen (Abteilungen, Standorte)
- Kontaktverwaltung (Kunden, Lieferanten, Mitarbeiter)
- Produktkatalog und Artikelstamm
- Kategorisierung und Klassifizierung

### 🔮 **Modul 3: Finanzverwaltung** (Future)
- Buchhaltung und Kostenstellen
- Rechnungsstellung und Zahlungsverkehr
- Financial Reporting und Analytics

### 📊 **Modul 4: Analytics & Reporting** (Future)
- Business Intelligence Dashboard
- Custom Report Builder
- Data Export und Integration APIs

---

## 📞 Support & Community

### 🐛 **Bug Reports & Feature Requests**
- **GitHub Issues**: [Project Issues](../../issues)
- **Security Issues**: Siehe [SECURITY.md](SECURITY.md)

### 📚 **Documentation**
- **API Documentation**: http://localhost:3000/swagger-ui
- **Rustdoc**: `cargo doc --open`
- **Architecture Docs**: `docs/ARCHITECTURE.md`

### 🤝 **Contributing**
1. **Fork** das Repository
2. **Feature Branch** erstellen: `git checkout -b feature/AmazingFeature`
3. **Changes committen**: `git commit -m 'Add some AmazingFeature'`
4. **Branch pushen**: `git push origin feature/AmazingFeature`  
5. **Pull Request** erstellen

Siehe [CONTRIBUTING.md](CONTRIBUTING.md) für detaillierte Entwicklungsrichtlinien.

---

## 📄 Lizenz

Dieses Projekt ist unter der **MIT License** lizenziert. Siehe [LICENSE](LICENSE) für Details.

---

<div align="center">

**⚡ Entwickelt mit ❤️ in Rust für maximale Performance und Sicherheit**

*Enterprise-Grade ERP System - Modul 1 vollständig implementiert*

[![Rust](https://img.shields.io/badge/Built_with-Rust-orange.svg?style=for-the-badge&logo=rust)](https://rust-lang.org/)
[![PostgreSQL](https://img.shields.io/badge/Database-PostgreSQL-blue.svg?style=for-the-badge&logo=postgresql)](https://postgresql.org/)  
[![Docker](https://img.shields.io/badge/Containerized-Docker-blue.svg?style=for-the-badge&logo=docker)](https://docker.com/)

</div>