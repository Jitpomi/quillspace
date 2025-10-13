use uuid::Uuid;
use serde::{Deserialize, Serialize};
use tokio_postgres::Client;
use std::sync::Arc;

use crate::services::composition::{PuckComposition, RenderContext, RenderDefaults, composition_to_context};
use crate::services::template_cache::{TemplateCache, TemplateCacheError};

/// Page data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub site_id: Uuid,
    pub slug: String,
    pub title: String,
    pub template_id: Uuid,
    pub template_version: i32,
    pub draft_composition: PuckComposition,
    pub published_url: Option<String>,
    pub published_etag: Option<String>,
    pub is_published: bool,
    pub preview_image_url: Option<String>,
    pub preview_status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Request to save page draft
#[derive(Debug, Deserialize)]
pub struct SavePageDraftRequest {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub template_id: Option<Uuid>,
    pub template_version: Option<i32>,
    pub draft_composition: PuckComposition,
}

/// Request to switch page template
#[derive(Debug, Deserialize)]
pub struct SwitchTemplateRequest {
    pub template_id: Uuid,
    pub template_version: i32,
}

/// Preview link response
#[derive(Debug, Serialize)]
pub struct PreviewLinkResponse {
    pub url: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

/// Page service for managing page operations
#[derive(Debug)]
pub struct PageService {
    db_client: Arc<Client>,
    template_cache: Arc<TemplateCache>,
    render_defaults: RenderDefaults,
}

impl PageService {
    pub fn new(
        db_client: Arc<Client>,
        template_cache: Arc<TemplateCache>,
        render_defaults: RenderDefaults,
    ) -> Self {
        Self {
            db_client,
            template_cache,
            render_defaults,
        }
    }

    /// Save page draft (Puck composition JSON)
    pub async fn save_draft(
        &self,
        page_id: Uuid,
        tenant_id: Uuid,
        request: SavePageDraftRequest,
    ) -> Result<Page, PageServiceError> {
        // Serialize composition to JSON
        let composition_json = serde_json::to_value(&request.draft_composition)
            .map_err(|e| PageServiceError::SerializationError(e.to_string()))?;

        // Build update query dynamically based on provided fields
        let mut query_parts = vec!["draft_composition = $3", "updated_at = now()"];
        let mut param_count = 3;
        let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![
            &page_id,
            &tenant_id,
            &composition_json,
        ];

        if let Some(ref title) = request.title {
            param_count += 1;
            query_parts.push(&format!("title = ${}", param_count));
            params.push(title);
        }

        if let Some(ref slug) = request.slug {
            param_count += 1;
            query_parts.push(&format!("slug = ${}", param_count));
            params.push(slug);
        }

        if let Some(ref template_id) = request.template_id {
            param_count += 1;
            query_parts.push(&format!("template_id = ${}", param_count));
            params.push(template_id);
        }

        if let Some(ref template_version) = request.template_version {
            param_count += 1;
            query_parts.push(&format!("template_version = ${}", param_count));
            params.push(template_version);
        }

        let query = format!(
            r#"
            UPDATE pages 
            SET {}
            WHERE id = $1 AND tenant_id = $2
            RETURNING id, tenant_id, site_id, slug, title, template_id, template_version,
                      draft_composition, published_url, published_etag, is_published,
                      preview_image_url, preview_status, created_at, updated_at
            "#,
            query_parts.join(", ")
        );

        let row = self.db_client
            .query_opt(&query, &params)
            .await
            .map_err(PageServiceError::DatabaseError)?
            .ok_or(PageServiceError::PageNotFound(page_id))?;

        let page = self.row_to_page(row)?;
        
        // Queue preview thumbnail generation
        self.queue_preview_generation(page_id).await?;

        Ok(page)
    }

