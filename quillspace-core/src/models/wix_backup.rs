use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDate};
use rust_decimal::Decimal;

/// Author record backed up from Wix Authors collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    pub id: Uuid,
    pub tenant_id: Uuid,
    
    // Core author data (matches Wix Authors schema exactly)
    pub name: String, // Primary field in Wix
    pub tagline: Option<String>,
    pub bio: Option<String>, // Rich text field in Wix
    pub portrait_image: Option<String>, // Image field in Wix
    pub slug: Option<String>,
    pub external_id: Option<String>, // External Id field for QuillSpace sync
    
    // Social media fields (URL/Text fields in Wix)
    pub facebook: Option<String>,
    pub x: Option<String>, // Twitter/X field
    pub instagram: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    
    // Sync metadata
    pub wix_id: Option<String>, // The _id from Wix Authors collection
    pub wix_site_id: String,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub sync_status: SyncStatus,
    
    // Standard fields
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Book record backed up from Wix Books collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Book {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub author_id: Option<Uuid>, // Reference to QuillSpace authors table
    
    // Core book data (matches Wix Books schema exactly)
    pub title: String, // Primary field in Wix
    pub sub_title: Option<String>, // "Sub Title" field in Wix
    pub description: Option<String>, // Rich text field in Wix
    pub cover_image: Option<String>, // Image field in Wix
    pub tilted_cover_image: Option<String>, // "Tilted Cover Image" field in Wix
    pub buy_link: Option<String>, // URL field in Wix
    pub status: Option<String>, // Text field in Wix
    pub featured: Option<bool>, // Boolean field in Wix
    pub price: Option<Decimal>, // Number field in Wix
    pub external_id: Option<String>, // External Id field for QuillSpace sync
    
    // Sync metadata
    pub wix_id: Option<String>, // The _id from Wix Books collection
    pub wix_site_id: String,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub sync_status: SyncStatus,
    
    // Standard fields
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Sync operation log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WixSyncLog {
    pub id: Uuid,
    pub tenant_id: Uuid,
    
    // Sync operation details
    pub operation_type: OperationType,
    pub collection_name: CollectionName,
    pub record_id: Uuid, // QuillSpace record ID
    pub wix_record_id: Option<String>, // Wix _id
    pub wix_site_id: String,
    
    // Operation results
    pub status: SyncLogStatus,
    pub error_message: Option<String>,
    
    // Data snapshots (for conflict resolution)
    pub quillspace_data: Option<serde_json::Value>,
    pub wix_data: Option<serde_json::Value>,
    
    // Timestamps
    pub created_at: DateTime<Utc>,
}

/// Sync status for records
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SyncStatus {
    Pending,
    Synced,
    Conflict,
    Error,
}

impl std::fmt::Display for SyncStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncStatus::Pending => write!(f, "pending"),
            SyncStatus::Synced => write!(f, "synced"),
            SyncStatus::Conflict => write!(f, "conflict"),
            SyncStatus::Error => write!(f, "error"),
        }
    }
}

/// Operation types for sync log
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OperationType {
    Create,
    Update,
    Delete,
    Sync,
}

impl std::fmt::Display for OperationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperationType::Create => write!(f, "create"),
            OperationType::Update => write!(f, "update"),
            OperationType::Delete => write!(f, "delete"),
            OperationType::Sync => write!(f, "sync"),
        }
    }
}

/// Collection names for sync operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CollectionName {
    Authors,
    Books,
}

impl std::fmt::Display for CollectionName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CollectionName::Authors => write!(f, "authors"),
            CollectionName::Books => write!(f, "books"),
        }
    }
}

/// Sync log status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SyncLogStatus {
    Success,
    Error,
    Conflict,
}

impl std::fmt::Display for SyncLogStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncLogStatus::Success => write!(f, "success"),
            SyncLogStatus::Error => write!(f, "error"),
            SyncLogStatus::Conflict => write!(f, "conflict"),
        }
    }
}

