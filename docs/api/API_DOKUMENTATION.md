# ERP System - API Dokumentation

**Status**: ⚠️ **Development API - Limited Implementation**

Diese Dokumentation beschreibt die **tatsächlich implementierten** API-Endpoints des ERP Systems. Die meisten Endpoints sind noch in Entwicklung oder geben Mock-Daten zurück.

## 📋 Inhaltsverzeichnis

1. [API Überblick](#api-überblick)
2. [Health Check APIs](#health-check-apis)
3. [Authentifizierung APIs](#authentifizierung-apis)
4. [Customer Management APIs](#customer-management-apis)
5. [Fehlerbehandlung](#fehlerbehandlung)
6. [Entwicklungshinweise](#entwicklungshinweise)

## 🌐 API Überblick

### Base URL (Development)
```
http://localhost:3000/api/v1
```

### Content-Type
Alle API-Anfragen verwenden JSON:
```
Content-Type: application/json
```

### Authentifizierung
Die meisten APIs erwarten JWT Bearer Token (wenn implementiert):
```
Authorization: Bearer <jwt_token>
```

### Rate Limiting
- **Development**: Nicht aktiviert
- **Planned**: 1000 Anfragen pro Minute

## ✅ Health Check APIs

### System Health Check
**Status**: ✅ **Vollständig implementiert**

```http
GET /health

Response 200 OK:
{
  "status": "healthy",
  "timestamp": "2024-12-16T10:30:00Z"
}
```

### Readiness Check
**Status**: ✅ **Vollständig implementiert**

```http
GET /ready

Response 200 OK:
{
  "status": "ready",
  "timestamp": "2024-12-16T10:30:00Z"
}
```

## 🔐 Authentifizierung APIs

### User Login
**Status**: 🚧 **Teilweise implementiert - Mock Responses**

```http
POST /auth/login
Content-Type: application/json

Request:
{
  "email": "user@example.com",
  "password": "secure_password"
}

Success Response 200 OK:
{
  "success": true,
  "access_token": "mock_jwt_token_here",
  "refresh_token": "mock_refresh_token_here",
  "expires_in": 1800,
  "requires_2fa": false,
  "session_token": null
}

Error Response 200 OK (Invalid Credentials):
{
  "success": false,
  "access_token": null,
  "refresh_token": null,
  "expires_in": null,
  "requires_2fa": null,
  "session_token": null
}
```

**Implementierungsstand**:
- ✅ Endpoint vorhanden und erreichbar
- ✅ JSON Request/Response Verarbeitung
- 🚧 Passwort-Validierung ist Mock-Implementation
- ❌ Keine echte Authentifizierung gegen Datenbank
- ❌ JWT Tokens sind Platzhalter

### Tenant Registration
**Status**: 🚧 **Teilweise implementiert**

```http
POST /auth/register
Content-Type: application/json

Request:
{
  "company_name": "Beispiel GmbH",
  "email": "admin@beispiel.com",
  "password": "secure_password",
  "first_name": "Max",
  "last_name": "Mustermann"
}

Success Response 200 OK:
{
  "success": true,
  "tenant_id": "550e8400-e29b-41d4-a716-446655440000",
  "user_id": "123e4567-e89b-12d3-a456-426614174000",
  "message": "Registration successful"
}

Error Response 200 OK:
{
  "success": false,
  "error": "Registration failed",
  "message": "Error details here"
}
```

**Implementierungsstand**:
- ✅ Endpoint vorhanden
- 🚧 Basis-Validierung implementiert
- ❌ Echte Tenant-Erstellung nicht vollständig
- ❌ Email-Verifizierung nicht implementiert

### Token Refresh
**Status**: 🚧 **Mock Implementation**

```http
POST /auth/refresh-token
Content-Type: application/json

Request:
{
  "refresh_token": "mock_refresh_token"
}

Success Response 200 OK:
{
  "success": true,
  "access_token": "new_mock_access_token",
  "refresh_token": "new_mock_refresh_token",
  "expires_in": 1800
}
```

### User Logout
**Status**: 🚧 **Placeholder Implementation**

```http
POST /auth/logout

Response 200 OK:
{
  "success": true,
  "message": "Logged out successfully"
}
```

**Hinweis**: Aktuell nur Placeholder-Response, keine echte Session-Verwaltung.

### Token Validation
**Status**: 🚧 **Mock Implementation mit Error Handling Demo**

```http
POST /auth/validate
Content-Type: application/json

Request:
{
  "token": "jwt_token_to_validate"
}

Success Response 200 OK:
{
  "valid": true,
  "message": "Token is valid",
  "token_type": "bearer"
}

Error Response (Various HTTP Status Codes):
// 400 Bad Request für leere/fehlende Token
// 401 Unauthorized für ungültige Token
// Siehe Fehlerbehandlung für Details
```

**Hinweis**: Demonstriert strukturierte Fehlerbehandlung, aber Token-Validierung ist Mock-Implementation.

## 👥 Customer Management APIs

**Status**: ❌ **Mock Implementation - Keine echten Daten**

### Customer List
```http
GET /customers

Response 200 OK:
{
  "customers": [],
  "total_count": 0,
  "has_more": false,
  "pagination": {
    "current_page": 1,
    "total_pages": 0,
    "limit": 20,
    "offset": 0
  }
}
```

### Customer Create
```http
POST /customers
Content-Type: application/json

Request:
{
  "customer_number": "CUST-001",
  "legal_name": "Beispiel GmbH",
  "display_name": "Beispiel Firma"
}

Response 201 Created:
{
  "id": "generated_uuid_placeholder",
  "message": "Customer creation endpoint received data"
}
```

### Customer Read/Update/Delete
```http
GET /customers/{id}
PUT /customers/{id}
DELETE /customers/{id}

Response: Mock responses with placeholder data
```

**Implementierungsstand**:
- ✅ Alle REST-Endpoints definiert und erreichbar
- ✅ JSON Request/Response Processing
- ❌ Keine Verbindung zur Customer Repository
- ❌ Keine echte Datenbank-Integration
- ❌ Keine Validierung der Business Logic

## ❌ Fehlerbehandlung

### Standard Error Format
```json
{
  "error": {
    "code": "VALIDATION_FAILED",
    "message": "Token is required",
    "timestamp": "2024-12-16T10:30:00Z",
    "request_id": "generated_request_id"
  }
}
```

### HTTP Status Codes

| Code | Verwendung | Implementiert |
|------|-----------|---------------|
| 200 | OK - Erfolgreiche Anfrage | ✅ |
| 400 | Bad Request - Validation Errors | ✅ (Teilweise) |
| 401 | Unauthorized - Auth erforderlich | ✅ (Mock) |
| 404 | Not Found - Ressource nicht gefunden | ✅ |
| 500 | Internal Server Error | ✅ |

### Aktuelle Error Codes

| Code | Beschreibung | Status |
|------|--------------|---------|
| `VALIDATION_FAILED` | Input validation fehler | ✅ Implementiert |
| `AUTHENTICATION_FAILED` | Login fehlgeschlagen | 🚧 Mock |
| `RESOURCE_NOT_FOUND` | 404 Fehler | ✅ Implementiert |
| `INTERNAL_ERROR` | Server-Fehler | ✅ Implementiert |

## 🚧 Entwicklungshinweise

### Was funktioniert aktuell:
1. **HTTP Server**: Startet erfolgreich und ist erreichbar
2. **Routing**: Alle definierten Endpoints sind verfügbar
3. **Middleware**: Security Headers, CORS, Request ID generation
4. **JSON Processing**: Request/Response Serialisierung funktioniert
5. **Error Handling**: Strukturierte Fehlerantworten
6. **Health Checks**: Vollständig funktionsfähig

### Was NICHT funktioniert:
1. **Echte Authentifizierung**: JWT Validation ist nicht implementiert
2. **Datenbank Integration**: API-Handler sind nicht mit Repository verbunden
3. **Business Logic**: Keine echten ERP-Workflows
4. **Multi-Tenant**: Tenant-Kontext wird nicht verarbeitet
5. **Autorisierung**: Keine Rollen- oder Permissions-Prüfung

### Nächste Entwicklungsschritte:
1. Verbindung von API-Handlern mit Repository-Layer
2. Implementierung echter JWT-Authentifizierung
3. Integration der Customer Repository in die API
4. Tenant-Context Middleware aktivieren
5. Input-Validierung erweitern

## 🛠️ Lokale Entwicklung

### API Server starten:
```bash
# Mit Datenbank-Verbindung
DATABASE_URL="postgresql://user:pass@localhost:5432/erp" cargo run -p erp-api

# Server läuft auf: http://localhost:3000
# Health Check: http://localhost:3000/health
# API Base: http://localhost:3000/api/v1/
```

### API Testen:
```bash
# Health Check
curl http://localhost:3000/health

# Login (Mock)
curl -X POST http://localhost:3000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@test.com","password":"test123"}'

# Customer List (Mock)
curl http://localhost:3000/api/v1/customers
```

---

## ⚠️ Wichtige Hinweise

**Diese API ist aktuell NUR für die Entwicklung geeignet:**

1. **Keine Produktionstauglichkeit**: Mock-Implementierungen überall
2. **Keine echte Sicherheit**: Authentifizierung ist Placeholder
3. **Keine Datenpersistierung**: Customer-APIs speichern nichts
4. **Entwicklung im Gange**: Funktionalität wird kontinuierlich erweitert

**Für echte Integration warten Sie auf:**
- Repository-Integration (Q1 2025)
- Echte Authentifizierung (Q1 2025)
- Business Logic Implementation (Q2 2025)

---

**© 2024 Enterprise ERP System - API Dokumentation v0.1.0-alpha**
**Status**: Foundation Layer - Mock Implementation Phase