    /// Switch page template
    pub async fn switch_template(
        &self,
        page_id: Uuid,
        tenant_id: Uuid,
        request: SwitchTemplateRequest,
    ) -> Result<Page, PageServiceError> {
        // Verify template exists and is accessible
        self.template_cache
            .get_template_by_id(request.template_id)
            .await
            .map_err(|e| match e {
                TemplateCacheError::TemplateNotFoundById(_) => {
                    PageServiceError::TemplateNotFound(request.template_id)
                }
                _ => PageServiceError::TemplateCacheError(e),
            })?;

        let query = r#"
            UPDATE pages 
            SET template_id = $3, template_version = $4, updated_at = now()
            WHERE id = $1 AND tenant_id = $2
            RETURNING id, tenant_id, site_id, slug, title, template_id, template_version,
                      draft_composition, published_url, published_etag, is_published,
                      preview_image_url, preview_status, created_at, updated_at
        "#;

        let row = self.db_client
            .query_opt(query, &[&page_id, &tenant_id, &request.template_id, &request.template_version])
            .await
            .map_err(PageServiceError::DatabaseError)?
            .ok_or(PageServiceError::PageNotFound(page_id))?;

        let page = self.row_to_page(row)?;
        
        // Queue preview thumbnail generation with new template
        self.queue_preview_generation(page_id).await?;

        Ok(page)
    }

    /// Get page by ID
    pub async fn get_page(&self, page_id: Uuid, tenant_id: Uuid) -> Result<Page, PageServiceError> {
        let query = r#"
            SELECT id, tenant_id, site_id, slug, title, template_id, template_version,
                   draft_composition, published_url, published_etag, is_published,
                   preview_image_url, preview_status, created_at, updated_at
            FROM pages 
            WHERE id = $1 AND tenant_id = $2
        "#;

        let row = self.db_client
            .query_opt(query, &[&page_id, &tenant_id])
            .await
            .map_err(PageServiceError::DatabaseError)?
            .ok_or(PageServiceError::PageNotFound(page_id))?;

        self.row_to_page(row)
    }

    /// Render page for preview (SSR from draft)
    pub async fn render_preview(
        &self,
        page_id: Uuid,
        tenant_id: Uuid,
        site_slug: Option<&str>,
    ) -> Result<String, PageServiceError> {
        // Get page data
        let page = self.get_page(page_id, tenant_id).await?;
        
        // Get template
        let cached_template = self.template_cache
            .get_template_by_id(page.template_id)
            .await
            .map_err(PageServiceError::TemplateCacheError)?;

        // Transform composition to render context
        let context = composition_to_context(
            &page.draft_composition,
            &self.render_defaults,
            site_slug,
        ).map_err(PageServiceError::CompositionError)?;

        // Render with MiniJinja
        let html = cached_template.environment
            .get_template(&cached_template.template.main_name)
            .map_err(|e| PageServiceError::TemplateRenderError(e.to_string()))?
            .render(&context)
            .map_err(|e| PageServiceError::TemplateRenderError(e.to_string()))?;

        // Add preview meta tags
        let html_with_meta = self.add_preview_meta_tags(html);

        Ok(html_with_meta)
    }

    /// Generate preview link token
    pub async fn generate_preview_link(
        &self,
        page_id: Uuid,
        tenant_id: Uuid,
        base_url: &str,
    ) -> Result<PreviewLinkResponse, PageServiceError> {
        // Verify page exists
        let _page = self.get_page(page_id, tenant_id).await?;

        // Generate JWT token (simplified - in production use proper JWT library)
        let expires_at = chrono::Utc::now() + chrono::Duration::minutes(30);
        let token = format!("{}:{}:{}", tenant_id, page_id, expires_at.timestamp());
        let encoded_token = base64::encode(&token);

        let url = format!("{}/preview/{}", base_url, encoded_token);

        Ok(PreviewLinkResponse {
            url,
            expires_at,
        })
    }

    /// Parse preview token
    pub fn parse_preview_token(&self, token: &str) -> Result<(Uuid, Uuid, chrono::DateTime<chrono::Utc>), PageServiceError> {
        let decoded = base64::decode(token)
            .map_err(|_| PageServiceError::InvalidPreviewToken)?;
        
        let token_str = String::from_utf8(decoded)
            .map_err(|_| PageServiceError::InvalidPreviewToken)?;
        
        let parts: Vec<&str> = token_str.split(':').collect();
        if parts.len() != 3 {
            return Err(PageServiceError::InvalidPreviewToken);
        }

        let tenant_id = Uuid::parse_str(parts[0])
            .map_err(|_| PageServiceError::InvalidPreviewToken)?;
        
        let page_id = Uuid::parse_str(parts[1])
            .map_err(|_| PageServiceError::InvalidPreviewToken)?;
        
        let timestamp = parts[2].parse::<i64>()
            .map_err(|_| PageServiceError::InvalidPreviewToken)?;
        
        let expires_at = chrono::DateTime::from_timestamp(timestamp, 0)
            .ok_or(PageServiceError::InvalidPreviewToken)?;

        // Check if token is expired
        if chrono::Utc::now() > expires_at {
            return Err(PageServiceError::PreviewTokenExpired);
        }

        Ok((tenant_id, page_id, expires_at))
    }

