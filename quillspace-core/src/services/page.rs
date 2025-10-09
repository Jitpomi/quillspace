use crate::types::TenantId;
use anyhow::{Context, Result};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio_postgres::Row;
use uuid::Uuid;

/// Page entity representing a single page within a site
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub id: Uuid,
    pub site_id: Uuid,
    pub slug: String,
    pub title: String,
    pub meta_description: Option<String>,
    pub meta_keywords: Option<String>,
    pub puck_data: Value,
    pub is_published: bool,
    pub published_html: Option<String>,
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,
    pub sort_order: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Page creation request
#[derive(Debug, Deserialize)]
pub struct CreatePageRequest {
    pub slug: String,
    pub title: String,
    pub meta_description: Option<String>,
    pub meta_keywords: Option<String>,
    pub puck_data: Option<Value>,
    pub sort_order: Option<i32>,
}

/// Page update request
#[derive(Debug, Deserialize)]
pub struct UpdatePageRequest {
    pub slug: Option<String>,
    pub title: Option<String>,
    pub meta_description: Option<String>,
    pub meta_keywords: Option<String>,
    pub puck_data: Option<Value>,
    pub sort_order: Option<i32>,
}

/// Page publish request
#[derive(Debug, Deserialize)]
pub struct PublishPageRequest {
    pub rendered_html: String,
}

/// Page service for managing individual pages within sites
pub struct PageService {
    db: Pool,
}

impl PageService {
    pub fn new(db: Pool) -> Self {
        Self { db }
    }

    /// Create a new page
    pub async fn create_page(
        &self,
        tenant_id: &TenantId,
        site_id: Uuid,
        request: CreatePageRequest,
    ) -> Result<Page> {
        let client = self.db.get().await
            .context("Failed to get database connection")?;

        // Set RLS context
        client
            .execute("SELECT set_config('app.current_tenant_id', $1, true)", &[&tenant_id.to_string()])
            .await
            .context("Failed to set RLS tenant context")?;

        // Verify site exists and belongs to tenant
        let site_exists = client
            .query_opt("SELECT id FROM sites WHERE id = $1", &[&site_id])
            .await
            .context("Failed to verify site existence")?;

        if site_exists.is_none() {
            return Err(anyhow::anyhow!("Site not found or access denied"));
        }

        // Clean and validate slug
        let clean_slug: String = client
            .query_one("SELECT clean_slug($1)", &[&request.slug])
            .await
            .context("Failed to clean slug")?
            .get(0);

        // Check if slug already exists for this site
        let slug_exists = client
            .query_opt(
                "SELECT id FROM pages WHERE site_id = $1 AND slug = $2",
                &[&site_id, &clean_slug],
            )
            .await
            .context("Failed to check slug uniqueness")?;

        if slug_exists.is_some() {
            return Err(anyhow::anyhow!("Page with slug '{}' already exists", clean_slug));
        }

        let puck_data = request.puck_data.unwrap_or_else(|| serde_json::json!({}));
        let sort_order = request.sort_order.unwrap_or(0);

        let row = client
            .query_one(
                "INSERT INTO pages (site_id, slug, title, meta_description, meta_keywords, puck_data, sort_order) 
                 VALUES ($1, $2, $3, $4, $5, $6, $7) 
                 RETURNING *",
                &[
                    &site_id,
                    &clean_slug,
                    &request.title,
                    &request.meta_description,
                    &request.meta_keywords,
                    &puck_data,
                    &sort_order,
                ],
            )
            .await
            .context("Failed to create page")?;

        Ok(row_to_page(&row)?)
    }

    /// Get page by ID
    pub async fn get_page(
        &self,
        tenant_id: &TenantId,
        page_id: Uuid,
    ) -> Result<Option<Page>> {
        let client = self.db.get().await
            .context("Failed to get database connection")?;

        // Set RLS context
        client
            .execute("SELECT set_config('app.current_tenant_id', $1, true)", &[&tenant_id.to_string()])
            .await
            .context("Failed to set RLS tenant context")?;

        let row = client
            .query_opt(
                "SELECT p.* FROM pages p 
                 JOIN sites s ON s.id = p.site_id 
                 WHERE p.id = $1",
                &[&page_id],
            )
            .await
            .context("Failed to get page")?;

        match row {
            Some(row) => Ok(Some(row_to_page(&row)?)),
            None => Ok(None),
        }
    }

    /// Get page by site and slug
    pub async fn get_page_by_slug(
        &self,
        site_id: Uuid,
        slug: &str,
    ) -> Result<Option<Page>> {
        let client = self.db.get().await
            .context("Failed to get database connection")?;

        let row = client
            .query_opt(
                "SELECT * FROM pages WHERE site_id = $1 AND slug = $2",
                &[&site_id, &slug],
            )
            .await
            .context("Failed to get page by slug")?;

        match row {
            Some(row) => Ok(Some(row_to_page(&row)?)),
            None => Ok(None),
        }
    }

