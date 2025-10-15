use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, delete, put},
    Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::{
    services::connected_websites::{ConnectedWebsitesService, ConnectedWebsite, ConnectWebsiteRequest, SiteInfo, BuilderType},
    middleware::auth::AuthContext,
    AppState,
};

#[derive(Debug, Deserialize)]
pub struct AddExistingWebsiteRequest {
    pub builder_type: BuilderType,
    pub external_site_id: String,
    pub name: String,
    pub url: Option<String>,
    pub domain: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ConnectedWebsitesResponse {
    pub websites: Vec<ConnectedWebsite>,
}

pub fn connected_websites_routes() -> Router<AppState> {
    Router::new()
        .route("/connected-websites", get(get_user_websites))
        .route("/connected-websites", post(connect_website))
        .route("/connected-websites/add-existing", post(add_existing_website))
        .route("/connected-websites/:website_id", delete(disconnect_website))
        .route("/connected-websites/:website_id/refresh", put(refresh_website))
        // Wix editing routes
        .route("/connected-websites/wix/:site_id/pages/:page_id/edit", get(load_wix_page_for_editing))
        .route("/connected-websites/wix/:site_id/pages/:page_id/save", put(save_wix_page_content))
        .route("/connected-websites/wix/:site_id/publish", post(publish_wix_site))
        .route("/connected-websites/wix/test-connection", post(test_wix_connection))
}

/// Get all connected websites for the authenticated user
pub async fn get_user_websites(
    auth: AuthContext,
    State(state): State<AppState>,
) -> Result<Json<ConnectedWebsitesResponse>, StatusCode> {
    let service = ConnectedWebsitesService::new(state.db.clone());
    
    match service.get_user_websites(auth.user_id).await {
        Ok(websites) => Ok(Json(ConnectedWebsitesResponse { websites })),
        Err(e) => {
            tracing::error!("Failed to get user websites: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Connect a new website through API credentials
pub async fn connect_website(
    auth: AuthContext,
    State(state): State<AppState>,
    Json(request): Json<ConnectWebsiteRequest>,
) -> Result<Json<ConnectedWebsite>, StatusCode> {
    let service = ConnectedWebsitesService::new(state.db.clone());
    
    match service.connect_website(auth.user_id, auth.tenant_id, request).await {
        Ok(website) => Ok(Json(website)),
        Err(e) => {
            tracing::error!("Failed to connect website: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Add an existing website manually (like Yasin's Wix site)
pub async fn add_existing_website(
    auth: AuthContext,
    State(state): State<AppState>,
    Json(request): Json<AddExistingWebsiteRequest>,
) -> Result<Json<ConnectedWebsite>, StatusCode> {
    let service = ConnectedWebsitesService::new(state.db.clone());
    
    let site_info = SiteInfo {
        external_site_id: request.external_site_id,
        name: request.name,
        url: request.url,
        domain: request.domain,
    };
    
    match service.add_existing_website(
        auth.user_id,
        auth.tenant_id,
        request.builder_type,
        site_info,
    ).await {
        Ok(website) => Ok(Json(website)),
        Err(e) => {
            tracing::error!("Failed to add existing website: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Disconnect a website
pub async fn disconnect_website(
    auth: AuthContext,
    State(state): State<AppState>,
    Path(website_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let service = ConnectedWebsitesService::new(state.db.clone());
    
    match service.disconnect_website(website_id, auth.user_id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            tracing::error!("Failed to disconnect website: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Refresh website data
pub async fn refresh_website(
    auth: AuthContext,
    State(state): State<AppState>,
    Path(website_id): Path<Uuid>,
) -> Result<Json<ConnectedWebsite>, StatusCode> {
    let service = ConnectedWebsitesService::new(state.db.clone());
    
    // Verify the website belongs to the user
    let user_websites = service.get_user_websites(auth.user_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    if !user_websites.iter().any(|w| w.id == website_id) {
        return Err(StatusCode::NOT_FOUND);
    }
    
    match service.refresh_website(website_id).await {
        Ok(website) => Ok(Json(website)),
        Err(e) => {
            tracing::error!("Failed to refresh website: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Load Wix page for editing in QuillSpace
pub async fn load_wix_page_for_editing(
    auth: AuthContext,
    State(state): State<AppState>,
    Path((site_id, page_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let service = ConnectedWebsitesService::new(state.db.clone());
    
    match service.load_wix_page_for_editing(auth.user_id, &site_id, &page_id).await {
        Ok(puck_data) => Ok(Json(puck_data)),
        Err(e) => {
            tracing::error!("Failed to load Wix page for editing: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Save edited Wix page content from QuillSpace
pub async fn save_wix_page_content(
    auth: AuthContext,
    State(state): State<AppState>,
    Path((site_id, page_id)): Path<(String, String)>,
    Json(puck_data): Json<serde_json::Value>,
) -> Result<StatusCode, StatusCode> {
    let service = ConnectedWebsitesService::new(state.db.clone());
    
    match service.save_wix_page_from_quillspace(auth.user_id, &site_id, &page_id, &puck_data).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => {
            tracing::error!("Failed to save Wix page content: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Publish Wix site changes
pub async fn publish_wix_site(
    auth: AuthContext,
    State(state): State<AppState>,
    Path(site_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let service = ConnectedWebsitesService::new(state.db.clone());
    
    match service.publish_wix_site(auth.user_id, &site_id).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => {
            tracing::error!("Failed to publish Wix site: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct TestWixConnectionRequest {
    pub api_key: String,
    pub account_id: String,
}

/// Test Wix API connection
pub async fn test_wix_connection(
    State(state): State<AppState>,
    Json(request): Json<TestWixConnectionRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let service = ConnectedWebsitesService::new(state.db.clone());
    
    match service.test_wix_connection(&request.api_key, &request.account_id).await {
        Ok(is_valid) => Ok(Json(serde_json::json!({ "valid": is_valid }))),
        Err(e) => {
            tracing::error!("Failed to test Wix connection: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
