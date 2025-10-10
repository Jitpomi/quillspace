use crate::types::{TenantId, UserId};
use anyhow::{Context, Result};
use deadpool_postgres::Pool;
use uuid::Uuid;

/// RLS Context Service for managing Row-Level Security session variables
pub struct RlsService {
    db: Pool,
}

impl RlsService {
    pub fn new(db: Pool) -> Self {
        Self { db }
    }

    /// Set tenant context for RLS policies
    pub async fn set_tenant_context(&self, tenant_id: &TenantId) -> Result<()> {
        let client = self.db.get().await
            .context("Failed to get database connection")?;

        client
            .execute("SELECT set_tenant_context($1)", &[tenant_id.as_uuid()])
            .await
            .context("Failed to set tenant context")?;

        Ok(())
    }

    /// Set user context for RLS policies
    pub async fn set_user_context(&self, user_id: &UserId) -> Result<()> {
        let client = self.db.get().await
            .context("Failed to get database connection")?;

        client
            .execute("SELECT set_user_context($1)", &[user_id.as_uuid()])
            .await
            .context("Failed to set user context")?;

        Ok(())
    }

    /// Set both tenant and user context in a single call
    pub async fn set_full_context(&self, tenant_id: &TenantId, user_id: &UserId) -> Result<()> {
        let mut client = self.db.get().await
            .context("Failed to get database connection")?;

        // Use a transaction to ensure both contexts are set atomically
        let transaction = client.transaction().await
            .context("Failed to start transaction")?;

        transaction
            .execute("SELECT set_tenant_context($1)", &[tenant_id.as_uuid()])
            .await
            .context("Failed to set tenant context")?;

        transaction
            .execute("SELECT set_user_context($1)", &[user_id.as_uuid()])
            .await
            .context("Failed to set user context")?;

        transaction.commit().await
            .context("Failed to commit RLS context transaction")?;

        Ok(())
    }

    /// Get current tenant isolation mode
    pub async fn get_tenant_isolation_mode(&self, tenant_id: &TenantId) -> Result<String> {
        let client = self.db.get().await
            .context("Failed to get database connection")?;

        // Set context first
        client
            .execute("SELECT set_tenant_context($1)", &[tenant_id.as_uuid()])
            .await
            .context("Failed to set tenant context")?;

        let row = client
            .query_one("SELECT get_tenant_isolation_mode()", &[])
            .await
            .context("Failed to get tenant isolation mode")?;

        Ok(row.get(0))
    }

    /// Set tenant isolation mode (admin only)
    pub async fn set_tenant_isolation_mode(
        &self, 
        tenant_id: &TenantId, 
        user_id: &UserId, 
        mode: &str
    ) -> Result<String> {
        let client = self.db.get().await
            .context("Failed to get database connection")?;

        // Set full context
        self.set_full_context(tenant_id, user_id).await?;

        let row = client
            .query_one("SELECT set_tenant_user_isolation($1)", &[&mode])
            .await
            .context("Failed to set tenant isolation mode")?;

        Ok(row.get(0))
    }

    /// Verify RLS security status
    pub async fn verify_security(&self, tenant_id: &TenantId, user_id: &UserId) -> Result<Vec<SecurityTest>> {
        let client = self.db.get().await
            .context("Failed to get database connection")?;

        // Set full context
        self.set_full_context(tenant_id, user_id).await?;

        let rows = client
            .query("SELECT * FROM verify_rls_security()", &[])
            .await
            .context("Failed to verify RLS security")?;

        let mut tests = Vec::new();
        for row in rows {
            tests.push(SecurityTest {
                test_name: row.get("test_name"),
                status: row.get("status"),
                details: row.get("details"),
            });
        }

        Ok(tests)
    }

    /// Get tenant security status
    pub async fn get_security_status(&self, tenant_id: &TenantId) -> Result<TenantSecurityStatus> {
        let client = self.db.get().await
            .context("Failed to get database connection")?;

        // Set context
        self.set_tenant_context(tenant_id).await?;

        let row = client
            .query_one("SELECT * FROM tenant_security_status", &[])
            .await
            .context("Failed to get tenant security status")?;

        Ok(TenantSecurityStatus {
            tenant_id: row.get("tenant_id"),
            tenant_name: row.get("tenant_name"),
            user_isolation_mode: row.get("user_isolation_mode"),
            total_users: row.get("total_users"),
            admin_users: row.get("admin_users"),
            active_users: row.get("active_users"),
            last_security_change: row.get("last_security_change"),
        })
    }
}

#[derive(Debug, serde::Serialize)]
pub struct SecurityTest {
    pub test_name: String,
    pub status: String,
    pub details: String,
}

#[derive(Debug, serde::Serialize)]
pub struct TenantSecurityStatus {
    pub tenant_id: Uuid,
    pub tenant_name: String,
    pub user_isolation_mode: String,
    pub total_users: i64,
    pub admin_users: i64,
    pub active_users: i64,
    pub last_security_change: chrono::DateTime<chrono::Utc>,
}