    /// Queue preview thumbnail generation
    async fn queue_preview_generation(&self, page_id: Uuid) -> Result<(), PageServiceError> {
        let query = r#"
            UPDATE pages 
            SET preview_status = 'queued', updated_at = now()
            WHERE id = $1
        "#;

        self.db_client
            .execute(query, &[&page_id])
            .await
            .map_err(PageServiceError::DatabaseError)?;

        // TODO: Send to actual queue (Redis, SQS, etc.)
        tracing::info!("Queued preview generation for page {}", page_id);

        Ok(())
    }

    /// Add preview-specific meta tags
    fn add_preview_meta_tags(&self, html: String) -> String {
        let preview_meta = r#"<meta name="robots" content="noindex,nofollow">
<style>
.preview-banner {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    background: #3b82f6;
    color: white;
    padding: 8px 16px;
    text-align: center;
    font-size: 14px;
    z-index: 9999;
    font-family: -apple-system, BlinkMacSystemFont, sans-serif;
}
body { margin-top: 40px !important; }
</style>
<div class="preview-banner">🔍 Preview Mode - This page is not published</div>"#;

        html.replace("</head>", &format!("{}</head>", preview_meta))
    }

    /// Convert database row to Page struct
    fn row_to_page(&self, row: tokio_postgres::Row) -> Result<Page, PageServiceError> {
        let composition_json: serde_json::Value = row.get("draft_composition");
        let draft_composition: PuckComposition = serde_json::from_value(composition_json)
            .map_err(|e| PageServiceError::SerializationError(e.to_string()))?;

        Ok(Page {
            id: row.get("id"),
            tenant_id: row.get("tenant_id"),
            site_id: row.get("site_id"),
            slug: row.get("slug"),
            title: row.get("title"),
            template_id: row.get("template_id"),
            template_version: row.get("template_version"),
            draft_composition,
            published_url: row.get("published_url"),
            published_etag: row.get("published_etag"),
            is_published: row.get("is_published"),
            preview_image_url: row.get("preview_image_url"),
            preview_status: row.get("preview_status"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }
}

/// Page service errors
#[derive(Debug, thiserror::Error)]
pub enum PageServiceError {
    #[error("Page not found: {0}")]
    PageNotFound(Uuid),
    
    #[error("Template not found: {0}")]
    TemplateNotFound(Uuid),
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] tokio_postgres::Error),
    
    #[error("Template cache error: {0}")]
    TemplateCacheError(#[from] TemplateCacheError),
    
    #[error("Composition error: {0}")]
    CompositionError(#[from] crate::services::composition::CompositionError),
    
    #[error("Template render error: {0}")]
    TemplateRenderError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Invalid preview token")]
    InvalidPreviewToken,
    
    #[error("Preview token expired")]
    PreviewTokenExpired,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_preview_token_parsing() {
        let service = PageService::new(
            Arc::new(unsafe { std::mem::zeroed() }), // Mock for test
            Arc::new(unsafe { std::mem::zeroed() }), // Mock for test
            RenderDefaults::default(),
        );

        let tenant_id = Uuid::new_v4();
        let page_id = Uuid::new_v4();
        let expires_at = chrono::Utc::now() + chrono::Duration::minutes(30);
        
        let token = format!("{}:{}:{}", tenant_id, page_id, expires_at.timestamp());
        let encoded_token = base64::encode(&token);

        let result = service.parse_preview_token(&encoded_token);
        assert!(result.is_ok());
        
        let (parsed_tenant_id, parsed_page_id, _) = result.unwrap();
        assert_eq!(parsed_tenant_id, tenant_id);
        assert_eq!(parsed_page_id, page_id);
    }
}
