use serde::{Deserialize, Serialize};
use anyhow::Result;
use reqwest::Client;

pub struct WixApiClient {
    client: Client,
    api_key: String,
    account_id: String,
    base_url: String,
}

impl WixApiClient {
    pub fn new(api_key: String, account_id: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            account_id,
            base_url: "https://www.wixapis.com".to_string(),
        }
    }

    fn create_headers(&self, site_id: &str) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("Authorization", self.api_key.parse().unwrap());
        headers.insert("wix-account-id", self.account_id.parse().unwrap());
        headers.insert("wix-site-id", site_id.parse().unwrap());
        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers
    }

    /// Get items from a Wix Data collection
    pub async fn get_collection_items(&self, site_id: &str, collection_id: &str) -> Result<serde_json::Value> {
        let url = format!("{}/wix-data/v2/items/query", self.base_url);
        let headers = self.create_headers(site_id);
        
        let body = serde_json::json!({
            "dataCollectionId": collection_id,
            "query": {}
        });
        
        let response = self.client
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let error_text = response.text().await?;
            Err(anyhow::anyhow!("Wix Data API error: {}", error_text))
        }
    }

    /// Get a single item from a Wix Data collection by ID
    pub async fn get_collection_item(&self, site_id: &str, collection_id: &str, item_id: &str) -> Result<serde_json::Value> {
        let url = format!("{}/wix-data/v2/items/{}", self.base_url, item_id);
        let mut headers = self.create_headers(site_id);
        
        // Add collection ID as query parameter for single item retrieval
        let url_with_collection = format!("{}?dataCollectionId={}", url, collection_id);
        
        let response = self.client
            .get(&url_with_collection)
            .headers(headers)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let error_text = response.text().await?;
            Err(anyhow::anyhow!("Wix Data API error: {}", error_text))
        }
    }

    /// Partially update (patch) an item in Wix Data collection
    pub async fn patch_collection_item(&self, site_id: &str, collection_id: &str, item_id: &str, patch_data: serde_json::Value) -> Result<serde_json::Value> {
        // First, get the existing item
        let existing_item = self.get_collection_item(site_id, collection_id, item_id).await?;
        
        // Extract the current data
        let mut current_data = existing_item
            .get("dataItem")
            .and_then(|item| item.get("data"))
            .cloned()
            .unwrap_or_default();
        
        // Merge the patch data into current data
        if let Some(patch_obj) = patch_data.get("data") {
            if let (Some(current_obj), Some(patch_obj)) = (current_data.as_object_mut(), patch_obj.as_object()) {
                for (key, value) in patch_obj {
                    current_obj.insert(key.clone(), value.clone());
                }
            }
        }
        
        // Update with the merged data
        let update_payload = serde_json::json!({
            "data": current_data
        });
        
        self.update_collection_item(site_id, collection_id, item_id, update_payload).await
    }

    /// Insert item into Wix Data collection
    pub async fn insert_collection_item(&self, site_id: &str, collection_id: &str, item_data: serde_json::Value) -> Result<serde_json::Value> {
        let url = format!("{}/wix-data/v2/items", self.base_url);
        let headers = self.create_headers(site_id);
        
        let body = serde_json::json!({
            "dataCollectionId": collection_id,
            "dataItem": item_data
        });
        
        let response = self.client
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let error_text = response.text().await?;
            Err(anyhow::anyhow!("Wix Data API error: {}", error_text))
        }
    }

    /// Update item in Wix Data collection
    pub async fn update_collection_item(&self, site_id: &str, collection_id: &str, item_id: &str, item_data: serde_json::Value) -> Result<serde_json::Value> {
        let url = format!("{}/wix-data/v2/items/{}", self.base_url, item_id);
        let headers = self.create_headers(site_id);
        
        // Wix Data API v2 requires PUT with full dataItem, not PATCH
        let body = serde_json::json!({
            "dataCollectionId": collection_id,
            "dataItem": item_data
        });
        
        let response = self.client
            .put(&url)  // Changed from patch to put
            .headers(headers)
            .json(&body)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let error_text = response.text().await?;
            Err(anyhow::anyhow!("Wix Data API error: {}", error_text))
        }
    }

    /// Create or update collection field with proper type
    pub async fn ensure_collection_field(&self, site_id: &str, collection_id: &str, field_key: &str, field_type: &str, display_name: &str) -> Result<serde_json::Value> {
        let url = format!("{}/wix-data/v1/collections/{}/fields", self.base_url, collection_id);
        let headers = self.create_headers(site_id);
        
        // Use the correct Wix field type format
        let body = serde_json::json!({
            "field": {
                "key": field_key,
                "displayName": display_name,
                "type": field_type.to_uppercase(), // Wix expects uppercase: NUMBER, TEXT, etc.
                "queryOperators": ["eq", "ne", "gt", "gte", "lt", "lte"],
                "sortable": true
            }
        });
        
        tracing::info!("Creating Wix field with body: {}", serde_json::to_string_pretty(&body).unwrap_or_default());
        
        let response = self.client
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        let response_text = response.text().await?;
        
        tracing::info!("Wix field creation response: {} - {}", status, response_text);

        if status.is_success() {
            Ok(serde_json::from_str(&response_text).unwrap_or_default())
        } else {
            // Field might already exist, that's okay
            if response_text.contains("already exists") || response_text.contains("FIELD_ALREADY_EXISTS") {
                Ok(serde_json::json!({"status": "field_exists"}))
            } else {
                Err(anyhow::anyhow!("Wix Collection Field API error: {} - {}", status, response_text))
            }
        }
    }

    /// Get site properties
    pub async fn get_site_properties(&self, site_id: &str) -> Result<serde_json::Value> {
        let url = format!("{}/site-properties/v4/properties", self.base_url);
        let headers = self.create_headers(site_id);
        
        let response = self.client
            .get(&url)
            .headers(headers)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let error_text = response.text().await?;
            Err(anyhow::anyhow!("Site properties API error: {}", error_text))
        }
    }

    /// Query sites using Wix Sites API
    /// This fetches site information including thumbnails, names, and URLs
    pub async fn query_sites(&self, site_ids: Option<Vec<String>>) -> Result<serde_json::Value> {
        let url = format!("{}/site-list/v2/sites/query", self.base_url);
        
        // Create headers for account-level API call (no site-id needed)
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("Authorization", self.api_key.parse().unwrap());
        headers.insert("wix-account-id", self.account_id.parse().unwrap());
        headers.insert("Content-Type", "application/json".parse().unwrap());
        
        let mut query_body = serde_json::json!({
            "query": {
                "paging": {
                    "limit": 100
                },
                "sort": [{
                    "fieldName": "updatedDate",
                    "order": "DESC"
                }]
            }
        });
        
        // Filter by specific site IDs if provided
        if let Some(ids) = site_ids {
            if !ids.is_empty() {
                query_body["query"]["filter"] = serde_json::json!({
                    "id": {
                        "$in": ids
                    }
                });
            }
        }
        
        let response = self.client
            .post(&url)
            .headers(headers)
            .json(&query_body)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let error_text = response.text().await?;
            Err(anyhow::anyhow!("Wix Sites API error: {}", error_text))
        }
    }

}
