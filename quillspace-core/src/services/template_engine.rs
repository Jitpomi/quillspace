use anyhow::{Context, Result};
use minijinja::Environment;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio_postgres::Row;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::database::{DatabaseConnections, rls_helper::RlsHelper};

/// Template engine service with database loader for MiniJinja templates
pub struct TemplateEngine {
    env: Environment<'static>,
    db: Arc<DatabaseConnections>,
    template_cache: std::sync::RwLock<HashMap<String, String>>,
}

/// Template data structure matching database schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub html_source: String,
    pub default_schema: Value,
    pub preview_image_url: Option<String>,
    pub is_public: bool,
    pub version: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Context data for template rendering
#[derive(Debug, Serialize)]
pub struct TemplateContext {
    pub site: SiteContext,
    pub page: PageContext,
    pub puck_content: String,
    pub user: Option<UserContext>,
}

#[derive(Debug, Serialize)]
pub struct SiteContext {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub subdomain: String,
    pub custom_domain: Option<String>,
    pub seo_settings: Value,
}

#[derive(Debug, Serialize)]
pub struct PageContext {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub meta_description: Option<String>,
    pub meta_keywords: Option<String>,
    pub is_published: bool,
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize)]
pub struct UserContext {
    pub id: Uuid,
    pub name: String,
    pub email: String,
}

impl TemplateEngine {
    /// Create a new template engine with database loader
    pub fn new(db: Arc<DatabaseConnections>) -> Result<Self> {
        let mut env = Environment::new();
        
        // Configure MiniJinja environment
        env.set_auto_escape_callback(|name| {
            // Auto-escape HTML files, but not text files
            if name.ends_with(".html") || name.ends_with(".htm") {
                minijinja::AutoEscape::Html
            } else {
                minijinja::AutoEscape::None
            }
        });
        
        // Add custom filters
        env.add_filter("markdown", markdown_filter);
        env.add_filter("truncate", truncate_filter);
        env.add_filter("date", date_filter);
        
        // Add custom functions
        env.add_function("asset_url", asset_url_function);
        env.add_function("url", url_function);
        
        // Add global variables
        env.add_global("now", minijinja::Value::from_serialize(&chrono::Utc::now()));
        
        Ok(Self {
            env,
            db,
            template_cache: std::sync::RwLock::new(HashMap::new()),
        })
    }
    
    /// Load template from database with caching
    pub async fn load_template(&self, name: &str, tenant_id: Uuid) -> Result<String> {
        let cache_key = format!("{}:{}", tenant_id, name);
        
        // Check cache first
        if let Ok(cache) = self.template_cache.read() {
            if let Some(cached_template) = cache.get(&cache_key) {
                return Ok(cached_template.clone());
            }
        }
        
        // Load from database
        let query = "
            SELECT html_source 
            FROM templates 
            WHERE name = $1 AND (tenant_id = $2 OR is_public = true)
            ORDER BY tenant_id = $2 DESC, version DESC
            LIMIT 1
        ";
        
        let client = self.db.postgres().get().await
            .context("Failed to get database connection")?;
        let row = client
            .query_opt(query, &[&name, &tenant_id])
            .await
            .context("Failed to query template from database")?;
        
        match row {
            Some(row) => {
                let html_source: String = row.get("html_source");
                
                // Cache the template
                if let Ok(mut cache) = self.template_cache.write() {
                    cache.insert(cache_key, html_source.clone());
                }
                
                Ok(html_source)
            }
            None => {
                error!("Template '{}' not found for tenant {}", name, tenant_id);
                Err(anyhow::anyhow!("Template '{}' not found", name))
            }
        }
    }
    
    /// Render template with context
    pub async fn render_template(
        &self,
        template_name: &str,
        tenant_id: Uuid,
        context: &TemplateContext,
    ) -> Result<String> {
        // Load template source
        let template_source = self.load_template(template_name, tenant_id).await?;
        
        // Create a new environment for this render to avoid lifetime issues
        let mut env = Environment::new();
        env.add_template(template_name, &template_source)
            .context("Failed to add template to environment")?;
        
        // Get template and render
        let template = env.get_template(template_name)
            .context("Failed to get template from environment")?;
        
        let rendered = template.render(context)
            .context("Failed to render template")?;
        
        Ok(rendered)
    }
    
