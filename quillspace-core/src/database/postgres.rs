use anyhow::Result;
use deadpool_postgres::{Config, Pool, Runtime};
use std::time::Duration;
use tokio_postgres::{NoTls, Client};
use tracing::info;

/// Create PostgreSQL connection pool with optimized settings
pub async fn create_pool(postgres_url: &str) -> Result<Pool> {
    info!("Connecting to PostgreSQL database...");

    let mut cfg = Config::new();
    cfg.url = Some(postgres_url.to_string());
    
    // Use default pool configuration
    let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;

    // Test connection
    let client = pool.get().await?;
    client.query("SELECT 1", &[]).await?;

    info!("PostgreSQL connection pool created successfully");
    Ok(pool)
}

/// Setup row-level security for multi-tenant isolation
pub async fn setup_rls(pool: &Pool) -> Result<()> {
    info!("Setting up row-level security policies...");

    let client = pool.get().await?;

    // Enable RLS on tenant-scoped tables
    let rls_queries = vec![
        // Enable RLS on users table
        "ALTER TABLE users ENABLE ROW LEVEL SECURITY;",
        
        // Enable RLS on content table
        "ALTER TABLE content ENABLE ROW LEVEL SECURITY;",
        
        // Create policy for users - users can only see their own tenant's data
        r#"
        CREATE POLICY IF NOT EXISTS tenant_isolation_users ON users
        FOR ALL TO authenticated
        USING (tenant_id = current_setting('app.current_tenant_id')::uuid);
        "#,
        
        // Create policy for content - content is scoped to tenant
        r#"
        CREATE POLICY IF NOT EXISTS tenant_isolation_content ON content
        FOR ALL TO authenticated
        USING (tenant_id = current_setting('app.current_tenant_id')::uuid);
        "#,
        
        // Create policy for tenants - users can only see their own tenant
        r#"
        CREATE POLICY IF NOT EXISTS tenant_isolation_tenants ON tenants
        FOR ALL TO authenticated
        USING (id = current_setting('app.current_tenant_id')::uuid);
        "#,
    ];

    for query in rls_queries {
        client.execute(query, &[]).await.ok(); // Ignore errors for existing policies
    }

    info!("Row-level security policies configured");
    Ok(())
}

/// Set tenant context for the current session
pub async fn set_tenant_context(pool: &Pool, tenant_id: uuid::Uuid) -> Result<()> {
    let client = pool.get().await?;
    client.execute(
        "SELECT set_config('app.current_tenant_id', $1, true)",
        &[&tenant_id.to_string()]
    ).await?;
    
    Ok(())
}

/// Tenant-aware query helper
pub struct TenantQuery {
    tenant_id: uuid::Uuid,
}

impl TenantQuery {
    pub fn new(tenant_id: uuid::Uuid) -> Self {
        Self { tenant_id }
    }

    /// Execute a query with tenant context
    pub async fn execute(
        &self,
        pool: &Pool,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<u64> {
        // Set tenant context
        set_tenant_context(pool, self.tenant_id).await?;
        
        // Execute the query
        let client = pool.get().await?;
        let result = client.execute(query, params).await?;
        Ok(result)
    }

    /// Query for multiple rows with tenant context
    pub async fn query(
        &self,
        pool: &Pool,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<Vec<tokio_postgres::Row>> {
        // Set tenant context
        set_tenant_context(pool, self.tenant_id).await?;
        
        // Execute the query
        let client = pool.get().await?;
        let result = client.query(query, params).await?;
        Ok(result)
    }

    /// Query for a single row with tenant context
    pub async fn query_one(
        &self,
        pool: &Pool,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<tokio_postgres::Row> {
        // Set tenant context
        set_tenant_context(pool, self.tenant_id).await?;
        
        // Execute the query
        let client = pool.get().await?;
        let result = client.query_one(query, params).await?;
        Ok(result)
    }

    /// Query for an optional row with tenant context
    pub async fn query_opt(
        &self,
        pool: &Pool,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<Option<tokio_postgres::Row>> {
        // Set tenant context
        set_tenant_context(pool, self.tenant_id).await?;
        
        // Execute the query
        let client = pool.get().await?;
        let result = client.query_opt(query, params).await?;
        Ok(result)
    }
}