/// Request/Response DTOs for API endpoints

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAuthorRequest {
    pub name: String,
    pub tagline: Option<String>,
    pub bio: Option<String>,
    pub portrait_image_url: Option<String>,
    pub slug: Option<String>,
    pub twitter_url: Option<String>,
    pub facebook_url: Option<String>,
    pub instagram_url: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateAuthorRequest {
    pub name: Option<String>,
    pub tagline: Option<String>,
    pub bio: Option<String>,
    pub portrait_image_url: Option<String>,
    pub slug: Option<String>,
    pub twitter_url: Option<String>,
    pub facebook_url: Option<String>,
    pub instagram_url: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateBookRequest {
    pub author_id: Option<Uuid>,
    pub title: String,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub cover_image_url: Option<String>,
    pub price: Option<Decimal>,
    pub buy_link: Option<String>,
    pub slug: Option<String>,
    pub published_date: Option<NaiveDate>,
    pub featured: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateBookRequest {
    pub author_id: Option<Uuid>,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub cover_image_url: Option<String>,
    pub price: Option<Decimal>,
    pub buy_link: Option<String>,
    pub slug: Option<String>,
    pub published_date: Option<NaiveDate>,
    pub featured: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncResponse {
    pub success: bool,
    pub message: String,
    pub synced_records: u32,
    pub conflicts: u32,
    pub errors: u32,
}

/// Helper functions for converting between QuillSpace and Wix formats

impl Author {
    /// Convert to Wix Authors collection format
    pub fn to_wix_format(&self) -> serde_json::Value {
        serde_json::json!({
            "data": {
                "name": self.name,
                "tagline": self.tagline,
                "bio": self.bio,
                "portraitImage": self.portrait_image,
                "slug": self.slug,
                "x": self.x,
                "facebook": self.facebook,
                "instagram": self.instagram,
                "email": self.email,
                "phone": self.phone,
                "address": self.address,
                "externalId": self.id.to_string() // QuillSpace ID as External ID
            }
        })
    }
    
    /// Create from Wix Authors collection data
    pub fn from_wix_data(wix_data: &serde_json::Value, tenant_id: Uuid, wix_site_id: String) -> Option<Self> {
        let data = wix_data.get("data")?;
        let wix_id = wix_data.get("id")?.as_str().map(|s| s.to_string());
        
        // Try to get QuillSpace ID from externalId, otherwise generate new
        let id = data.get("externalId")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .unwrap_or_else(Uuid::new_v4);
        
        Some(Author {
            id,
            tenant_id,
            name: data.get("name")?.as_str()?.to_string(),
            tagline: data.get("tagline").and_then(|v| v.as_str()).map(|s| s.to_string()),
            bio: data.get("bio").and_then(|v| v.as_str()).map(|s| s.to_string()),
            portrait_image: data.get("portraitImage").and_then(|v| v.as_str()).map(|s| s.to_string()),
            slug: data.get("slug").and_then(|v| v.as_str()).map(|s| s.to_string()),
            x: data.get("x").and_then(|v| v.as_str()).map(|s| s.to_string()),
            facebook: data.get("facebook").and_then(|v| v.as_str()).map(|s| s.to_string()),
            instagram: data.get("instagram").and_then(|v| v.as_str()).map(|s| s.to_string()),
            email: data.get("email").and_then(|v| v.as_str()).map(|s| s.to_string()),
            phone: data.get("phone").and_then(|v| v.as_str()).map(|s| s.to_string()),
            address: data.get("address").and_then(|v| v.as_str()).map(|s| s.to_string()),
            external_id: data.get("externalId").and_then(|v| v.as_str()).map(|s| s.to_string()),
            wix_id,
            wix_site_id,
            last_synced_at: Some(Utc::now()),
            sync_status: SyncStatus::Synced,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }
}

impl Book {
    /// Convert to Wix Books collection format
    pub fn to_wix_format(&self) -> serde_json::Value {
        serde_json::json!({
            "data": {
                "title": self.title,
                "subTitle": self.sub_title,
                "description": self.description,
                "coverImage": self.cover_image,
                "tiltedCoverImage": self.tilted_cover_image,
                "buyLink": self.buy_link,
                "status": self.status,
                "price": self.price,
                "featured": self.featured,
                "externalId": self.id.to_string() // QuillSpace ID as External ID
                // Note: author field would need to be resolved to Wix Authors._id
            }
        })
    }
    
    /// Create from Wix Books collection data
    pub fn from_wix_data(wix_data: &serde_json::Value, tenant_id: Uuid, wix_site_id: String) -> Option<Self> {
        let data = wix_data.get("data")?;
        let wix_id = wix_data.get("id")?.as_str().map(|s| s.to_string());
        
        // Try to get QuillSpace ID from externalId, otherwise generate new
        let id = data.get("externalId")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .unwrap_or_else(Uuid::new_v4);
        
        Some(Book {
            id,
            tenant_id,
            author_id: None, // Would need to be resolved from Wix author reference
            title: data.get("title")?.as_str()?.to_string(),
            sub_title: data.get("subTitle").and_then(|v| v.as_str()).map(|s| s.to_string()),
            description: data.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()),
            cover_image: data.get("coverImage").and_then(|v| v.as_str()).map(|s| s.to_string()),
            tilted_cover_image: data.get("tiltedCoverImage").and_then(|v| v.as_str()).map(|s| s.to_string()),
            buy_link: data.get("buyLink").and_then(|v| v.as_str()).map(|s| s.to_string()),
            status: data.get("status").and_then(|v| v.as_str()).map(|s| s.to_string()),
            price: data.get("price").and_then(|v| v.as_f64()).and_then(|f| Decimal::try_from(f).ok()),
            featured: data.get("featured").and_then(|v| v.as_bool()),
            external_id: data.get("externalId").and_then(|v| v.as_str()).map(|s| s.to_string()),
            wix_id,
            wix_site_id,
            last_synced_at: Some(Utc::now()),
            sync_status: SyncStatus::Synced,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }
}