    /// Get template by name and tenant
    pub async fn get_template(&self, name: &str, tenant_id: Uuid) -> Result<Template> {
        let query = "
            SELECT id, tenant_id, name, description, category, html_source, 
                   default_schema, preview_image_url, is_public, version,
                   created_at, updated_at
            FROM templates 
            WHERE name = $1 AND (tenant_id = $2 OR is_public = true)
            ORDER BY tenant_id = $2 DESC, version DESC
            LIMIT 1
        ";
        
        let client = self.db.postgres().get().await
            .context("Failed to get database connection")?;
        let row = client
            .query_opt(query, &[&name, &tenant_id])
            .await
            .context("Failed to query template")?;
        
        match row {
            Some(row) => Ok(self.row_to_template(row)?),
            None => Err(anyhow::anyhow!("Template '{}' not found", name)),
        }
    }
    
    /// List templates for tenant
    pub async fn list_templates(
        &self,
        tenant_id: Uuid,
        include_public: bool,
        category: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Template>> {
        // With RLS enabled, we don't need to filter by tenant_id explicitly
        // The RLS policy will handle tenant isolation automatically
        let mut query = "
            SELECT id, tenant_id, name, description, category, html_source, 
                   default_schema, preview_image_url, is_public, version,
                   created_at, updated_at
            FROM templates 
            WHERE 1=1".to_string();
        
        // Note: RLS policy will automatically filter by tenant_id
        // We only need to handle the public templates logic if needed
        
        let mut param_count = 0;
        if let Some(_cat) = category {
            param_count += 1;
            query.push_str(&format!(" AND category = ${}", param_count));
        }
        
        param_count += 1;
        query.push_str(&format!(" ORDER BY created_at DESC LIMIT ${}", param_count));
        
        param_count += 1;
        query.push_str(&format!(" OFFSET ${}", param_count));
        
        let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![];
        let category_ref;
        if let Some(cat) = category {
            category_ref = cat;
            params.push(&category_ref);
        }
        params.push(&limit);
        params.push(&offset);
        
        let mut client = self.db.postgres().get().await
            .context("Failed to get database connection")?;
        
        // Use a transaction to ensure RLS context persists
        let transaction = client.transaction().await
            .context("Failed to start transaction")?;
        
        // Set RLS context for tenant isolation within transaction
        info!("Setting RLS context for tenant: {}", tenant_id);
        transaction
            .execute("SELECT set_config('app.current_tenant_id', $1, true)", &[&tenant_id.to_string()])
            .await
            .context("Failed to set RLS tenant context")?;
        
        info!("Executing query: {} with params: {:?}", query, params.len());
        
        let rows = transaction
            .query(&query, &params)
            .await
            .context("Failed to list templates")?;
        
        // Commit transaction
        transaction.commit().await
            .context("Failed to commit transaction")?;
        
        info!("Found {} template rows", rows.len());
        
        let mut templates = Vec::new();
        for row in rows {
            templates.push(self.row_to_template(row)?);
        }
        
        Ok(templates)
    }
    
    /// Create new template
    pub async fn create_template(
        &self,
        tenant_id: Uuid,
        name: &str,
        description: Option<&str>,
        category: &str,
        html_source: &str,
        default_schema: &Value,
    ) -> Result<Template> {
        // Validate template syntax
        self.validate_template_syntax(html_source)?;
        
        let query = "
            INSERT INTO templates (tenant_id, name, description, category, html_source, default_schema)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, tenant_id, name, description, category, html_source, 
                      default_schema, preview_image_url, is_public, version,
                      created_at, updated_at
        ";
        
        let client = self.db.postgres().get().await
            .context("Failed to get database connection")?;
        
        // Set RLS context for tenant isolation
        RlsHelper::set_tenant_context(&client, &tenant_id).await?;
        
        let row = client
            .query_one(query, &[&tenant_id, &name, &description, &category, &html_source, &default_schema])
            .await
            .context("Failed to create template")?;
        
        let template = self.row_to_template(row)?;
        
        // Clear cache for this tenant
        self.clear_cache_for_tenant(tenant_id);
        
        info!("Created template '{}' for tenant {}", name, tenant_id);
        Ok(template)
    }
    
    /// Update template
    pub async fn update_template(
        &self,
        template_id: Uuid,
        tenant_id: Uuid,
        html_source: Option<&str>,
        description: Option<&str>,
        default_schema: Option<&Value>,
    ) -> Result<Template> {
        if let Some(html) = html_source {
            self.validate_template_syntax(html)?;
        }
        
        let query = "
            UPDATE templates 
            SET html_source = COALESCE($3, html_source),
                description = COALESCE($4, description),
                default_schema = COALESCE($5, default_schema),
                updated_at = NOW()
            WHERE id = $1 AND tenant_id = $2
            RETURNING id, tenant_id, name, description, category, html_source, 
                      default_schema, preview_image_url, is_public, version,
                      created_at, updated_at
        ";
        
        let client = self.db.postgres().get().await
            .context("Failed to get database connection")?;
        let row = client
            .query_opt(query, &[&template_id, &tenant_id, &html_source, &description, &default_schema])
            .await
            .context("Failed to update template")?;
        
        match row {
            Some(row) => {
                let template = self.row_to_template(row)?;
                self.clear_cache_for_tenant(tenant_id);
                info!("Updated template {} for tenant {}", template_id, tenant_id);
                Ok(template)
            }
            None => Err(anyhow::anyhow!("Template not found or access denied")),
        }
    }
    
    /// Delete template
    pub async fn delete_template(&self, template_id: Uuid, tenant_id: Uuid) -> Result<()> {
        let query = "DELETE FROM templates WHERE id = $1 AND tenant_id = $2";
        
        let client = self.db.postgres().get().await
            .context("Failed to get database connection")?;
        let rows_affected = client
            .execute(query, &[&template_id, &tenant_id])
            .await
            .context("Failed to delete template")?;
        
        if rows_affected == 0 {
            return Err(anyhow::anyhow!("Template not found or access denied"));
        }
        
        self.clear_cache_for_tenant(tenant_id);
        info!("Deleted template {} for tenant {}", template_id, tenant_id);
        Ok(())
    }
    
    /// Validate template syntax
    fn validate_template_syntax(&self, html_source: &str) -> Result<()> {
        // Create a temporary environment for validation
        let mut temp_env = Environment::new();
        
        // Try to parse the template to check for syntax errors
        match temp_env.add_template("__validation__", html_source) {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Template syntax validation failed: {}", e);
                Err(anyhow::anyhow!("Template syntax error: {}", e))
            }
        }
    }
    
