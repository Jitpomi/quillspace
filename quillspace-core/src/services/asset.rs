use crate::types::TenantId;
use anyhow::{Context, Result};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;
use uuid::Uuid;

/// Asset entity representing uploaded files and media
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub site_id: Option<Uuid>,
    pub filename: String,
    pub original_filename: String,
    pub mime_type: String,
    pub file_size: i64,
    pub storage_path: String,
    pub cdn_url: Option<String>,
    pub alt_text: Option<String>,
    pub is_optimized: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Asset creation request
#[derive(Debug, Deserialize)]
pub struct CreateAssetRequest {
    pub site_id: Option<Uuid>,
    pub filename: String,
    pub original_filename: String,
    pub mime_type: String,
    pub file_size: i64,
    pub storage_path: String,
    pub cdn_url: Option<String>,
    pub alt_text: Option<String>,
}

/// Asset update request
#[derive(Debug, Deserialize)]
pub struct UpdateAssetRequest {
    pub alt_text: Option<String>,
    pub cdn_url: Option<String>,
    pub is_optimized: Option<bool>,
}

/// Asset list query parameters
#[derive(Debug, Deserialize)]
pub struct AssetListQuery {
    pub site_id: Option<Uuid>,
    pub mime_type_filter: Option<String>, // e.g., "image/", "video/", "application/"
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Asset service for managing uploaded files and media
pub struct AssetService {
    db: Pool,
}

impl AssetService {
    pub fn new(db: Pool) -> Self {
        Self { db }
    }

    /// Create a new asset record
    pub async fn create_asset(
        &self,
        tenant_id: &TenantId,
        request: CreateAssetRequest,
    ) -> Result<Asset> {
        let client = self.db.get().await
            .context("Failed to get database connection")?;

        // Set RLS context
        client
            .execute("SELECT set_config('quillspace.tenant_id', $1, true)", &[&tenant_id.to_string()])
            .await
            .context("Failed to set RLS tenant context")?;

        // Validate site_id if provided
        if let Some(site_id) = request.site_id {
            let site_exists = client
                .query_opt("SELECT id FROM sites WHERE id = $1", &[&site_id])
                .await
                .context("Failed to verify site existence")?;

            if site_exists.is_none() {
                return Err(anyhow::anyhow!("Site not found or access denied"));
            }
        }

        let row = client
            .query_one(
                "INSERT INTO assets (tenant_id, site_id, filename, original_filename, mime_type, file_size, storage_path, cdn_url, alt_text) 
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) 
                 RETURNING *",
                &[
                    tenant_id.as_uuid(),
                    &request.site_id,
                    &request.filename,
                    &request.original_filename,
                    &request.mime_type,
                    &request.file_size,
                    &request.storage_path,
                    &request.cdn_url,
                    &request.alt_text,
                ],
            )
            .await
            .context("Failed to create asset")?;

        Ok(row_to_asset(&row)?)
    }

    /// Get asset by ID
    pub async fn get_asset(
        &self,
        tenant_id: &TenantId,
        asset_id: Uuid,
    ) -> Result<Option<Asset>> {
        let client = self.db.get().await
            .context("Failed to get database connection")?;

        // Set RLS context
        client
            .execute("SELECT set_config('quillspace.tenant_id', $1, true)", &[&tenant_id.to_string()])
            .await
            .context("Failed to set RLS tenant context")?;

        let row = client
            .query_opt("SELECT * FROM assets WHERE id = $1", &[&asset_id])
            .await
            .context("Failed to get asset")?;

        match row {
            Some(row) => Ok(Some(row_to_asset(&row)?)),
            None => Ok(None),
        }
    }

    /// List assets for a tenant
    pub async fn list_assets(
        &self,
        tenant_id: &TenantId,
        query: AssetListQuery,
    ) -> Result<Vec<Asset>> {
        let client = self.db.get().await
            .context("Failed to get database connection")?;

        // Set RLS context
        client
            .execute("SELECT set_config('quillspace.tenant_id', $1, true)", &[&tenant_id.to_string()])
            .await
            .context("Failed to set RLS tenant context")?;

        let limit = query.limit.unwrap_or(50).min(100);
        let offset = query.offset.unwrap_or(0);

        // Build dynamic query
        let mut sql = "SELECT * FROM assets WHERE 1=1".to_string();
        let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![];
        let mut param_count = 0;

        if let Some(site_id) = &query.site_id {
            param_count += 1;
            sql.push_str(&format!(" AND site_id = ${}", param_count));
            params.push(site_id);
        }

        if let Some(mime_filter) = &query.mime_type_filter {
            param_count += 1;
            sql.push_str(&format!(" AND mime_type LIKE ${}", param_count));
            params.push(mime_filter);
        }

        sql.push_str(" ORDER BY created_at DESC");
        
        param_count += 1;
        sql.push_str(&format!(" LIMIT ${}", param_count));
        params.push(&limit);
        
        param_count += 1;
        sql.push_str(&format!(" OFFSET ${}", param_count));
        params.push(&offset);

        let rows = client
            .query(&sql, &params)
            .await
            .context("Failed to list assets")?;

        let mut assets = Vec::new();
        for row in rows {
            assets.push(row_to_asset(&row)?);
        }

        Ok(assets)
    }

