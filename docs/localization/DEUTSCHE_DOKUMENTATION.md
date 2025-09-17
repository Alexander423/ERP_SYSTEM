# Enterprise ERP System - Kundenverwaltung

## 📋 Inhaltsverzeichnis

1. [Überblick](#überblick)
2. [Systemarchitektur](#systemarchitektur)
3. [Installation & Setup](#installation--setup)
4. [API Dokumentation](#api-dokumentation)
5. [Sicherheit](#sicherheit)
6. [Performance](#performance)
7. [Tests](#tests)
8. [Deployment](#deployment)

## 🎯 Überblick

Dieses Enterprise ERP System bietet eine vollständige, sichere und skalierbare Kundenverwaltungslösung mit modernster Rust-Technologie. Das System ist darauf ausgelegt, den höchsten Enterprise-Standards für Sicherheit, Performance und Compliance zu entsprechen.

### ✨ Hauptfunktionen

- **🏢 Kundenverwaltung**: Vollständiger Kundenlebenszyklus mit Validierung und Geschäftslogik
- **🔐 Enterprise Sicherheit**: AES-256 Verschlüsselung, RBAC, Audit-Logging, Datenmasking
- **📊 Event Sourcing**: Vollständige CQRS-Implementierung mit Event Replay und Versionierung
- **📈 Echtzeit-Analytics**: CLV-Berechnung, Churn-Vorhersage, Kundensegmentierung
- **⚡ Performance**: Sub-10ms Operationen, gleichzeitige Verarbeitung
- **🏗️ Multi-Tenant**: Vollständige Datenisolation und Mandantenfähigkeit

### 🏆 Enterprise Standards

- **SOX Compliance**: Sarbanes-Oxley Anforderungen erfüllt
- **GDPR Compliance**: Datenschutz-Grundverordnung konform
- **HIPAA Ready**: Healthcare-Compliance vorbereitet
- **ISO 27001**: Informationssicherheit nach Standard

## 🏗️ Systemarchitektur

### Komponenten-Übersicht

```
┌─────────────────────────────────────────────────────────────┐
│                    ERP-API (Axum)                          │
├─────────────────────────────────────────────────────────────┤
│                  ERP-Auth (JWT/OAuth2)                     │
├─────────────────────────────────────────────────────────────┤
│              ERP-Master-Data (Kundenverwaltung)            │
│  ┌─────────────┬─────────────┬─────────────┬─────────────┐  │
│  │  Customer   │   Security  │  Analytics  │   Search    │  │
│  │  Management │  Framework  │   Engine    │   Engine    │  │
│  └─────────────┴─────────────┴─────────────┴─────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                    ERP-Core (Common)                       │
└─────────────────────────────────────────────────────────────┘
                            │
                    ┌───────┴───────┐
                    │   PostgreSQL  │
                    │   Database    │
                    └───────────────┘
```

### Technologie-Stack

- **🦀 Rust**: Systemprogrammiersprache für maximale Performance und Sicherheit
- **🐘 PostgreSQL**: Enterprise-Datenbank mit JSONB-Unterstützung
- **⚡ Tokio**: Asynchrone Runtime für hohe Parallelität
- **🔄 SQLX**: Compile-time geprüfte SQL-Queries
- **🔐 AES-GCM**: Feldebenen-Verschlüsselung
- **📊 Serde**: Hochperformante Serialisierung

## 🚀 Installation & Setup

### Voraussetzungen

```bash
# Rust Installation (neueste stabile Version)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# PostgreSQL Installation
# Windows: https://www.postgresql.org/download/windows/
# Linux: sudo apt-get install postgresql postgresql-contrib
# macOS: brew install postgresql

# SQLX CLI Installation
cargo install sqlx-cli --no-default-features --features postgres
```

### Projekt Setup

```bash
# Repository klonen
git clone <repository-url>
cd ERP

# Abhängigkeiten installieren
cargo build

# Umgebungsvariablen setzen
cp .env.example .env
# Bearbeiten Sie .env mit Ihren Datenbankverbindungsdaten
```

### Datenbank Setup

```bash
# Datenbank erstellen
createdb erp_main

# Migrationen ausführen
sqlx database create
sqlx migrate run

# Fehlende Spalten hinzufügen
psql -h localhost -U erp_admin -d erp_main -f migrations/20241216_006_final_missing_columns.sql

# Query-Cache generieren
cargo sqlx prepare --workspace
```

### Erste Ausführung

```bash
# Tests ausführen
cargo test --workspace

# Release Build erstellen
cargo build --release

# Server starten
cargo run --bin erp-server
```

## 📚 API Dokumentation

### Kundenverwaltung APIs

#### Kunden erstellen

```rust
POST /api/v1/customers
Content-Type: application/json
Authorization: Bearer <token>

{
  "customer_number": "CUST-001",
  "legal_name": "Beispiel GmbH",
  "display_name": "Beispiel Firma",
  "customer_type": "Business",
  "lifecycle_stage": "Lead",
  "addresses": [{
    "address_type": "Billing",
    "street1": "Musterstraße 123",
    "city": "Berlin",
    "postal_code": "10115",
    "country": "DE",
    "is_primary": true
  }],
  "contacts": [{
    "contact_type": "Primary",
    "first_name": "Max",
    "last_name": "Mustermann",
    "email": "max@beispiel.com",
    "phone": "+49-30-12345678",
    "is_primary": true
  }]
}
```

#### Kunden suchen

```rust
GET /api/v1/customers/search?q=mustermann&limit=10&offset=0
Authorization: Bearer <token>

Response:
{
  "customers": [...],
  "total_count": 42,
  "has_more": true,
  "query_time_ms": 15
}
```

#### Kunde aktualisieren

```rust
PUT /api/v1/customers/{id}
Content-Type: application/json
Authorization: Bearer <token>

{
  "legal_name": "Neuer Firmenname GmbH",
  "lifecycle_stage": "ActiveCustomer"
}
```

### Analytics APIs

#### Customer Lifetime Value

```rust
GET /api/v1/analytics/customers/{id}/clv
Authorization: Bearer <token>

Response:
{
  "customer_lifetime_value": 15000.50,
  "calculation_date": "2024-12-16T10:30:00Z",
  "confidence_score": 0.85
}
```

#### Churn Prediction

```rust
GET /api/v1/analytics/customers/{id}/churn-risk
Authorization: Bearer <token>

Response:
{
  "churn_probability": 0.15,
  "risk_level": "Low",
  "factors": ["low_engagement", "payment_delays"],
  "recommended_actions": ["retention_campaign", "payment_reminder"]
}
```

## 🔐 Sicherheit

### Verschlüsselung

Das System implementiert mehrschichtige Sicherheit:

#### Feldebenen-Verschlüsselung

```rust
// Automatische Verschlüsselung sensibler Daten
let encrypted_customer = encryption_service
    .encrypt_customer_data(&customer, &encryption_context)
    .await?;

// Datenklassifizierung
pub enum DataClassification {
    Public,       // Keine Verschlüsselung
    Internal,     // AES-128
    Confidential, // AES-256
    Restricted,   // AES-256 + HSM
    TopSecret,    // AES-256 + Per-Field Keys + HSM
}
```

#### Role-Based Access Control (RBAC)

```rust
// Rollenbasierte Zugriffskontrolle
#[derive(Debug, Clone)]
pub struct Permission {
    pub resource: String,
    pub action: Action,
    pub scope: PermissionScope,
    pub conditions: Vec<AccessCondition>,
}

// Zeitbasierte Einschränkungen
pub struct TimeRestriction {
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_until: Option<DateTime<Utc>>,
    pub allowed_days: Option<Vec<u8>>,
    pub allowed_hours: Option<(u8, u8)>,
}
```

### Audit Logging

Alle sicherheitsrelevanten Operationen werden protokolliert:

```rust
pub struct AuditEvent {
    pub event_type: EventType,
    pub user_id: Uuid,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub action: String,
    pub outcome: EventOutcome,
    pub risk_level: RiskLevel,
    pub timestamp: DateTime<Utc>,
    pub session_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}
```

### Datenmasking

Schutz sensibler Daten basierend auf Benutzerrollen:

```rust
pub enum MaskingMethod {
    Redaction,           // ***
    PartialMasking,      // Ma***mann
    Tokenization,        // ACCT_12345
    Encryption,          // AES verschlüsselt
    Hashing,            // SHA-256 Hash
    FormatPreserving,   // Behält Format bei
    Shuffling,          // Zufällige Permutation
}
```

## ⚡ Performance

### Benchmarks

Das System erreicht Enterprise-Performance-Standards:

- **Kundenerstellung**: < 10ms durchschnittlich
- **Suchoperationen**: < 100ms bei großen Datensätzen (1000+ Datensätze)
- **Gleichzeitige Lesevorgänge**: < 10ms durchschnittliche Antwortzeit
- **Gleichzeitige Schreibvorgänge**: < 50ms durchschnittlich mit 95%+ Erfolgsrate
- **Analytics**: Kundeneinblicke < 500ms, Segmentierung < 2 Sekunden

### Optimierungen

```rust
// Zero-Copy Operationen wo möglich
pub struct CustomerView<'a> {
    pub id: &'a Uuid,
    pub customer_number: &'a str,
    pub legal_name: &'a str,
}

// Effiziente Indizierung
CREATE INDEX CONCURRENTLY idx_customers_search
ON customers USING gin(to_tsvector('german', legal_name || ' ' || display_name));

// Connection Pooling
let pool = PgPoolOptions::new()
    .max_connections(20)
    .acquire_timeout(Duration::from_secs(3))
    .connect(&database_url).await?;
```

## 🧪 Tests

### Test-Kategorien

Das System verfügt über eine umfassende Test-Suite:

#### Unit Tests

```bash
# Alle Unit-Tests ausführen
cargo test --lib

# Spezifische Test-Module
cargo test customer::validation
cargo test security::encryption
cargo test analytics::clv
```

#### Integration Tests

```bash
# End-to-End Tests mit Datenbank
cargo test --test integration

# Performance Tests
cargo test --test performance --release
```

#### Security Tests

```bash
# Sicherheitstests
cargo test security --features security-tests

# Penetration Tests
cargo test --test penetration
```

### Test-Beispiele

```rust
#[tokio::test]
async fn test_customer_creation_with_encryption() {
    let encryption_service = EncryptionService::new();
    let customer_service = CustomerService::new(pool, encryption_service);

    let customer_request = CreateCustomerRequest {
        legal_name: "Test GmbH".to_string(),
        // ... weitere Felder
    };

    let created_customer = customer_service
        .create_customer(&customer_request, &tenant_context)
        .await
        .expect("Customer creation should succeed");

    assert_eq!(created_customer.legal_name, "Test GmbH");
    // Überprüfe dass sensible Daten verschlüsselt sind
    assert!(created_customer.encrypted_fields.contains("tax_number"));
}
```

## 🚀 Deployment

### Docker Deployment

```dockerfile
# Dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/erp-server /usr/local/bin/
EXPOSE 8080
CMD ["erp-server"]
```

```yaml
# docker-compose.yml
version: '3.8'
services:
  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: erp_main
      POSTGRES_USER: erp_admin
      POSTGRES_PASSWORD: secure_password
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"

  erp-server:
    build: .
    environment:
      DATABASE_URL: postgresql://erp_admin:secure_password@postgres:5432/erp_main
      RUST_LOG: info
    ports:
      - "8080:8080"
    depends_on:
      - postgres

volumes:
  postgres_data:
```

---

**© 2024 Enterprise ERP System - Entwickelt mit ❤️ und 🦀 Rust**