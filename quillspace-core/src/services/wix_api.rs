use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use anyhow::Result;
use reqwest::Client;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct WixSite {
    pub site_id: String,
    pub display_name: String,
    pub url: String,
    pub status: String,
    pub created_date: DateTime<Utc>,
    pub published_date: Option<DateTime<Utc>>,
    pub site_type: String,
    pub locale: String,
    pub premium_features: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WixPage {
    pub page_id: String,
    pub title: String,
    pub url_path: String,
    pub page_type: String,
    pub is_home_page: bool,
    pub is_hidden: bool,
    pub seo_data: Option<WixSeoData>,
    pub content: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WixSeoData {
    pub title: Option<String>,
    pub description: Option<String>,
    pub keywords: Vec<String>,
    pub no_index: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WixComponent {
    pub component_id: String,
    pub component_type: String,
    pub data: serde_json::Value,
    pub style: Option<serde_json::Value>,
    pub position: WixPosition,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WixPosition {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Deserialize)]
pub struct WixApiResponse<T> {
    pub data: T,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct WixErrorResponse {
    pub error: WixError,
}

#[derive(Debug, Deserialize)]
pub struct WixError {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

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

    /// Create headers for Wix API calls (based on working implementation)
    fn create_headers(&self, site_id: Option<&str>) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        
        // Use Authorization header with API key (not Bearer)
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&self.api_key).unwrap()
        );
        
        // Add account ID header
        headers.insert(
            "wix-account-id",
            reqwest::header::HeaderValue::from_str(&self.account_id).unwrap()
        );
        
        // Add site ID header if provided
        if let Some(site_id) = site_id {
            headers.insert(
                "wix-site-id",
                reqwest::header::HeaderValue::from_str(site_id).unwrap()
            );
        }
        
        // Add content type
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/json")
        );
        
        headers
    }

    /// Get all sites for the authenticated account
    pub async fn get_sites(&self) -> Result<Vec<WixSite>> {
        let url = format!("{}/site-list/v2/sites", self.base_url);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("wix-account-id", &self.account_id)
            .send()
            .await?;

        if response.status().is_success() {
            let api_response: WixApiResponse<Vec<WixSite>> = response.json().await?;
            Ok(api_response.data)
        } else {
            let error: WixErrorResponse = response.json().await?;
            Err(anyhow::anyhow!("Wix API error: {}", error.error.message))
        }
    }

    /// Get specific site details
    pub async fn get_site(&self, site_id: &str) -> Result<WixSite> {
        let url = format!("{}/site-list/v2/sites/{}", self.base_url, site_id);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("wix-account-id", &self.account_id)
            .send()
            .await?;

        if response.status().is_success() {
            let api_response: WixApiResponse<WixSite> = response.json().await?;
            Ok(api_response.data)
        } else {
            let error: WixErrorResponse = response.json().await?;
            Err(anyhow::anyhow!("Wix API error: {}", error.error.message))
        }
    }

    /// Get all pages for a site
    pub async fn get_pages(&self, site_id: &str) -> Result<Vec<WixPage>> {
        let url = format!("{}/pages/v1/sites/{}/pages", self.base_url, site_id);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("wix-site-id", site_id)
            .send()
            .await?;

        if response.status().is_success() {
            let api_response: WixApiResponse<Vec<WixPage>> = response.json().await?;
            Ok(api_response.data)
        } else {
            let error: WixErrorResponse = response.json().await?;
            Err(anyhow::anyhow!("Wix API error: {}", error.error.message))
        }
    }

    /// Get specific page content
    pub async fn get_page(&self, site_id: &str, page_id: &str) -> Result<WixPage> {
        let url = format!("{}/pages/v1/sites/{}/pages/{}", self.base_url, site_id, page_id);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("wix-site-id", site_id)
            .send()
            .await?;

        if response.status().is_success() {
            let api_response: WixApiResponse<WixPage> = response.json().await?;
            Ok(api_response.data)
        } else {
            let error: WixErrorResponse = response.json().await?;
            Err(anyhow::anyhow!("Wix API error: {}", error.error.message))
        }
    }

    /// Update page content
    pub async fn update_page(&self, site_id: &str, page_id: &str, page_data: &WixPage) -> Result<WixPage> {
        let url = format!("{}/pages/v1/sites/{}/pages/{}", self.base_url, site_id, page_id);
        
        let response = self.client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("wix-site-id", site_id)
            .header("Content-Type", "application/json")
            .json(page_data)
            .send()
            .await?;

        if response.status().is_success() {
            let api_response: WixApiResponse<WixPage> = response.json().await?;
            Ok(api_response.data)
        } else {
            let error: WixErrorResponse = response.json().await?;
            Err(anyhow::anyhow!("Wix API error: {}", error.error.message))
        }
    }

    /// Get page components (for editing)
    pub async fn get_page_components(&self, site_id: &str, page_id: &str) -> Result<Vec<WixComponent>> {
        let url = format!("{}/editor/v1/sites/{}/pages/{}/components", self.base_url, site_id, page_id);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("wix-site-id", site_id)
            .send()
            .await?;

        if response.status().is_success() {
            let api_response: WixApiResponse<Vec<WixComponent>> = response.json().await?;
            Ok(api_response.data)
        } else {
            let error: WixErrorResponse = response.json().await?;
            Err(anyhow::anyhow!("Wix API error: {}", error.error.message))
        }
    }

    /// Update page component
    pub async fn update_component(
        &self, 
        site_id: &str, 
        page_id: &str, 
        component_id: &str, 
        component_data: &WixComponent
    ) -> Result<WixComponent> {
        let url = format!(
            "{}/editor/v1/sites/{}/pages/{}/components/{}", 
            self.base_url, site_id, page_id, component_id
        );
        
        let response = self.client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("wix-site-id", site_id)
            .header("Content-Type", "application/json")
            .json(component_data)
            .send()
            .await?;

        if response.status().is_success() {
            let api_response: WixApiResponse<WixComponent> = response.json().await?;
            Ok(api_response.data)
        } else {
            let error: WixErrorResponse = response.json().await?;
            Err(anyhow::anyhow!("Wix API error: {}", error.error.message))
        }
    }

    /// Publish site changes
    pub async fn publish_site(&self, site_id: &str) -> Result<()> {
        let url = format!("{}/site-actions/v1/sites/{}/publish", self.base_url, site_id);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("wix-site-id", site_id)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error: WixErrorResponse = response.json().await?;
            Err(anyhow::anyhow!("Wix API error: {}", error.error.message))
        }
    }

    /// Test API connection
    pub async fn test_connection(&self) -> Result<bool> {
        match self.get_sites().await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Query Wix data using the correct API format (based on working implementation)
    pub async fn query_data(&self, site_id: &str, collection: &str, query: serde_json::Value) -> Result<serde_json::Value> {
        let url = format!("{}/wix-data/v2/items/query", self.base_url);
        let headers = self.create_headers(Some(site_id));
        
        let body = serde_json::json!({
            "dataCollectionId": collection,
            "query": query
        });
        
        let response = self.client
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            Ok(result)
        } else {
            let error_text = response.text().await?;
            Err(anyhow::anyhow!("Wix API error: {}", error_text))
        }
    }

    /// Get business info for a site
    pub async fn get_business_info(&self, site_id: &str) -> Result<serde_json::Value> {
        let url = format!("{}/business-info/v1/business-info", self.base_url);
        let headers = self.create_headers(Some(site_id));
        
        let response = self.client
            .get(&url)
            .headers(headers)
            .send()
            .await?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            Ok(result)
        } else {
            let error_text = response.text().await?;
            Err(anyhow::anyhow!("Business info API error: {}", error_text))
        }
    }

    /// Update business info for a site
    pub async fn update_business_info(&self, site_id: &str, business_info: serde_json::Value) -> Result<serde_json::Value> {
        let url = format!("{}/business-info/v1/business-info", self.base_url);
        let headers = self.create_headers(Some(site_id));
        
        let response = self.client
            .patch(&url)
            .headers(headers)
            .json(&business_info)
            .send()
            .await?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            Ok(result)
        } else {
            let error_text = response.text().await?;
            Err(anyhow::anyhow!("Update business info error: {}", error_text))
        }
    }
}

