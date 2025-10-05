# QuillSpace Multi-Tenancy Guide

## Overview

QuillSpace is designed as a multi-tenant SaaS platform from the ground up, providing secure data isolation, scalable resource sharing, and flexible customization for thousands of tenants. This guide covers the multi-tenancy architecture, implementation patterns, and best practices.

## Multi-Tenancy Architecture

### Tenant Isolation Strategy

QuillSpace uses a **shared database, shared schema** approach with row-level security (RLS) for optimal resource utilization and security:

```mermaid
graph TB
    subgraph "Application Layer"
        A1[Tenant A Request]
        A2[Tenant B Request]
        A3[Tenant C Request]
    end
    
    subgraph "Middleware Layer"
        TM[Tenant Middleware<br/>Extract & Validate Tenant Context]
    end
    
    subgraph "Database Layer"
        RLS[Row Level Security<br/>WHERE tenant_id = current_tenant()]
        
        subgraph "Shared Tables"
            T1[users<br/>tenant_id | data]
            T2[content<br/>tenant_id | data]
            T3[analytics<br/>tenant_id | data]
        end
    end
    
    A1 --> TM
    A2 --> TM
    A3 --> TM
    TM --> RLS
    RLS --> T1
    RLS --> T2
    RLS --> T3
```

### Benefits of This Approach

1. **Resource Efficiency**: Shared infrastructure reduces per-tenant costs
2. **Scalability**: Support for millions of tenants in a single deployment
3. **Security**: Database-level isolation prevents data leakage
4. **Maintenance**: Single codebase and schema for all tenants
5. **Performance**: Optimized queries with tenant-aware indexing

## Database Design

### Tenant Context Management

Every table includes a `tenant_id` column as part of the primary key or with a unique constraint:

```sql
-- Core tenant table
CREATE TABLE tenants (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR NOT NULL,
    domain VARCHAR UNIQUE,
    settings JSONB DEFAULT '{}',
    plan_type VARCHAR NOT NULL DEFAULT 'starter',
    status VARCHAR NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Example multi-tenant table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    email VARCHAR NOT NULL,
    password_hash VARCHAR NOT NULL,
    role VARCHAR NOT NULL DEFAULT 'user',
    active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- Ensure email uniqueness per tenant
    UNIQUE(tenant_id, email)
);

-- Enable Row Level Security
ALTER TABLE users ENABLE ROW LEVEL SECURITY;

-- Create tenant isolation policy
CREATE POLICY tenant_isolation ON users
    USING (tenant_id = current_setting('app.current_tenant_id')::UUID);

-- Optimized indexes for multi-tenant queries
CREATE INDEX idx_users_tenant_id ON users(tenant_id);
CREATE INDEX idx_users_tenant_email ON users(tenant_id, email);
CREATE INDEX idx_users_tenant_created ON users(tenant_id, created_at DESC);
```

### ClickHouse Multi-Tenancy

ClickHouse uses row policies for tenant isolation:

```sql
-- Analytics events table
CREATE TABLE analytics_events (
    tenant_id UUID,
    event_type String,
    user_id UUID,
    session_id String,
    properties Map(String, String),
    timestamp DateTime64(3),
    date Date MATERIALIZED toDate(timestamp)
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (tenant_id, event_type, timestamp)
SETTINGS index_granularity = 8192;

-- Row policy for tenant isolation
CREATE ROW POLICY tenant_policy ON analytics_events
    FOR SELECT USING tenant_id = toUUID(getSetting('tenant_id'));

-- Materialized view for tenant-specific aggregations
CREATE MATERIALIZED VIEW tenant_daily_stats
ENGINE = SummingMergeTree()
ORDER BY (tenant_id, date, event_type)
AS SELECT
    tenant_id,
    date,
    event_type,
    count() as event_count,
    uniq(user_id) as unique_users
FROM analytics_events
GROUP BY tenant_id, date, event_type;
```

## Application Layer Implementation

### Tenant Context Middleware

