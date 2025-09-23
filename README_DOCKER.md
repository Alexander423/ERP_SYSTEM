# ERP Docker Stack - Professional Setup

## Übersicht

Das ERP-System verwendet jetzt einen professionellen Docker-Stack mit automatischer Datenbankinitialisierung. Alle Migration-Konflikte sind gelöst durch einen kompletten Neustart mit strukturierten Migrationen.

## Schnellstart

### 1. Stack komplett zurücksetzen und neu starten
```bash
scripts\docker_reset.bat
```

**Was passiert:**
- Löscht alle existierenden Container und Volumes
- Startet PostgreSQL und Redis neu
- Initialisiert automatisch das komplette ERP-Schema
- Lädt Demo-Daten für Entwicklung

### 2. Stack normal starten
```bash
scripts\docker_start.bat
```

### 3. System validieren
```bash
scripts\validate_docker_stack.bat
```

## Stack-Komponenten

### PostgreSQL 15
- **Port:** 5432
- **Database:** erp_main
- **User:** erp_admin
- **Password:** erp_secure_password_change_in_production
- **Automatische Schema-Initialisierung:** ✅
- **Performance-optimiert für Entwicklung:** ✅

### Redis 7
- **Port:** 6379
- **Password:** erp_redis_password_change_in_production
- **Persistence:** AOF + RDB
- **Optimiert für Session-Management:** ✅

### Optionale Admin-Tools
```bash
# pgAdmin starten (http://localhost:8080)
docker-compose --profile admin up -d pgadmin

# Redis Commander starten (http://localhost:8081)
docker-compose --profile admin up -d redis-commander
```

**Zugangsdaten für Admin-Tools:**
- pgAdmin: admin@erp.local / admin123
- Redis Commander: admin / admin123

## Automatische Datenbankinitialisierung

Das System initialisiert sich automatisch mit:

1. **Foundation Layer** (`001_init_complete_schema.sql`)
   - PostgreSQL Extensions (uuid-ossp, pgcrypto)
   - Alle Enum-Typen
   - Utility-Funktionen
   - Tenants-Tabelle
   - Komplette Core-Tabellen (Products, Customers, Suppliers, Addresses)

2. **Inventory System** (`002_inventory_system.sql`)
   - Location Items mit ABC-Klassifizierung
   - Stock Reservations und Cycle Counting
   - Inventory Transactions und Transfers
   - Stock Alerts und Snapshots

3. **Analytics & Indexes** (`003_analytics_and_indexes.sql`)
   - Turnover Analysis und Demand Forecasting
   - ABC Analysis Results
   - Performance-Indexes
   - Full-text Search

4. **Seed Data** (`004_seed_data.sql`)
   - Demo-Tenant "ACME Corporation"
   - Produkte, Kunden, Lieferanten
   - Inventory-Daten und Transaktionen

## Rust-Integration

Nach dem Stack-Reset:

```bash
# Environment Variable setzen
set DATABASE_URL=postgresql://erp_admin:erp_secure_password_change_in_production@localhost:5432/erp_main

# SQLx Metadata vorbereiten
cargo sqlx prepare --workspace

# System kompilieren
cargo check --all --message-format=short

# Tests ausführen
cargo test --lib --all
```

## Entwicklungsworkflow

### Tägliche Entwicklung
```bash
# Stack starten
scripts\docker_start.bat

# Logs anzeigen
scripts\docker_logs.bat postgres

# System validieren
scripts\validate_docker_stack.bat

# Stack stoppen
scripts\docker_stop.bat
```

### Problembehebung
```bash
# Kompletter Reset bei Problemen
scripts\docker_reset.bat

# Schema-Status prüfen
docker exec erp-postgres psql -U erp_admin -d erp_main -c "\dt"

# Sample-Daten prüfen
docker exec erp-postgres psql -U erp_admin -d erp_main -c "SELECT count(*) FROM products;"
```

## Vorteile dieser Lösung

### ✅ Keine Migration-Konflikte
- Komplett sauberer Zustand bei jedem Reset
- Keine SQLx-Historie-Probleme
- Reproduzierbar auf jedem System

### ✅ Professional Development Setup
- Infrastructure as Code
- Versionskontrollierte Konfiguration
- Einfaches Onboarding für neue Entwickler

### ✅ Performance-Optimiert
- PostgreSQL-Konfiguration für Entwicklung
- Redis für Caching und Sessions
- Strategische Indexes für optimale Performance

### ✅ Vollständiges ERP-Schema
- Multi-tenant Architektur
- Komplette Inventory-Verwaltung
- ABC-Analyse und Forecasting
- Audit-Trails und Change-Tracking

## Datenbankschema-Highlights

### Core-Funktionalität
- **Multi-tenant:** Isolation durch tenant_id
- **Audit-Trails:** created_at, updated_at, created_by, updated_by
- **Flexible Attribute:** JSONB-Felder für Erweiterungen
- **Referenzielle Integrität:** Foreign Keys mit CASCADE/RESTRICT

### Inventory-Management
- **Multi-location:** Verschiedene Lagerorte pro Produkt
- **ABC-Klassifizierung:** Automatische Kategorisierung nach Wert
- **Cycle Counting:** Geplante und Ad-hoc Inventuren
- **Demand Forecasting:** Bedarfsprognosen mit Konfidenzintervallen

### Performance-Features
- **Full-text Search:** Produktsuche über Name, Beschreibung, SKU
- **Strategische Indexes:** Optimiert für häufige Abfragen
- **Materialized Views:** Für Inventory-Health-Dashboard
- **Partial Indexes:** Für aktive Records und Low-Stock-Alerts

## Technische Details

### Container-Architektur
```
docker-compose.yml
├── postgres (PostgreSQL 15)
│   ├── Automatische Initialisierung
│   ├── Performance-Konfiguration
│   ├── Health Checks
│   └── Persistent Volume
├── redis (Redis 7)
│   ├── AOF + RDB Persistence
│   ├── Memory-optimiert
│   ├── Security-konfiguriert
│   └── Persistent Volume
└── admin-tools (optional)
    ├── pgAdmin (Port 8080)
    └── redis-commander (Port 8081)
```

### Konfigurationsdateien
- `docker/postgres-config/postgresql.conf` - PostgreSQL Performance-Tuning
- `docker/redis-config/redis.conf` - Redis Session-Management
- `docker/postgres-init/*.sql` - Automatische Schema-Initialisierung

### Volume-Management
- `erp_postgres_data` - PostgreSQL Datenbank-Dateien
- `erp_redis_data` - Redis Persistence-Dateien
- `erp_pgadmin_data` - pgAdmin Konfiguration

## Migration von altem Setup

Falls Sie von einem bestehenden Setup migrieren:

1. **Daten sichern** (falls erforderlich)
2. **Altes System stoppen**
3. **Docker Stack reset:** `scripts\docker_reset.bat`
4. **System validieren:** `scripts\validate_docker_stack.bat`
5. **Rust neu kompilieren:** `cargo sqlx prepare --workspace && cargo check --all`

## Support

Bei Problemen:

1. **Logs prüfen:** `scripts\docker_logs.bat postgres`
2. **Validation laufen lassen:** `scripts\validate_docker_stack.bat`
3. **Kompletter Reset:** `scripts\docker_reset.bat`
4. **Container-Status:** `docker-compose ps`

Das System ist jetzt vollständig professionell aufgesetzt und bereit für die Entwicklung! 🎉