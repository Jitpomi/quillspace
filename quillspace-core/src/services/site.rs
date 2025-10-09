use crate::types::{TenantId, UserId};
use anyhow::{Context, Result};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio_postgres::Row;
use uuid::Uuid;

/// Site entity representing an author's website
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Site {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub template_id: Option<Uuid>,
    pub custom_domain: Option<String>,
    pub subdomain: String,
    pub is_published: bool,
    pub seo_settings: Value,
    pub build_status: String,
    pub theme_config: Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Site creation request
#[derive(Debug, Deserialize)]
pub struct CreateSiteRequest {
    pub name: String,
    pub description: Option<String>,
    pub template_id: Option<Uuid>,
    pub custom_domain: Option<String>,
    pub subdomain: Option<String>, // If not provided, will be auto-generated
    pub seo_settings: Option<Value>,
    pub theme_config: Option<Value>,
}

/// Site update request
#[derive(Debug, Deserialize)]
pub struct UpdateSiteRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub template_id: Option<Uuid>,
    pub custom_domain: Option<String>,
    pub seo_settings: Option<Value>,
    pub theme_config: Option<Value>,
    pub is_published: Option<bool>,
}

/// Site service for managing author websites
pub struct SiteService {
    db: Pool,
}

impl SiteService {
    pub fn new(db: Pool) -> Self {
        Self { db }
    }

    /// Create a new site
    pub async fn create_site(
        &self,
        tenant_id: &TenantId,
        request: CreateSiteRequest,
    ) -> Result<Site> {
        let client = self.db.get().await
            .context("Failed to get database connection")?;

        // Set RLS context
        client
            .execute("SELECT set_config('app.current_tenant_id', $1, true)", &[&tenant_id.to_string()])
            .await
            .context("Failed to set RLS tenant context")?;

        // Generate subdomain if not provided
        let subdomain = if let Some(subdomain) = request.subdomain {
            // Validate the provided subdomain
            self.validate_subdomain(&subdomain)?;
            subdomain
        } else {
            // Generate unique subdomain from site name
            let generated: String = client
                .query_one(
                    "SELECT generate_unique_subdomain($1)",
                    &[&request.name],
                )
                .await
                .context("Failed to generate unique subdomain")?
                .get(0);
            generated
        };

        // Check if subdomain is already taken
        let exists = client
            .query_opt("SELECT id FROM sites WHERE subdomain = $1", &[&subdomain])
            .await
            .context("Failed to check subdomain availability")?;

        if exists.is_some() {
            return Err(anyhow::anyhow!("Subdomain '{}' is already taken", subdomain));
        }

        // Set RLS context
        client
            .execute("SELECT set_config('app.current_tenant_id', $1, true)", &[&tenant_id.to_string()])
            .await
            .context("Failed to set RLS tenant context")?;

        let seo_settings = request.seo_settings.unwrap_or_else(|| serde_json::json!({}));
        let theme_config = request.theme_config.unwrap_or_else(|| serde_json::json!({}));

        let row = client
            .query_one(
                "INSERT INTO sites (tenant_id, name, description, template_id, custom_domain, subdomain, seo_settings, theme_config) 
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8) 
                 RETURNING *",
                &[
                    tenant_id.as_uuid(),
                    &request.name,
                    &request.description,
                    &request.template_id,
                    &request.custom_domain,
                    &subdomain,
                    &seo_settings,
                    &theme_config,
                ],
            )
            .await
            .context("Failed to create site")?;

