# QuillSpace Deployment Guide

## Overview

This guide covers deploying QuillSpace in production environments, from single-server setups to large-scale Kubernetes clusters. QuillSpace is designed to be cloud-native and supports various deployment strategies.

## Deployment Options

### 1. Development/Staging
- **Single Server**: Docker Compose for quick setup
- **Resources**: 2 CPU, 4GB RAM, 50GB storage
- **Use Case**: Development, testing, small-scale demos

### 2. Production (Small)
- **Container Platform**: Docker Swarm or single Kubernetes node
- **Resources**: 4 CPU, 8GB RAM, 200GB storage
- **Use Case**: Small businesses, up to 100 tenants

### 3. Production (Large)
- **Platform**: Kubernetes cluster with auto-scaling
- **Resources**: 16+ CPU, 32GB+ RAM, 1TB+ storage
- **Use Case**: Enterprise, 1000+ tenants, high availability

## Prerequisites

### Infrastructure Requirements

| Component | Minimum | Recommended | Notes |
|-----------|---------|-------------|-------|
| **CPU** | 2 cores | 4+ cores | Rust benefits from multiple cores |
| **Memory** | 4GB | 8GB+ | ClickHouse requires significant RAM |
| **Storage** | 50GB | 200GB+ SSD | Fast storage for database performance |
| **Network** | 100Mbps | 1Gbps+ | High throughput for API requests |

### Software Dependencies

- **Container Runtime**: Docker 20.10+ or containerd
- **Orchestration**: Kubernetes 1.24+ (recommended) or Docker Compose
- **Database**: PostgreSQL 15+, ClickHouse 23+
- **Cache**: Redis 7+
- **Load Balancer**: Nginx, Traefik, or cloud LB

## Docker Compose Deployment

### Quick Start

Create a `docker-compose.yml` file:

```yaml
version: '3.8'

services:
  # PostgreSQL - System of Record
  postgres:
    image: postgres:15-alpine
    environment:
      POSTGRES_DB: quillspace
      POSTGRES_USER: quillspace
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./init.sql:/docker-entrypoint-initdb.d/init.sql
    ports:
      - "5432:5432"
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U quillspace"]
      interval: 10s
      timeout: 5s
      retries: 5

  # ClickHouse - Analytics
  clickhouse:
    image: clickhouse/clickhouse-server:23-alpine
    environment:
      CLICKHOUSE_DB: analytics
      CLICKHOUSE_USER: quillspace
      CLICKHOUSE_PASSWORD: ${CLICKHOUSE_PASSWORD}
    volumes:
      - clickhouse_data:/var/lib/clickhouse
      - ./clickhouse-config.xml:/etc/clickhouse-server/config.xml
    ports:
      - "8123:8123"
      - "9000:9000"
    healthcheck:
      test: ["CMD", "wget", "--no-verbose", "--tries=1", "--spider", "http://localhost:8123/ping"]
      interval: 10s
      timeout: 5s
      retries: 5

  # Redis - Cache & Sessions
  redis:
    image: redis:7-alpine
    command: redis-server --requirepass ${REDIS_PASSWORD}
    volumes:
      - redis_data:/data
    ports:
      - "6379:6379"
    healthcheck:
      test: ["CMD", "redis-cli", "--raw", "incr", "ping"]
      interval: 10s
      timeout: 3s
      retries: 5

  # QuillSpace API Server
  api:
    image: quillspace/api:latest
    environment:
      DATABASE_URL: postgresql://quillspace:${POSTGRES_PASSWORD}@postgres:5432/quillspace
      CLICKHOUSE_URL: http://quillspace:${CLICKHOUSE_PASSWORD}@clickhouse:8123/analytics
      REDIS_URL: redis://:${REDIS_PASSWORD}@redis:6379
      JWT_SECRET: ${JWT_SECRET}
      RUST_LOG: info
      PORT: 3000
    ports:
      - "3000:3000"
    depends_on:
      postgres:
        condition: service_healthy
      clickhouse:
        condition: service_healthy
      redis:
        condition: service_healthy
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3
    restart: unless-stopped

  # Qwik Frontend
  frontend:
    image: quillspace/frontend:latest
    environment:
      QWIK_API_URL: http://api:3000
      NODE_ENV: production
    ports:
      - "80:3000"
    depends_on:
      - api
    restart: unless-stopped

  # Nginx Load Balancer
  nginx:
    image: nginx:alpine
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
      - ./ssl:/etc/nginx/ssl
    ports:
      - "443:443"
      - "80:80"
    depends_on:
      - frontend
      - api
    restart: unless-stopped

volumes:
  postgres_data:
  clickhouse_data:
  redis_data:
```

