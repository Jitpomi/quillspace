# QuillSpace Platform Guide: Development & Deployment

## Overview

This comprehensive guide consolidates all platform operations documentation for QuillSpace, covering local development setup, production deployment strategies, and operational best practices in a single authoritative document.

**Consolidated from**: development.md and deployment.md

## Development Environment

### Prerequisites

#### System Requirements

| Component | Minimum | Recommended | Notes |
|-----------|---------|-------------|-------|
| **OS** | macOS 10.15+, Ubuntu 20.04+, Windows 10+ | Latest stable | WSL2 recommended for Windows |
| **RAM** | 8GB | 16GB+ | ClickHouse and compilation require memory |
| **Storage** | 20GB free | 50GB+ SSD | Fast storage improves build times |
| **CPU** | 2 cores | 4+ cores | Rust compilation benefits from parallelism |

#### Required Software

##### 1. Rust Toolchain

```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Verify installation
rustc --version
cargo --version

# Install additional components
rustup component add rustfmt clippy
cargo install cargo-watch cargo-audit
```

##### 2. Node.js & Package Manager

```bash
# Install Node.js (via nvm recommended)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 18
nvm use 18

# Verify installation
node --version
npm --version

# Install pnpm (recommended for Qwik)
npm install -g pnpm
```

##### 3. Database Tools

```bash
# PostgreSQL client
# macOS
brew install postgresql

# Ubuntu
sudo apt-get install postgresql-client

# Windows (via Chocolatey)
choco install postgresql

# ClickHouse client
# macOS
brew install clickhouse

# Ubuntu
curl https://clickhouse.com/ | sh

# Verify installations
psql --version
clickhouse-client --version
```

##### 4. Container Tools

```bash
# Docker Desktop (recommended)
# Download from https://www.docker.com/products/docker-desktop

# Or Docker Engine (Linux)
curl -fsSL https://get.docker.com -o get-docker.sh
sh get-docker.sh

# Docker Compose
sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose

# Verify installation
docker --version
docker-compose --version
```

### Project Setup

#### 1. Clone Repository

```bash
# Clone the repository
git clone <repository-url>
cd quillspace

# Check project structure
tree -L 2
```

#### 2. Environment Configuration

```bash
# Copy environment template
cp .env.example .env

# Edit environment variables
nano .env
```

Example `.env` file:

```bash
# Database Configuration
DATABASE_URL=postgresql://quillspace:dev_password@localhost:5432/quillspace_dev
CLICKHOUSE_URL=http://localhost:8123/analytics_dev
REDIS_URL=redis://localhost:6379

# Application Configuration
JWT_SECRET=your_jwt_secret_key_minimum_32_characters_for_dev
API_BASE_URL=http://localhost:3000
FRONTEND_URL=http://localhost:5173

# Development Settings
RUST_LOG=debug
RUST_BACKTRACE=1
NODE_ENV=development

# Feature Flags
ENABLE_METRICS=true
ENABLE_TRACING=true
ENABLE_HOT_RELOAD=true
```

#### 3. Start Infrastructure Services

```bash
# Start databases and cache
docker compose up -d postgres clickhouse redis

# Verify services are running
docker compose ps
```

#### 4. Run Database Migrations

```bash
# Apply all migrations
docker exec -i quillspace-postgres psql -U quillspace -d quillspace_dev < quillspace-core/migrations/001_initial_schema.sql
docker exec -i quillspace-postgres psql -U quillspace -d quillspace_dev < quillspace-core/migrations/002_complete_rls_security.sql
docker exec -i quillspace-postgres psql -U quillspace -d quillspace_dev < quillspace-core/migrations/004_web_builder_schema.sql

# Seed development data
docker exec -i quillspace-postgres psql -U quillspace -d quillspace_dev < scripts/seed-dev.sql
```

#### 5. Start Development Servers

```bash
# Terminal 1: Start Rust backend
cd quillspace-core
cargo run

# Terminal 2: Start Qwik frontend (when available)
cd frontend
pnpm dev

# Terminal 3: Watch for changes
cargo watch -x run
```

### Development Workflows

#### Backend Development

```bash
# Run tests
cargo test

# Check code formatting
cargo fmt --check

# Run linter
cargo clippy

# Security audit
cargo audit

# Build for production
cargo build --release
```

#### Database Operations

```bash
# Connect to PostgreSQL
docker exec -it quillspace-postgres psql -U quillspace -d quillspace_dev

# Connect to ClickHouse
docker exec -it quillspace-clickhouse clickhouse-client

# Backup database
docker exec quillspace-postgres pg_dump -U quillspace quillspace_dev > backup.sql

# Restore database
docker exec -i quillspace-postgres psql -U quillspace -d quillspace_dev < backup.sql
```

#### Testing & Verification

