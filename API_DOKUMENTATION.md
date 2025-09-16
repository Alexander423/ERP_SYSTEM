# ERP System - API Dokumentation

## 📋 Inhaltsverzeichnis

1. [API Überblick](#api-überblick)
2. [Authentifizierung](#authentifizierung)
3. [Kundenverwaltung APIs](#kundenverwaltung-apis)
4. [Sicherheits APIs](#sicherheits-apis)
5. [Analytics APIs](#analytics-apis)
6. [Search APIs](#search-apis)
7. [Admin APIs](#admin-apis)
8. [Fehlerbehandlung](#fehlerbehandlung)

## 🌐 API Überblick

### Base URL
```
https://api.erp-system.com/api/v1
```

### Content-Type
Alle API-Anfragen verwenden JSON:
```
Content-Type: application/json
```

### Authentifizierung
Alle APIs erfordern JWT Bearer Token:
```
Authorization: Bearer <jwt_token>
```

### Rate Limiting
- Standard: 1000 Anfragen pro Minute
- Burst: 100 Anfragen pro Sekunde
- Enterprise: Unbegrenzt (konfigurierbar)

## 🔐 Authentifizierung

### Login
```http
POST /auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "secure_password",
  "tenant_id": "550e8400-e29b-41d4-a716-446655440000"
}

Response:
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_in": 3600,
  "user": {
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "email": "user@example.com",
    "roles": ["customer_manager"],
    "permissions": ["customers:read", "customers:write"]
  }
}
```

### Token Refresh
```http
POST /auth/refresh
Content-Type: application/json

{
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}

Response:
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_in": 3600
}
```

## 🏢 Kundenverwaltung APIs

### Kunden erstellen
```http
POST /customers
Content-Type: application/json
Authorization: Bearer <token>

{
  "customer_number": "CUST-001",
  "legal_name": "Beispiel GmbH",
  "display_name": "Beispiel Firma",
  "customer_type": "Business",
  "lifecycle_stage": "Lead",
  "industry_classification": "Technology",
  "business_size": "Medium",
  "parent_customer_id": null,
  "addresses": [
    {
      "address_type": "Billing",
      "street1": "Musterstraße 123",
      "street2": "2. OG",
      "city": "Berlin",
      "state_province": "Berlin",
      "postal_code": "10115",
      "country": "DE",
      "is_primary": true
    }
  ],
  "contacts": [
    {
      "contact_type": "Primary",
      "first_name": "Max",
      "last_name": "Mustermann",
      "email": "max@beispiel.com",
      "phone": "+49-30-12345678",
      "position": "Geschäftsführer",
      "is_primary": true
    }
  ],
  "financial_info": {
    "currency_code": "EUR",
    "credit_limit": 50000.00,
    "payment_terms": {
      "payment_method": "BankTransfer",
      "net_days": 30
    },
    "tax_exempt": false
  },
  "tags": ["premium", "technology"],
  "notes": "Wichtiger Kunde für Technologie-Bereich",
  "custom_fields": {
    "segment": "enterprise",
    "acquisition_channel": "website"
  }
}

Response:
{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "customer_number": "CUST-001",
  "legal_name": "Beispiel GmbH",
  "display_name": "Beispiel Firma",
  "customer_type": "Business",
  "lifecycle_stage": "Lead",
  "status": "Active",
  "created_at": "2024-12-16T10:30:00Z",
  "modified_at": "2024-12-16T10:30:00Z",
  "version": 1
}
```

### Kunde abrufen
```http
GET /customers/{id}
Authorization: Bearer <token>

Response:
{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "customer_number": "CUST-001",
  "legal_name": "Beispiel GmbH",
  "display_name": "Beispiel Firma",
  "customer_type": "Business",
  "lifecycle_stage": "Lead",
  "addresses": [...],
  "contacts": [...],
  "financial_info": {...},
  "performance_metrics": {
    "total_revenue": 125000.50,
    "total_orders": 45,
    "customer_lifetime_value": 15000.00,
    "satisfaction_score": 8.5,
    "last_order_date": "2024-12-10T14:20:00Z"
  },
  "created_at": "2024-12-16T10:30:00Z",
  "modified_at": "2024-12-16T10:30:00Z",
  "version": 3
}
```

### Kunde aktualisieren
```http
PUT /customers/{id}
Content-Type: application/json
Authorization: Bearer <token>

{
  "legal_name": "Neue Beispiel GmbH",
  "lifecycle_stage": "ActiveCustomer",
  "version": 3
}

Response:
{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "legal_name": "Neue Beispiel GmbH",
  "lifecycle_stage": "ActiveCustomer",
  "modified_at": "2024-12-16T11:45:00Z",
  "version": 4
}
```

### Kunden auflisten
```http
GET /customers?limit=20&offset=0&sort=created_at&order=desc
Authorization: Bearer <token>

Query Parameters:
- limit: 1-100 (default: 20)
- offset: 0+ (default: 0)
- sort: created_at|modified_at|legal_name|customer_number
- order: asc|desc (default: asc)
- status: Active|Inactive|Pending|Suspended
- customer_type: B2B|B2C|B2G|Internal
- lifecycle_stage: Lead|Prospect|NewCustomer|ActiveCustomer|...

Response:
{
  "customers": [
    {
      "id": "123e4567-e89b-12d3-a456-426614174000",
      "customer_number": "CUST-001",
      "legal_name": "Beispiel GmbH",
      "customer_type": "Business",
      "lifecycle_stage": "Lead",
      "created_at": "2024-12-16T10:30:00Z"
    }
  ],
  "total_count": 1,
  "has_more": false,
  "pagination": {
    "current_page": 1,
    "total_pages": 1,
    "limit": 20,
    "offset": 0
  }
}
```

### Kunde löschen
```http
DELETE /customers/{id}?version=4
Authorization: Bearer <token>

Response:
{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "status": "deleted",
  "deleted_at": "2024-12-16T12:00:00Z"
}
```

## 🔍 Search APIs

### Volltext-Suche
```http
GET /customers/search?q=mustermann&limit=10&offset=0
Authorization: Bearer <token>

Query Parameters:
- q: Suchbegriff (required)
- limit: 1-100 (default: 10)
- offset: 0+ (default: 0)
- include_deleted: true|false (default: false)
- fields: legal_name,display_name,email,phone (default: all)

Response:
{
  "customers": [
    {
      "id": "123e4567-e89b-12d3-a456-426614174000",
      "customer_number": "CUST-001",
      "legal_name": "Max Mustermann GmbH",
      "display_name": "Mustermann Firma",
      "relevance_score": 0.95,
      "highlighted_fields": {
        "legal_name": "<mark>Mustermann</mark> GmbH"
      }
    }
  ],
  "total_count": 1,
  "query_time_ms": 15,
  "suggestions": ["musterman", "muster"]
}
```

### Erweiterte Suche
```http
POST /customers/advanced-search
Content-Type: application/json
Authorization: Bearer <token>

{
  "query": {
    "must": [
      {"term": {"customer_type": "Business"}},
      {"range": {"created_at": {"gte": "2024-01-01"}}}
    ],
    "should": [
      {"match": {"legal_name": "GmbH"}},
      {"term": {"tags": "premium"}}
    ]
  },
  "filters": {
    "lifecycle_stage": ["Lead", "Prospect"],
    "business_size": ["Medium", "Large"],
    "min_revenue": 10000.00,
    "max_revenue": 100000.00
  },
  "sort": [
    {"customer_lifetime_value": {"order": "desc"}},
    {"created_at": {"order": "asc"}}
  ],
  "limit": 50,
  "offset": 0
}

Response:
{
  "customers": [...],
  "total_count": 25,
  "facets": {
    "customer_types": {
      "Business": 20,
      "Individual": 5
    },
    "lifecycle_stages": {
      "Lead": 15,
      "Prospect": 10
    }
  },
  "query_time_ms": 45
}
```

## 📊 Analytics APIs

### Customer Lifetime Value
```http
GET /analytics/customers/{id}/clv
Authorization: Bearer <token>

Response:
{
  "customer_id": "123e4567-e89b-12d3-a456-426614174000",
  "customer_lifetime_value": 15000.50,
  "calculation_method": "predicted",
  "confidence_score": 0.85,
  "calculation_date": "2024-12-16T10:30:00Z",
  "breakdown": {
    "historical_value": 8500.00,
    "predicted_value": 6500.50,
    "time_horizon_months": 24
  },
  "factors": {
    "purchase_frequency": 0.8,
    "average_order_value": 850.00,
    "retention_probability": 0.75
  }
}
```

### Churn Prediction
```http
GET /analytics/customers/{id}/churn-risk
Authorization: Bearer <token>

Response:
{
  "customer_id": "123e4567-e89b-12d3-a456-426614174000",
  "churn_probability": 0.15,
  "risk_level": "Low",
  "prediction_confidence": 0.92,
  "prediction_date": "2024-12-16T10:30:00Z",
  "risk_factors": [
    {
      "factor": "payment_delays",
      "impact": 0.05,
      "severity": "low"
    },
    {
      "factor": "reduced_engagement",
      "impact": 0.10,
      "severity": "medium"
    }
  ],
  "recommended_actions": [
    {
      "action": "retention_campaign",
      "priority": "medium",
      "expected_impact": 0.08
    },
    {
      "action": "personal_outreach",
      "priority": "high",
      "expected_impact": 0.12
    }
  ]
}
```

### Kundensegmentierung
```http
GET /analytics/segmentation?criteria=behavioral&min_customers=10
Authorization: Bearer <token>

Response:
{
  "segments": [
    {
      "id": "high-value-frequent",
      "name": "High-Value Frequent Buyers",
      "description": "Customers with high CLV and frequent purchases",
      "customer_count": 45,
      "criteria": {
        "clv_min": 10000.00,
        "purchase_frequency_min": 0.5,
        "last_purchase_days_max": 90
      },
      "metrics": {
        "avg_clv": 18500.00,
        "avg_order_value": 950.00,
        "retention_rate": 0.89
      }
    },
    {
      "id": "at-risk-high-value",
      "name": "At-Risk High-Value",
      "description": "High-value customers with declining engagement",
      "customer_count": 12,
      "criteria": {
        "clv_min": 15000.00,
        "churn_probability_min": 0.3,
        "last_purchase_days_min": 180
      },
      "metrics": {
        "avg_clv": 22000.00,
        "avg_churn_probability": 0.45,
        "recommended_actions": ["urgent_retention", "personal_call"]
      }
    }
  ],
  "total_customers_segmented": 157,
  "segmentation_date": "2024-12-16T10:30:00Z"
}
```

### Performance Dashboard
```http
GET /analytics/dashboard?period=last_30_days
Authorization: Bearer <token>

Response:
{
  "period": {
    "start_date": "2024-11-16T00:00:00Z",
    "end_date": "2024-12-16T23:59:59Z",
    "duration_days": 30
  },
  "key_metrics": {
    "total_customers": 1247,
    "new_customers": 89,
    "active_customers": 967,
    "churn_rate": 0.03,
    "avg_clv": 12450.00,
    "total_revenue": 1250000.00
  },
  "trends": {
    "customer_acquisition": {
      "current_period": 89,
      "previous_period": 76,
      "change_percent": 17.1
    },
    "revenue_per_customer": {
      "current_period": 1001.60,
      "previous_period": 945.20,
      "change_percent": 5.97
    }
  },
  "top_segments": [
    {
      "segment_name": "Enterprise",
      "customer_count": 45,
      "revenue_contribution": 0.42
    }
  ]
}
```

## 🔐 Sicherheits APIs

### Verschlüsselung Management
```http
POST /security/encryption/encrypt-field
Content-Type: application/json
Authorization: Bearer <token>

{
  "field_name": "tax_number",
  "value": "DE123456789",
  "data_classification": "Confidential",
  "encryption_context": {
    "customer_id": "123e4567-e89b-12d3-a456-426614174000",
    "user_id": "456e7890-e12b-34c5-d678-901234567890"
  }
}

Response:
{
  "encrypted_value": "AQECAHjR...encrypted_data...",
  "encryption_key_id": "arn:aws:kms:eu-central-1:123456789:key/12345678-1234",
  "algorithm": "AES-256-GCM",
  "nonce": "random_nonce_value"
}
```

### Audit Log Abfrage
```http
GET /security/audit/events?user_id=456e7890&resource_type=customer&limit=50
Authorization: Bearer <token>

Query Parameters:
- user_id: UUID (optional)
- resource_type: string (optional)
- action: string (optional)
- start_date: ISO 8601 date (optional)
- end_date: ISO 8601 date (optional)
- risk_level: Low|Medium|High|Critical (optional)

Response:
{
  "events": [
    {
      "id": "789e0123-e45f-67g8-h901-234567890123",
      "event_type": "DataAccess",
      "user_id": "456e7890-e12b-34c5-d678-901234567890",
      "resource_type": "customer",
      "resource_id": "123e4567-e89b-12d3-a456-426614174000",
      "action": "read",
      "outcome": "Success",
      "risk_level": "Low",
      "timestamp": "2024-12-16T10:30:00Z",
      "ip_address": "192.168.1.100",
      "user_agent": "Mozilla/5.0...",
      "session_id": "sess_123456789",
      "details": {
        "fields_accessed": ["legal_name", "customer_number"],
        "access_reason": "customer_inquiry"
      }
    }
  ],
  "total_count": 1,
  "query_time_ms": 25
}
```

### Rollenmanagement
```http
POST /security/roles
Content-Type: application/json
Authorization: Bearer <token>

{
  "name": "Customer Manager",
  "description": "Vollzugriff auf Kundenverwaltung",
  "permissions": [
    {
      "resource": "customers",
      "action": "read",
      "scope": "tenant"
    },
    {
      "resource": "customers",
      "action": "write",
      "scope": "tenant",
      "conditions": [
        {
          "type": "TimeRestriction",
          "allowed_hours": [9, 17],
          "allowed_days": [1, 2, 3, 4, 5]
        }
      ]
    }
  ],
  "is_system_role": false,
  "priority": 100
}

Response:
{
  "id": "role_789e0123-e45f-67g8-h901-234567890123",
  "name": "Customer Manager",
  "created_at": "2024-12-16T10:30:00Z",
  "permission_count": 2
}
```

## 🛠️ Admin APIs

### System Health
```http
GET /admin/health
Authorization: Bearer <admin_token>

Response:
{
  "status": "healthy",
  "version": "1.0.0",
  "timestamp": "2024-12-16T10:30:00Z",
  "components": {
    "database": {
      "status": "healthy",
      "response_time_ms": 5,
      "connection_pool": {
        "active": 8,
        "idle": 12,
        "max": 20
      }
    },
    "cache": {
      "status": "healthy",
      "hit_rate": 0.95,
      "memory_usage_mb": 256
    },
    "external_services": {
      "email_service": {
        "status": "healthy",
        "last_check": "2024-12-16T10:29:00Z"
      }
    }
  },
  "metrics": {
    "requests_per_minute": 1250,
    "avg_response_time_ms": 85,
    "error_rate": 0.001
  }
}
```

### Performance Metriken
```http
GET /admin/metrics?period=last_hour
Authorization: Bearer <admin_token>

Response:
{
  "period": {
    "start": "2024-12-16T09:30:00Z",
    "end": "2024-12-16T10:30:00Z"
  },
  "performance": {
    "requests_total": 75000,
    "requests_per_second_avg": 20.8,
    "response_time_p50_ms": 45,
    "response_time_p95_ms": 120,
    "response_time_p99_ms": 250,
    "error_rate": 0.0015
  },
  "database": {
    "queries_total": 125000,
    "query_time_avg_ms": 12,
    "slow_queries_count": 3,
    "connection_pool_utilization": 0.65
  },
  "memory": {
    "heap_used_mb": 512,
    "heap_max_mb": 2048,
    "gc_collections": 15
  }
}
```

## ❌ Fehlerbehandlung

### Standardfehlerformat
```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Customer validation failed",
    "details": [
      {
        "field": "email",
        "message": "Invalid email format",
        "code": "INVALID_FORMAT"
      }
    ],
    "timestamp": "2024-12-16T10:30:00Z",
    "request_id": "req_123456789"
  }
}
```

### HTTP Status Codes

| Code | Bedeutung | Beschreibung |
|------|-----------|--------------|
| 200 | OK | Anfrage erfolgreich |
| 201 | Created | Ressource erstellt |
| 400 | Bad Request | Ungültige Anfrage |
| 401 | Unauthorized | Authentifizierung erforderlich |
| 403 | Forbidden | Zugriff verweigert |
| 404 | Not Found | Ressource nicht gefunden |
| 409 | Conflict | Ressourcenkonflikt (z.B. Versionierung) |
| 422 | Unprocessable Entity | Validierungsfehler |
| 429 | Too Many Requests | Rate Limit überschritten |
| 500 | Internal Server Error | Serverfehler |

### Fehlercodes

| Code | Beschreibung |
|------|--------------|
| VALIDATION_ERROR | Eingabevalidierung fehlgeschlagen |
| AUTHENTICATION_FAILED | Authentifizierung fehlgeschlagen |
| AUTHORIZATION_DENIED | Berechtigung verweigert |
| RESOURCE_NOT_FOUND | Ressource nicht gefunden |
| DUPLICATE_RESOURCE | Ressource bereits vorhanden |
| VERSION_CONFLICT | Versionierungskonflikt |
| RATE_LIMIT_EXCEEDED | Rate Limit überschritten |
| INTERNAL_ERROR | Interner Serverfehler |

---

**© 2024 Enterprise ERP System - API Dokumentation v1.0**