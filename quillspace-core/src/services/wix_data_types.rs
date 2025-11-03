use serde_json::{Value, Map};
use chrono::{DateTime, Utc};
use anyhow::{Result, anyhow};
use uuid::Uuid;

/// Wix Data Type Validator and Formatter
/// Ensures all data sent to Wix conforms to their expected data type formats
/// Reference: https://dev.wix.com/docs/api-reference/business-solutions/cms/data-types-in-wix-data
pub struct WixDataTypeValidator;

impl WixDataTypeValidator {
    /// Validate and format data according to Wix data type requirements
    /// Based on the new schema: Books collection with proper field types
    pub fn validate_and_format_book_data(data: &Value) -> Result<Value> {
        let mut formatted_data = Map::new();
        
        if let Some(obj) = data.as_object() {
            for (key, value) in obj {
                match key.as_str() {
                    // Text fields - match exact Wix field names
                    "title" | "subTitle" | "buyLink" | "status" | "externalId" => {
                        formatted_data.insert(key.clone(), Self::format_text_field(value)?);
                    },
                    
                    // Rich Content fields (description is Rich Content in Wix)
                    "description" => {
                        formatted_data.insert(key.clone(), Self::format_rich_text_field(value)?);
                    },
                    
                    // URL fields (images are URLs, not media objects)
                    "coverImage" | "tiltedCoverImage" => {
                        formatted_data.insert(key.clone(), Self::format_url_field(value)?);
                    },
                    
                    // Number fields
                    "price" => {
                        formatted_data.insert(key.clone(), Self::format_number_field(value)?);
                    },
                    
                    // Reference fields (Books.author → Authors) - resolve name to _id
                    "author" => {
                        formatted_data.insert(key.clone(), Self::format_author_reference_field_sync(value)?);
                    },
                    
                    // DateTime fields
                    "publishedDate" => {
                        formatted_data.insert(key.clone(), Self::format_date_field(value)?);
                    },
                    
                    // Boolean fields
                    "featured" => {
                        formatted_data.insert(key.clone(), Self::format_boolean_field(value)?);
                    },
                    
                    // Number fields - ensure they are numbers
                    "pages" | "rating" => {
                        formatted_data.insert(key.clone(), Self::format_number_field(value)?);
                    },
                    
                    // Date fields - ensure they are in Wix date format
                    "publishedDate" | "createdDate" | "updatedDate" => {
                        formatted_data.insert(key.clone(), Self::format_date_field(value)?);
                    },
                    
                    // Boolean fields - ensure they are booleans
                    "featured" | "available" | "bestseller" => {
                        formatted_data.insert(key.clone(), Self::format_boolean_field(value)?);
                    },
                    
                    // Media fields - ensure they are in Wix media format
                    "coverImage" | "authorPhoto" | "bookCover" => {
                        formatted_data.insert(key.clone(), Self::format_media_field(value)?);
                    },
                    
                    // URL fields - ensure they are valid URLs
                    "previewUrl" | "purchaseUrl" | "websiteUrl" => {
                        formatted_data.insert(key.clone(), Self::format_url_field(value)?);
                    },
                    
                    // Address fields - ensure they are in Wix address format
                    "publisherAddress" | "authorLocation" => {
                        formatted_data.insert(key.clone(), Self::format_address_field(value)?);
                    },
                    
                    // Time fields - ensure they are in Wix time format
                    "readingTime" | "eventTime" => {
                        formatted_data.insert(key.clone(), Self::format_time_field(value)?);
                    },
                    
                    // Reference fields - ensure they reference valid items
                    "authorRef" | "seriesRef" | "publisherRef" => {
                        formatted_data.insert(key.clone(), Self::format_reference_field(value)?);
                    },
                    
                    // Object/JSON fields - ensure they are valid JSON objects
                    "metadata" | "settings" | "customData" => {
                        formatted_data.insert(key.clone(), Self::format_object_field(value)?);
                    },
                    
                    // Rich text fields - ensure they are in Wix rich text format
                    "summary" | "excerpt" => {
                        formatted_data.insert(key.clone(), Self::format_rich_text_field(value)?);
                    },
                    
                    // Array fields - ensure they are arrays
                    "tags" | "categories" | "keywords" => {
                        formatted_data.insert(key.clone(), Self::format_array_field(value)?);
                    },
                    
                    // Pass through system fields unchanged
                    "_id" | "_createdDate" | "_updatedDate" | "_owner" => {
                        formatted_data.insert(key.clone(), value.clone());
                    },
                    
                    // Unknown fields - validate as text by default
                    _ => {
                        tracing::warn!("Unknown field '{}' in book data, treating as text", key);
                        formatted_data.insert(key.clone(), Self::format_text_field(value)?);
                    }
                }
            }
        }
        
        Ok(Value::Object(formatted_data))
    }
    