### Environment Configuration

Create a `.env` file:

```bash
# Database Passwords
POSTGRES_PASSWORD=your_secure_postgres_password
CLICKHOUSE_PASSWORD=your_secure_clickhouse_password
REDIS_PASSWORD=your_secure_redis_password

# Application Secrets
JWT_SECRET=your_jwt_secret_key_minimum_32_characters
API_KEY=your_api_key_for_external_services

# Domain Configuration
DOMAIN=yourdomain.com
SSL_EMAIL=admin@yourdomain.com

# Monitoring
PROMETHEUS_ENABLED=true
GRAFANA_PASSWORD=your_grafana_password
```

### Deployment Commands

```bash
# Clone the repository
git clone <repository-url>
cd quillspace

# Create environment file
cp .env.example .env
# Edit .env with your values

# Start services
docker-compose up -d

# Check service health
docker-compose ps
docker-compose logs -f api

# Run database migrations
docker-compose exec api cargo run --bin migrate

# Scale API servers
docker-compose up -d --scale api=3
```

## Kubernetes Deployment

### Namespace and ConfigMap

```yaml
# namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: quillspace
---
# configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: quillspace-config
  namespace: quillspace
data:
  RUST_LOG: "info"
  DATABASE_HOST: "postgres-service"
  CLICKHOUSE_HOST: "clickhouse-service"
  REDIS_HOST: "redis-service"
```

### Database Deployments

```yaml
# postgres.yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: postgres
  namespace: quillspace
spec:
  serviceName: postgres-service
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
          value: quillspace
        - name: POSTGRES_USER
          value: quillspace
        - name: POSTGRES_PASSWORD
          valueFrom:
            secretKeyRef:
              name: postgres-secret
              key: password
        ports:
        - containerPort: 5432
        volumeMounts:
        - name: postgres-storage
          mountPath: /var/lib/postgresql/data
        resources:
          requests:
            memory: "1Gi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "1000m"
  volumeClaimTemplates:
  - metadata:
      name: postgres-storage
    spec:
      accessModes: ["ReadWriteOnce"]
      resources:
        requests:
          storage: 100Gi
---
apiVersion: v1
kind: Service
metadata:
  name: postgres-service
  namespace: quillspace
spec:
  selector:
    app: postgres
  ports:
  - port: 5432
    targetPort: 5432
```

### API Server Deployment

```yaml
# api-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: quillspace-api
  namespace: quillspace
spec:
  replicas: 3
  selector:
    matchLabels:
      app: quillspace-api
  template:
    metadata:
      labels:
        app: quillspace-api
    spec:
      containers:
      - name: api
        image: quillspace/api:latest
        ports:
        - containerPort: 3000
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: database-secret
              key: url
        - name: JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: jwt-secret
              key: secret
        envFrom:
        - configMapRef:
            name: quillspace-config
        livenessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 5
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
---
apiVersion: v1
kind: Service
metadata:
  name: api-service
  namespace: quillspace
spec:
  selector:
    app: quillspace-api
  ports:
  - port: 80
    targetPort: 3000
  type: ClusterIP
```

### Horizontal Pod Autoscaler

```yaml
# hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: quillspace-api-hpa
  namespace: quillspace
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: quillspace-api
  minReplicas: 3
  maxReplicas: 20
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

### Ingress Configuration

```yaml
# ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: quillspace-ingress
  namespace: quillspace
  annotations:
    kubernetes.io/ingress.class: nginx
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/rate-limit: "100"
    nginx.ingress.kubernetes.io/rate-limit-window: "1m"
spec:
  tls:
  - hosts:
    - api.yourdomain.com
    - app.yourdomain.com
    secretName: quillspace-tls
  rules:
  - host: api.yourdomain.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: api-service
            port:
              number: 80
  - host: app.yourdomain.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: frontend-service
            port:
              number: 80
