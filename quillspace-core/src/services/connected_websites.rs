use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::database::Database;
use crate::services::wix_api::{WixApiClient, WixIntegrationService};
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectedWebsite {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub credentials_id: Uuid,
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
pub struct WebsiteBuilderCredentials {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub builder_type: BuilderType,
    pub encrypted_credentials: String,
    pub is_active: bool,
    pub last_sync: Option<DateTime<Utc>>,
    pub sync_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BuilderType {
    Wix,
    WordPress,
    Squarespace,
    Jflux,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionStatus {
    Active,
    Inactive,
    Syncing,
    Error,
}

#[derive(Debug, Deserialize)]
pub struct ConnectWebsiteRequest {
    pub builder_type: BuilderType,
    pub credentials: serde_json::Value,
    pub site_info: Option<SiteInfo>,
}

#[derive(Debug, Deserialize)]
pub struct SiteInfo {
    pub external_site_id: String,
    pub name: String,
    pub url: Option<String>,
    pub domain: Option<String>,
}

pub struct ConnectedWebsitesService {
    db: Database,
    wix_service: WixIntegrationService,
}

impl ConnectedWebsitesService {
    pub fn new(db: Database) -> Self {
        Self { 
            db,
            wix_service: WixIntegrationService::new(),
        }
    }

    /// Get all connected websites for a user (includes QuillSpace-built sites from Wix)
    pub async fn get_user_websites(&self, user_id: Uuid) -> Result<Vec<ConnectedWebsite>> {
        // First get manually connected websites from database
        let query = "
            SELECT id, tenant_id, user_id, credentials_id, builder_type,
                   external_site_id, name, url, domain, status,
                   last_sync, sync_error, metadata, created_at, updated_at
            FROM connected_websites 
            WHERE user_id = $1
            ORDER BY created_at DESC
        ";

        let rows = self.db.query(query, &[&user_id]).await?;
        let mut websites = Vec::new();

        for row in rows {
            websites.push(ConnectedWebsite {
                id: row.get(0),
                tenant_id: row.get(1),
                user_id: row.get(2),
                credentials_id: row.get(3),
                builder_type: row.get(4),
                external_site_id: row.get(5),
                name: row.get(6),
                url: row.get(7),
                domain: row.get(8),
                status: row.get(9),
                last_sync: row.get(10),
                sync_error: row.get(11),
                metadata: row.get(12),
                created_at: row.get(13),
                updated_at: row.get(14),
            });
        }

        // Also fetch websites built by QuillSpace team from Wix
        let quillspace_websites = self.get_quillspace_built_websites(user_id).await?;
        websites.extend(quillspace_websites);

        Ok(websites)
    }

    /// Get websites built by QuillSpace team for this user using Wix site ID mapping
    async fn get_quillspace_built_websites(&self, user_id: Uuid) -> Result<Vec<ConnectedWebsite>> {
        // Get user's mapped Wix sites from database (using site ID as key)
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

        let rows = self.db.query(query, &[&user_id]).await?;
        let mut websites = Vec::new();

        // Get QuillSpace's Wix credentials for API calls
        let wix_api_key = std::env::var("QUILLSPACE_WIX_API_KEY")
            .map_err(|_| anyhow::anyhow!("QuillSpace Wix API key not configured"))?;
        let wix_account_id = std::env::var("QUILLSPACE_WIX_ACCOUNT_ID")
            .map_err(|_| anyhow::anyhow!("QuillSpace Wix account ID not configured"))?;

        let client = WixApiClient::new(wix_api_key, wix_account_id);

        for row in rows {
            let wix_site_id: String = row.get(0);
            let tenant_id: Uuid = row.get(1);
            let display_name: Option<String> = row.get(2);
            let custom_domain: Option<String> = row.get(3);
            let project_status: String = row.get(4);
            let service_type: String = row.get(5);
            let client_can_edit: bool = row.get(6);
            let metadata: serde_json::Value = row.get(7);
            let created_at: chrono::DateTime<chrono::Utc> = row.get(8);
            let updated_at: chrono::DateTime<chrono::Utc> = row.get(9);

            // Get site details from Wix API using the site ID
            match client.get_site(&wix_site_id).await {
                Ok(wix_site) => {
                    let website = ConnectedWebsite {
                        id: uuid::Uuid::parse_str(&wix_site_id).unwrap_or_else(|_| uuid::Uuid::new_v4()),
                        tenant_id,
                        user_id,
                        credentials_id: uuid::Uuid::new_v4(), // Temp ID for QuillSpace-managed sites
                        builder_type: BuilderType::Wix,
                        external_site_id: wix_site_id.clone(),
                        name: display_name.unwrap_or(wix_site.display_name), // Use override or Wix name
                        url: Some(wix_site.url.clone()),
                        domain: custom_domain.or_else(|| {
                            Some(wix_site.url.replace("https://", "").replace("http://", ""))
                        }),
                        status: if project_status == "active" { 
                            ConnectionStatus::Active 
                        } else { 
                            ConnectionStatus::Inactive 
                        },
                        last_sync: Some(chrono::Utc::now()),
                        sync_error: None,
                        metadata: serde_json::json!({
                            "wix_site_id": wix_site_id,
                            "built_by": "quillspace_team",
                            "service_type": service_type,
                            "project_status": project_status,
                            "editable_by_client": client_can_edit,
                            "managed_by_quillspace": true,
                            "wix_created_date": wix_site.created_date,
                            "wix_published_date": wix_site.published_date,
                            "original_metadata": metadata
                        }),
                        created_at,
                        updated_at,
                    };
                    websites.push(website);
                }
                Err(e) => {
                    tracing::warn!("Wix site {} not found for user {}: {}", wix_site_id, user_id, e);
                    // Could mark as error status in database here
                }
            }
        }
        
        Ok(websites)
    }


    /// Connect a new website
    pub async fn connect_website(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
        request: ConnectWebsiteRequest,
    ) -> Result<ConnectedWebsite> {
        // First, create or update credentials
        let credentials_id = self.upsert_credentials(
            user_id,
            tenant_id,
            request.builder_type.clone(),
            request.credentials,
        ).await?;

        // If site_info is provided, create the website connection directly
        if let Some(site_info) = request.site_info {
            return self.create_website_connection(
                user_id,
                tenant_id,
                credentials_id,
                request.builder_type,
                site_info,
            ).await;
        }

        // Otherwise, sync websites from the platform
        let websites = self.sync_websites_from_platform(
            user_id,
            tenant_id,
            credentials_id,
            request.builder_type,
        ).await?;

        // Return the first website (or error if none found)
        websites.into_iter().next()
            .ok_or_else(|| anyhow::anyhow!("No websites found for this account"))
    }

    /// Manually add a website (for existing sites like Yasin's)
    pub async fn add_existing_website(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
        builder_type: BuilderType,
        site_info: SiteInfo,
    ) -> Result<ConnectedWebsite> {
        // Create dummy credentials for manual connections
        let credentials_id = self.create_manual_credentials(
            user_id,
            tenant_id,
            builder_type.clone(),
        ).await?;

        self.create_website_connection(
            user_id,
            tenant_id,
            credentials_id,
            builder_type,
            site_info,
        ).await
    }

    /// Create website connection record
    async fn create_website_connection(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
        credentials_id: Uuid,
        builder_type: BuilderType,
        site_info: SiteInfo,
    ) -> Result<ConnectedWebsite> {
        let website_id = Uuid::new_v4();
        let now = Utc::now();

        let query = "
            INSERT INTO connected_websites (
                id, tenant_id, user_id, credentials_id, builder_type,
                external_site_id, name, url, domain, status,
                last_sync, metadata, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING id, tenant_id, user_id, credentials_id, builder_type,
                      external_site_id, name, url, domain, status,
                      last_sync, sync_error, metadata, created_at, updated_at
        ";

        let metadata = match builder_type {
            BuilderType::Wix => serde_json::json!({
                "wix_site_id": site_info.external_site_id,
                "plan_type": "free",
                "last_published": now
            }),
            _ => serde_json::json!({}),
        };

        let row = self.db.query_one(query, &[
            &website_id,
            &tenant_id,
            &user_id,
            &credentials_id,
            &builder_type,
            &site_info.external_site_id,
            &site_info.name,
            &site_info.url,
            &site_info.domain,
            &ConnectionStatus::Active,
            &now,
            &metadata,
            &now,
            &now,
        ]).await?;

        Ok(ConnectedWebsite {
            id: row.get(0),
            tenant_id: row.get(1),
            user_id: row.get(2),
            credentials_id: row.get(3),
            builder_type: row.get(4),
            external_site_id: row.get(5),
            name: row.get(6),
            url: row.get(7),
            domain: row.get(8),
            status: row.get(9),
            last_sync: row.get(10),
            sync_error: row.get(11),
            metadata: row.get(12),
            created_at: row.get(13),
            updated_at: row.get(14),
        })
    }

    /// Create manual credentials for existing websites
    async fn create_manual_credentials(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
        builder_type: BuilderType,
    ) -> Result<Uuid> {
        let credentials_id = Uuid::new_v4();
        let now = Utc::now();

        let query = "
            INSERT INTO website_builder_credentials (
                id, tenant_id, user_id, builder_type, encrypted_credentials,
                is_active, last_sync, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (user_id, builder_type) 
            DO UPDATE SET updated_at = EXCLUDED.updated_at
            RETURNING id
        ";

        let row = self.db.query_one(query, &[
            &credentials_id,
            &tenant_id,
            &user_id,
            &builder_type,
            &"manual_connection", // Placeholder for manual connections
            &true,
            &now,
            &now,
            &now,
        ]).await?;

        Ok(row.get(0))
    }

    /// Upsert credentials
    async fn upsert_credentials(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
        builder_type: BuilderType,
        credentials: serde_json::Value,
    ) -> Result<Uuid> {
        // In production, encrypt the credentials
        let encrypted_credentials = serde_json::to_string(&credentials)?;
        let credentials_id = Uuid::new_v4();
        let now = Utc::now();

        let query = "
            INSERT INTO website_builder_credentials (
                id, tenant_id, user_id, builder_type, encrypted_credentials,
                is_active, last_sync, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (user_id, builder_type) 
            DO UPDATE SET 
                encrypted_credentials = EXCLUDED.encrypted_credentials,
                updated_at = EXCLUDED.updated_at
            RETURNING id
        ";

        let row = self.db.query_one(query, &[
            &credentials_id,
            &tenant_id,
            &user_id,
            &builder_type,
            &encrypted_credentials,
            &true,
            &now,
            &now,
            &now,
        ]).await?;

        Ok(row.get(0))
    }

    /// Sync websites from external platform
    async fn sync_websites_from_platform(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
        credentials_id: Uuid,
        builder_type: BuilderType,
    ) -> Result<Vec<ConnectedWebsite>> {
        // This would integrate with actual APIs
        // For now, return empty vector
        match builder_type {
            BuilderType::Wix => self.sync_wix_websites(user_id, tenant_id, credentials_id).await,
            BuilderType::WordPress => self.sync_wordpress_websites(user_id, tenant_id, credentials_id).await,
            _ => Ok(Vec::new()),
        }
    }

    /// Sync Wix websites using actual API
    async fn sync_wix_websites(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
        credentials_id: Uuid,
    ) -> Result<Vec<ConnectedWebsite>> {
        // Get decrypted credentials
        let credentials = self.get_decrypted_credentials(credentials_id).await?;
        let api_key = credentials.get("api_key").ok_or_else(|| anyhow::anyhow!("Missing API key"))?;
        let account_id = credentials.get("account_id").ok_or_else(|| anyhow::anyhow!("Missing account ID"))?;
        
        // Create Wix client
        let client = WixApiClient::new(api_key.clone(), account_id.clone());
        
        // Get sites from Wix
        let wix_sites = client.get_sites().await?;
        let mut connected_websites = Vec::new();
        
        for wix_site in wix_sites {
            // Create or update connected website record
            let website = self.create_website_connection(
                user_id,
                tenant_id,
                credentials_id,
                BuilderType::Wix,
                SiteInfo {
                    external_site_id: wix_site.site_id.clone(),
                    name: wix_site.display_name.clone(),
                    url: Some(wix_site.url.clone()),
                    domain: Some(wix_site.url.replace("https://", "").replace("http://", "")),
                },
            ).await?;
            
            connected_websites.push(website);
        }
        
        Ok(connected_websites)
    }

    /// Sync WordPress websites (placeholder implementation)
    async fn sync_wordpress_websites(
        &self,
        _user_id: Uuid,
        _tenant_id: Uuid,
        _credentials_id: Uuid,
    ) -> Result<Vec<ConnectedWebsite>> {
        // TODO: Implement WordPress API integration
        Ok(Vec::new())
    }

    /// Update website status
    pub async fn update_website_status(
        &self,
        website_id: Uuid,
        status: ConnectionStatus,
        error_message: Option<String>,
    ) -> Result<()> {
        let query = "
            UPDATE connected_websites 
            SET status = $1, sync_error = $2, updated_at = NOW()
            WHERE id = $3
        ";

        self.db.execute(query, &[&status, &error_message, &website_id]).await?;
        Ok(())
    }

    /// Delete website connection
    pub async fn disconnect_website(&self, website_id: Uuid, user_id: Uuid) -> Result<()> {
        let query = "
            DELETE FROM connected_websites 
            WHERE id = $1 AND user_id = $2
        ";

        self.db.execute(query, &[&website_id, &user_id]).await?;
        Ok(())
    }

    /// Refresh website data
    pub async fn refresh_website(&self, website_id: Uuid) -> Result<ConnectedWebsite> {
        // Update last_sync timestamp
        let query = "
            UPDATE connected_websites 
            SET last_sync = NOW(), status = 'active', updated_at = NOW()
            WHERE id = $1
            RETURNING id, tenant_id, user_id, credentials_id, builder_type,
                      external_site_id, name, url, domain, status,
                      last_sync, sync_error, metadata, created_at, updated_at
        ";

        let row = self.db.query_one(query, &[&website_id]).await?;

        Ok(ConnectedWebsite {
            id: row.get(0),
            tenant_id: row.get(1),
            user_id: row.get(2),
            credentials_id: row.get(3),
            builder_type: row.get(4),
            external_site_id: row.get(5),
            name: row.get(6),
            url: row.get(7),
            domain: row.get(8),
            status: row.get(9),
            last_sync: row.get(10),
            sync_error: row.get(11),
            metadata: row.get(12),
            created_at: row.get(13),
            updated_at: row.get(14),
        })
    }

    /// Get decrypted credentials for a credentials record
    async fn get_decrypted_credentials(&self, credentials_id: Uuid) -> Result<std::collections::HashMap<String, String>> {
        let query = "SELECT encrypted_credentials FROM website_builder_credentials WHERE id = $1";
        let row = self.db.query_one(query, &[&credentials_id]).await?;
        let encrypted_credentials: String = row.get(0);
        
        // In production, decrypt the credentials
        // For now, assume they're JSON
        let credentials: std::collections::HashMap<String, String> = 
            serde_json::from_str(&encrypted_credentials)?;
        
        Ok(credentials)
    }

    /// Load Wix page content for editing in QuillSpace
    pub async fn load_wix_page_for_editing(
        &self,
        user_id: Uuid,
        site_id: &str,
        page_id: &str,
    ) -> Result<serde_json::Value> {
        // Get user's Wix credentials
        let credentials = self.get_user_wix_credentials(user_id).await?;
        let client = WixApiClient::new(
            credentials.get("api_key").unwrap().clone(),
            credentials.get("account_id").unwrap().clone(),
        );
        
        // Get page content from Wix
        let page = client.get_page(site_id, page_id).await?;
        let components = client.get_page_components(site_id, page_id).await?;
        
        // Convert to Puck format for editing
        let mut wix_service = WixIntegrationService::new();
        let puck_data = wix_service.convert_to_quillspace_format(
            &user_id.to_string(),
            site_id,
            page_id,
        ).await?;
        
        Ok(puck_data)
    }

    /// Save edited content back to Wix
    pub async fn save_wix_page_from_quillspace(
        &self,
        user_id: Uuid,
        site_id: &str,
        page_id: &str,
        puck_data: &serde_json::Value,
    ) -> Result<()> {
        // Get user's Wix credentials
        let credentials = self.get_user_wix_credentials(user_id).await?;
        
        // Set up Wix client in the integration service
        let mut wix_service = WixIntegrationService::new();
        wix_service.add_client(
            &user_id.to_string(),
            credentials.get("api_key").unwrap().clone(),
            credentials.get("account_id").unwrap().clone(),
        );
        
        // Convert Puck data back to Wix format and save
        wix_service.convert_from_quillspace_format(
            &user_id.to_string(),
            site_id,
            page_id,
            puck_data,
        ).await?;
        
        Ok(())
    }

    /// Publish Wix site changes
    pub async fn publish_wix_site(&self, user_id: Uuid, site_id: &str) -> Result<()> {
        let credentials = self.get_user_wix_credentials(user_id).await?;
        let client = WixApiClient::new(
            credentials.get("api_key").unwrap().clone(),
            credentials.get("account_id").unwrap().clone(),
        );
        
        client.publish_site(site_id).await?;
        Ok(())
    }

    /// Get user's Wix credentials
    async fn get_user_wix_credentials(&self, user_id: Uuid) -> Result<std::collections::HashMap<String, String>> {
        let query = "
            SELECT encrypted_credentials 
            FROM website_builder_credentials 
            WHERE user_id = $1 AND builder_type = 'wix' AND is_active = true
            LIMIT 1
        ";
        
        let row = self.db.query_one(query, &[&user_id]).await?;
        let encrypted_credentials: String = row.get(0);
        
        // Decrypt credentials (in production)
        let credentials: std::collections::HashMap<String, String> = 
            serde_json::from_str(&encrypted_credentials)?;
        
        Ok(credentials)
    }

    /// Test Wix API connection
    pub async fn test_wix_connection(&self, api_key: &str, account_id: &str) -> Result<bool> {
        let client = WixApiClient::new(api_key.to_string(), account_id.to_string());
        client.test_connection().await
    }
}