        Ok(row_to_site(&row)?)
    }

    /// Get site by ID
    pub async fn get_site(
        &self,
        tenant_id: &TenantId,
        site_id: Uuid,
    ) -> Result<Option<Site>> {
        let client = self.db.get().await
            .context("Failed to get database connection")?;

        // Set RLS context
        client
            .execute("SELECT set_config('app.current_tenant_id', $1, true)", &[&tenant_id.to_string()])
            .await
            .context("Failed to set RLS tenant context")?;

        let row = client
            .query_opt("SELECT * FROM sites WHERE id = $1", &[&site_id])
            .await
            .context("Failed to get site")?;

        match row {
            Some(row) => Ok(Some(row_to_site(&row)?)),
            None => Ok(None),
        }
    }

    /// Get site by subdomain
    pub async fn get_site_by_subdomain(&self, subdomain: &str) -> Result<Option<Site>> {
        let client = self.db.get().await
            .context("Failed to get database connection")?;

        let row = client
            .query_opt("SELECT * FROM sites WHERE subdomain = $1", &[&subdomain])
            .await
            .context("Failed to get site by subdomain")?;

        match row {
            Some(row) => Ok(Some(row_to_site(&row)?)),
            None => Ok(None),
        }
    }

    /// Get site by custom domain
    pub async fn get_site_by_domain(&self, domain: &str) -> Result<Option<Site>> {
        let client = self.db.get().await
            .context("Failed to get database connection")?;

        let row = client
            .query_opt("SELECT * FROM sites WHERE custom_domain = $1", &[&domain])
            .await
            .context("Failed to get site by domain")?;

        match row {
            Some(row) => Ok(Some(row_to_site(&row)?)),
            None => Ok(None),
        }
    }

    /// List sites for a tenant
    pub async fn list_sites(
        &self,
        tenant_id: &TenantId,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Site>> {
        let mut client = self.db.get().await
            .context("Failed to get database connection")?;

        // Use a transaction to ensure RLS context persists
        let transaction = client.transaction().await
            .context("Failed to start transaction")?;

        // Set RLS context within transaction
        transaction
            .execute("SELECT set_config('app.current_tenant_id', $1, true)", &[&tenant_id.to_string()])
            .await
            .context("Failed to set RLS tenant context")?;

        let rows = transaction
            .query(
                "SELECT * FROM sites ORDER BY created_at DESC LIMIT $1 OFFSET $2",
                &[&limit, &offset],
            )
            .await
            .context("Failed to list sites")?;

        // Commit transaction
        transaction.commit().await
            .context("Failed to commit transaction")?;

        let mut sites = Vec::new();
        for row in rows {
            sites.push(row_to_site(&row)?);
        }

        Ok(sites)
    }

    /// List sites for a tenant without RLS (application-level filtering)
    pub async fn list_sites_without_rls(
        &self,
        tenant_id: &TenantId,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Site>> {
        let client = self.db.get().await
            .context("Failed to get database connection")?;
        let rows = client
            .query(
                "SELECT * FROM sites WHERE tenant_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
                &[tenant_id.as_uuid(), &limit, &offset],
            )
            .await
            .context("Failed to list sites")?;

        let mut sites = Vec::new();
        for row in rows {
            sites.push(row_to_site(&row)?);
        }

        Ok(sites)
    }

    /// Update site
    pub async fn update_site(
        &self,
        tenant_id: &TenantId,
        site_id: Uuid,
        request: UpdateSiteRequest,
    ) -> Result<Option<Site>> {
        let client = self.db.get().await
            .context("Failed to get database connection")?;

        // Set RLS context
        client
            .execute("SELECT set_config('app.current_tenant_id', $1, true)", &[&tenant_id.to_string()])
            .await
            .context("Failed to set RLS tenant context")?;

        // Build dynamic update query
        let mut set_clauses = Vec::new();
        let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![&site_id];
        let mut param_count = 1;

        if let Some(name) = &request.name {
            param_count += 1;
            set_clauses.push(format!("name = ${}", param_count));
            params.push(name);
        }

        if let Some(description) = &request.description {
            param_count += 1;
            set_clauses.push(format!("description = ${}", param_count));
            params.push(description);
        }

        if let Some(template_id) = &request.template_id {
            param_count += 1;
            set_clauses.push(format!("template_id = ${}", param_count));
            params.push(template_id);
        }

        if let Some(custom_domain) = &request.custom_domain {
            param_count += 1;
            set_clauses.push(format!("custom_domain = ${}", param_count));
            params.push(custom_domain);
        }

        if let Some(seo_settings) = &request.seo_settings {
            param_count += 1;
            set_clauses.push(format!("seo_settings = ${}", param_count));
            params.push(seo_settings);
        }

        if let Some(theme_config) = &request.theme_config {
            param_count += 1;
            set_clauses.push(format!("theme_config = ${}", param_count));
            params.push(theme_config);
        }

        if let Some(is_published) = &request.is_published {
            param_count += 1;
            set_clauses.push(format!("is_published = ${}", param_count));
            params.push(is_published);
        }

        if set_clauses.is_empty() {
            // No updates requested, just return the current site
            return self.get_site(tenant_id, site_id).await;
        }

        let query = format!(
            "UPDATE sites SET {}, updated_at = NOW() WHERE id = $1 RETURNING *",
            set_clauses.join(", ")
        );

        let row = client
            .query_opt(&query, &params)
            .await
            .context("Failed to update site")?;

        match row {
            Some(row) => Ok(Some(row_to_site(&row)?)),
            None => Ok(None),
        }
    }

    /// Delete site
    pub async fn delete_site(
        &self,
        tenant_id: &TenantId,
        site_id: Uuid,
    ) -> Result<bool> {
        let client = self.db.get().await
            .context("Failed to get database connection")?;

        // Set RLS context
        client
            .execute("SELECT set_config('app.current_tenant_id', $1, true)", &[&tenant_id.to_string()])
            .await
            .context("Failed to set RLS tenant context")?;

        let rows_affected = client
            .execute("DELETE FROM sites WHERE id = $1", &[&site_id])
            .await
            .context("Failed to delete site")?;

        Ok(rows_affected > 0)
    }

    /// Publish site
    pub async fn publish_site(
        &self,
        tenant_id: &TenantId,
        site_id: Uuid,
    ) -> Result<Option<Site>> {
        let client = self.db.get().await
            .context("Failed to get database connection")?;

        // Set RLS context
        client
            .execute("SELECT set_config('app.current_tenant_id', $1, true)", &[&tenant_id.to_string()])
            .await
            .context("Failed to set RLS tenant context")?;

        let row = client
            .query_opt(
                "UPDATE sites SET is_published = true, build_status = 'published', updated_at = NOW() 
                 WHERE id = $1 RETURNING *",
                &[&site_id],
            )
            .await
            .context("Failed to publish site")?;

        match row {
            Some(row) => Ok(Some(row_to_site(&row)?)),
            None => Ok(None),
        }
    }

    /// Unpublish site
    pub async fn unpublish_site(
        &self,
        tenant_id: &TenantId,
        site_id: Uuid,
    ) -> Result<Option<Site>> {
        let client = self.db.get().await
            .context("Failed to get database connection")?;

        // Set RLS context
        client
            .execute("SELECT set_config('app.current_tenant_id', $1, true)", &[&tenant_id.to_string()])
            .await
            .context("Failed to set RLS tenant context")?;

        let row = client
            .query_opt(
                "UPDATE sites SET is_published = false, build_status = 'draft', updated_at = NOW() 
                 WHERE id = $1 RETURNING *",
                &[&site_id],
            )
            .await
            .context("Failed to unpublish site")?;

        match row {
            Some(row) => Ok(Some(row_to_site(&row)?)),
            None => Ok(None),
        }
    }

    /// Count sites for a tenant
    pub async fn count_sites(&self, tenant_id: &TenantId) -> Result<i64> {
        let client = self.db.get().await
            .context("Failed to get database connection")?;
        // Set RLS context
        client
            .execute("SELECT set_config('app.current_tenant_id', $1, true)", &[&tenant_id.to_string()])
            .await
            .context("Failed to set RLS tenant context")?;

        let count: i64 = client
            .query_one("SELECT COUNT(*) FROM sites", &[])
            .await
            .context("Failed to count sites")?
            .get(0);

        Ok(count)
    }

    /// Check if subdomain is available (global check across all tenants)
    pub async fn is_subdomain_available(&self, subdomain: &str) -> Result<bool> {
        let mut client = self.db.get().await
            .context("Failed to get database connection")?;

        // Use a transaction to temporarily disable FORCE RLS for global subdomain check
        let transaction = client.transaction().await
            .context("Failed to start transaction")?;

        // Temporarily disable FORCE RLS for this global check
        transaction
            .execute("ALTER TABLE sites NO FORCE ROW LEVEL SECURITY", &[])
            .await
            .context("Failed to disable FORCE RLS for subdomain check")?;

        transaction
            .execute("SET LOCAL row_security = off", &[])
            .await
            .context("Failed to disable RLS for subdomain check")?;

        let exists = transaction
            .query_opt("SELECT id FROM sites WHERE subdomain = $1", &[&subdomain])
            .await
            .context("Failed to check subdomain availability")?;

        // Re-enable FORCE RLS
        transaction
            .execute("ALTER TABLE sites FORCE ROW LEVEL SECURITY", &[])
            .await
            .context("Failed to re-enable FORCE RLS")?;

        // Commit transaction
        transaction.commit().await
            .context("Failed to commit transaction")?;

        Ok(exists.is_none())
    }

    /// Validate subdomain format
    fn validate_subdomain(&self, subdomain: &str) -> Result<()> {
        if subdomain.is_empty() {
            return Err(anyhow::anyhow!("Subdomain cannot be empty"));
        }

        if subdomain.len() > 63 {
            return Err(anyhow::anyhow!("Subdomain cannot be longer than 63 characters"));
        }

        // Check if subdomain matches the pattern: alphanumeric, can contain hyphens but not at start/end
        let re = regex::Regex::new(r"^[a-z0-9][a-z0-9-]*[a-z0-9]$|^[a-z0-9]$")
            .context("Failed to compile subdomain regex")?;

        if !re.is_match(subdomain) {
            return Err(anyhow::anyhow!(
                "Subdomain must contain only lowercase letters, numbers, and hyphens (not at start/end)"
            ));
        }

        // Check for reserved subdomains
        let reserved = ["www", "api", "admin", "app", "mail", "ftp", "blog", "shop", "store"];
        if reserved.contains(&subdomain) {
            return Err(anyhow::anyhow!("Subdomain '{}' is reserved", subdomain));
        }

        Ok(())
    }
}

/// Convert database row to Site struct
fn row_to_site(row: &Row) -> Result<Site> {
    Ok(Site {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        name: row.get("name"),
        description: row.get("description"),
        template_id: row.get("template_id"),
        custom_domain: row.get("custom_domain"),
        subdomain: row.get("subdomain"),
        is_published: row.get("is_published"),
        seo_settings: row.get("seo_settings"),
        build_status: row.get("build_status"),
        theme_config: row.get("theme_config"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}
