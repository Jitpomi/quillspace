use crate::config::DatabaseConfig;
use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;
use tracing::info;

/// Create PostgreSQL connection pool with optimized settings
pub async fn create_pool(config: &DatabaseConfig) -> Result<PgPool> {
    info!("Connecting to PostgreSQL database...");

    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(Duration::from_secs(config.connect_timeout))
        .idle_timeout(Duration::from_secs(600)) // 10 minutes
        .max_lifetime(Duration::from_secs(1800)) // 30 minutes
        .connect(&config.url)
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    info!("PostgreSQL connection pool created successfully");
    Ok(pool)
}

/// Setup row-level security for multi-tenant isolation
pub async fn setup_rls(pool: &PgPool) -> Result<()> {
    info!("Setting up row-level security policies...");

    // Enable RLS on tenant-scoped tables
    let rls_queries = vec![
        // Enable RLS on users table
        "ALTER TABLE users ENABLE ROW LEVEL SECURITY;",
        
        // Enable RLS on content table
        "ALTER TABLE content ENABLE ROW LEVEL SECURITY;",
        
        // Create policy for users - users can only see their own tenant's data
        r#"
        CREATE POLICY tenant_isolation_users ON users
        FOR ALL TO authenticated
        USING (tenant_id = current_setting('app.current_tenant_id')::uuid);
        "#,
        
        // Create policy for content - content is scoped to tenant
        r#"
        CREATE POLICY tenant_isolation_content ON content
        FOR ALL TO authenticated
        USING (tenant_id = current_setting('app.current_tenant_id')::uuid);
        "#,
        
        // Create policy for tenants - users can only see their own tenant
        r#"
        CREATE POLICY tenant_isolation_tenants ON tenants
        FOR ALL TO authenticated
        USING (id = current_setting('app.current_tenant_id')::uuid);
        "#,
    ];

    for query in rls_queries {
        sqlx::query(query).execute(pool).await.ok(); // Ignore errors for existing policies
    }

    info!("Row-level security policies configured");
    Ok(())
}

/// Set tenant context for the current session
pub async fn set_tenant_context(pool: &PgPool, tenant_id: uuid::Uuid) -> Result<()> {
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(pool)
        .await?;
    
    Ok(())
}

/// Tenant-aware query builder
pub struct TenantQuery {
    tenant_id: uuid::Uuid,
}

impl TenantQuery {
    pub fn new(tenant_id: uuid::Uuid) -> Self {
        Self { tenant_id }
    }

    /// Execute a query with tenant context
    pub async fn execute<'q>(
        &self,
        pool: &PgPool,
        query: sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments>,
    ) -> Result<sqlx::postgres::PgQueryResult> {
        // Set tenant context
        set_tenant_context(pool, self.tenant_id).await?;
        
        // Execute the query
        let result = query.execute(pool).await?;
        Ok(result)
    }

    /// Fetch all with tenant context
    pub async fn fetch_all<'q, T>(
        &self,
        pool: &PgPool,
        query: sqlx::query::QueryAs<'q, sqlx::Postgres, T, sqlx::postgres::PgArguments>,
    ) -> Result<Vec<T>>
    where
        T: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
    {
        // Set tenant context
        set_tenant_context(pool, self.tenant_id).await?;
        
        // Execute the query
        let result = query.fetch_all(pool).await?;
        Ok(result)
    }

    /// Fetch one with tenant context
    pub async fn fetch_one<'q, T>(
        &self,
        pool: &PgPool,
        query: sqlx::query::QueryAs<'q, sqlx::Postgres, T, sqlx::postgres::PgArguments>,
    ) -> Result<T>
    where
        T: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
    {
        // Set tenant context
        set_tenant_context(pool, self.tenant_id).await?;
        
        // Execute the query
        let result = query.fetch_one(pool).await?;
        Ok(result)
    }

    /// Fetch optional with tenant context
    pub async fn fetch_optional<'q, T>(
        &self,
        pool: &PgPool,
        query: sqlx::query::QueryAs<'q, sqlx::Postgres, T, sqlx::postgres::PgArguments>,
    ) -> Result<Option<T>>
    where
        T: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
    {
        // Set tenant context
        set_tenant_context(pool, self.tenant_id).await?;
        
        // Execute the query
        let result = query.fetch_optional(pool).await?;
        Ok(result)
    }
}