    /// Format text fields according to Wix requirements
    fn format_text_field(value: &Value) -> Result<Value> {
        match value {
            Value::String(s) => {
                // Ensure text is not empty and within reasonable limits
                if s.is_empty() {
                    return Err(anyhow!("Text field cannot be empty"));
                }
                if s.len() > 10000 {
                    return Err(anyhow!("Text field too long (max 10000 characters)"));
                }
                Ok(Value::String(s.clone()))
            },
            Value::Number(n) => Ok(Value::String(n.to_string())),
            Value::Bool(b) => Ok(Value::String(b.to_string())),
            _ => Err(anyhow!("Invalid value for text field: {:?}", value))
        }
    }
    
    /// Format number fields according to Wix requirements
    fn format_number_field(value: &Value) -> Result<Value> {
        match value {
            Value::Number(n) => Ok(Value::Number(n.clone())),
            Value::String(s) => {
                if let Ok(num) = s.parse::<f64>() {
                    Ok(serde_json::Number::from_f64(num)
                        .map(Value::Number)
                        .unwrap_or(Value::Null))
                } else {
                    Err(anyhow!("Invalid number format: {}", s))
                }
            },
            _ => Err(anyhow!("Invalid value for number field: {:?}", value))
        }
    }
    
    /// Format date fields according to Wix requirements
    /// Wix expects: {"$date": "YYYY-MM-DDTHH:mm:ss.sssZ"}
    fn format_date_field(value: &Value) -> Result<Value> {
        match value {
            Value::String(s) => {
                // Try to parse as ISO 8601 date
                if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
                    Ok(serde_json::json!({
                        "$date": dt.to_utc().to_rfc3339()
                    }))
                } else if let Ok(dt) = s.parse::<DateTime<Utc>>() {
                    Ok(serde_json::json!({
                        "$date": dt.to_rfc3339()
                    }))
                } else {
                    Err(anyhow!("Invalid date format: {}", s))
                }
            },
            Value::Object(obj) if obj.contains_key("$date") => {
                // Already in Wix format
                Ok(value.clone())
            },
            _ => {
                // Default to current date if no valid date provided
                let now = Utc::now();
                Ok(serde_json::json!({
                    "$date": now.to_rfc3339()
                }))
            }
        }
    }
    
    /// Format boolean fields according to Wix requirements
    fn format_boolean_field(value: &Value) -> Result<Value> {
        match value {
            Value::Bool(b) => Ok(Value::Bool(*b)),
            Value::String(s) => {
                match s.to_lowercase().as_str() {
                    "true" | "1" | "yes" | "on" => Ok(Value::Bool(true)),
                    "false" | "0" | "no" | "off" => Ok(Value::Bool(false)),
                    _ => Err(anyhow!("Invalid boolean format: {}", s))
                }
            },
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(Value::Bool(i != 0))
                } else {
                    Ok(Value::Bool(n.as_f64().unwrap_or(0.0) != 0.0))
                }
            },
            _ => Err(anyhow!("Invalid value for boolean field: {:?}", value))
        }
    }
    
    /// Format URL fields according to Wix requirements
    fn format_url_field(value: &Value) -> Result<Value> {
        match value {
            Value::String(s) => {
                if s.is_empty() {
                    return Ok(Value::Null);
                }
                
                // Basic URL validation
                if s.starts_with("http://") || s.starts_with("https://") || s.starts_with("wix:") {
                    Ok(Value::String(s.clone()))
                } else {
                    // Assume it's a relative URL and make it absolute
                    Ok(Value::String(format!("https://{}", s)))
                }
            },
            Value::Null => Ok(Value::Null),
            _ => Err(anyhow!("Invalid value for URL field: {:?}", value))
        }
    }
    
    /// Format rich text fields according to Wix Ricos Document requirements
    fn format_rich_text_field(value: &Value) -> Result<Value> {
        match value {
            Value::String(s) => {
                // Try to parse as JSON first (from frontend rich text editor)
                if let Ok(parsed) = serde_json::from_str::<Value>(s) {
                    if parsed.is_object() {
                        // Already in Wix Rich Text format
                        return Ok(parsed);
                    }
                }
                // Otherwise, convert HTML/plain text to Wix Rich Text format
                Self::html_to_wix_rich_text(s)
            },
            Value::Object(_) => {
                // Already in Ricos format
                Ok(value.clone())
            },
            _ => Err(anyhow!("Invalid value for rich text field: {:?}", value))
        }
    }
    
    /// Convert HTML string to Wix Rich Text format
    fn html_to_wix_rich_text(html: &str) -> Result<Value> {
        // For now, create a simple paragraph with the HTML content
        // This preserves the HTML structure while putting it in Wix format
        let mut nodes = Vec::new();
        let mut node_id = 1;
        
        // Split by paragraph tags
        let paragraphs: Vec<&str> = html.split("</p>").collect();
        
        for (i, paragraph) in paragraphs.iter().enumerate() {
            if paragraph.trim().is_empty() {
                continue;
            }
            
            // Remove opening <p> tag and any class attributes
            let clean_text = paragraph
                .trim_start_matches("<p>")
                .trim_start_matches(|c: char| c != '>' && c != '<')
                .trim_start_matches('>');
            
            if !clean_text.trim().is_empty() {
                nodes.push(serde_json::json!({
                    "type": "PARAGRAPH",
                    "id": node_id.to_string(),
                    "nodes": [{
                        "type": "TEXT", 
                        "id": (node_id + 1).to_string(),
                        "textData": {
                            "text": clean_text.trim(),
                            "decorations": []
                        }
                    }]
                }));
                node_id += 2;
            }
        }
        
        // If no paragraphs found, treat as plain text
        if nodes.is_empty() {
            nodes.push(serde_json::json!({
                "type": "PARAGRAPH",
                "id": "1",
                "nodes": [{
                    "type": "TEXT",
                    "id": "2", 
                    "textData": {
                        "text": html,
                        "decorations": []
                    }
                }]
            }));
        }
        
        Ok(serde_json::json!({
            "nodes": nodes,
            "documentStyle": {}
        }))
    }
    
    /// Format array fields according to Wix requirements
    fn format_array_field(value: &Value) -> Result<Value> {
        match value {
            Value::Array(arr) => Ok(Value::Array(arr.clone())),
            Value::String(s) => {
                // Split comma-separated string into array
                let items: Vec<Value> = s.split(',')
                    .map(|item| Value::String(item.trim().to_string()))
                    .collect();
                Ok(Value::Array(items))
            },
            _ => Err(anyhow!("Invalid value for array field: {:?}", value))
        }
    }
    
    /// Format media fields according to Wix requirements
    /// Wix media format: {"id": "string", "url": "string", "filename": "string", etc.}
    fn format_media_field(value: &Value) -> Result<Value> {
        match value {
            Value::String(s) => {
                if s.is_empty() {
                    return Ok(Value::Null);
                }
                
                // If it's a URL, create a media object
                if s.starts_with("http://") || s.starts_with("https://") || s.starts_with("wix:") {
                    Ok(serde_json::json!({
                        "id": format!("media_{}", Uuid::new_v4()),
                        "url": s,
                        "filename": s.split('/').last().unwrap_or("unknown"),
                        "mediaType": "image"
                    }))
                } else {
                    // Assume it's already a media ID
                    Ok(Value::String(s.clone()))
                }
            },
            Value::Object(_) => {
                // Already in media format
                Ok(value.clone())
            },
            Value::Null => Ok(Value::Null),
            _ => Err(anyhow!("Invalid value for media field: {:?}", value))
        }
    }
    
    /// Format address fields according to Wix requirements
    /// Wix address format: {"formatted": "string", "location": {...}, etc.}
    fn format_address_field(value: &Value) -> Result<Value> {
        match value {
            Value::String(s) => {
                if s.is_empty() {
                    return Ok(Value::Null);
                }
                
                // Convert string address to Wix address format
                Ok(serde_json::json!({
                    "formatted": s,
                    "location": {
                        "latitude": 0.0,
                        "longitude": 0.0
                    },
                    "streetAddress": {
                        "name": s
                    },
                    "addressLine": s
                }))
            },
            Value::Object(_) => {
                // Already in address format
                Ok(value.clone())
            },
            Value::Null => Ok(Value::Null),
            _ => Err(anyhow!("Invalid value for address field: {:?}", value))
        }
    }
    
    /// Format time fields according to Wix requirements
    /// Wix time format: "hh:mm:ss.SSS"
    fn format_time_field(value: &Value) -> Result<Value> {
        match value {
            Value::String(s) => {
                // Validate and format time string
                if s.is_empty() {
                    return Ok(Value::Null);
                }
                
                // Try to parse various time formats
                if s.contains(':') {
                    // Already in time format, validate it
                    let parts: Vec<&str> = s.split(':').collect();
                    if parts.len() >= 2 {
                        let hours: u32 = parts[0].parse().map_err(|_| anyhow!("Invalid hours: {}", parts[0]))?;
                        let minutes: u32 = parts[1].parse().map_err(|_| anyhow!("Invalid minutes: {}", parts[1]))?;
                        
                        if hours > 23 || minutes > 59 {
                            return Err(anyhow!("Invalid time values: {}:{}", hours, minutes));
                        }
                        
                        let seconds = if parts.len() > 2 { 
                            parts[2].parse::<f32>().unwrap_or(0.0) 
                        } else { 
                            0.0 
                        };
                        
                        Ok(Value::String(format!("{:02}:{:02}:{:06.3}", hours, minutes, seconds)))
                    } else {
                        Err(anyhow!("Invalid time format: {}", s))
                    }
                } else {
                    // Try to parse as number of minutes or seconds
                    if let Ok(minutes) = s.parse::<u32>() {
                        let hours = minutes / 60;
                        let mins = minutes % 60;
                        Ok(Value::String(format!("{:02}:{:02}:00.000", hours, mins)))
                    } else {
                        Err(anyhow!("Invalid time format: {}", s))
                    }
                }
            },
            Value::Number(n) => {
                // Assume it's minutes
                if let Some(minutes) = n.as_u64() {
                    let hours = minutes / 60;
                    let mins = minutes % 60;
                    Ok(Value::String(format!("{:02}:{:02}:00.000", hours, mins)))
                } else {
                    Err(anyhow!("Invalid number for time field: {:?}", n))
                }
            },
            Value::Null => Ok(Value::Null),
            _ => Err(anyhow!("Invalid value for time field: {:?}", value))
        }
    }
    
    /// Format reference fields according to Wix requirements
    /// Wix reference format: "collection_id/item_id" or {"id": "item_id", "collection": "collection_id"}
    fn format_reference_field(value: &Value) -> Result<Value> {
        match value {
            Value::String(s) => {
                if s.is_empty() {
                    return Ok(Value::Null);
                }
                
                // If it contains a slash, assume it's in "collection/id" format
                if s.contains('/') {
                    Ok(Value::String(s.clone()))
                } else {
                    // Assume it's just an ID, need to determine collection
                    // For now, return as-is and let Wix handle it
                    Ok(Value::String(s.clone()))
                }
            },
            Value::Object(_) => {
                // Already in reference format
                Ok(value.clone())
            },
            Value::Null => Ok(Value::Null),
            _ => Err(anyhow!("Invalid value for reference field: {:?}", value))
        }
    }
    
    /// Format object/JSON fields according to Wix requirements
    fn format_object_field(value: &Value) -> Result<Value> {
        match value {
            Value::Object(_) => Ok(value.clone()),
            Value::String(s) => {
                if s.is_empty() {
                    return Ok(serde_json::json!({}));
                }
                
                // Try to parse as JSON
                match serde_json::from_str::<Value>(s) {
                    Ok(parsed) => Ok(parsed),
                    Err(_) => {
                        // If not valid JSON, wrap in an object
                        Ok(serde_json::json!({"value": s}))
                    }
                }
            },
            Value::Null => Ok(serde_json::json!({})),
            _ => {
                // Wrap other types in an object
                Ok(serde_json::json!({"value": value}))
            }
        }
    }
    
    /// Format author field - handle both Object and Text types based on Wix schema
    fn format_author_field(value: &Value) -> Result<Value> {
        match value {
            Value::String(s) => {
                if s.is_empty() {
                    return Err(anyhow!("Author field cannot be empty"));
                }
                
                // Based on your Wix collection schema showing "Object" type for author,
                // we need to wrap the string in an object structure
                Ok(serde_json::json!({
                    "name": s,
                    "displayName": s
                }))
            },
            Value::Object(_) => {
                // Already in object format
                Ok(value.clone())
            },
            _ => Err(anyhow!("Invalid value for author field: {:?}", value))
        }
    }
    
    /// Format author reference field - resolve author name to Wix _id (synchronous version)
    /// Since name is the Primary Key, we map known names to their _ids
    fn format_author_reference_field_sync(value: &Value) -> Result<Value> {
        match value {
            Value::String(s) => {
                if s.is_empty() {
                    return Err(anyhow!("Author field cannot be empty"));
                }
                
                // Check if it's already a Wix _id (UUID format with dashes)
                if s.len() > 30 && s.contains('-') && !s.parse::<uuid::Uuid>().is_ok() {
                    tracing::info!("Author field appears to be Wix _id, passing through: {}", s);
                    return Ok(Value::String(s.clone()));
                }
                
                // Map known author names to their Wix _ids
                let wix_id = match s.as_str() {
                    "Yasin Kakande" => "31ac11d6-ecb6-4348-a3b4-125cb1965d39", // Yasin's actual Wix _id
                    _ => {
                        tracing::warn!("Unknown author name '{}' - using name as fallback", s);
                        // For unknown authors, return the name and let Wix handle it
                        // This will work if the author exists in Wix with that exact name
                        return Ok(Value::String(s.clone()));
                    }
                };
                
                tracing::info!("Resolved author '{}' to Wix _id: {}", s, wix_id);
                Ok(Value::String(wix_id.to_string()))
            },
            _ => Err(anyhow!("Invalid value for author reference field: {:?}", value))
        }
    }
    
    /// Format author reference field - resolve author name to Wix _id (async version)
    /// Since name is the Primary Key, we query by name to get the _id
    pub async fn format_author_reference_field(value: &Value, wix_client: &crate::services::wix_api::WixApiClient, site_id: &str) -> Result<Value> {
        match value {
            Value::String(s) => {
                if s.is_empty() {
                    return Err(anyhow!("Author field cannot be empty"));
                }
                
                // Check if it's already a Wix _id (UUID format with dashes)
                if s.len() > 30 && s.contains('-') && !s.parse::<uuid::Uuid>().is_ok() {
                    tracing::info!("Author field appears to be Wix _id, passing through: {}", s);
                    return Ok(Value::String(s.clone()));
                }
                
                // It's a name (Primary Key), look up by name to get _id
                tracing::info!("Author field is name (PK), looking up by name: {}", s);
                Self::lookup_author_by_name(wix_client, site_id, s).await
            },
            _ => Err(anyhow!("Invalid value for author reference field: {:?}", value))
        }
    }
    
    /// Look up author by External Id in Wix Authors collection
    async fn lookup_author_by_external_id(wix_client: &crate::services::wix_api::WixApiClient, site_id: &str, external_id: &str) -> Result<Value> {
        // Query Authors collection by externalId
        let query_body = serde_json::json!({
            "filter": {
                "externalId": {
                    "$eq": external_id
                }
            },
            "paging": {
                "limit": 1
            }
        });
        
        match wix_client.query_collection_items_with_filter(site_id, "Authors", query_body).await {
            Ok(response) => {
                if let Some(items) = response.get("dataItems").and_then(|v| v.as_array()) {
                    if let Some(author) = items.first() {
                        if let Some(wix_id) = author.get("id").and_then(|v| v.as_str()) {
                            tracing::info!("Found author by externalId '{}' -> Wix _id: {}", external_id, wix_id);
                            return Ok(Value::String(wix_id.to_string()));
                        }
                    }
                }
                Err(anyhow!("Author with externalId '{}' not found", external_id))
            },
            Err(e) => {
                tracing::error!("Failed to query Authors by externalId: {}", e);
                Err(anyhow!("Failed to lookup author by externalId: {}", e))
            }
        }
    }
    
    /// Look up author by name in Wix Authors collection
    async fn lookup_author_by_name(wix_client: &crate::services::wix_api::WixApiClient, site_id: &str, name: &str) -> Result<Value> {
        // Query Authors collection by name
        let query_body = serde_json::json!({
            "filter": {
                "name": {
                    "$eq": name
                }
            },
            "paging": {
                "limit": 1
            }
        });
        
        match wix_client.query_collection_items_with_filter(site_id, "Authors", query_body).await {
            Ok(response) => {
                if let Some(items) = response.get("dataItems").and_then(|v| v.as_array()) {
                    if let Some(author) = items.first() {
                        if let Some(wix_id) = author.get("id").and_then(|v| v.as_str()) {
                            tracing::info!("Found author by name '{}' -> Wix _id: {}", name, wix_id);
                            return Ok(Value::String(wix_id.to_string()));
                        }
                    }
                }
                Err(anyhow!("Author with name '{}' not found", name))
            },
            Err(e) => {
                tracing::error!("Failed to query Authors by name: {}", e);
                Err(anyhow!("Failed to lookup author by name: {}", e))
            }
        }
    }
    
    /// Validate Authors collection data based on the new schema
    /// Authors schema: name (text), tagline (text), bio (rich text), portraitImage (image), 
    /// twitter/facebook/instagram (text/url), slug (text, unique)
    pub fn validate_and_format_author_data(data: &Value) -> Result<Value> {
        let mut formatted_data = Map::new();
        
        if let Some(obj) = data.as_object() {
            for (key, value) in obj {
                match key.as_str() {
                    // Text fields
                    "name" | "tagline" | "slug" | "externalId" => {
                        formatted_data.insert(key.clone(), Self::format_text_field(value)?);
                    },
                    
                    // Bio field - allow HTML and empty values
                    "bio" => {
                        match value {
                            Value::String(s) => {
                                // Allow empty bio and HTML content, just validate length
                                if s.len() > 50000 {  // Larger limit for bio with HTML
                                    return Err(anyhow!("Bio field too long (max 50000 characters)"));
                                }
                                formatted_data.insert(key.clone(), Value::String(s.clone()));
                            },
                            Value::Null => {
                                formatted_data.insert(key.clone(), Value::String("".to_string()));
                            },
                            _ => {
                                formatted_data.insert(key.clone(), Value::String(value.to_string()));
                            }
                        }
                    },
                    
                    // Image fields
                    "portraitImage" => {
                        formatted_data.insert(key.clone(), Self::format_media_field(value)?);
                    },
                    
                    // URL/Text fields (social media) - match exact Wix field names
                    "x" | "facebook" | "instagram" | "email" | "phone" | "address" => {
                        formatted_data.insert(key.clone(), Self::format_text_field(value)?);
                    },
                    
                    // System fields
                    "_id" | "_createdDate" | "_updatedDate" | "_owner" => {
                        formatted_data.insert(key.clone(), value.clone());
                    },
                    
                    // Unknown fields
                    _ => {
                        tracing::warn!("Unknown field '{}' in author data, treating as text", key);
                        formatted_data.insert(key.clone(), Self::format_text_field(value)?);
                    }
                }
            }
        }
        
        Ok(Value::Object(formatted_data))
    }
    
    /// Validate Posts collection data based on the new schema
    /// Posts schema: title (text), body (rich text), coverImage (image), 
    /// date (datetime), author (reference → Authors), slug (text)
    pub fn validate_and_format_post_data(data: &Value) -> Result<Value> {
        let mut formatted_data = Map::new();
        
        if let Some(obj) = data.as_object() {
            for (key, value) in obj {
                match key.as_str() {
                    // Text fields
                    "title" | "slug" => {
                        formatted_data.insert(key.clone(), Self::format_text_field(value)?);
                    },
                    
                    // Rich text fields
                    "body" => {
                        formatted_data.insert(key.clone(), Self::format_rich_text_field(value)?);
                    },
                    
                    // Image fields
                    "coverImage" => {
                        formatted_data.insert(key.clone(), Self::format_media_field(value)?);
                    },
                    
                    // DateTime fields
                    "date" => {
                        formatted_data.insert(key.clone(), Self::format_date_field(value)?);
                    },
                    
                    // Reference fields (Posts.author → Authors)
                    "author" => {
                        formatted_data.insert(key.clone(), Self::format_author_reference_field_sync(value)?);
                    },
                    
                    // System fields
                    "_id" | "_createdDate" | "_updatedDate" | "_owner" => {
                        formatted_data.insert(key.clone(), value.clone());
                    },
                    
                    // Unknown fields
                    _ => {
                        tracing::warn!("Unknown field '{}' in post data, treating as text", key);
                        formatted_data.insert(key.clone(), Self::format_text_field(value)?);
                    }
                }
            }
        }
        
        Ok(Value::Object(formatted_data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_validate_book_data() {
        let input = json!({
            "title": "Test Book",
            "author": "Test Author",
            "price": "29.99",
            "featured": "true",
            "publishedDate": "2023-01-01T00:00:00Z"
        });
        
        let result = WixDataTypeValidator::validate_and_format_book_data(&input).unwrap();
        
        assert_eq!(result["title"], "Test Book");
        assert_eq!(result["author"], "Test Author");
        assert_eq!(result["price"], 29.99);
        assert_eq!(result["featured"], true);
        assert!(result["publishedDate"]["$date"].is_string());
    }
    
    #[test]
    fn test_invalid_data_types() {
        let input = json!({
            "title": "",  // Empty string should fail
            "price": "invalid_number"  // Invalid number should fail
        });
        
        let result = WixDataTypeValidator::validate_and_format_book_data(&input);
        assert!(result.is_err());
    }
}
