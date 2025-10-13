use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use minijinja::{Environment, Source};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tokio_postgres::Client;

/// Template data from database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub name: String,
    pub version: i32,
    pub display_name: String,
    pub description: Option<String>,
    pub main_name: String,
    pub html_main: String,
    pub html_partials: HashMap<String, String>,
    pub manifest: serde_json::Value,
}

/// Cache key for templates
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct TemplateCacheKey {
    pub tenant_id: Option<Uuid>,
    pub name: String,
    pub version: i32,
}

impl TemplateCacheKey {
    pub fn new(tenant_id: Option<Uuid>, name: String, version: i32) -> Self {
        Self { tenant_id, name, version }
    }
}

/// Cached template with MiniJinja environment
#[derive(Debug, Clone)]
pub struct CachedTemplate {
    pub template: Template,
    pub environment: Environment<'static>,
    pub cached_at: chrono::DateTime<chrono::Utc>,
}

/// In-memory template cache with MiniJinja environments
#[derive(Debug)]
pub struct TemplateCache {
    cache: Arc<RwLock<HashMap<TemplateCacheKey, CachedTemplate>>>,
    db_client: Arc<Client>,
}

impl TemplateCache {
    pub fn new(db_client: Arc<Client>) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            db_client,
        }
    }

    /// Get a template from cache or load from database
    pub async fn get_template(
        &self,
        tenant_id: Option<Uuid>,
        name: &str,
        version: i32,
    ) -> Result<CachedTemplate, TemplateCacheError> {
        let cache_key = TemplateCacheKey::new(tenant_id, name.to_string(), version);

        // Try to get from cache first
        {
            let cache = self.cache.read().map_err(|_| TemplateCacheError::LockError)?;
            if let Some(cached) = cache.get(&cache_key) {
                // Check if cache is still fresh (5 minutes)
                let cache_age = chrono::Utc::now() - cached.cached_at;
                if cache_age.num_minutes() < 5 {
                    return Ok(cached.clone());
                }
            }
        }

        // Load from database
        let template = self.load_template_from_db(tenant_id, name, version).await?;
        let environment = self.create_environment(&template)?;

        let cached_template = CachedTemplate {
            template,
            environment,
            cached_at: chrono::Utc::now(),
        };

        // Update cache
        {
            let mut cache = self.cache.write().map_err(|_| TemplateCacheError::LockError)?;
            cache.insert(cache_key, cached_template.clone());
        }

        Ok(cached_template)
    }

    /// Load template by ID (for pages that reference template_id)
    pub async fn get_template_by_id(&self, template_id: Uuid) -> Result<CachedTemplate, TemplateCacheError> {
        // First check if we have it in cache by ID
        {
            let cache = self.cache.read().map_err(|_| TemplateCacheError::LockError)?;
            for cached in cache.values() {
                if cached.template.id == template_id {
                    let cache_age = chrono::Utc::now() - cached.cached_at;
                    if cache_age.num_minutes() < 5 {
                        return Ok(cached.clone());
                    }
                }
            }
        }

        // Load from database by ID
        let template = self.load_template_by_id_from_db(template_id).await?;
        let environment = self.create_environment(&template)?;

        let cached_template = CachedTemplate {
            template: template.clone(),
            environment,
            cached_at: chrono::Utc::now(),
        };

        // Update cache
        let cache_key = TemplateCacheKey::new(template.tenant_id, template.name.clone(), template.version);
        {
            let mut cache = self.cache.write().map_err(|_| TemplateCacheError::LockError)?;
            cache.insert(cache_key, cached_template.clone());
        }

        Ok(cached_template)
    }

    /// Invalidate cache for a specific template
    pub fn invalidate_template(&self, tenant_id: Option<Uuid>, name: &str, version: i32) -> Result<(), TemplateCacheError> {
        let cache_key = TemplateCacheKey::new(tenant_id, name.to_string(), version);
        let mut cache = self.cache.write().map_err(|_| TemplateCacheError::LockError)?;
        cache.remove(&cache_key);
        Ok(())
    }

    /// Clear entire cache
    pub fn clear_cache(&self) -> Result<(), TemplateCacheError> {
        let mut cache = self.cache.write().map_err(|_| TemplateCacheError::LockError)?;
        cache.clear();
        Ok(())
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> Result<CacheStats, TemplateCacheError> {
        let cache = self.cache.read().map_err(|_| TemplateCacheError::LockError)?;
        Ok(CacheStats {
            total_templates: cache.len(),
            memory_usage_estimate: cache.len() * 1024, // Rough estimate
        })
    }

    /// Load template from database
    async fn load_template_from_db(
        &self,
        tenant_id: Option<Uuid>,
        name: &str,
        version: i32,
    ) -> Result<Template, TemplateCacheError> {
        let query = r#"
            SELECT id, tenant_id, name, version, display_name, description, 
                   main_name, html_main, html_partials, manifest
            FROM templates 
            WHERE (tenant_id IS NULL OR tenant_id = $1) 
              AND name = $2 
              AND version = $3 
              AND is_active = true
            ORDER BY tenant_id NULLS LAST
            LIMIT 1
        "#;

        let row = self.db_client
            .query_opt(query, &[&tenant_id, &name, &version])
            .await
            .map_err(TemplateCacheError::DatabaseError)?
            .ok_or_else(|| TemplateCacheError::TemplateNotFound(name.to_string(), version))?;

        let html_partials: serde_json::Value = row.get("html_partials");
        let html_partials: HashMap<String, String> = serde_json::from_value(html_partials)
            .map_err(|e| TemplateCacheError::SerializationError(e.to_string()))?;

        Ok(Template {
            id: row.get("id"),
            tenant_id: row.get("tenant_id"),
            name: row.get("name"),
            version: row.get("version"),
            display_name: row.get("display_name"),
            description: row.get("description"),
            main_name: row.get("main_name"),
            html_main: row.get("html_main"),
            html_partials,
            manifest: row.get("manifest"),
        })
    }

    /// Load template by ID from database
    async fn load_template_by_id_from_db(&self, template_id: Uuid) -> Result<Template, TemplateCacheError> {
        let query = r#"
            SELECT id, tenant_id, name, version, display_name, description, 
                   main_name, html_main, html_partials, manifest
            FROM templates 
            WHERE id = $1 AND is_active = true
        "#;

        let row = self.db_client
            .query_opt(query, &[&template_id])
            .await
            .map_err(TemplateCacheError::DatabaseError)?
            .ok_or_else(|| TemplateCacheError::TemplateNotFoundById(template_id))?;

        let html_partials: serde_json::Value = row.get("html_partials");
        let html_partials: HashMap<String, String> = serde_json::from_value(html_partials)
            .map_err(|e| TemplateCacheError::SerializationError(e.to_string()))?;

        Ok(Template {
            id: row.get("id"),
            tenant_id: row.get("tenant_id"),
            name: row.get("name"),
            version: row.get("version"),
            display_name: row.get("display_name"),
            description: row.get("description"),
            main_name: row.get("main_name"),
            html_main: row.get("html_main"),
            html_partials,
            manifest: row.get("manifest"),
        })
    }

    /// Create MiniJinja environment with template and partials
    fn create_environment(&self, template: &Template) -> Result<Environment<'static>, TemplateCacheError> {
        let mut env = Environment::new();
        
        // Create source with main template and partials
        let mut source = Source::new();
        
        // Add main template
        source.add_template(&template.main_name, &template.html_main)
            .map_err(|e| TemplateCacheError::TemplateError(e.to_string()))?;
        
        // Add partials
        for (name, content) in &template.html_partials {
            source.add_template(name, content)
                .map_err(|e| TemplateCacheError::TemplateError(e.to_string()))?;
        }
        
        env.set_source(source);
        
        // Configure environment
        env.set_auto_escape_callback(|name| {
            // Auto-escape HTML files
            name.ends_with(".html") || name.ends_with(".htm")
        });
        
        // Add custom filters if needed
        env.add_filter("truncate", truncate_filter);
        env.add_filter("slugify", slugify_filter);
        
        Ok(env)
    }
}

