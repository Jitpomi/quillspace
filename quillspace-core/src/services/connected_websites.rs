use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::database::DatabaseConnections;
use crate::services::wix_api::WixApiClient;
use anyhow::Result;

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

    /// Get websites built by QuillSpace for a user
    pub async fn get_user_websites(&self, user_id: Uuid) -> Result<Vec<ConnectedWebsite>> {
        let client = self.db.postgres().get().await
            .map_err(|e| anyhow::anyhow!("Failed to get database connection: {}", e))?;
            
        let query = "
            SELECT uws.wix_site_id, uws.tenant_id, uws.display_name, 
                   uws.custom_domain, uws.project_status, uws.service_type, 
                   uws.client_can_edit, uws.metadata, uws.created_at, uws.updated_at
            FROM user_wix_sites uws
            WHERE uws.user_id = $1 
            AND uws.project_status IN ('review', 'active')
            AND uws.client_can_edit = TRUE
            ORDER BY uws.created_at DESC
        ";

        let rows = client.query(query, &[&user_id]).await?;
        let mut websites = Vec::new();

        for row in rows {
            let wix_site_id: String = row.get(0);
            let tenant_id: Uuid = row.get(1);
            let display_name: Option<String> = row.get(2);
            let custom_domain: Option<String> = row.get(3);
            let project_status: String = row.get(4);
            let service_type: String = row.get(5);
            let metadata: serde_json::Value = row.get(7);
            let created_at: DateTime<Utc> = row.get(8);
            let updated_at: DateTime<Utc> = row.get(9);

            websites.push(ConnectedWebsite {
                id: Uuid::new_v4(),
                tenant_id,
                user_id,
                builder_type: BuilderType::Wix,
                external_site_id: wix_site_id.clone(),
                name: display_name.unwrap_or_else(|| format!("Wix Site {}", &wix_site_id[..8])),
                url: custom_domain.clone().map(|d| format!("https://{}", d)),
                domain: custom_domain,
                status: if project_status == "active" { 
                    ConnectionStatus::Active 
                } else { 
                    ConnectionStatus::Inactive 
                },
                last_sync: Some(Utc::now()),
                sync_error: None,
                metadata: serde_json::json!({
                    "wix_site_id": wix_site_id,
                    "service_type": service_type,
                    "project_status": project_status,
                    "managed_by_quillspace": true,
                    "original_metadata": metadata
                }),
                created_at,
                updated_at,
            });
        }

        Ok(websites)
    }


}
