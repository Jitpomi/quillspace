use crate::types::{Content, ContentStatus, TenantId, UserId};
use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

/// Content management service
#[derive(Clone)]
pub struct ContentService {
    db: PgPool,
}

impl ContentService {
    pub fn new(db: PgPool) -> Self {
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

        let content = sqlx::query_as::<_, Content>(
            r#"
            INSERT INTO content (id, tenant_id, title, slug, body, status, author_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#
        )
        .bind(content_id)
        .bind(tenant_id.as_uuid())
        .bind(&title)
        .bind(&slug)
        .bind(&body)
        .bind(ContentStatus::Draft)
        .bind(author_id.as_uuid())
        .bind(now)
        .bind(now)
        .fetch_one(&self.db)
        .await?;

        Ok(content)
    }

    /// Get content by ID
    pub async fn get_content(
        &self,
        tenant_id: &TenantId,
        content_id: Uuid,
    ) -> Result<Option<Content>> {
        let content = sqlx::query_as::<_, Content>(
            "SELECT * FROM content WHERE id = $1 AND tenant_id = $2"
        )
        .bind(content_id)
        .bind(tenant_id.as_uuid())
        .fetch_optional(&self.db)
        .await?;

        Ok(content)
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

        let content = sqlx::query_as::<_, Content>(
            r#"
            UPDATE content 
            SET title = COALESCE($3, title),
                slug = COALESCE($4, slug),
                body = COALESCE($5, body),
                updated_at = $6
            WHERE id = $1 AND tenant_id = $2
            RETURNING *
            "#
        )
        .bind(content_id)
        .bind(tenant_id.as_uuid())
        .bind(title)
        .bind(slug)
        .bind(body)
        .bind(now)
        .fetch_optional(&self.db)
        .await?;

        Ok(content)
    }

    /// Publish content
    pub async fn publish_content(
        &self,
        tenant_id: &TenantId,
        content_id: Uuid,
    ) -> Result<Option<Content>> {
        let now = chrono::Utc::now();

        let content = sqlx::query_as::<_, Content>(
            r#"
            UPDATE content 
            SET status = $3, published_at = $4, updated_at = $5
            WHERE id = $1 AND tenant_id = $2
            RETURNING *
            "#
        )
        .bind(content_id)
        .bind(tenant_id.as_uuid())
        .bind(ContentStatus::Published)
        .bind(now)
        .bind(now)
        .fetch_optional(&self.db)
        .await?;

        Ok(content)
    }

    /// List content for tenant
    pub async fn list_content(
        &self,
        tenant_id: &TenantId,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Content>> {
        let content = sqlx::query_as::<_, Content>(
            r#"
            SELECT * FROM content 
            WHERE tenant_id = $1 
            ORDER BY created_at DESC 
            LIMIT $2 OFFSET $3
            "#
        )
        .bind(tenant_id.as_uuid())
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db)
        .await?;

        Ok(content)
    }

    /// Delete content
    pub async fn delete_content(
        &self,
        tenant_id: &TenantId,
        content_id: Uuid,
    ) -> Result<bool> {
        let result = sqlx::query(
            "DELETE FROM content WHERE id = $1 AND tenant_id = $2"
        )
        .bind(content_id)
        .bind(tenant_id.as_uuid())
        .execute(&self.db)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