```

## Cloud Provider Deployments

### AWS EKS

```bash
# Create EKS cluster
eksctl create cluster \
  --name quillspace-prod \
  --region us-west-2 \
  --nodegroup-name standard-workers \
  --node-type m5.large \
  --nodes 3 \
  --nodes-min 1 \
  --nodes-max 10 \
  --managed

# Install AWS Load Balancer Controller
kubectl apply -k "github.com/aws/eks-charts/stable/aws-load-balancer-controller//crds?ref=master"

# Deploy application
kubectl apply -f k8s/
```

### Google GKE

```bash
# Create GKE cluster
gcloud container clusters create quillspace-prod \
  --zone us-central1-a \
  --num-nodes 3 \
  --enable-autoscaling \
  --min-nodes 1 \
  --max-nodes 10 \
  --machine-type n1-standard-2

# Get credentials
gcloud container clusters get-credentials quillspace-prod --zone us-central1-a

# Deploy application
kubectl apply -f k8s/
```

### Azure AKS

```bash
# Create resource group
az group create --name quillspace-rg --location eastus

# Create AKS cluster
az aks create \
  --resource-group quillspace-rg \
  --name quillspace-prod \
  --node-count 3 \
  --enable-addons monitoring \
  --generate-ssh-keys

# Get credentials
az aks get-credentials --resource-group quillspace-rg --name quillspace-prod

# Deploy application
kubectl apply -f k8s/
```

## Database Setup

### PostgreSQL Configuration

```sql
-- init.sql
-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Create application user
CREATE USER quillspace_app WITH PASSWORD 'secure_password';

-- Create database
CREATE DATABASE quillspace OWNER quillspace_app;

-- Connect to application database
\c quillspace;

-- Enable Row Level Security
ALTER DATABASE quillspace SET row_security = on;

-- Create tenant context function
CREATE OR REPLACE FUNCTION current_tenant_id()
RETURNS UUID AS $$
BEGIN
  RETURN current_setting('app.current_tenant_id')::UUID;
EXCEPTION
  WHEN OTHERS THEN
    RETURN NULL;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;
```

### ClickHouse Configuration

```xml
<!-- clickhouse-config.xml -->
<yandex>
    <logger>
        <level>information</level>
        <console>1</console>
    </logger>

    <http_port>8123</http_port>
    <tcp_port>9000</tcp_port>

    <users>
        <quillspace>
            <password>secure_password</password>
            <networks>
                <ip>::/0</ip>
            </networks>
            <profile>default</profile>
            <quota>default</quota>
        </quillspace>
    </users>

    <profiles>
        <default>
            <max_memory_usage>10000000000</max_memory_usage>
            <use_uncompressed_cache>0</use_uncompressed_cache>
            <load_balancing>random</load_balancing>
        </default>
    </profiles>

    <quotas>
        <default>
            <interval>
                <duration>3600</duration>
                <queries>0</queries>
                <errors>0</errors>
                <result_rows>0</result_rows>
                <read_rows>0</read_rows>
                <execution_time>0</execution_time>
            </interval>
        </default>
    </quotas>
</yandex>
```

## Monitoring and Observability

### Prometheus Configuration

```yaml
# prometheus.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: prometheus-config
  namespace: quillspace
data:
  prometheus.yml: |
    global:
      scrape_interval: 15s
    scrape_configs:
    - job_name: 'quillspace-api'
      static_configs:
      - targets: ['api-service:80']
      metrics_path: /metrics
    - job_name: 'postgres'
      static_configs:
      - targets: ['postgres-exporter:9187']
    - job_name: 'clickhouse'
      static_configs:
      - targets: ['clickhouse-exporter:9116']
```

### Grafana Dashboards

Key metrics to monitor:

1. **API Performance**
   - Request rate (RPS)
   - Response time (p95, p99)
   - Error rate
   - Active connections

2. **Database Health**
   - Connection pool usage
   - Query performance
   - Lock contention
   - Replication lag

3. **System Resources**
   - CPU utilization
   - Memory usage
   - Disk I/O
   - Network throughput

4. **Business Metrics**
   - Active tenants
   - User sessions
   - Content operations
   - Revenue metrics

## Security Configuration

### SSL/TLS Setup

```bash
# Generate SSL certificates with Let's Encrypt
certbot certonly --webroot \
  -w /var/www/html \
  -d api.yourdomain.com \
  -d app.yourdomain.com \
  --email admin@yourdomain.com \
  --agree-tos
