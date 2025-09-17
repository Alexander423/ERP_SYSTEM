# ERP System - Deployment Guide

## 📋 Inhaltsverzeichnis

1. [Deployment Übersicht](#deployment-übersicht)
2. [Systemanforderungen](#systemanforderungen)
3. [Lokale Entwicklung](#lokale-entwicklung)
4. [Docker Deployment](#docker-deployment)
5. [Kubernetes Deployment](#kubernetes-deployment)
6. [Cloud Deployment](#cloud-deployment)
7. [Monitoring & Logging](#monitoring--logging)
8. [Backup & Recovery](#backup--recovery)
9. [Security Configuration](#security-configuration)

## 🎯 Deployment Übersicht

Das ERP System kann auf verschiedene Weise deployed werden:

- **🔧 Lokale Entwicklung**: Direkt mit Cargo
- **🐳 Docker**: Containerisiert für einfache Bereitstellung
- **☸️ Kubernetes**: Orchestriert für Skalierbarkeit
- **☁️ Cloud**: AWS, Azure, GCP mit managed Services

## 🖥️ Systemanforderungen

### Minimale Anforderungen

| Komponente | Minimum | Empfohlen |
|------------|---------|-----------|
| CPU | 2 Cores | 4+ Cores |
| RAM | 4 GB | 8+ GB |
| Storage | 20 GB SSD | 100+ GB SSD |
| Network | 100 Mbps | 1 Gbps |

### Software Abhängigkeiten

```bash
# Rust (neueste stabile Version)
rustc 1.70.0+

# PostgreSQL
PostgreSQL 14.0+

# Optional: Redis für Caching
Redis 6.0+

# Optional: Elasticsearch für erweiterte Suche
Elasticsearch 8.0+
```

## 🔧 Lokale Entwicklung

### 1. Repository Setup

```bash
# Repository klonen
git clone https://github.com/your-org/erp-system.git
cd erp-system

# Rust Dependencies installieren
cargo build

# PostgreSQL starten (abhängig vom System)
# Windows: net start postgresql
# Linux: sudo systemctl start postgresql
# macOS: brew services start postgresql
```

### 2. Umgebungsvariablen

Erstellen Sie eine `.env` Datei:

```bash
# .env
DATABASE_URL=postgresql://erp_admin:erp_secure_password_change_in_production@localhost:5432/erp_main
REDIS_URL=redis://localhost:6379
JWT_SECRET=your-super-secret-jwt-key-change-this-in-production
ENCRYPTION_KEY=your-32-byte-encryption-key-base64-encoded
RUST_LOG=info
BIND_ADDRESS=0.0.0.0:8080

# Optional: External Services
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-app-password

# Optional: Cloud Services
AWS_REGION=eu-central-1
AWS_ACCESS_KEY_ID=your-access-key
AWS_SECRET_ACCESS_KEY=your-secret-key
```

### 3. Datenbank Setup

```bash
# Datenbank erstellen
createdb erp_main

# SQLX CLI installieren
cargo install sqlx-cli --no-default-features --features postgres

# Migrationen ausführen
sqlx migrate run

# Fehlende Spalten hinzufügen
psql -h localhost -U erp_admin -d erp_main -f migrations/20241216_006_final_missing_columns.sql

# Query Cache generieren
cargo sqlx prepare --workspace
```

### 4. Anwendung starten

```bash
# Development Mode
cargo run --bin erp-server

# Release Mode (für Testing)
cargo run --release --bin erp-server

# Mit spezifischer Konfiguration
RUST_LOG=debug cargo run --bin erp-server
```

## 🐳 Docker Deployment

### 1. Dockerfile

```dockerfile
# Multi-stage build für optimale Größe
FROM rust:1.70-slim as builder

# Build dependencies installieren
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

# Dependencies cachen
RUN cargo build --release

FROM debian:bullseye-slim

# Runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq5 \
    libssl1.1 \
    && rm -rf /var/lib/apt/lists/*

# Benutzer erstellen (Security Best Practice)
RUN useradd -m -u 1000 erp
USER erp

# Binary kopieren
COPY --from=builder /app/target/release/erp-server /usr/local/bin/
COPY --from=builder /app/migrations /app/migrations

EXPOSE 8080

CMD ["erp-server"]
```

### 2. Docker Compose

```yaml
# docker-compose.yml
version: '3.8'

services:
  postgres:
    image: postgres:15-alpine
    restart: unless-stopped
    environment:
      POSTGRES_DB: erp_main
      POSTGRES_USER: erp_admin
      POSTGRES_PASSWORD: ${DB_PASSWORD:-erp_secure_password_change_in_production}
      PGDATA: /var/lib/postgresql/data/pgdata
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./migrations:/docker-entrypoint-initdb.d
    ports:
      - "5432:5432"
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U erp_admin -d erp_main"]
      interval: 10s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    restart: unless-stopped
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 3s
      retries: 3

  erp-server:
    build: .
    restart: unless-stopped
    environment:
      DATABASE_URL: postgresql://erp_admin:${DB_PASSWORD:-erp_secure_password_change_in_production}@postgres:5432/erp_main
      REDIS_URL: redis://redis:6379
      JWT_SECRET: ${JWT_SECRET}
      ENCRYPTION_KEY: ${ENCRYPTION_KEY}
      RUST_LOG: ${RUST_LOG:-info}
      BIND_ADDRESS: 0.0.0.0:8080
    ports:
      - "8080:8080"
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  # Optional: Nginx Reverse Proxy
  nginx:
    image: nginx:alpine
    restart: unless-stopped
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
      - ./ssl:/etc/nginx/ssl
    depends_on:
      - erp-server

volumes:
  postgres_data:
  redis_data:
```

### 3. Environment Setup

```bash
# .env für Docker Compose
DB_PASSWORD=super_secure_production_password
JWT_SECRET=your-256-bit-secret-key-for-jwt-tokens
ENCRYPTION_KEY=your-32-byte-base64-encoded-encryption-key
RUST_LOG=info

# SSL Zertifikate (für HTTPS)
# Letsencrypt oder eigene Zertifikate in ./ssl/
```

### 4. Deployment ausführen

```bash
# Images bauen
docker-compose build

# Services starten
docker-compose up -d

# Logs überprüfen
docker-compose logs -f

# Health Check
curl http://localhost:8080/health

# Services stoppen
docker-compose down
```

## ☸️ Kubernetes Deployment

### 1. Namespace und ConfigMap

```yaml
# k8s/namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: erp-system

---
# k8s/configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: erp-config
  namespace: erp-system
data:
  RUST_LOG: "info"
  BIND_ADDRESS: "0.0.0.0:8080"
```

### 2. Secrets

```yaml
# k8s/secrets.yaml
apiVersion: v1
kind: Secret
metadata:
  name: erp-secrets
  namespace: erp-system
type: Opaque
data:
  # Base64 encoded values
  database-url: cG9zdGdyZXNxbDovL2VycF9hZG1pbjpwYXNzd29yZEBwb3N0Z3Jlczo1NDMyL2VycF9tYWlu
  jwt-secret: eW91ci1qd3Qtc2VjcmV0LWtleQ==
  encryption-key: eW91ci1lbmNyeXB0aW9uLWtleQ==
```

### 3. PostgreSQL Deployment

```yaml
# k8s/postgres.yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: postgres
  namespace: erp-system
spec:
  serviceName: postgres
  replicas: 1
  selector:
    matchLabels:
      app: postgres
  template:
    metadata:
      labels:
        app: postgres
    spec:
      containers:
      - name: postgres
        image: postgres:15-alpine
        env:
        - name: POSTGRES_DB
          value: erp_main
        - name: POSTGRES_USER
          value: erp_admin
        - name: POSTGRES_PASSWORD
          valueFrom:
            secretKeyRef:
              name: erp-secrets
              key: postgres-password
        ports:
        - containerPort: 5432
        volumeMounts:
        - name: postgres-storage
          mountPath: /var/lib/postgresql/data
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "2Gi"
            cpu: "1000m"
  volumeClaimTemplates:
  - metadata:
      name: postgres-storage
    spec:
      accessModes: ["ReadWriteOnce"]
      storageClassName: "fast-ssd"
      resources:
        requests:
          storage: 50Gi

---
apiVersion: v1
kind: Service
metadata:
  name: postgres
  namespace: erp-system
spec:
  selector:
    app: postgres
  ports:
  - port: 5432
    targetPort: 5432
```

### 4. ERP Server Deployment

```yaml
# k8s/erp-server.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: erp-server
  namespace: erp-system
spec:
  replicas: 3
  selector:
    matchLabels:
      app: erp-server
  template:
    metadata:
      labels:
        app: erp-server
    spec:
      containers:
      - name: erp-server
        image: erp-server:latest
        ports:
        - containerPort: 8080
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: erp-secrets
              key: database-url
        - name: JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: erp-secrets
              key: jwt-secret
        - name: ENCRYPTION_KEY
          valueFrom:
            secretKeyRef:
              name: erp-secrets
              key: encryption-key
        envFrom:
        - configMapRef:
            name: erp-config
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "2Gi"
            cpu: "1000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5

---
apiVersion: v1
kind: Service
metadata:
  name: erp-server
  namespace: erp-system
spec:
  selector:
    app: erp-server
  ports:
  - port: 80
    targetPort: 8080
  type: ClusterIP
```

### 5. Ingress Configuration

```yaml
# k8s/ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: erp-ingress
  namespace: erp-system
  annotations:
    kubernetes.io/ingress.class: nginx
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/rate-limit: "100"
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
spec:
  tls:
  - hosts:
    - api.erp-system.com
    secretName: erp-tls
  rules:
  - host: api.erp-system.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: erp-server
            port:
              number: 80
```

### 6. Deployment ausführen

```bash
# Namespace erstellen
kubectl apply -f k8s/namespace.yaml

# Secrets erstellen (mit echten Werten)
kubectl apply -f k8s/secrets.yaml

# ConfigMap erstellen
kubectl apply -f k8s/configmap.yaml

# PostgreSQL deployen
kubectl apply -f k8s/postgres.yaml

# Warten bis PostgreSQL bereit ist
kubectl wait --for=condition=ready pod -l app=postgres -n erp-system --timeout=300s

# ERP Server deployen
kubectl apply -f k8s/erp-server.yaml

# Ingress konfigurieren
kubectl apply -f k8s/ingress.yaml

# Status überprüfen
kubectl get pods -n erp-system
kubectl get services -n erp-system
kubectl logs -f deployment/erp-server -n erp-system
```

## ☁️ Cloud Deployment

### AWS Deployment mit EKS

```bash
# EKS Cluster erstellen
eksctl create cluster --name erp-cluster --region eu-central-1 --nodes 3

# RDS PostgreSQL erstellen
aws rds create-db-instance \
  --db-instance-identifier erp-postgres \
  --db-instance-class db.t3.medium \
  --engine postgres \
  --engine-version 15.4 \
  --allocated-storage 100 \
  --db-name erp_main \
  --master-username erp_admin \
  --master-user-password your-secure-password

# ElastiCache Redis erstellen
aws elasticache create-cache-cluster \
  --cache-cluster-id erp-redis \
  --cache-node-type cache.t3.micro \
  --engine redis \
  --num-cache-nodes 1

# Kubernetes Secrets aktualisieren mit AWS Endpunkten
kubectl create secret generic erp-secrets \
  --from-literal=database-url="postgresql://erp_admin:password@erp-postgres.region.rds.amazonaws.com:5432/erp_main" \
  --from-literal=redis-url="redis://erp-redis.cache.amazonaws.com:6379" \
  -n erp-system
```

### Azure Deployment mit AKS

```bash
# Resource Group erstellen
az group create --name erp-rg --location westeurope

# AKS Cluster erstellen
az aks create \
  --resource-group erp-rg \
  --name erp-cluster \
  --node-count 3 \
  --enable-addons monitoring

# Azure Database for PostgreSQL erstellen
az postgres server create \
  --resource-group erp-rg \
  --name erp-postgres \
  --location westeurope \
  --admin-user erp_admin \
  --admin-password your-secure-password \
  --sku-name GP_Gen5_2

# Azure Cache for Redis erstellen
az redis create \
  --resource-group erp-rg \
  --name erp-redis \
  --location westeurope \
  --sku Basic \
  --vm-size c0
```

## 📊 Monitoring & Logging

### Prometheus & Grafana

```yaml
# k8s/monitoring.yaml
apiVersion: v1
kind: ServiceMonitor
metadata:
  name: erp-server
  namespace: erp-system
spec:
  selector:
    matchLabels:
      app: erp-server
  endpoints:
  - port: metrics
    path: /metrics

---
apiVersion: v1
kind: ConfigMap
metadata:
  name: grafana-dashboard
  namespace: erp-system
data:
  erp-dashboard.json: |
    {
      "dashboard": {
        "title": "ERP System Dashboard",
        "panels": [
          {
            "title": "Request Rate",
            "type": "graph",
            "targets": [
              {
                "expr": "rate(http_requests_total[5m])"
              }
            ]
          }
        ]
      }
    }
```

### ELK Stack für Logging

```yaml
# k8s/elasticsearch.yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: elasticsearch
  namespace: erp-system
spec:
  serviceName: elasticsearch
  replicas: 1
  selector:
    matchLabels:
      app: elasticsearch
  template:
    metadata:
      labels:
        app: elasticsearch
    spec:
      containers:
      - name: elasticsearch
        image: docker.elastic.co/elasticsearch/elasticsearch:8.11.0
        env:
        - name: discovery.type
          value: single-node
        - name: ES_JAVA_OPTS
          value: "-Xms512m -Xmx512m"
        ports:
        - containerPort: 9200
        volumeMounts:
        - name: es-data
          mountPath: /usr/share/elasticsearch/data
  volumeClaimTemplates:
  - metadata:
      name: es-data
    spec:
      accessModes: ["ReadWriteOnce"]
      resources:
        requests:
          storage: 10Gi
```

## 🔒 Security Configuration

### SSL/TLS Setup

```bash
# Letsencrypt mit cert-manager
kubectl apply -f https://github.com/jetstack/cert-manager/releases/download/v1.13.0/cert-manager.yaml

# ClusterIssuer für Letsencrypt
kubectl apply -f - <<EOF
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: letsencrypt-prod
spec:
  acme:
    server: https://acme-v02.api.letsencrypt.org/directory
    email: admin@yourdomain.com
    privateKeySecretRef:
      name: letsencrypt-prod
    solvers:
    - http01:
        ingress:
          class: nginx
EOF
```

### Network Policies

```yaml
# k8s/network-policy.yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: erp-network-policy
  namespace: erp-system
spec:
  podSelector:
    matchLabels:
      app: erp-server
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - podSelector:
        matchLabels:
          app: nginx-ingress
    ports:
    - protocol: TCP
      port: 8080
  egress:
  - to:
    - podSelector:
        matchLabels:
          app: postgres
    ports:
    - protocol: TCP
      port: 5432
  - to:
    - podSelector:
        matchLabels:
          app: redis
    ports:
    - protocol: TCP
      port: 6379
```

## 💾 Backup & Recovery

### Automatisches Datenbank Backup

```bash
#!/bin/bash
# backup.sh

BACKUP_DIR="/var/backups/erp"
RETENTION_DAYS=30
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# PostgreSQL Backup
pg_dump $DATABASE_URL | gzip > "$BACKUP_DIR/erp_backup_$TIMESTAMP.sql.gz"

# Upload zu S3 (optional)
aws s3 cp "$BACKUP_DIR/erp_backup_$TIMESTAMP.sql.gz" s3://your-backup-bucket/

# Alte Backups löschen
find "$BACKUP_DIR" -name "*.sql.gz" -mtime +$RETENTION_DAYS -delete

echo "Backup completed: erp_backup_$TIMESTAMP.sql.gz"
```

### Kubernetes CronJob für Backups

```yaml
# k8s/backup-cronjob.yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: postgres-backup
  namespace: erp-system
spec:
  schedule: "0 2 * * *"  # Täglich um 2 Uhr
  jobTemplate:
    spec:
      template:
        spec:
          containers:
          - name: postgres-backup
            image: postgres:15-alpine
            env:
            - name: PGPASSWORD
              valueFrom:
                secretKeyRef:
                  name: erp-secrets
                  key: postgres-password
            command:
            - /bin/bash
            - -c
            - |
              TIMESTAMP=$(date +%Y%m%d_%H%M%S)
              pg_dump -h postgres -U erp_admin erp_main | gzip > /backup/erp_backup_$TIMESTAMP.sql.gz
              echo "Backup completed: erp_backup_$TIMESTAMP.sql.gz"
            volumeMounts:
            - name: backup-storage
              mountPath: /backup
          volumes:
          - name: backup-storage
            persistentVolumeClaim:
              claimName: backup-pvc
          restartPolicy: OnFailure
```

## 🔍 Troubleshooting

### Häufige Probleme

#### 1. Datenbankverbindung fehlgeschlagen

```bash
# Verbindung testen
psql $DATABASE_URL -c "SELECT version();"

# Network connectivity prüfen
kubectl exec -it deployment/erp-server -n erp-system -- nc -zv postgres 5432
```

#### 2. Memory/CPU Issues

```bash
# Resource Usage prüfen
kubectl top pods -n erp-system

# Limits anpassen
kubectl patch deployment erp-server -n erp-system -p '{"spec":{"template":{"spec":{"containers":[{"name":"erp-server","resources":{"limits":{"memory":"4Gi","cpu":"2000m"}}}]}}}}'
```

#### 3. SSL/TLS Probleme

```bash
# Zertifikat Status prüfen
kubectl describe certificate erp-tls -n erp-system

# cert-manager Logs prüfen
kubectl logs -n cert-manager deployment/cert-manager
```

### Logs und Debugging

```bash
# Application Logs
kubectl logs -f deployment/erp-server -n erp-system

# Structured Logs mit JSON
kubectl logs deployment/erp-server -n erp-system | jq .

# Fehlerbasierte Filterung
kubectl logs deployment/erp-server -n erp-system | grep ERROR

# Performance Profiling
kubectl port-forward -n erp-system deployment/erp-server 6060:6060
go tool pprof http://localhost:6060/debug/pprof/profile
```

---

**© 2024 Enterprise ERP System - Deployment Guide v1.0**