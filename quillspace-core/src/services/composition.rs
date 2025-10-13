use serde::{Deserialize, Serialize};
use serde_json::{Value, Map};
use std::collections::HashMap;
use uuid::Uuid;

/// Puck composition structure from the editor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PuckComposition {
    pub version: Option<u32>,
    pub content: Vec<PuckBlock>,
    pub root: PuckRoot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PuckBlock {
    #[serde(rename = "type")]
    pub block_type: String,
    pub props: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PuckRoot {
    pub props: Map<String, Value>,
}

/// Structured context for MiniJinja rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderContext {
    pub content: Vec<RenderBlock>,
    pub page_title: String,
    pub site_name: String,
    pub current_year: i32,
    pub meta: RenderMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderBlock {
    #[serde(rename = "type")]
    pub block_type: String,
    pub props: Map<String, Value>,
    pub children: Option<Vec<RenderBlock>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderMeta {
    pub description: Option<String>,
    pub keywords: Vec<String>,
    pub og_image: Option<String>,
    pub canonical_url: Option<String>,
}

/// Default values for rendering context
#[derive(Debug, Clone)]
pub struct RenderDefaults {
    pub site_name: String,
    pub default_title: String,
    pub default_description: String,
    pub base_url: String,
}

impl Default for RenderDefaults {
    fn default() -> Self {
        Self {
            site_name: "Author Website".to_string(),
            default_title: "My Page".to_string(),
            default_description: "A beautiful author website".to_string(),
            base_url: "https://example.com".to_string(),
        }
    }
}

/// Transform Puck composition JSON into MiniJinja render context
pub fn composition_to_context(
    composition: &PuckComposition,
    defaults: &RenderDefaults,
    site_slug: Option<&str>,
) -> Result<RenderContext, CompositionError> {
    // Extract page title from root props or use default
    let page_title = composition
        .root
        .props
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or(&defaults.default_title)
        .to_string();

    // Transform blocks
    let content = composition
        .content
        .iter()
        .map(|block| transform_block(block))
        .collect::<Result<Vec<_>, _>>()?;

    // Extract meta information from content
    let meta = extract_meta_from_content(&content, defaults);

    // Build site name (could be from tenant settings)
    let site_name = site_slug
        .map(|slug| slug.replace('-', " ").to_title_case())
        .unwrap_or_else(|| defaults.site_name.clone());

    Ok(RenderContext {
        content,
        page_title,
        site_name,
        current_year: chrono::Utc::now().year(),
        meta,
    })
}

/// Transform a single Puck block into a render block
fn transform_block(block: &PuckBlock) -> Result<RenderBlock, CompositionError> {
    let mut props = block.props.clone();
    
    // Validate and sanitize props based on block type
    match block.block_type.as_str() {
        "HeroBlock" => validate_hero_block(&mut props)?,
        "TextBlock" => validate_text_block(&mut props)?,
        "CardBlock" => validate_card_block(&mut props)?,
        "ImageBlock" => validate_image_block(&mut props)?,
        "ButtonBlock" => validate_button_block(&mut props)?,
        "SectionBlock" => validate_section_block(&mut props)?,
        "GridBlock" => validate_grid_block(&mut props)?,
        _ => {
            // Unknown block type - log warning but allow it
            tracing::warn!("Unknown block type: {}", block.block_type);
        }
    }

    Ok(RenderBlock {
        block_type: block.block_type.clone(),
        props,
        children: None, // TODO: Handle nested blocks if needed
    })
}

/// Extract SEO meta information from content blocks
fn extract_meta_from_content(content: &[RenderBlock], defaults: &RenderDefaults) -> RenderMeta {
    let mut description = None;
    let mut keywords = Vec::new();
    let mut og_image = None;

    for block in content {
        match block.block_type.as_str() {
            "HeroBlock" => {
                if description.is_none() {
                    description = block.props.get("subtitle")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                }
            }
            "TextBlock" => {
                if description.is_none() {
                    description = block.props.get("children")
                        .and_then(|v| v.as_str())
                        .map(|s| truncate_text(s, 160));
                }
            }
            "ImageBlock" => {
                if og_image.is_none() {
                    og_image = block.props.get("src")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                }
            }
            _ => {}
        }
    }

    RenderMeta {
        description: description.or_else(|| Some(defaults.default_description.clone())),
        keywords,
        og_image,
        canonical_url: None, // Set by caller based on site URL
    }
}

/// Validation functions for different block types
fn validate_hero_block(props: &mut Map<String, Value>) -> Result<(), CompositionError> {
    // Ensure required fields have defaults
    if !props.contains_key("title") {
        props.insert("title".to_string(), Value::String("Welcome".to_string()));
    }
    if !props.contains_key("subtitle") {
        props.insert("subtitle".to_string(), Value::String("Your story starts here".to_string()));
    }
    if !props.contains_key("buttonText") {
        props.insert("buttonText".to_string(), Value::String("Learn More".to_string()));
    }
    if !props.contains_key("buttonHref") {
        props.insert("buttonHref".to_string(), Value::String("#".to_string()));
    }
    
    // Sanitize background image URL
    if let Some(bg_img) = props.get_mut("backgroundImage") {
        if let Some(url) = bg_img.as_str() {
            if !is_valid_image_url(url) {
                *bg_img = Value::String("https://images.unsplash.com/photo-1557804506-669a67965ba0?w=1200&h=600&fit=crop".to_string());
            }
        }
    }
    
    Ok(())
}

fn validate_text_block(props: &mut Map<String, Value>) -> Result<(), CompositionError> {
    if !props.contains_key("children") {
        props.insert("children".to_string(), Value::String("Add your text here.".to_string()));
    }
    
    // Sanitize HTML content if needed
    if let Some(content) = props.get_mut("children") {
        if let Some(text) = content.as_str() {
            *content = Value::String(sanitize_html(text));
        }
    }
    
    Ok(())
}

fn validate_card_block(props: &mut Map<String, Value>) -> Result<(), CompositionError> {
    if !props.contains_key("title") {
        props.insert("title".to_string(), Value::String("Card Title".to_string()));
    }
    if !props.contains_key("content") {
        props.insert("content".to_string(), Value::String("Card content goes here.".to_string()));
    }
    if !props.contains_key("imageUrl") {
        props.insert("imageUrl".to_string(), Value::String("https://images.unsplash.com/photo-1557804506-669a67965ba0?w=400&h=200&fit=crop".to_string()));
    }
    
    Ok(())
}

fn validate_image_block(props: &mut Map<String, Value>) -> Result<(), CompositionError> {
    if !props.contains_key("src") {
        props.insert("src".to_string(), Value::String("https://images.unsplash.com/photo-1557804506-669a67965ba0?w=400&h=300&fit=crop".to_string()));
    }
    if !props.contains_key("alt") {
        props.insert("alt".to_string(), Value::String("Image".to_string()));
    }
    
    // Validate image URL
    if let Some(src) = props.get_mut("src") {
        if let Some(url) = src.as_str() {
            if !is_valid_image_url(url) {
                return Err(CompositionError::InvalidImageUrl(url.to_string()));
            }
        }
    }
    
    Ok(())
}

fn validate_button_block(props: &mut Map<String, Value>) -> Result<(), CompositionError> {
    if !props.contains_key("children") {
        props.insert("children".to_string(), Value::String("Click Me".to_string()));
    }
    if !props.contains_key("href") {
        props.insert("href".to_string(), Value::String("#".to_string()));
    }
    if !props.contains_key("variant") {
        props.insert("variant".to_string(), Value::String("primary".to_string()));
    }
    
    Ok(())
}

fn validate_section_block(props: &mut Map<String, Value>) -> Result<(), CompositionError> {
    if !props.contains_key("backgroundColor") {
        props.insert("backgroundColor".to_string(), Value::String("#ffffff".to_string()));
    }
    if !props.contains_key("padding") {
        props.insert("padding".to_string(), Value::Number(serde_json::Number::from(60)));
    }
    
    Ok(())
}

fn validate_grid_block(props: &mut Map<String, Value>) -> Result<(), CompositionError> {
    if !props.contains_key("columns") {
        props.insert("columns".to_string(), Value::Number(serde_json::Number::from(2)));
    }
    
    Ok(())
}

/// Utility functions
fn is_valid_image_url(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://") || url.starts_with("/")
}

fn sanitize_html(content: &str) -> String {
    // Basic HTML sanitization - in production, use a proper library like ammonia
    content
        .replace("<script", "&lt;script")
        .replace("</script>", "&lt;/script&gt;")
        .replace("javascript:", "")
        .replace("data:", "")
}

fn truncate_text(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        format!("{}...", &text[..max_len.saturating_sub(3)])
    }
}

trait ToTitleCase {
    fn to_title_case(&self) -> String;
}

impl ToTitleCase for str {
    fn to_title_case(&self) -> String {
        self.split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Error types for composition transformation
#[derive(Debug, thiserror::Error)]
pub enum CompositionError {
    #[error("Invalid image URL: {0}")]
    InvalidImageUrl(String),
    
    #[error("Missing required field: {0}")]
    MissingRequiredField(String),
    
    #[error("Invalid block type: {0}")]
    InvalidBlockType(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_composition_to_context() {
        let composition = PuckComposition {
            version: Some(1),
            content: vec![
                PuckBlock {
                    block_type: "HeroBlock".to_string(),
                    props: {
                        let mut map = Map::new();
                        map.insert("title".to_string(), json!("Welcome to My Site"));
                        map.insert("subtitle".to_string(), json!("A beautiful author website"));
                        map
                    },
                },
                PuckBlock {
                    block_type: "TextBlock".to_string(),
                    props: {
                        let mut map = Map::new();
                        map.insert("children".to_string(), json!("This is some sample text content."));
                        map
                    },
                },
            ],
            root: PuckRoot {
                props: {
                    let mut map = Map::new();
                    map.insert("title".to_string(), json!("My Page"));
                    map
                },
            },
        };

        let defaults = RenderDefaults::default();
        let context = composition_to_context(&composition, &defaults, Some("test-site")).unwrap();

        assert_eq!(context.page_title, "My Page");
        assert_eq!(context.site_name, "Test Site");
        assert_eq!(context.content.len(), 2);
        assert_eq!(context.content[0].block_type, "HeroBlock");
        assert_eq!(context.content[1].block_type, "TextBlock");
    }

    #[test]
    fn test_validate_hero_block() {
        let mut props = Map::new();
        validate_hero_block(&mut props).unwrap();
        
        assert!(props.contains_key("title"));
        assert!(props.contains_key("subtitle"));
        assert!(props.contains_key("buttonText"));
        assert!(props.contains_key("buttonHref"));
    }
}