```

### Nginx Security Configuration

```nginx
# nginx.conf
events {
    worker_connections 1024;
}

http {
    # Security headers
    add_header X-Frame-Options DENY;
    add_header X-Content-Type-Options nosniff;
    add_header X-XSS-Protection "1; mode=block";
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains";

    # Rate limiting
    limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;
    limit_req_zone $binary_remote_addr zone=login:10m rate=1r/s;

    upstream api_backend {
        server api:3000;
        keepalive 32;
    }

    server {
        listen 443 ssl http2;
        server_name api.yourdomain.com;

        ssl_certificate /etc/nginx/ssl/fullchain.pem;
        ssl_certificate_key /etc/nginx/ssl/privkey.pem;

        location / {
            limit_req zone=api burst=20 nodelay;
            proxy_pass http://api_backend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
        }

        location /auth/login {
            limit_req zone=login burst=5 nodelay;
            proxy_pass http://api_backend;
        }
    }
}
```

## Backup and Disaster Recovery

### Database Backups

```bash
#!/bin/bash
# backup.sh

DATE=$(date +%Y%m%d_%H%M%S)

# PostgreSQL backup
pg_dump -h postgres -U quillspace -d quillspace | \
  gzip > /backups/postgres_${DATE}.sql.gz

# ClickHouse backup
clickhouse-client --host clickhouse --query "BACKUP DATABASE analytics TO S3('s3://backups/clickhouse/${DATE}/')"

# Upload to cloud storage
aws s3 sync /backups/ s3://quillspace-backups/

# Cleanup old backups (keep 30 days)
find /backups -name "*.gz" -mtime +30 -delete
```

### Disaster Recovery Plan

1. **RTO (Recovery Time Objective)**: 4 hours
2. **RPO (Recovery Point Objective)**: 1 hour
3. **Backup Frequency**: Every 6 hours
4. **Cross-region Replication**: Enabled for critical data

## Performance Tuning

### Database Optimization

```sql
-- PostgreSQL tuning
ALTER SYSTEM SET shared_buffers = '256MB';
ALTER SYSTEM SET effective_cache_size = '1GB';
ALTER SYSTEM SET maintenance_work_mem = '64MB';
ALTER SYSTEM SET checkpoint_completion_target = 0.9;
ALTER SYSTEM SET wal_buffers = '16MB';
ALTER SYSTEM SET default_statistics_target = 100;
SELECT pg_reload_conf();
```

### ClickHouse Optimization

```xml
<!-- Performance settings -->
<max_memory_usage>8000000000</max_memory_usage>
<max_bytes_before_external_group_by>4000000000</max_bytes_before_external_group_by>
<max_bytes_before_external_sort>4000000000</max_bytes_before_external_sort>
```

### Application Tuning

```rust
// Connection pool optimization
let pool = PgPoolOptions::new()
    .max_connections(20)
    .min_connections(5)
    .acquire_timeout(Duration::from_secs(30))
    .idle_timeout(Duration::from_secs(600))
    .max_lifetime(Duration::from_secs(1800))
    .connect(&database_url)
    .await?;
```

## Troubleshooting

### Common Issues

1. **High Memory Usage**
   - Check ClickHouse query complexity
   - Monitor connection pool sizes
   - Review caching strategies

2. **Slow API Responses**
   - Analyze database query performance
   - Check connection pool exhaustion
   - Review middleware overhead

3. **Database Connection Issues**
   - Verify network connectivity
   - Check authentication credentials
   - Monitor connection limits

### Debug Commands

```bash
# Check API health
curl -f http://localhost:3000/health

# View API logs
docker-compose logs -f api

# Check database connectivity
docker-compose exec postgres psql -U quillspace -d quillspace -c "SELECT version();"

# Monitor resource usage
docker stats

# Check Kubernetes pod status
kubectl get pods -n quillspace
kubectl describe pod <pod-name> -n quillspace
kubectl logs <pod-name> -n quillspace
```

This deployment guide provides comprehensive coverage for deploying QuillSpace across different environments and scales. Choose the appropriate deployment strategy based on your requirements and infrastructure constraints.
