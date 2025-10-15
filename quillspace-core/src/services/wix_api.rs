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
        
        let body = serde_json::json!({
            "dataCollectionId": collection_id,
            "dataItem": item_data
        });
        
        let response = self.client
            .patch(&url)
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

}
