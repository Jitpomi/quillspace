use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::database::DatabaseConnections;
use crate::services::wix_api::WixApiClient;
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct WixSite {
    pub id: String,
    pub display_name: String,
    pub view_url: Option<String>,
    pub edit_url: Option<String>,
    pub thumbnail: Option<String>,
    pub owner_account_id: Option<String>,
    pub published: Option<bool>,
    pub premium: Option<bool>,
    pub created_date: Option<String>,
    pub updated_date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectedWebsite {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub builder_type: BuilderType,
    pub external_site_id: String,
    pub name: String,
    pub url: Option<String>,
    pub domain: Option<String>,
    pub status: ConnectionStatus,
    pub last_sync: Option<DateTime<Utc>>,
    pub sync_error: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BuilderType {
    Wix,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionStatus {
    Active,
    Inactive,
    Error,
}

pub struct ConnectedWebsitesService {
    db: DatabaseConnections,
}

impl ConnectedWebsitesService {
    pub fn new(db: DatabaseConnections) -> Self {
        Self { db }
    }

    /// Get Wix books for a specific site
    pub async fn get_wix_books(&self, site_id: &str) -> Result<serde_json::Value> {
        let api_key = std::env::var("QUILLSPACE_WIX_API_KEY")
            .map_err(|_| anyhow::anyhow!("QUILLSPACE_WIX_API_KEY not configured"))?;
        let account_id = std::env::var("QUILLSPACE_WIX_ACCOUNT_ID")
            .map_err(|_| anyhow::anyhow!("QUILLSPACE_WIX_ACCOUNT_ID not configured"))?;

        let client = WixApiClient::new(api_key, account_id);
        client.get_collection_items(site_id, "Books").await
    }

    /// Create a new book in Wix site
    pub async fn create_wix_book(&self, site_id: &str, book_data: serde_json::Value) -> Result<serde_json::Value> {
        let api_key = std::env::var("QUILLSPACE_WIX_API_KEY")
            .map_err(|_| anyhow::anyhow!("QUILLSPACE_WIX_API_KEY not configured"))?;
        let account_id = std::env::var("QUILLSPACE_WIX_ACCOUNT_ID")
            .map_err(|_| anyhow::anyhow!("QUILLSPACE_WIX_ACCOUNT_ID not configured"))?;

        let client = WixApiClient::new(api_key, account_id);
        client.insert_collection_item(site_id, "Books", book_data).await
    }

    /// Update a book in Wix site
    pub async fn update_wix_book(&self, site_id: &str, book_id: &str, book_data: serde_json::Value) -> Result<serde_json::Value> {
        let api_key = std::env::var("QUILLSPACE_WIX_API_KEY")
            .map_err(|_| anyhow::anyhow!("QUILLSPACE_WIX_API_KEY not configured"))?;
        let account_id = std::env::var("QUILLSPACE_WIX_ACCOUNT_ID")
            .map_err(|_| anyhow::anyhow!("QUILLSPACE_WIX_ACCOUNT_ID not configured"))?;

        let client = WixApiClient::new(api_key, account_id);
        client.update_collection_item(site_id, "Books", book_id, book_data).await
    }

    /// Get author info from Wix site
    pub async fn get_wix_author_info(&self, site_id: &str) -> Result<serde_json::Value> {
        let api_key = std::env::var("QUILLSPACE_WIX_API_KEY")
            .map_err(|_| anyhow::anyhow!("QUILLSPACE_WIX_API_KEY not configured"))?;
        let account_id = std::env::var("QUILLSPACE_WIX_ACCOUNT_ID")
            .map_err(|_| anyhow::anyhow!("QUILLSPACE_WIX_ACCOUNT_ID not configured"))?;

        let client = WixApiClient::new(api_key, account_id);
        client.get_collection_items(site_id, "AuthorInfo").await
    }

    /// Update author info in Wix site
    pub async fn update_wix_author_info(&self, site_id: &str, author_data: serde_json::Value) -> Result<serde_json::Value> {
        let api_key = std::env::var("QUILLSPACE_WIX_API_KEY")
            .map_err(|_| anyhow::anyhow!("QUILLSPACE_WIX_API_KEY not configured"))?;
        let account_id = std::env::var("QUILLSPACE_WIX_ACCOUNT_ID")
            .map_err(|_| anyhow::anyhow!("QUILLSPACE_WIX_ACCOUNT_ID not configured"))?;

        let client = WixApiClient::new(api_key, account_id);
        
        // Get existing AuthorInfo to update it
        match client.get_collection_items(site_id, "AuthorInfo").await {
            Ok(existing_data) => {
                if let Some(items) = existing_data.get("dataItems").and_then(|v| v.as_array()) {
                    if let Some(first_item) = items.first() {
                        if let Some(item_id) = first_item.get("id").and_then(|v| v.as_str()) {
                            return client.update_collection_item(site_id, "AuthorInfo", item_id, author_data).await;
                        }
                    }
                }
                // No existing author info, create new one
                client.insert_collection_item(site_id, "AuthorInfo", author_data).await
            }
            Err(_) => {
                // Create new if can't get existing
                client.insert_collection_item(site_id, "AuthorInfo", author_data).await
            }
        }
    }

    /// Get websites from Wix Sites API for the authenticated user
    pub async fn get_user_websites(&self, user_id: Uuid) -> Result<Vec<ConnectedWebsite>> {
        // For now, we'll fetch from Wix API directly
        // In the future, we can store user's connected sites in the database
        let api_key = std::env::var("QUILLSPACE_WIX_API_KEY")
            .map_err(|_| anyhow::anyhow!("QUILLSPACE_WIX_API_KEY not configured"))?;
        let account_id = std::env::var("QUILLSPACE_WIX_ACCOUNT_ID")
            .map_err(|_| anyhow::anyhow!("QUILLSPACE_WIX_ACCOUNT_ID not configured"))?;

        let client = WixApiClient::new(api_key, account_id);
        
        // For demo purposes, we'll show the known site. In production, you'd:
        // 1. Store user's connected site IDs in database
        // 2. Or use OAuth to fetch sites they own
        // 3. Or allow users to add sites manually
        let demo_site_ids = vec!["1e4e0091-f4d5-4a4c-a66a-4d09e7a5b4e9".to_string()];
        
        match client.query_sites(Some(demo_site_ids)).await {
            Ok(response) => {
                let mut websites = Vec::new();
                
                if let Some(sites) = response.get("sites").and_then(|s| s.as_array()) {
                    for site in sites {
                        let wix_site = self.parse_wix_site(site)?;
                        let connected_website = self.wix_site_to_connected_website(wix_site, user_id);
                        websites.push(connected_website);
                    }
                }
                
                Ok(websites)
            }
            Err(e) => {
                tracing::error!("Failed to fetch sites from Wix API: {}", e);
                // Return empty list instead of error to avoid breaking the UI
                Ok(vec![])
            }
        }
    }
    
    /// Parse Wix site JSON into WixSite struct
    fn parse_wix_site(&self, site_json: &serde_json::Value) -> Result<WixSite> {
        Ok(WixSite {
            id: site_json.get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            display_name: site_json.get("displayName")
                .and_then(|v| v.as_str())
                .unwrap_or("Untitled Site")
                .to_string(),
            view_url: site_json.get("viewUrl")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            edit_url: site_json.get("editUrl")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            thumbnail: site_json.get("thumbnail")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            owner_account_id: site_json.get("ownerAccountId")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            published: site_json.get("published")
                .and_then(|v| v.as_bool()),
            premium: site_json.get("premium")
                .and_then(|v| v.as_bool()),
            created_date: site_json.get("createdDate")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            updated_date: site_json.get("updatedDate")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        })
    }
    
    /// Convert WixSite to ConnectedWebsite
    fn wix_site_to_connected_website(&self, wix_site: WixSite, user_id: Uuid) -> ConnectedWebsite {
        ConnectedWebsite {
            id: Uuid::new_v4(), // Generate a new UUID for the connected website
            tenant_id: user_id, // Use user_id as tenant_id for now
            user_id,
            builder_type: BuilderType::Wix,
            external_site_id: wix_site.id.clone(),
            name: wix_site.display_name.clone(),
            url: wix_site.view_url.clone(),
            domain: None, // Could extract from view_url if needed
            status: if wix_site.published.unwrap_or(false) {
                ConnectionStatus::Active
            } else {
                ConnectionStatus::Inactive
            },
            last_sync: Some(Utc::now()),
            sync_error: None,
            metadata: serde_json::json!({
                "wix_site_id": wix_site.id,
                "display_name": wix_site.display_name,
                "view_url": wix_site.view_url,
                "edit_url": wix_site.edit_url,
                "thumbnail": wix_site.thumbnail,
                "published": wix_site.published,
                "premium": wix_site.premium,
                "created_date": wix_site.created_date,
                "updated_date": wix_site.updated_date,
                "owner_account_id": wix_site.owner_account_id,
                "fetched_from_api": true
            }),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }


}
