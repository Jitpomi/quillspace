use crate::types::{Content, ContentStatus, TenantId, UserId};
use anyhow::Result;
use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;
use tokio_postgres::{Row, Error as PgError};
use uuid::Uuid;

/// Helper function to convert a tokio-postgres Row to Content
fn row_to_content(row: &Row) -> Result<Content, PgError> {
    let status_str: String = row.try_get("status")?;
    let status = match status_str.as_str() {
        "Draft" => ContentStatus::Draft,
        "Published" => ContentStatus::Published,
        "Archived" => ContentStatus::Archived,
        _ => ContentStatus::Draft,
    };

    Ok(Content {
        id: row.try_get("id")?,
        tenant_id: row.try_get("tenant_id")?,
        title: row.try_get("title")?,
        slug: row.try_get("slug")?,
        body: row.try_get("body")?,
        status,
        author_id: row.try_get("author_id")?,
        published_at: row.try_get("published_at")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

/// Helper function to convert ContentStatus to string for database
fn content_status_to_string(status: &ContentStatus) -> &'static str {
    match status {
        ContentStatus::Draft => "Draft",
        ContentStatus::Published => "Published",
        ContentStatus::Archived => "Archived",
    }
}

/// Content management service
#[derive(Clone)]
pub struct ContentService {
    db: Pool,
}

impl ContentService {
    pub fn new(db: Pool) -> Self {
        Self { db }
    }

    /// Create new content
    pub async fn create_content(
        &self,
        tenant_id: &TenantId,
        author_id: &UserId,
        title: String,
        slug: String,
        body: String,
    ) -> Result<Content> {
        let content_id = Uuid::new_v4();
        let now = chrono::Utc::now();

        // Get database connection
        let client = self.db.get().await?;

        let query = r#"
            INSERT INTO content (id, tenant_id, title, slug, body, status, author_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#;

        let status_str = content_status_to_string(&ContentStatus::Draft);
        let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![
            &content_id,
            tenant_id.as_uuid(),
            &title,
            &slug,
            &body,
            &status_str,
            author_id.as_uuid(),
            &now,
            &now,
        ];

        let row = client.query_one(query, &params).await?;
        let content = row_to_content(&row)?;

        Ok(content)
    }

    /// Get content by ID
    pub async fn get_content(
        &self,
        tenant_id: &TenantId,
        content_id: Uuid,
    ) -> Result<Option<Content>> {
        let client = self.db.get().await?;

        let query = "SELECT * FROM content WHERE id = $1 AND tenant_id = $2";
        let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![&content_id, tenant_id.as_uuid()];

        match client.query_opt(query, &params).await? {
            Some(row) => {
                let content = row_to_content(&row)?;
                Ok(Some(content))
            }
            None => Ok(None),
        }
    }

    /// Update content
    pub async fn update_content(
        &self,
        tenant_id: &TenantId,
        content_id: Uuid,
        title: Option<String>,
        slug: Option<String>,
        body: Option<String>,
    ) -> Result<Option<Content>> {
        let now = chrono::Utc::now();
        let client = self.db.get().await?;

        let query = r#"
            UPDATE content 
            SET title = COALESCE($3, title),
                slug = COALESCE($4, slug),
                body = COALESCE($5, body),
                updated_at = $6
            WHERE id = $1 AND tenant_id = $2
            RETURNING *
            "#;

        let title_ref = title.as_deref();
        let slug_ref = slug.as_deref();
        let body_ref = body.as_deref();
        
        let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![
            &content_id,
            tenant_id.as_uuid(),
            &title_ref,
            &slug_ref,
            &body_ref,
            &now,
        ];

        match client.query_opt(query, &params).await? {
            Some(row) => {
                let content = row_to_content(&row)?;
                Ok(Some(content))
            }
            None => Ok(None),
        }
    }

    /// Publish content
    pub async fn publish_content(
        &self,
        tenant_id: &TenantId,
        content_id: Uuid,
    ) -> Result<Option<Content>> {
        let now = chrono::Utc::now();
        let client = self.db.get().await?;

        let query = r#"
            UPDATE content 
            SET status = $3, published_at = $4, updated_at = $5
            WHERE id = $1 AND tenant_id = $2
            RETURNING *
            "#;

        let status_str = content_status_to_string(&ContentStatus::Published);
        let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![
            &content_id,
            tenant_id.as_uuid(),
            &status_str,
            &now,
            &now,
        ];

        match client.query_opt(query, &params).await? {
            Some(row) => {
                let content = row_to_content(&row)?;
                Ok(Some(content))
            }
            None => Ok(None),
        }
    }

    /// List content for tenant
    pub async fn list_content(
        &self,
        tenant_id: &TenantId,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Content>> {
        let client = self.db.get().await?;

        let query = r#"
            SELECT * FROM content 
            WHERE tenant_id = $1 
            ORDER BY created_at DESC 
            LIMIT $2 OFFSET $3
            "#;

        let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![
            tenant_id.as_uuid(),
            &limit,
            &offset,
        ];

        let rows = client.query(query, &params).await?;
        let content: Result<Vec<Content>, _> = rows.iter().map(row_to_content).collect();
        
        Ok(content?)
    }

    /// Delete content
    pub async fn delete_content(
        &self,
        tenant_id: &TenantId,
        content_id: Uuid,
    ) -> Result<bool> {
        let client = self.db.get().await?;

        let query = "DELETE FROM content WHERE id = $1 AND tenant_id = $2";
        let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![&content_id, tenant_id.as_uuid()];

        let rows_affected = client.execute(query, &params).await?;
        
        Ok(rows_affected > 0)
    }
}