```rust
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct TenantContext {
    pub tenant_id: Uuid,
    pub tenant_name: String,
    pub plan_type: String,
    pub features: TenantFeatures,
}

#[derive(Clone, Debug)]
pub struct TenantFeatures {
    pub analytics_enabled: bool,
    pub custom_domains: bool,
    pub api_access: bool,
    pub storage_limit_gb: u64,
}

pub async fn tenant_middleware(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract tenant identifier from various sources
    let tenant_id = extract_tenant_id(&headers, &request)?;
    
    // Load tenant context from database
    let tenant_context = load_tenant_context(&app_state.db, tenant_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Validate tenant status
    if tenant_context.status != "active" {
        return Err(StatusCode::FORBIDDEN);
    }
    
    // Set database session variable for RLS
    set_tenant_context(&app_state.db, tenant_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Add tenant context to request extensions
    request.extensions_mut().insert(tenant_context);
    
    Ok(next.run(request).await)
}

fn extract_tenant_id(headers: &HeaderMap, request: &Request) -> Result<Uuid, StatusCode> {
    // Priority order for tenant identification:
    // 1. X-Tenant-ID header (for API clients)
    // 2. Subdomain from Host header
    // 3. Custom domain lookup
    // 4. JWT token claims
    
    if let Some(tenant_header) = headers.get("x-tenant-id") {
        let tenant_str = tenant_header.to_str()
            .map_err(|_| StatusCode::BAD_REQUEST)?;
        return Uuid::parse_str(tenant_str)
            .map_err(|_| StatusCode::BAD_REQUEST);
    }
    
    if let Some(host_header) = headers.get("host") {
        let host = host_header.to_str()
            .map_err(|_| StatusCode::BAD_REQUEST)?;
        
        // Extract subdomain: tenant.quillspace.com
        if let Some(subdomain) = extract_subdomain(host) {
            return lookup_tenant_by_subdomain(subdomain).await;
        }
        
        // Check for custom domain
        return lookup_tenant_by_domain(host).await;
    }
    
    Err(StatusCode::BAD_REQUEST)
}

async fn set_tenant_context(db: &PgPool, tenant_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(db)
        .await?;
    Ok(())
}
```

### Tenant-Scoped Services

```rust
use uuid::Uuid;
use sqlx::PgPool;

pub struct ContentService {
    db: PgPool,
}

impl ContentService {
    pub async fn list_content(
        &self,
        tenant_id: Uuid,
        filters: ContentFilters,
        pagination: Pagination,
    ) -> Result<PaginatedResult<Content>, ServiceError> {
        // All queries automatically respect RLS policies
        let content = sqlx::query_as!(
            Content,
            r#"
            SELECT id, title, content, status, author_id, created_at, updated_at
            FROM content 
            WHERE ($1::text IS NULL OR status = $1)
            AND ($2::uuid IS NULL OR author_id = $2)
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
            filters.status,
            filters.author_id,
            pagination.limit,
            pagination.offset
        )
        .fetch_all(&self.db)
        .await?;
        
        Ok(PaginatedResult {
            data: content,
            pagination: pagination.with_total_count(total_count),
        })
    }
    
    pub async fn create_content(
        &self,
        tenant_id: Uuid,
        request: CreateContentRequest,
    ) -> Result<Content, ServiceError> {
        let content = sqlx::query_as!(
            Content,
            r#"
            INSERT INTO content (tenant_id, title, content, status, author_id)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, title, content, status, author_id, created_at, updated_at
            "#,
            tenant_id,
            request.title,
            request.content,
            request.status,
            request.author_id
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(content)
    }
}
```

### Route Handlers with Tenant Context

```rust
use axum::{extract::Extension, response::Json};

pub async fn list_content(
    Extension(tenant_context): Extension<TenantContext>,
    Query(params): Query<ContentListParams>,
) -> Result<Json<PaginatedResult<Content>>, StatusCode> {
    let content_service = ContentService::new(db);
    
    let result = content_service
        .list_content(
            tenant_context.tenant_id,
            params.into_filters(),
            params.into_pagination(),
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(result))
}

pub async fn create_content(
    Extension(tenant_context): Extension<TenantContext>,
    Json(request): Json<CreateContentRequest>,
) -> Result<Json<Content>, StatusCode> {
    // Validate tenant features
    if !tenant_context.features.api_access {
        return Err(StatusCode::FORBIDDEN);
    }
    
    let content_service = ContentService::new(db);
    
    let content = content_service
        .create_content(tenant_context.tenant_id, request)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(content))
}
```

