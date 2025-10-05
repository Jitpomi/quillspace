# 🚀 QuillSpace - Getting Started Guide

This guide will help you get QuillSpace running locally, implementing the recommended architecture from the 2025 stack evaluation.

## 📋 Prerequisites

- **Rust** 1.75+ with Cargo
- **Node.js** 18+ with npm
- **Docker** and Docker Compose
- **Git**

## 🏃‍♂️ Quick Start (Recommended)

### 1. Clone and Setup

```bash
git clone <repository-url>
cd quillspace

# Copy environment configuration
cp .env.example .env
```

### 2. Start Infrastructure with Docker

```bash
# Start PostgreSQL, ClickHouse, Redis, and monitoring stack
docker-compose -f docker-compose.dev.yml up -d postgres clickhouse redis prometheus grafana

# Wait for services to be ready (about 30 seconds)
docker-compose -f docker-compose.dev.yml logs -f postgres clickhouse
```

### 3. Setup Database

```bash
# Run PostgreSQL migrations
export DATABASE_URL="postgresql://postgres:password@localhost:5432/quillspace_dev"
cd quillspace-core
sqlx migrate run

# Verify ClickHouse is ready
curl http://localhost:8123/ping
```

### 4. Start the API Server

```bash
# From the quillspace-core directory
cargo run

# Or with environment variables
RUN_MODE=development cargo run
```

### 5. Start the Frontend

```bash
cd frontend
npm install
npm run dev
```

### 6. Access the Application

- **Frontend**: http://localhost:5173
- **API**: http://localhost:3000
- **API Health**: http://localhost:3000/health
- **Metrics**: http://localhost:9090 (Prometheus)
- **Dashboards**: http://localhost:3001 (Grafana, admin/admin)
- **ClickHouse**: http://localhost:8123

## 🔧 Manual Setup (Development)

If you prefer to run services individually:

### PostgreSQL Setup

```bash
# Install PostgreSQL 15+
createdb quillspace_dev
export DATABASE_URL="postgresql://postgres:password@localhost:5432/quillspace_dev"

# Run migrations
cd quillspace-core
sqlx migrate run
```

### ClickHouse Setup

```bash
# Install ClickHouse
# Create database
echo "CREATE DATABASE quillspace_analytics_dev" | curl 'http://localhost:8123/' --data-binary @-
```

### Configuration

Create `config/local.toml`:

```toml
[server]
host = "127.0.0.1"
port = 3000

[database]
url = "postgresql://postgres:password@localhost:5432/quillspace_dev"

[clickhouse]
url = "http://localhost:8123"
database = "quillspace_analytics_dev"

[auth]
jwt_secret = "your-local-dev-secret"
```

## 🧪 Testing the Implementation

### 1. API Health Check

```bash
curl http://localhost:3000/health
# Expected: "OK"

curl http://localhost:3000/ready
# Expected: "Ready"
```

### 2. Test Multi-Tenant API

```bash
# Create a tenant (requires admin token in production)
curl -X POST http://localhost:3000/api/v1/tenants \
  -H "Content-Type: application/json" \
  -H "X-Tenant-ID: 550e8400-e29b-41d4-a716-446655440000" \
  -d '{"name": "Test Tenant", "slug": "test-tenant"}'
```

### 3. Test Analytics

```bash
# Record an analytics event
curl -X POST http://localhost:3000/api/v1/analytics/events \
  -H "Content-Type: application/json" \
  -H "X-Tenant-ID: 550e8400-e29b-41d4-a716-446655440000" \
  -d '{
    "event_type": "page_view",
    "event_data": {"page": "/dashboard"},
    "session_id": "session123"
  }'

# Get tenant statistics
curl http://localhost:3000/api/v1/analytics/stats?days=7 \
  -H "X-Tenant-ID: 550e8400-e29b-41d4-a716-446655440000"
```

### 4. Test Frontend Integration

