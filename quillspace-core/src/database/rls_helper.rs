use anyhow::{Context, Result};
use deadpool_postgres::Client;
use uuid::Uuid;

/// RLS (Row Level Security) helper functions for consistent tenant context management
pub struct RlsHelper;

impl RlsHelper {
    /// Set tenant context for RLS policies
    /// This must be called before any database operations that require tenant isolation
    pub async fn set_tenant_context(client: &Client, tenant_id: &Uuid) -> Result<()> {
        client
            .execute(
                "SELECT set_config('quillspace.tenant_id', $1, true)",
                &[&tenant_id.to_string()],
            )
            .await
            .context("Failed to set RLS tenant context")?;
        Ok(())
    }

    /// Set user context for RLS policies (for widgets and user-specific data)
    pub async fn set_user_context(client: &Client, user_id: &Uuid) -> Result<()> {
        client
            .execute(
                "SELECT set_config('rls.user_id', $1, true)",
                &[&user_id.to_string()],
            )
            .await
            .context("Failed to set RLS user context")?;
        Ok(())
    }

    /// Set both tenant and user context
    pub async fn set_full_context(
        client: &Client,
        tenant_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<()> {
        Self::set_tenant_context(client, tenant_id).await?;
        Self::set_user_context(client, user_id).await?;
        Ok(())
    }

    /// Clear all RLS context (useful for cleanup or global operations)
    pub async fn clear_context(client: &Client) -> Result<()> {
        client
            .execute("SELECT set_config('quillspace.tenant_id', NULL, true)", &[])
            .await
            .context("Failed to clear RLS tenant context")?;
        
        client
            .execute("SELECT set_config('rls.user_id', NULL, true)", &[])
            .await
            .context("Failed to clear RLS user context")?;
        
        Ok(())
    }

    /// Get current tenant context (for debugging)
    pub async fn get_tenant_context(client: &Client) -> Result<Option<String>> {
        let row = client
            .query_opt("SELECT current_setting('quillspace.tenant_id', true)", &[])
            .await
            .context("Failed to get current tenant context")?;

        match row {
            Some(row) => {
                let value: String = row.get(0);
                if value.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(value))
                }
            }
            None => Ok(None),
        }
    }
}