```bash
# Test API endpoints
curl http://localhost:3000/health

# Test authentication
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "yasinkak@gmail.com", "password": "secret"}'

# Verify RLS implementation
docker exec -i quillspace-postgres psql -U quillspace -d quillspace_dev < scripts/verify-complete-setup.sql
```

## Production Deployment

### Deployment Options

#### 1. Development/Staging
- **Single Server**: Docker Compose for quick setup
- **Resources**: 2 CPU, 4GB RAM, 50GB storage
- **Use Case**: Development, testing, small-scale demos

#### 2. Production (Small)
- **Container Platform**: Docker Swarm or single Kubernetes node
- **Resources**: 4 CPU, 8GB RAM, 200GB storage
- **Use Case**: Small businesses, up to 100 tenants

#### 3. Production (Large)
- **Platform**: Kubernetes cluster with auto-scaling
- **Resources**: 16+ CPU, 32GB+ RAM, 1TB+ storage
- **Use Case**: Enterprise, 1000+ tenants, high availability

### Infrastructure Requirements

| Component | Minimum | Recommended | Notes |
|-----------|---------|-------------|-------|
| **CPU** | 2 cores | 4+ cores | Rust benefits from multiple cores |
| **Memory** | 4GB | 8GB+ | ClickHouse requires significant RAM |
| **Storage** | 50GB | 200GB+ SSD | Fast storage for database performance |
| **Network** | 100Mbps | 1Gbps+ | High throughput for API requests |

### Docker Compose Deployment

#### Production Docker Compose

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
      - ./quillspace-core/migrations:/docker-entrypoint-initdb.d
      - ./scripts:/docker-entrypoint-initdb.d/scripts
    ports:
      - "5432:5432"
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U quillspace"]
      interval: 10s
      timeout: 5s
      retries: 5
    restart: unless-stopped

  # ClickHouse - Analytics
  clickhouse:
    image: clickhouse/clickhouse-server:23-alpine
    environment:
      CLICKHOUSE_DB: analytics
      CLICKHOUSE_USER: quillspace
      CLICKHOUSE_PASSWORD: ${CLICKHOUSE_PASSWORD}
    volumes:
      - clickhouse_data:/var/lib/clickhouse
    ports:
      - "8123:8123"
      - "9000:9000"
    healthcheck:
      test: ["CMD", "wget", "--no-verbose", "--tries=1", "--spider", "http://localhost:8123/ping"]
      interval: 10s
      timeout: 5s
      retries: 5
    restart: unless-stopped

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
    restart: unless-stopped

  # QuillSpace API Server
  backend:
    build:
      context: .
      dockerfile: docker/Dockerfile.api
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
    build:
      context: ./frontend
      dockerfile: Dockerfile
    environment:
      QWIK_API_URL: http://backend:3000
      NODE_ENV: production
    ports:
      - "80:3000"
    depends_on:
      - backend
    restart: unless-stopped

volumes:
  postgres_data:
    driver: local
  clickhouse_data:
    driver: local
  redis_data:
    driver: local

networks:
  default:
    driver: bridge
```

#### Environment Configuration

```bash
# Production .env file
POSTGRES_PASSWORD=secure_production_password_change_me
CLICKHOUSE_PASSWORD=secure_clickhouse_password_change_me
REDIS_PASSWORD=secure_redis_password_change_me
JWT_SECRET=your_super_secure_jwt_secret_key_minimum_32_characters_change_in_production

# Optional: External services
SMTP_HOST=smtp.example.com
SMTP_USER=noreply@quillspace.com
SMTP_PASSWORD=smtp_password

# Monitoring
SENTRY_DSN=https://your-sentry-dsn
PROMETHEUS_ENDPOINT=http://prometheus:9090
```

### Kubernetes Deployment

#### Namespace and ConfigMap

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: quillspace

---
apiVersion: v1
kind: ConfigMap
metadata:
  name: quillspace-config
  namespace: quillspace
data:
  RUST_LOG: "info"
  NODE_ENV: "production"
  POSTGRES_DB: "quillspace"
  POSTGRES_USER: "quillspace"
  CLICKHOUSE_DB: "analytics"
  CLICKHOUSE_USER: "quillspace"
```

#### PostgreSQL Deployment

```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: postgres
  namespace: quillspace
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
          valueFrom:
            configMapKeyRef:
              name: quillspace-config
              key: POSTGRES_DB
        - name: POSTGRES_USER
          valueFrom:
            configMapKeyRef:
              name: quillspace-config
              key: POSTGRES_USER
        - name: POSTGRES_PASSWORD
          valueFrom:
            secretKeyRef:
              name: quillspace-secrets
              key: postgres-password
        ports:
        - containerPort: 5432
        volumeMounts:
        - name: postgres-storage
          mountPath: /var/lib/postgresql/data
  volumeClaimTemplates:
  - metadata:
      name: postgres-storage
    spec:
      accessModes: ["ReadWriteOnce"]
      resources:
        requests:
          storage: 100Gi
```

