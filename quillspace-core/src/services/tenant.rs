use crate::types::{Tenant, TenantId};
use anyhow::Result;
use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;
use tokio_postgres::{Row, Error as PgError};
use uuid::Uuid;

/// Helper function to convert a tokio-postgres Row to Tenant
fn row_to_tenant(row: &Row) -> Result<Tenant, PgError> {
    Ok(Tenant {
        id: row.try_get("id")?,
        name: row.try_get("name")?,
        slug: row.try_get("slug")?,
        settings: row.try_get("settings")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
        is_active: row.try_get("is_active")?,
    })
}

/// Tenant management service
#[derive(Clone)]
pub struct TenantService {
    db: Pool,
}

impl TenantService {
    pub fn new(db: Pool) -> Self {
        Self { db }
    }

    /// Create a new tenant
    pub async fn create_tenant(
        &self,
        name: String,
        slug: String,
    ) -> Result<Tenant> {
        let tenant_id = Uuid::new_v4();
        let now = chrono::Utc::now();

        // Get database connection
        let client = self.db.get().await?;

        let query = r#"
            INSERT INTO tenants (id, name, slug, settings, created_at, updated_at, is_active)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#;

        let settings = serde_json::json!({});
        let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![
            &tenant_id,
            &name,
            &slug,
            &settings,
            &now,
            &now,
            &true,
        ];

        let row = client.query_one(query, &params).await?;
        let tenant = row_to_tenant(&row)?;

        Ok(tenant)
    }

    /// Get tenant by ID
    pub async fn get_tenant(&self, tenant_id: &TenantId) -> Result<Option<Tenant>> {
        let client = self.db.get().await?;

        let query = "SELECT * FROM tenants WHERE id = $1";
        let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![tenant_id.as_uuid()];

        match client.query_opt(query, &params).await? {
            Some(row) => {
                let tenant = row_to_tenant(&row)?;
                Ok(Some(tenant))
            }
            None => Ok(None),
        }
    }

    /// Get tenant by slug
    pub async fn get_tenant_by_slug(&self, slug: &str) -> Result<Option<Tenant>> {
        let client = self.db.get().await?;

        let query = "SELECT * FROM tenants WHERE slug = $1 AND is_active = true";
        let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![&slug];

        match client.query_opt(query, &params).await? {
            Some(row) => {
                let tenant = row_to_tenant(&row)?;
                Ok(Some(tenant))
            }
            None => Ok(None),
        }
    }

    /// Update tenant
    pub async fn update_tenant(
        &self,
        tenant_id: &TenantId,
        name: Option<String>,
        slug: Option<String>,
    ) -> Result<Option<Tenant>> {
        let now = chrono::Utc::now();
        let client = self.db.get().await?;

        let query = r#"
            UPDATE tenants 
            SET name = COALESCE($2, name),
                slug = COALESCE($3, slug),
                updated_at = $4
            WHERE id = $1
            RETURNING *
            "#;

        let name_ref = name.as_deref();
        let slug_ref = slug.as_deref();
        
        let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![
            tenant_id.as_uuid(),
            &name_ref,
            &slug_ref,
            &now,
        ];

        match client.query_opt(query, &params).await? {
            Some(row) => {
                let tenant = row_to_tenant(&row)?;
                Ok(Some(tenant))
            }
            None => Ok(None),
        }
    }

    /// Update tenant settings
    pub async fn update_tenant_settings(
        &self,
        tenant_id: &TenantId,
        settings: serde_json::Value,
    ) -> Result<Option<Tenant>> {
        let now = chrono::Utc::now();
        let client = self.db.get().await?;

        let query = r#"
            UPDATE tenants 
            SET settings = $2, updated_at = $3
            WHERE id = $1
            RETURNING *
            "#;

        let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![
            tenant_id.as_uuid(),
            &settings,
            &now,
        ];

        match client.query_opt(query, &params).await? {
            Some(row) => {
                let tenant = row_to_tenant(&row)?;
                Ok(Some(tenant))
            }
            None => Ok(None),
        }
    }

    /// List all tenants (admin only)
    pub async fn list_tenants(&self, limit: i64, offset: i64) -> Result<Vec<Tenant>> {
        let client = self.db.get().await?;

        let query = r#"
            SELECT * FROM tenants 
            WHERE is_active = true 
            ORDER BY created_at DESC 
            LIMIT $1 OFFSET $2
            "#;

        let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![
            &limit,
            &offset,
        ];

        let rows = client.query(query, &params).await?;
        let tenants: Result<Vec<Tenant>, _> = rows.iter().map(row_to_tenant).collect();
        
        Ok(tenants?)
    }

    /// Deactivate tenant (soft delete)
    pub async fn deactivate_tenant(&self, tenant_id: &TenantId) -> Result<bool> {
        let now = chrono::Utc::now();
        let client = self.db.get().await?;

        let query = "UPDATE tenants SET is_active = false, updated_at = $2 WHERE id = $1";
        let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![
            tenant_id.as_uuid(),
            &now,
        ];

        let rows_affected = client.execute(query, &params).await?;
        
        Ok(rows_affected > 0)
    }
}


