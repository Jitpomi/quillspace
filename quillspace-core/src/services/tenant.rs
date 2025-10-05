use crate::types::{Tenant, TenantId};
use anyhow::Result;
use sqlx::{PgPool, FromRow};
use uuid::Uuid;

/// Tenant management service
#[derive(Clone)]
pub struct TenantService {
    db: PgPool,
}

impl TenantService {
    pub fn new(db: PgPool) -> Self {
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

        let tenant = sqlx::query_as::<_, Tenant>(
            r#"
            INSERT INTO tenants (id, name, slug, settings, created_at, updated_at, is_active)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#
        )
        .bind(tenant_id)
        .bind(&name)
        .bind(&slug)
        .bind(serde_json::json!({}))
        .bind(now)
        .bind(now)
        .bind(true)
        .fetch_one(&self.db)
        .await?;

        Ok(tenant)
    }

    /// Get tenant by ID
    pub async fn get_tenant(&self, tenant_id: &TenantId) -> Result<Option<Tenant>> {
        let tenant = sqlx::query_as::<_, Tenant>(
            "SELECT * FROM tenants WHERE id = $1"
        )
        .bind(tenant_id.as_uuid())
        .fetch_optional(&self.db)
        .await?;

        Ok(tenant)
    }

    /// Get tenant by slug
    pub async fn get_tenant_by_slug(&self, slug: &str) -> Result<Option<Tenant>> {
        let tenant = sqlx::query_as::<_, Tenant>(
            "SELECT * FROM tenants WHERE slug = $1 AND is_active = true"
        )
        .bind(slug)
        .fetch_optional(&self.db)
        .await?;

        Ok(tenant)
    }

    /// Update tenant
    pub async fn update_tenant(
        &self,
        tenant_id: &TenantId,
        name: Option<String>,
        slug: Option<String>,
    ) -> Result<Option<Tenant>> {
        let now = chrono::Utc::now();

        let tenant = sqlx::query_as::<_, Tenant>(
            r#"
            UPDATE tenants 
            SET name = COALESCE($2, name),
                slug = COALESCE($3, slug),
                updated_at = $4
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(tenant_id.as_uuid())
        .bind(name)
        .bind(slug)
        .bind(now)
        .fetch_optional(&self.db)
        .await?;

        Ok(tenant)
    }

    /// Update tenant settings
    pub async fn update_tenant_settings(
        &self,
        tenant_id: &TenantId,
        settings: serde_json::Value,
    ) -> Result<Option<Tenant>> {
        let now = chrono::Utc::now();

        let tenant = sqlx::query_as::<_, Tenant>(
            r#"
            UPDATE tenants 
            SET settings = $2, updated_at = $3
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(tenant_id.as_uuid())
        .bind(settings)
        .bind(now)
        .fetch_optional(&self.db)
        .await?;

        Ok(tenant)
    }

    /// List all tenants (admin only)
    pub async fn list_tenants(&self, limit: i64, offset: i64) -> Result<Vec<Tenant>> {
        let tenants = sqlx::query_as::<_, Tenant>(
            r#"
            SELECT * FROM tenants 
            WHERE is_active = true 
            ORDER BY created_at DESC 
            LIMIT $1 OFFSET $2
            "#
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db)
        .await?;

        Ok(tenants)
    }

    /// Deactivate tenant (soft delete)
    pub async fn deactivate_tenant(&self, tenant_id: &TenantId) -> Result<bool> {
        let now = chrono::Utc::now();

        let result = sqlx::query!(
            "UPDATE tenants SET is_active = false, updated_at = $2 WHERE id = $1",
            tenant_id.as_uuid(),
            now
        )
        .execute(&self.db)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}