    /// Clear cache for tenant
    fn clear_cache_for_tenant(&self, tenant_id: Uuid) {
        if let Ok(mut cache) = self.template_cache.write() {
            let tenant_prefix = format!("{}:", tenant_id);
            cache.retain(|key, _| !key.starts_with(&tenant_prefix));
        }
    }
    
    /// Convert database row to Template struct
    fn row_to_template(&self, row: Row) -> Result<Template> {
        Ok(Template {
            id: row.get("id"),
            tenant_id: row.get("tenant_id"),
            name: row.get("name"),
            description: row.get("description"),
            category: row.get("category"),
            html_source: row.get("html_source"),
            default_schema: row.get("default_schema"),
            preview_image_url: row.get("preview_image_url"),
            is_public: row.get("is_public"),
            version: row.get("version"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }
}

// Custom MiniJinja filters
fn markdown_filter(value: String) -> Result<String, minijinja::Error> {
    // Simple markdown to HTML conversion (in production, use a proper markdown parser)
    let html = value
        .replace("\n\n", "</p><p>")
        .replace("**", "<strong>")
        .replace("*", "<em>");
    Ok(format!("<p>{}</p>", html))
}

fn truncate_filter(value: String, length: usize) -> Result<String, minijinja::Error> {
    if value.len() <= length {
        Ok(value)
    } else {
        Ok(format!("{}...", &value[..length]))
    }
}

fn date_filter(value: String, format: Option<String>) -> Result<String, minijinja::Error> {
    let format_str = format.as_deref().unwrap_or("%Y-%m-%d");
    
    if value == "now" {
        let now = chrono::Utc::now();
        Ok(now.format(format_str).to_string())
    } else {
        // Try to parse the date string
        match chrono::DateTime::parse_from_rfc3339(&value) {
            Ok(dt) => Ok(dt.format(format_str).to_string()),
            Err(_) => Ok(value), // Return original if parsing fails
        }
    }
}

// Custom MiniJinja functions
fn asset_url_function(path: String) -> Result<String, minijinja::Error> {
    // In production, this would use your CDN URL
    Ok(format!("https://cdn.quillspace.com/{}", path))
}

fn url_function(path: String) -> Result<String, minijinja::Error> {
    // Generate URLs relative to site root
    if path.starts_with('/') {
        Ok(path)
    } else {
        Ok(format!("/{}", path))
    }
}