/// Service for managing Wix integrations
pub struct WixIntegrationService {
    clients: HashMap<String, WixApiClient>,
}

impl WixIntegrationService {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }

    /// Add a Wix client for a user
    pub fn add_client(&mut self, user_id: &str, api_key: String, account_id: String) {
        let client = WixApiClient::new(api_key, account_id);
        self.clients.insert(user_id.to_string(), client);
    }

    /// Get client for user
    pub fn get_client(&self, user_id: &str) -> Option<&WixApiClient> {
        self.clients.get(user_id)
    }

    /// Sync user's Wix sites
    pub async fn sync_user_sites(&self, user_id: &str) -> Result<Vec<WixSite>> {
        if let Some(client) = self.get_client(user_id) {
            client.get_sites().await
        } else {
            Err(anyhow::anyhow!("No Wix client found for user"))
        }
    }

    /// Convert Wix page to QuillSpace format (for editing)
    pub async fn convert_to_quillspace_format(
        &self, 
        user_id: &str, 
        site_id: &str, 
        page_id: &str
    ) -> Result<serde_json::Value> {
        if let Some(client) = self.get_client(user_id) {
            let page = client.get_page(site_id, page_id).await?;
            let components = client.get_page_components(site_id, page_id).await?;
            
            // Convert Wix components to Puck-compatible format
            let puck_data = self.convert_wix_to_puck(&page, &components)?;
            Ok(puck_data)
        } else {
            Err(anyhow::anyhow!("No Wix client found for user"))
        }
    }

    /// Convert Puck data back to Wix format
    pub async fn convert_from_quillspace_format(
        &self,
        user_id: &str,
        site_id: &str,
        page_id: &str,
        puck_data: &serde_json::Value,
    ) -> Result<()> {
        if let Some(client) = self.get_client(user_id) {
            let (page_data, components) = self.convert_puck_to_wix(puck_data)?;
            
            // Update page
            client.update_page(site_id, page_id, &page_data).await?;
            
            // Update components
            for component in components {
                client.update_component(site_id, page_id, &component.component_id, &component).await?;
            }
            
            Ok(())
        } else {
            Err(anyhow::anyhow!("No Wix client found for user"))
        }
    }

    /// Convert Wix components to Puck format
    fn convert_wix_to_puck(&self, page: &WixPage, components: &[WixComponent]) -> Result<serde_json::Value> {
        let mut puck_components = Vec::new();
        
        for component in components {
            let puck_component = match component.component_type.as_str() {
                "Text" => self.convert_wix_text_to_puck(component)?,
                "Image" => self.convert_wix_image_to_puck(component)?,
                "Button" => self.convert_wix_button_to_puck(component)?,
                "Container" => self.convert_wix_container_to_puck(component)?,
                _ => self.convert_wix_generic_to_puck(component)?,
            };
            puck_components.push(puck_component);
        }
        
        Ok(serde_json::json!({
            "content": puck_components,
            "root": {
                "title": page.title,
                "props": {
                    "title": page.title
                }
            }
        }))
    }

    /// Convert Puck data to Wix format
    fn convert_puck_to_wix(&self, puck_data: &serde_json::Value) -> Result<(WixPage, Vec<WixComponent>)> {
        // This would implement the reverse conversion
        // For now, return placeholder data
        let page = WixPage {
            page_id: "placeholder".to_string(),
            title: "Updated Page".to_string(),
            url_path: "/".to_string(),
            page_type: "static".to_string(),
            is_home_page: false,
            is_hidden: false,
            seo_data: None,
            content: Some(puck_data.clone()),
        };
        
        let components = Vec::new(); // Would convert Puck components back to Wix
        
        Ok((page, components))
    }

    // Component conversion helpers
    fn convert_wix_text_to_puck(&self, component: &WixComponent) -> Result<serde_json::Value> {
        Ok(serde_json::json!({
            "type": "Text",
            "props": {
                "text": component.data.get("text").unwrap_or(&serde_json::Value::String("".to_string())),
                "size": "m"
            }
        }))
    }

    fn convert_wix_image_to_puck(&self, component: &WixComponent) -> Result<serde_json::Value> {
        Ok(serde_json::json!({
            "type": "Image",
            "props": {
                "src": component.data.get("src").unwrap_or(&serde_json::Value::String("".to_string())),
                "alt": component.data.get("alt").unwrap_or(&serde_json::Value::String("".to_string()))
            }
        }))
    }

    fn convert_wix_button_to_puck(&self, component: &WixComponent) -> Result<serde_json::Value> {
        Ok(serde_json::json!({
            "type": "Button",
            "props": {
                "text": component.data.get("text").unwrap_or(&serde_json::Value::String("Button".to_string())),
                "href": component.data.get("link").unwrap_or(&serde_json::Value::String("#".to_string())),
                "variant": "primary"
            }
        }))
    }

    fn convert_wix_container_to_puck(&self, component: &WixComponent) -> Result<serde_json::Value> {
        Ok(serde_json::json!({
            "type": "Container",
            "props": {
                "padding": "m"
            }
        }))
    }

    fn convert_wix_generic_to_puck(&self, component: &WixComponent) -> Result<serde_json::Value> {
        Ok(serde_json::json!({
            "type": "Custom",
            "props": {
                "componentType": component.component_type,
                "data": component.data
            }
        }))
    }
}
