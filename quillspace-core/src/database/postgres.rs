use anyhow::Result;
use deadpool_postgres::{Config, Pool, Runtime};
use tokio_postgres::NoTls;
use tracing::{info, warn};

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
/// Note: Most RLS policies are defined in migrations, this ensures they're enabled
pub async fn setup_rls(pool: &Pool) -> Result<()> {
    info!("Verifying row-level security configuration...");

    let client = pool.get().await?;

    // Verify RLS is enabled on core tables (these should be set by migrations)
    let rls_checks = vec![
        "SELECT schemaname, tablename, rowsecurity FROM pg_tables WHERE tablename IN ('users', 'tenants', 'templates', 'sites', 'pages', 'assets') AND schemaname = 'public'",
    ];

    for query in rls_checks {
        let rows = client.query(query, &[]).await?;
        for row in rows {
            let table_name: String = row.get("tablename");
            let rls_enabled: bool = row.get("rowsecurity");
            if rls_enabled {
                info!("✓ RLS enabled on table: {}", table_name);
            } else {
                warn!("⚠ RLS not enabled on table: {}", table_name);
            }
        }
    }

    // Ensure core tenant isolation policies exist for legacy tables
    let legacy_policies = vec![
        // Create policy for users if table exists (using correct setting name)
        r#"
        DO $$
        BEGIN
            IF EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'users' AND table_schema = 'public') THEN
                ALTER TABLE users ENABLE ROW LEVEL SECURITY;
                DROP POLICY IF EXISTS tenant_isolation_users ON users;
                CREATE POLICY tenant_isolation_users ON users
                FOR ALL
                USING (tenant_id = current_setting('quillspace.tenant_id')::uuid)
                WITH CHECK (tenant_id = current_setting('quillspace.tenant_id')::uuid);
            END IF;
        END $$;
        "#,
        
        // Create policy for tenants if table exists
        r#"
        DO $$
        BEGIN
            IF EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'tenants' AND table_schema = 'public') THEN
                ALTER TABLE tenants ENABLE ROW LEVEL SECURITY;
                DROP POLICY IF EXISTS tenant_isolation_tenants ON tenants;
                CREATE POLICY tenant_isolation_tenants ON tenants
                FOR ALL
                USING (id = current_setting('quillspace.tenant_id')::uuid);
            END IF;
        END $$;
        "#,
    ];

    for query in legacy_policies {
        if let Err(e) = client.execute(query, &[]).await {
            warn!("Failed to execute RLS policy: {}", e);
        }
    }

    info!("Row-level security policies configured");
    Ok(())
}

/// Set tenant context for the current session
pub async fn set_tenant_context(pool: &Pool, tenant_id: uuid::Uuid) -> Result<()> {
    let client = pool.get().await?;
    client.execute(
        "SELECT set_config('quillspace.tenant_id', $1, true)",
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