Visit http://localhost:5173 and:
- Click the interactive demo buttons
- Check browser network tab for API calls
- Verify analytics events are recorded

## 📊 Monitoring and Observability

### Prometheus Metrics

Visit http://localhost:9090 and query:

```promql
# HTTP request rate
rate(http_requests_total[5m])

# Request duration
histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))

# Active requests
http_requests_active
```

### Grafana Dashboards

1. Go to http://localhost:3001 (admin/admin)
2. Import dashboard from `docker/monitoring/grafana/dashboards/`
3. View real-time metrics

### ClickHouse Analytics

```sql
-- Connect to ClickHouse
clickhouse-client

-- View events
SELECT event_type, count() FROM events GROUP BY event_type;

-- Tenant analytics
SELECT tenant_id, count() as events FROM events GROUP BY tenant_id;

-- Daily stats
SELECT toDate(timestamp) as date, count() FROM events GROUP BY date ORDER BY date;
```

## 🏗️ Architecture Verification

This implementation demonstrates the key recommendations from the stack evaluation:

### ✅ **Axum Performance**
- High-throughput async Rust server
- Type-safe request handling
- Comprehensive middleware stack

### ✅ **Multi-Tenant Isolation**
- PostgreSQL row-level security
- Tenant-scoped API endpoints
- Isolated analytics per tenant

### ✅ **ClickHouse Analytics**
- Real-time event processing
- Materialized views for aggregations
- TTL policies for data retention

### ✅ **Qwik Frontend**
- O(1) JavaScript payload sizes
- Instant interactivity with resumability
- Server-side rendering with edge deployment

## 🚀 Production Deployment

### Environment Variables

```bash
# Required production environment variables
export DATABASE_URL="postgresql://user:pass@host:5432/quillspace"
export CLICKHOUSE_URL="https://clickhouse.company.com:8443"
export CLICKHOUSE_USERNAME="production_user"
export CLICKHOUSE_PASSWORD="secure_password"
export JWT_SECRET="your-production-jwt-secret-256-bits"
export RUN_MODE="production"
```

### Docker Production

```bash
# Build and deploy
docker-compose -f docker-compose.prod.yml up -d

# Scale API servers
docker-compose -f docker-compose.prod.yml up -d --scale quillspace-api=3
```

### Performance Tuning

1. **Database Connection Pooling**: Adjust `max_connections` in config
2. **ClickHouse Optimization**: Configure partitioning and indexes
3. **Rust Compilation**: Use `--release` flag for production builds
4. **Frontend Optimization**: Enable compression and CDN

## 🔍 Troubleshooting

### Common Issues

1. **Database Connection Failed**
   ```bash
   # Check PostgreSQL is running
   pg_isready -h localhost -p 5432
   
   # Verify connection string
   psql $DATABASE_URL -c "SELECT 1"
   ```

2. **ClickHouse Connection Failed**
   ```bash
   # Test ClickHouse connectivity
   curl http://localhost:8123/ping
   
   # Check database exists
   echo "SHOW DATABASES" | curl 'http://localhost:8123/' --data-binary @-
   ```

3. **Frontend Build Issues**
   ```bash
   # Clear node_modules and reinstall
   rm -rf node_modules package-lock.json
   npm install
   ```

4. **Rust Compilation Errors**
   ```bash
   # Clean and rebuild
   cargo clean
   cargo build
   
   # Update dependencies
   cargo update
   ```

## 📚 Next Steps

1. **Explore the API**: Check out the OpenAPI docs at `/docs`
2. **Customize Tenants**: Modify tenant settings and branding
3. **Add Content**: Use the content management endpoints
4. **Monitor Performance**: Set up alerts in Grafana
5. **Scale Horizontally**: Deploy multiple API instances

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes following the architecture patterns
4. Add tests and documentation
5. Submit a pull request

For questions or issues, please check the documentation or create an issue in the repository.

---

**Built with the power of Rust 🦀, Qwik ⚡, ClickHouse 📊, and PostgreSQL 🐘**