### Monitoring & Observability

#### Prometheus Configuration

```yaml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'quillspace-backend'
    static_configs:
      - targets: ['backend:9091']
    metrics_path: /metrics
    scrape_interval: 5s

  - job_name: 'postgres'
    static_configs:
      - targets: ['postgres-exporter:9187']

  - job_name: 'clickhouse'
    static_configs:
      - targets: ['clickhouse:9363']
```

#### Health Checks

```bash
# Backend health
curl -f http://localhost:3000/health

# Database connectivity
curl -f http://localhost:3000/health/db

# ClickHouse connectivity
curl -f http://localhost:8123/ping

# Redis connectivity
redis-cli -h localhost -p 6379 ping
```

### Security Considerations

#### SSL/TLS Configuration

```bash
# Generate SSL certificates (Let's Encrypt)
certbot certonly --standalone -d api.quillspace.com
certbot certonly --standalone -d app.quillspace.com

# Configure Nginx SSL
server {
    listen 443 ssl http2;
    server_name api.quillspace.com;
    
    ssl_certificate /etc/letsencrypt/live/api.quillspace.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/api.quillspace.com/privkey.pem;
    
    location / {
        proxy_pass http://backend:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

#### Database Security

```sql
-- Create read-only user for monitoring
CREATE USER quillspace_readonly WITH PASSWORD 'readonly_password';
GRANT CONNECT ON DATABASE quillspace TO quillspace_readonly;
GRANT USAGE ON SCHEMA public TO quillspace_readonly;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO quillspace_readonly;

-- Backup user
CREATE USER quillspace_backup WITH PASSWORD 'backup_password';
GRANT CONNECT ON DATABASE quillspace TO quillspace_backup;
GRANT USAGE ON SCHEMA public TO quillspace_backup;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO quillspace_backup;
```

### Backup & Recovery

#### Automated Backups

```bash
#!/bin/bash
# backup.sh - Daily backup script

DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="/backups"

# PostgreSQL backup
docker exec quillspace-postgres pg_dump -U quillspace quillspace > "$BACKUP_DIR/postgres_$DATE.sql"

# ClickHouse backup
docker exec quillspace-clickhouse clickhouse-client --query "BACKUP DATABASE analytics TO Disk('backups', 'clickhouse_$DATE')"

# Compress and upload to S3
tar -czf "$BACKUP_DIR/quillspace_backup_$DATE.tar.gz" "$BACKUP_DIR/postgres_$DATE.sql"
aws s3 cp "$BACKUP_DIR/quillspace_backup_$DATE.tar.gz" s3://quillspace-backups/

# Cleanup old backups (keep 30 days)
find "$BACKUP_DIR" -name "*.sql" -mtime +30 -delete
find "$BACKUP_DIR" -name "*.tar.gz" -mtime +30 -delete
```

#### Recovery Procedures

```bash
# Restore PostgreSQL from backup
docker exec -i quillspace-postgres psql -U quillspace -d quillspace < backup.sql

# Restore ClickHouse from backup
docker exec quillspace-clickhouse clickhouse-client --query "RESTORE DATABASE analytics FROM Disk('backups', 'clickhouse_backup')"

# Verify data integrity
docker exec -i quillspace-postgres psql -U quillspace -d quillspace -c "SELECT COUNT(*) FROM users;"
```

### Performance Optimization

#### Database Tuning

```sql
-- PostgreSQL optimization
ALTER SYSTEM SET shared_buffers = '256MB';
ALTER SYSTEM SET effective_cache_size = '1GB';
ALTER SYSTEM SET maintenance_work_mem = '64MB';
ALTER SYSTEM SET checkpoint_completion_target = 0.9;
ALTER SYSTEM SET wal_buffers = '16MB';
ALTER SYSTEM SET default_statistics_target = 100;

-- Reload configuration
SELECT pg_reload_conf();
```

#### Application Tuning

```toml
# Cargo.toml - Production optimizations
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
```

### Troubleshooting

#### Common Issues

1. **Database Connection Errors**
   ```bash
   # Check database status
   docker exec quillspace-postgres pg_isready -U quillspace
   
   # Check logs
   docker logs quillspace-postgres
   ```

2. **Memory Issues**
   ```bash
   # Monitor memory usage
   docker stats
   
   # Adjust ClickHouse memory settings
   echo "<max_memory_usage>4000000000</max_memory_usage>" >> /etc/clickhouse-server/config.xml
   ```

3. **Performance Issues**
   ```bash
   # Check slow queries
   docker exec quillspace-postgres psql -U quillspace -d quillspace -c "SELECT query, mean_time, calls FROM pg_stat_statements ORDER BY mean_time DESC LIMIT 10;"
   ```

---

**Document Version**: 2.0  
**Last Updated**: 2025-10-09  
**Consolidated By**: Engineering Team  
**Next Review**: 2025-11-09
