// Example of how ContentService would look with native postgres crate
// This is just for comparison - your current SQLx implementation is better

use crate::types::{Content, ContentStatus, TenantId, UserId};
use anyhow::Result;
use tokio_postgres::{Client, NoTls};
use uuid::Uuid;

pub struct ContentServiceNative {
    client: Client,
}

impl ContentServiceNative {
    pub async fn new(database_url: &str) -> Result<Self> {
        let (client, connection) = tokio_postgres::connect(database_url, NoTls).await?;
        
        // Spawn connection handler
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Connection error: {}", e);
            }
        });

        Ok(Self { client })
    }

    /// Create new content - more verbose than SQLx
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

        // Manual query construction - no compile-time verification
        let row = self.client
            .query_one(
                r#"
                INSERT INTO content (id, tenant_id, title, slug, body, status, author_id, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                RETURNING id, tenant_id, title, slug, body, status, author_id, published_at, created_at, updated_at
                "#,
                &[
                    &content_id,
                    tenant_id.as_uuid(),
                    &title,
                    &slug,
                    &body,
                    &"draft", // Manual enum conversion
                    author_id.as_uuid(),
                    &now,
                    &now,
                ],
            )
            .await?;

        // Manual type mapping - more error-prone
        let content = Content {
            id: row.get("id"),
            tenant_id: row.get("tenant_id"),
            title: row.get("title"),
            slug: row.get("slug"),
            body: row.get("body"),
            status: match row.get::<_, String>("status").as_str() {
                "draft" => ContentStatus::Draft,
                "published" => ContentStatus::Published,
                "archived" => ContentStatus::Archived,
                _ => ContentStatus::Draft,
            },
            author_id: row.get("author_id"),
            published_at: row.get("published_at"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };

        Ok(content)
    }

    /// Get content by ID - manual error handling
    pub async fn get_content(
        &self,
        tenant_id: &TenantId,
        content_id: Uuid,
    ) -> Result<Option<Content>> {
        let rows = self.client
            .query(
                "SELECT * FROM content WHERE id = $1 AND tenant_id = $2",
                &[&content_id, tenant_id.as_uuid()],
            )
            .await?;

        if rows.is_empty() {
            return Ok(None);
        }

        let row = &rows[0];
        
        // More manual mapping...
        let content = Content {
            id: row.get("id"),
            tenant_id: row.get("tenant_id"),
            title: row.get("title"),
            slug: row.get("slug"),
            body: row.get("body"),
            status: match row.get::<_, String>("status").as_str() {
                "draft" => ContentStatus::Draft,
                "published" => ContentStatus::Published,
                "archived" => ContentStatus::Archived,
                _ => ContentStatus::Draft,
            },
            author_id: row.get("author_id"),
            published_at: row.get("published_at"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };

        Ok(Some(content))
    }
}

// Comparison: Your current SQLx version is much cleaner:
/*
pub async fn create_content(&self, ...) -> Result<Content> {
    let content = sqlx::query_as::<_, Content>(
        "INSERT INTO content (...) VALUES (...) RETURNING *"
    )
    .bind(content_id)
    .bind(tenant_id.as_uuid())
    // ... other binds
    .fetch_one(&self.db)
    .await?;

    Ok(content)  // Much simpler!
}
*/