## Tenant Onboarding

### Tenant Registration Flow

```rust
#[derive(Deserialize)]
pub struct TenantRegistrationRequest {
    pub name: String,
    pub domain: String,
    pub admin_email: String,
    pub admin_password: String,
    pub plan_type: String,
}

pub async fn register_tenant(
    Json(request): Json<TenantRegistrationRequest>,
    Extension(db): Extension<PgPool>,
) -> Result<Json<TenantRegistrationResponse>, StatusCode> {
    // Start database transaction
    let mut tx = db.begin().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // 1. Create tenant
    let tenant = sqlx::query_as!(
        Tenant,
        r#"
        INSERT INTO tenants (name, domain, plan_type, settings)
        VALUES ($1, $2, $3, $4)
        RETURNING id, name, domain, plan_type, created_at
        "#,
        request.name,
        request.domain,
        request.plan_type,
        json!({
            "branding": {
                "primary_color": "#007bff"
            },
            "features": get_plan_features(&request.plan_type)
        })
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|_| StatusCode::CONFLICT)?; // Domain already exists
    
    // 2. Create admin user
    let password_hash = hash_password(&request.admin_password)?;
    let admin_user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (tenant_id, email, password_hash, role)
        VALUES ($1, $2, $3, 'admin')
        RETURNING id, email, role, created_at
        "#,
        tenant.id,
        request.admin_email,
        password_hash
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // 3. Initialize default content
    initialize_tenant_content(&mut tx, tenant.id).await?;
    
    // 4. Set up analytics
    initialize_tenant_analytics(&tenant.id).await?;
    
    // Commit transaction
    tx.commit().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // 5. Send welcome email
    send_welcome_email(&tenant, &admin_user).await?;
    
    Ok(Json(TenantRegistrationResponse {
        tenant_id: tenant.id,
        domain: format!("{}.quillspace.com", request.domain),
        admin_user_id: admin_user.id,
    }))
}

async fn initialize_tenant_content(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: Uuid,
) -> Result<(), sqlx::Error> {
    // Create default categories
    sqlx::query!(
        "INSERT INTO categories (tenant_id, name, slug) VALUES ($1, 'General', 'general')",
        tenant_id
    )
    .execute(&mut **tx)
    .await?;
    
    // Create welcome content
    sqlx::query!(
        r#"
        INSERT INTO content (tenant_id, title, content, status, type)
        VALUES ($1, 'Welcome to QuillSpace', $2, 'published', 'page')
        "#,
        tenant_id,
        "# Welcome to QuillSpace\n\nYour publishing platform is ready!"
    )
    .execute(&mut **tx)
    .await?;
    
    Ok(())
}
```

## Tenant Customization

### Settings Management

```rust
#[derive(Serialize, Deserialize)]
pub struct TenantSettings {
    pub branding: BrandingSettings,
    pub features: FeatureSettings,
    pub integrations: IntegrationSettings,
    pub limits: LimitSettings,
}

#[derive(Serialize, Deserialize)]
pub struct BrandingSettings {
    pub logo_url: Option<String>,
    pub primary_color: String,
    pub secondary_color: String,
    pub custom_css: Option<String>,
    pub favicon_url: Option<String>,
}

pub async fn update_tenant_settings(
    Extension(tenant_context): Extension<TenantContext>,
    Json(settings): Json<TenantSettings>,
) -> Result<Json<Tenant>, StatusCode> {
    // Validate settings against plan limits
    validate_settings_for_plan(&settings, &tenant_context.plan_type)?;
    
    let updated_tenant = sqlx::query_as!(
        Tenant,
        "UPDATE tenants SET settings = $1, updated_at = NOW() WHERE id = $2 RETURNING *",
        serde_json::to_value(settings).unwrap(),
        tenant_context.tenant_id
    )
    .fetch_one(&db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Invalidate tenant cache
    invalidate_tenant_cache(tenant_context.tenant_id).await;
    
    Ok(Json(updated_tenant))
}
```

### Custom Domains