/// Custom MiniJinja filters
fn truncate_filter(value: String, length: usize) -> String {
    if value.len() <= length {
        value
    } else {
        format!("{}...", &value[..length.saturating_sub(3)])
    }
}

fn slugify_filter(value: String) -> String {
    value
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Cache statistics
#[derive(Debug, Serialize)]
pub struct CacheStats {
    pub total_templates: usize,
    pub memory_usage_estimate: usize,
}

/// Template cache errors
#[derive(Debug, thiserror::Error)]
pub enum TemplateCacheError {
    #[error("Template not found: {0} version {1}")]
    TemplateNotFound(String, i32),
    
    #[error("Template not found by ID: {0}")]
    TemplateNotFoundById(Uuid),
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] tokio_postgres::Error),
    
    #[error("Template error: {0}")]
    TemplateError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Cache lock error")]
    LockError,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_creation() {
        let key = TemplateCacheKey::new(
            Some(Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap()),
            "literary-classic".to_string(),
            1,
        );
        
        assert_eq!(key.name, "literary-classic");
        assert_eq!(key.version, 1);
        assert!(key.tenant_id.is_some());
    }

    #[test]
    fn test_truncate_filter() {
        assert_eq!(truncate_filter("Hello World".to_string(), 5), "He...");
        assert_eq!(truncate_filter("Hi".to_string(), 5), "Hi");
    }

    #[test]
    fn test_slugify_filter() {
        assert_eq!(slugify_filter("Hello World!".to_string()), "hello-world");
        assert_eq!(slugify_filter("Test-123".to_string()), "test-123");
    }
}