    /// Update asset metadata
    pub async fn update_asset(
        &self,
        tenant_id: &TenantId,
        asset_id: Uuid,
        request: UpdateAssetRequest,
    ) -> Result<Option<Asset>> {
        let client = self.db.get().await
            .context("Failed to get database connection")?;

        // Set RLS context
        client
            .execute("SELECT set_config('quillspace.tenant_id', $1, true)", &[&tenant_id.to_string()])
            .await
            .context("Failed to set RLS tenant context")?;

        // Build dynamic update query
        let mut set_clauses = Vec::new();
        let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![&asset_id];
        let mut param_count = 1;

        if let Some(alt_text) = &request.alt_text {
            param_count += 1;
            set_clauses.push(format!("alt_text = ${}", param_count));
            params.push(alt_text);
        }

        if let Some(cdn_url) = &request.cdn_url {
            param_count += 1;
            set_clauses.push(format!("cdn_url = ${}", param_count));
            params.push(cdn_url);
        }

        if let Some(is_optimized) = &request.is_optimized {
            param_count += 1;
            set_clauses.push(format!("is_optimized = ${}", param_count));
            params.push(is_optimized);
        }

        if set_clauses.is_empty() {
            // No updates requested, just return the current asset
            return self.get_asset(tenant_id, asset_id).await;
        }

        let query = format!(
            "UPDATE assets SET {}, updated_at = NOW() WHERE id = $1 RETURNING *",
            set_clauses.join(", ")
        );

        let row = client
            .query_opt(&query, &params)
            .await
            .context("Failed to update asset")?;

        match row {
            Some(row) => Ok(Some(row_to_asset(&row)?)),
            None => Ok(None),
        }
    }

    /// Delete asset
    pub async fn delete_asset(
        &self,
        tenant_id: &TenantId,
        asset_id: Uuid,
    ) -> Result<bool> {
        let client = self.db.get().await
            .context("Failed to get database connection")?;

        // Set RLS context
        client
            .execute("SELECT set_config('quillspace.tenant_id', $1, true)", &[&tenant_id.to_string()])
            .await
            .context("Failed to set RLS tenant context")?;

        let rows_affected = client
            .execute("DELETE FROM assets WHERE id = $1", &[&asset_id])
            .await
            .context("Failed to delete asset")?;

        Ok(rows_affected > 0)
    }

    /// Get assets by site
    pub async fn get_assets_by_site(
        &self,
        tenant_id: &TenantId,
        site_id: Uuid,
    ) -> Result<Vec<Asset>> {
        let query = AssetListQuery {
            site_id: Some(site_id),
            mime_type_filter: None,
            limit: None,
            offset: None,
        };

        self.list_assets(tenant_id, query).await
    }

    /// Get image assets only
    pub async fn get_image_assets(
        &self,
        tenant_id: &TenantId,
        site_id: Option<Uuid>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Asset>> {
        let query = AssetListQuery {
            site_id,
            mime_type_filter: Some("image/%".to_string()),
            limit,
            offset,
        };

        self.list_assets(tenant_id, query).await
    }

    /// Count assets for a tenant
    pub async fn count_assets(
        &self,
        tenant_id: &TenantId,
        site_id: Option<Uuid>,
    ) -> Result<i64> {
        let client = self.db.get().await
            .context("Failed to get database connection")?;

        // Set RLS context
        client
            .execute("SELECT set_config('quillspace.tenant_id', $1, true)", &[&tenant_id.to_string()])
            .await
            .context("Failed to set RLS tenant context")?;

        let query = if let Some(site_id) = site_id {
            client
                .query_one("SELECT COUNT(*) FROM assets WHERE site_id = $1", &[&site_id])
                .await
        } else {
            client
                .query_one("SELECT COUNT(*) FROM assets", &[])
                .await
        };

        let count: i64 = query
            .context("Failed to count assets")?
            .get(0);

        Ok(count)
    }

    /// Get total storage usage for a tenant
    pub async fn get_storage_usage(
        &self,
        tenant_id: &TenantId,
    ) -> Result<i64> {
        let client = self.db.get().await
            .context("Failed to get database connection")?;

        // Set RLS context
        client
            .execute("SELECT set_config('quillspace.tenant_id', $1, true)", &[&tenant_id.to_string()])
            .await
            .context("Failed to set RLS tenant context")?;

        let total_size: Option<i64> = client
            .query_one("SELECT COALESCE(SUM(file_size), 0) FROM assets", &[])
            .await
            .context("Failed to calculate storage usage")?
            .get(0);

        Ok(total_size.unwrap_or(0))
    }
}

/// Convert database row to Asset struct
fn row_to_asset(row: &Row) -> Result<Asset> {
    Ok(Asset {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        site_id: row.get("site_id"),
        filename: row.get("filename"),
        original_filename: row.get("original_filename"),
        mime_type: row.get("mime_type"),
        file_size: row.get("file_size"),
        storage_path: row.get("storage_path"),
        cdn_url: row.get("cdn_url"),
        alt_text: row.get("alt_text"),
        is_optimized: row.get("is_optimized"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}