```rust
pub async fn add_custom_domain(
    Extension(tenant_context): Extension<TenantContext>,
    Json(request): Json<AddDomainRequest>,
) -> Result<Json<Domain>, StatusCode> {
    // Check if custom domains are enabled for this plan
    if !tenant_context.features.custom_domains {
        return Err(StatusCode::FORBIDDEN);
    }
    
    // Verify domain ownership
    verify_domain_ownership(&request.domain).await?;
    
    // Add domain to tenant
    let domain = sqlx::query_as!(
        Domain,
        r#"
        INSERT INTO tenant_domains (tenant_id, domain, verified, ssl_enabled)
        VALUES ($1, $2, true, false)
        RETURNING id, domain, verified, ssl_enabled, created_at
        "#,
        tenant_context.tenant_id,
        request.domain
    )
    .fetch_one(&db)
    .await
    .map_err(|_| StatusCode::CONFLICT)?;
    
    // Provision SSL certificate
    provision_ssl_certificate(&request.domain).await?;
    
    Ok(Json(domain))
}
```

## Performance Optimization

### Tenant-Aware Indexing

```sql
-- Composite indexes with tenant_id first for optimal performance
CREATE INDEX idx_content_tenant_status_created ON content(tenant_id, status, created_at DESC);
CREATE INDEX idx_users_tenant_role_active ON users(tenant_id, role, active) WHERE active = true;
CREATE INDEX idx_analytics_tenant_date_event ON analytics_events(tenant_id, date, event_type);

-- Partial indexes for common queries
CREATE INDEX idx_published_content ON content(tenant_id, created_at DESC) 
    WHERE status = 'published';

-- Covering indexes to avoid table lookups
CREATE INDEX idx_content_list_covering ON content(tenant_id, status, created_at DESC) 
    INCLUDE (id, title, excerpt, author_id);
```

### Connection Pool Optimization

```rust
pub struct TenantAwareConnectionPool {
    pools: HashMap<String, PgPool>, // Pool per plan type
    default_pool: PgPool,
}

impl TenantAwareConnectionPool {
    pub async fn get_connection(&self, tenant_context: &TenantContext) -> Result<PgPool, Error> {
        // Use dedicated pools for enterprise tenants
        if tenant_context.plan_type == "enterprise" {
            if let Some(pool) = self.pools.get(&tenant_context.plan_type) {
                return Ok(pool.clone());
            }
        }
        
        Ok(self.default_pool.clone())
    }
}
```

### Caching Strategy

```rust
use redis::AsyncCommands;

pub struct TenantCache {
    redis: redis::Client,
}

impl TenantCache {
    pub async fn get_tenant_settings(&self, tenant_id: Uuid) -> Result<Option<TenantSettings>, Error> {
        let mut conn = self.redis.get_async_connection().await?;
        let key = format!("tenant:{}:settings", tenant_id);
        
        let cached: Option<String> = conn.get(&key).await?;
        if let Some(data) = cached {
            return Ok(Some(serde_json::from_str(&data)?));
        }
        
        Ok(None)
    }
    
    pub async fn set_tenant_settings(
        &self, 
        tenant_id: Uuid, 
        settings: &TenantSettings,
        ttl_seconds: u64,
    ) -> Result<(), Error> {
        let mut conn = self.redis.get_async_connection().await?;
        let key = format!("tenant:{}:settings", tenant_id);
        let data = serde_json::to_string(settings)?;
        
        conn.setex(&key, ttl_seconds, data).await?;
        Ok(())
    }
}
```

## Security Considerations

### Data Isolation Validation

```rust
#[cfg(test)]
mod tenant_isolation_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_tenant_data_isolation() {
        let db = setup_test_db().await;
        
        // Create two tenants
        let tenant_a = create_test_tenant(&db, "tenant-a").await;
        let tenant_b = create_test_tenant(&db, "tenant-b").await;
        
        // Create content for tenant A
        set_tenant_context(&db, tenant_a.id).await.unwrap();
        let content_a = create_test_content(&db, "Tenant A Content").await;
        
        // Switch to tenant B context
        set_tenant_context(&db, tenant_b.id).await.unwrap();
        
        // Verify tenant B cannot access tenant A's content
        let result = sqlx::query_as!(
            Content,
            "SELECT * FROM content WHERE id = $1",
            content_a.id
        )
        .fetch_optional(&db)
        .await
        .unwrap();
        
        assert!(result.is_none(), "Tenant B should not see Tenant A's content");
    }
    
    #[tokio::test]
    async fn test_rls_policy_enforcement() {
        let db = setup_test_db().await;
        
        // Attempt to query without setting tenant context
        let result = sqlx::query_as!(Content, "SELECT * FROM content LIMIT 1")
            .fetch_optional(&db)
            .await;
        
        // Should return empty result due to RLS policy
        assert!(result.unwrap().is_none());
    }
}
```