    /// List pages for a site
    pub async fn list_pages(
        &self,
        tenant_id: &TenantId,
        site_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Page>> {
        let client = self.db.get().await
            .context("Failed to get database connection")?;

        // Set RLS context
        client
            .execute("SELECT set_config('app.current_tenant_id', $1, true)", &[&tenant_id.to_string()])
            .await
            .context("Failed to set RLS tenant context")?;

        let rows = client
            .query(
                "SELECT p.* FROM pages p 
                 JOIN sites s ON s.id = p.site_id 
                 WHERE p.site_id = $1 
                 ORDER BY p.sort_order ASC, p.created_at DESC 
                 LIMIT $2 OFFSET $3",
                &[&site_id, &limit, &offset],
            )
            .await
            .context("Failed to list pages")?;

        let mut pages = Vec::new();
        for row in rows {
            pages.push(row_to_page(&row)?);
        }

        Ok(pages)
    }

    /// Update page
    pub async fn update_page(
        &self,
        tenant_id: &TenantId,
        page_id: Uuid,
        request: UpdatePageRequest,
    ) -> Result<Option<Page>> {
        let client = self.db.get().await
            .context("Failed to get database connection")?;

        // Set RLS context
        client
            .execute("SELECT set_config('app.current_tenant_id', $1, true)", &[&tenant_id.to_string()])
            .await
            .context("Failed to set RLS tenant context")?;

        // Build dynamic update query
        let mut set_clauses = Vec::new();
        let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![&page_id];
        let mut param_count = 1;

        let clean_slug_ref;
        if let Some(slug) = &request.slug {
            // Clean the slug
            let clean_slug: String = client
                .query_one("SELECT clean_slug($1)", &[slug])
                .await
                .context("Failed to clean slug")?
                .get(0);
            
            clean_slug_ref = clean_slug;
            param_count += 1;
            set_clauses.push(format!("slug = ${}", param_count));
            params.push(&clean_slug_ref);
        }

        if let Some(title) = &request.title {
            param_count += 1;
            set_clauses.push(format!("title = ${}", param_count));
            params.push(title);
        }

        if let Some(meta_description) = &request.meta_description {
            param_count += 1;
            set_clauses.push(format!("meta_description = ${}", param_count));
            params.push(meta_description);
        }

        if let Some(meta_keywords) = &request.meta_keywords {
            param_count += 1;
            set_clauses.push(format!("meta_keywords = ${}", param_count));
            params.push(meta_keywords);
        }

        if let Some(puck_data) = &request.puck_data {
            param_count += 1;
            set_clauses.push(format!("puck_data = ${}", param_count));
            params.push(puck_data);
        }

        if let Some(sort_order) = &request.sort_order {
            param_count += 1;
            set_clauses.push(format!("sort_order = ${}", param_count));
            params.push(sort_order);
        }

        if set_clauses.is_empty() {
            // No updates requested, just return the current page
            return self.get_page(tenant_id, page_id).await;
        }

        let query = format!(
            "UPDATE pages SET {}, updated_at = NOW() 
             WHERE id = $1 AND EXISTS (
                 SELECT 1 FROM sites s WHERE s.id = pages.site_id
             ) 
             RETURNING *",
            set_clauses.join(", ")
        );

        let row = client
            .query_opt(&query, &params)
            .await
            .context("Failed to update page")?;

        match row {
            Some(row) => Ok(Some(row_to_page(&row)?)),
            None => Ok(None),
        }
    }

    /// Delete page
    pub async fn delete_page(
        &self,
        tenant_id: &TenantId,
        page_id: Uuid,
    ) -> Result<bool> {
        let client = self.db.get().await
            .context("Failed to get database connection")?;

        // Set RLS context
        client
            .execute("SELECT set_config('app.current_tenant_id', $1, true)", &[&tenant_id.to_string()])
            .await
            .context("Failed to set RLS tenant context")?;

        let rows_affected = client
            .execute(
                "DELETE FROM pages 
                 WHERE id = $1 AND EXISTS (
                     SELECT 1 FROM sites s WHERE s.id = pages.site_id
                 )",
                &[&page_id],
            )
            .await
            .context("Failed to delete page")?;

        Ok(rows_affected > 0)
    }

    /// Publish page with rendered HTML
    pub async fn publish_page(
        &self,
        tenant_id: &TenantId,
        page_id: Uuid,
        request: PublishPageRequest,
    ) -> Result<Option<Page>> {
        let client = self.db.get().await
            .context("Failed to get database connection")?;

        // Set RLS context
        client
            .execute("SELECT set_config('app.current_tenant_id', $1, true)", &[&tenant_id.to_string()])
            .await
            .context("Failed to set RLS tenant context")?;

        let row = client
            .query_opt(
                "UPDATE pages SET 
                     is_published = true, 
                     published_html = $2, 
                     published_at = NOW(), 
                     updated_at = NOW() 
                 WHERE id = $1 AND EXISTS (
                     SELECT 1 FROM sites s WHERE s.id = pages.site_id
                 ) 
                 RETURNING *",
                &[&page_id, &request.rendered_html],
            )
            .await
            .context("Failed to publish page")?;

        match row {
            Some(row) => Ok(Some(row_to_page(&row)?)),
            None => Ok(None),
        }
    }

    /// Unpublish page
    pub async fn unpublish_page(
        &self,
        tenant_id: &TenantId,
        page_id: Uuid,
    ) -> Result<Option<Page>> {
        let client = self.db.get().await
            .context("Failed to get database connection")?;

        // Set RLS context
        client
            .execute("SELECT set_config('app.current_tenant_id', $1, true)", &[&tenant_id.to_string()])
            .await
            .context("Failed to set RLS tenant context")?;

        let row = client
            .query_opt(
                "UPDATE pages SET 
                     is_published = false, 
                     published_at = NULL, 
                     updated_at = NOW() 
                 WHERE id = $1 AND EXISTS (
                     SELECT 1 FROM sites s WHERE s.id = pages.site_id
                 ) 
                 RETURNING *",
                &[&page_id],
            )
            .await
            .context("Failed to unpublish page")?;

        match row {
            Some(row) => Ok(Some(row_to_page(&row)?)),
            None => Ok(None),
        }
    }

    /// Get published pages for a site (for public access)
    pub async fn get_published_pages(&self, site_id: Uuid) -> Result<Vec<Page>> {
        let client = self.db.get().await
            .context("Failed to get database connection")?;

        let rows = client
            .query(
                "SELECT * FROM pages 
                 WHERE site_id = $1 AND is_published = true 
                 ORDER BY sort_order ASC, created_at DESC",
                &[&site_id],
            )
            .await
            .context("Failed to get published pages")?;

        let mut pages = Vec::new();
        for row in rows {
            pages.push(row_to_page(&row)?);
        }

        Ok(pages)
    }

    /// Count pages for a site
    pub async fn count_pages(
        &self,
        tenant_id: &TenantId,
        site_id: Uuid,
    ) -> Result<i64> {
        let client = self.db.get().await
            .context("Failed to get database connection")?;

        // Set RLS context
        client
            .execute("SELECT set_config('app.current_tenant_id', $1, true)", &[&tenant_id.to_string()])
            .await
            .context("Failed to set RLS tenant context")?;

        let count: i64 = client
            .query_one(
                "SELECT COUNT(*) FROM pages p 
                 JOIN sites s ON s.id = p.site_id 
                 WHERE p.site_id = $1",
                &[&site_id],
            )
            .await
            .context("Failed to count pages")?
            .get(0);

        Ok(count)
    }

    /// Reorder pages
    pub async fn reorder_pages(
        &self,
        tenant_id: &TenantId,
        site_id: Uuid,
        page_orders: Vec<(Uuid, i32)>, // (page_id, sort_order)
    ) -> Result<()> {
        let mut client = self.db.get().await
            .context("Failed to get database connection")?;

        // Set RLS context
        client
            .execute("SELECT set_config('app.current_tenant_id', $1, true)", &[&tenant_id.to_string()])
            .await
            .context("Failed to set RLS tenant context")?;

        let transaction = client.transaction().await
            .context("Failed to start transaction")?;

        for (page_id, sort_order) in page_orders {
            transaction
                .execute(
                    "UPDATE pages SET sort_order = $3, updated_at = NOW() 
                     WHERE id = $1 AND site_id = $2 AND EXISTS (
                         SELECT 1 FROM sites s WHERE s.id = $2
                     )",
                    &[&page_id, &site_id, &sort_order],
                )
                .await
                .context("Failed to update page sort order")?;
        }

        transaction.commit().await
            .context("Failed to commit page reorder transaction")?;

        Ok(())
    }
}

/// Convert database row to Page struct
fn row_to_page(row: &Row) -> Result<Page> {
    Ok(Page {
        id: row.get("id"),
        site_id: row.get("site_id"),
        slug: row.get("slug"),
        title: row.get("title"),
        meta_description: row.get("meta_description"),
        meta_keywords: row.get("meta_keywords"),
        puck_data: row.get("puck_data"),
        is_published: row.get("is_published"),
        published_html: row.get("published_html"),
        published_at: row.get("published_at"),
        sort_order: row.get("sort_order"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}