### Audit Logging

```rust
pub async fn log_tenant_action(
    db: &PgPool,
    tenant_id: Uuid,
    user_id: Uuid,
    action: &str,
    resource_type: &str,
    resource_id: Option<Uuid>,
    metadata: Option<serde_json::Value>,
) -> Result<(), Error> {
    sqlx::query!(
        r#"
        INSERT INTO audit_logs (tenant_id, user_id, action, resource_type, resource_id, metadata, timestamp)
        VALUES ($1, $2, $3, $4, $5, $6, NOW())
        "#,
        tenant_id,
        user_id,
        action,
        resource_type,
        resource_id,
        metadata
    )
    .execute(db)
    .await?;
    
    Ok(())
}
```

## Monitoring and Analytics

### Tenant Metrics

```rust
pub struct TenantMetrics {
    pub tenant_id: Uuid,
    pub active_users: u64,
    pub content_count: u64,
    pub storage_used_gb: f64,
    pub api_requests_today: u64,
    pub bandwidth_used_gb: f64,
}

pub async fn collect_tenant_metrics(
    db: &PgPool,
    clickhouse: &ClickHousePool,
    tenant_id: Uuid,
) -> Result<TenantMetrics, Error> {
    // Collect from PostgreSQL
    let (active_users, content_count) = sqlx::query_as!(
        (i64, i64),
        r#"
        SELECT 
            (SELECT COUNT(*) FROM users WHERE tenant_id = $1 AND active = true) as active_users,
            (SELECT COUNT(*) FROM content WHERE tenant_id = $1) as content_count
        "#,
        tenant_id
    )
    .fetch_one(db)
    .await?;
    
    // Collect from ClickHouse
    let analytics_data = clickhouse
        .query("
            SELECT 
                sum(storage_bytes) / 1024 / 1024 / 1024 as storage_gb,
                countIf(event_type = 'api_request' AND date = today()) as api_requests,
                sum(bandwidth_bytes) / 1024 / 1024 / 1024 as bandwidth_gb
            FROM tenant_usage_events 
            WHERE tenant_id = ?
            AND date >= today() - 30
        ")
        .bind(tenant_id)
        .fetch_one()
        .await?;
    
    Ok(TenantMetrics {
        tenant_id,
        active_users: active_users as u64,
        content_count: content_count as u64,
        storage_used_gb: analytics_data.storage_gb,
        api_requests_today: analytics_data.api_requests,
        bandwidth_used_gb: analytics_data.bandwidth_gb,
    })
}
```

## Migration and Scaling

### Tenant Migration

```rust
pub async fn migrate_tenant_to_dedicated_instance(
    source_db: &PgPool,
    target_db: &PgPool,
    tenant_id: Uuid,
) -> Result<(), Error> {
    let mut source_tx = source_db.begin().await?;
    let mut target_tx = target_db.begin().await?;
    
    // 1. Export tenant data
    let tenant_data = export_tenant_data(&mut source_tx, tenant_id).await?;
    
    // 2. Import to target database
    import_tenant_data(&mut target_tx, &tenant_data).await?;
    
    // 3. Verify data integrity
    verify_migration(&mut target_tx, tenant_id, &tenant_data).await?;
    
    // 4. Update tenant routing
    update_tenant_routing(tenant_id, &target_db_config).await?;
    
    // 5. Clean up source data (after verification period)
    schedule_source_cleanup(tenant_id).await?;
    
    target_tx.commit().await?;
    source_tx.commit().await?;
    
    Ok(())
}
```

This multi-tenancy guide provides comprehensive coverage of QuillSpace's tenant isolation strategy, implementation patterns, and operational considerations. The shared database approach with RLS provides the optimal balance of security, performance, and cost-effectiveness for a SaaS publishing platform.
