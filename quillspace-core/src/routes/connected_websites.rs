use axum::{
    extract::{Path, Request, State},
    http::StatusCode,
    response::Json,
    routing::{get, patch, post, put},
    Router,
};
use serde::Serialize;
use uuid::Uuid;
use crate::{
    auth::CasbinAuthContext,
    services::connected_websites::{ConnectedWebsitesService, ConnectedWebsite},
    AppState,
};


#[derive(Debug, Serialize)]
pub struct ConnectedWebsitesResponse {
    pub websites: Vec<ConnectedWebsite>,
}

pub fn connected_websites_routes() -> Router<AppState> {
    Router::new()
        .route("/test", get(|| async { "CONNECTED WEBSITES ROUTE WORKS!" }))
        .route("/websites", get(get_user_websites))
        .route("/wix/sites/:site_id/books", get(get_wix_books_simple))
        .route("/wix/sites/:site_id/books", post(create_wix_book))
        .route("/wix/sites/:site_id/books/with-schema", post(create_wix_book_with_proper_types))
        .route("/wix/sites/:site_id/books/:book_id", get(get_single_wix_book))
        .route("/wix/sites/:site_id/books/:book_id", put(update_wix_book))
        .route("/wix/sites/:site_id/books/:book_id", patch(patch_wix_book))
        // Keep legacy routes for backward compatibility
        .route("/wix/books", get(get_wix_books_legacy))
        .route("/wix/books/:book_id", get(get_single_wix_book_legacy))
        .route("/wix/author", get(get_wix_author_info))
        .route("/wix/author", put(update_wix_author_info))
}

/// Get connected websites for the authenticated user with Casbin authorization
pub async fn get_user_websites(
    State(state): State<AppState>,
    auth: CasbinAuthContext,
) -> Result<Json<ConnectedWebsitesResponse>, StatusCode> {
    // Check Casbin permission for connected_websites read
    auth.require_permission("connected_websites", "read").await?;
    
    tracing::info!(
        "Getting websites for user: {} (tenant: {}, role: {:?})", 
        auth.user_id, auth.tenant_id, auth.user_role
    );
    
    let service = ConnectedWebsitesService::new(state.db.clone());
    
    // Get websites with proper tenant/user isolation
    match service.get_user_websites_with_tenant(auth.user_id, auth.tenant_id).await {
        Ok(websites) => {
            tracing::info!(
                "Found {} websites for user {} in tenant {}", 
                websites.len(), auth.user_id, auth.tenant_id
            );
            Ok(Json(ConnectedWebsitesResponse { websites }))
        },
        Err(e) => {
            tracing::error!("Failed to get user websites: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get Wix books for a specific site with site ownership verification
pub async fn get_wix_books_simple(
    Path(site_id): Path<String>,
    State(state): State<AppState>,
    auth: CasbinAuthContext,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Check Casbin permission for books read
    auth.require_permission("books", "read").await?;
    
    tracing::info!(
        "Getting books for site {} by user: {} (tenant: {}, role: {:?})", 
        site_id, auth.user_id, auth.tenant_id, auth.user_role
    );
    
    // Verify site ownership - users can only read books from sites they own
    let service = ConnectedWebsitesService::new(state.db.clone());
    let owns_site = service.verify_site_ownership(&site_id, auth.tenant_id).await
        .map_err(|e| {
            tracing::error!("Failed to verify site ownership: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    if !owns_site {
        tracing::warn!(
            "Tenant {} attempted to access books for site {} - access denied (not site owner)", 
            auth.tenant_id, site_id
        );
        return Err(StatusCode::FORBIDDEN);
    }
    
    let api_key = std::env::var("QUILLSPACE_WIX_API_KEY")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let account_id = std::env::var("QUILLSPACE_WIX_ACCOUNT_ID")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let client = crate::services::wix_api::WixApiClient::new(api_key, account_id);
    
    match client.get_collection_items(&site_id, "Books").await {
        Ok(books) => {
            tracing::info!("Books for site {} accessed by tenant {}", site_id, auth.tenant_id);
            Ok(Json(books))
        },
        Err(e) => {
            tracing::error!("Failed to get Wix books for site {}: {}", site_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get single Wix book by ID with Casbin authorization (shared read access)
pub async fn get_single_wix_book(
    Path(book_id): Path<String>,
    auth: CasbinAuthContext,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Check Casbin permission for books read
    auth.require_permission("books", "read").await?;
    
    // Single book read is also shared - all tenants can view individual books
    
    let api_key = std::env::var("QUILLSPACE_WIX_API_KEY")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let account_id = std::env::var("QUILLSPACE_WIX_ACCOUNT_ID")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let client = crate::services::wix_api::WixApiClient::new(api_key, account_id);
    
    match client.get_collection_item("1e4e0091-f4d5-4a4c-a66a-4d09e7a5b4e9", "Books", &book_id).await {
        Ok(book) => Ok(Json(book)),
        Err(e) => {
            tracing::error!("Failed to get Wix book {}: {}", book_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Create new book in Wix for a specific site with site ownership verification
pub async fn create_wix_book(
    Path(site_id): Path<String>,
    State(state): State<AppState>,
    auth: CasbinAuthContext,
    Json(book_data): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Check Casbin permission for books write
    auth.require_permission("books", "write").await?;
    
    tracing::info!(
        "Creating book for site {} by user: {} (tenant: {}, role: {:?})", 
        site_id, auth.user_id, auth.tenant_id, auth.user_role
    );
    
    // Verify site ownership - users can only create books for sites they own
    let service = ConnectedWebsitesService::new(state.db.clone());
    let owns_site = service.verify_site_ownership(&site_id, auth.tenant_id).await
        .map_err(|e| {
            tracing::error!("Failed to verify site ownership: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    if !owns_site {
        tracing::warn!(
            "Tenant {} attempted to create book for site {} - access denied (not site owner)", 
            auth.tenant_id, site_id
        );
        return Err(StatusCode::FORBIDDEN);
    }
    
    let api_key = std::env::var("QUILLSPACE_WIX_API_KEY")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let account_id = std::env::var("QUILLSPACE_WIX_ACCOUNT_ID")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let client = crate::services::wix_api::WixApiClient::new(api_key, account_id);
    
    match client.insert_collection_item(&site_id, "Books", book_data).await {
        Ok(book) => {
            tracing::info!("Book created for site {} by tenant {}", site_id, auth.tenant_id);
            Ok(Json(book))
        },
        Err(e) => {
            tracing::error!("Failed to create Wix book for site {}: {}", site_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Update book in Wix with Casbin authorization (owner-only)
pub async fn update_wix_book(
    Path((site_id, book_id)): Path<(String, String)>,
    State(state): State<AppState>,
    auth: CasbinAuthContext,
    Json(book_data): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Check Casbin permission for books update
    auth.require_permission("books", "update").await?;
    
    tracing::info!(
        "Updating book {} for site {} by user: {} (tenant: {}, role: {:?})", 
        book_id, site_id, auth.user_id, auth.tenant_id, auth.user_role
    );
    
    // Verify site ownership - users can only update books for sites they own
    let service = ConnectedWebsitesService::new(state.db.clone());
    let owns_site = service.verify_site_ownership(&site_id, auth.tenant_id).await
        .map_err(|e| {
            tracing::error!("Failed to verify site ownership: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    if !owns_site {
        tracing::warn!(
            "Tenant {} attempted to update book {} for site {} - access denied (not site owner)", 
            auth.tenant_id, book_id, site_id
        );
        return Err(StatusCode::FORBIDDEN);
    }
    let api_key = std::env::var("QUILLSPACE_WIX_API_KEY")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let account_id = std::env::var("QUILLSPACE_WIX_ACCOUNT_ID")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let client = crate::services::wix_api::WixApiClient::new(api_key, account_id);
    
    match client.update_collection_item(&site_id, "Books", &book_id, book_data).await {
        Ok(book) => {
            tracing::info!("Book {} updated for site {} by tenant {}", book_id, site_id, auth.tenant_id);
            Ok(Json(book))
        },
        Err(e) => {
            tracing::error!("Failed to update Wix book {} for site {}: {}", book_id, site_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Partially update book in Wix (PATCH) with Casbin authorization (owner-only)
pub async fn patch_wix_book(
    Path((site_id, book_id)): Path<(String, String)>,
    State(state): State<AppState>,
    auth: CasbinAuthContext,
    Json(patch_data): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Check Casbin permission for books update
    auth.require_permission("books", "update").await?;
    
    tracing::info!(
        "Patching book {} for site {} by user: {} (tenant: {}, role: {:?})", 
        book_id, site_id, auth.user_id, auth.tenant_id, auth.user_role
    );
    
    // Verify site ownership - users can only patch books for sites they own
    let service = ConnectedWebsitesService::new(state.db.clone());
    let owns_site = service.verify_site_ownership(&site_id, auth.tenant_id).await
        .map_err(|e| {
            tracing::error!("Failed to verify site ownership: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    if !owns_site {
        tracing::warn!(
            "Tenant {} attempted to patch book {} for site {} - access denied (not site owner)", 
            auth.tenant_id, book_id, site_id
        );
        return Err(StatusCode::FORBIDDEN);
    }
    let api_key = std::env::var("QUILLSPACE_WIX_API_KEY")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let account_id = std::env::var("QUILLSPACE_WIX_ACCOUNT_ID")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let client = crate::services::wix_api::WixApiClient::new(api_key, account_id);
    
    match client.patch_collection_item(&site_id, "Books", &book_id, patch_data).await {
        Ok(book) => {
            tracing::info!("Book {} patched for site {} by tenant {}", book_id, site_id, auth.tenant_id);
            Ok(Json(book))
        },
        Err(e) => {
            tracing::error!("Failed to patch Wix book {} for site {}: {}", book_id, site_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Legacy: Get Wix books (defaults to Yasin's site for backward compatibility)
pub async fn get_wix_books_legacy(auth: CasbinAuthContext) -> Result<Json<serde_json::Value>, StatusCode> {
    // Check Casbin permission for books read
    auth.require_permission("books", "read").await?;
    
    // Default to Yasin's site for backward compatibility
    let default_site_id = "1e4e0091-f4d5-4a4c-a66a-4d09e7a5b4e9";
    
    // Only Yasin's tenant can access the legacy endpoint
    let yasin_tenant_id = "22222222-2222-2222-2222-222222222222";
    
    if auth.tenant_id.to_string() != yasin_tenant_id {
        tracing::warn!(
            "Tenant {} attempted to access legacy books endpoint - access denied", 
            auth.tenant_id
        );
        return Err(StatusCode::FORBIDDEN);
    }
    
    let api_key = std::env::var("QUILLSPACE_WIX_API_KEY")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let account_id = std::env::var("QUILLSPACE_WIX_ACCOUNT_ID")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let client = crate::services::wix_api::WixApiClient::new(api_key, account_id);
    
    match client.get_collection_items(default_site_id, "Books").await {
        Ok(books) => Ok(Json(books)),
        Err(e) => {
            tracing::error!("Failed to get legacy Wix books: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Legacy: Get single Wix book by ID (defaults to Yasin's site for backward compatibility)
pub async fn get_single_wix_book_legacy(
    Path(book_id): Path<String>,
    auth: CasbinAuthContext,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Check Casbin permission for books read
    auth.require_permission("books", "read").await?;
    
    // Default to Yasin's site for backward compatibility
    let default_site_id = "1e4e0091-f4d5-4a4c-a66a-4d09e7a5b4e9";
    
    // Only Yasin's tenant can access the legacy endpoint
    let yasin_tenant_id = "22222222-2222-2222-2222-222222222222";
    
    if auth.tenant_id.to_string() != yasin_tenant_id {
        tracing::warn!(
            "Tenant {} attempted to access legacy book {} - access denied", 
            auth.tenant_id, book_id
        );
        return Err(StatusCode::FORBIDDEN);
    }
    
    let api_key = std::env::var("QUILLSPACE_WIX_API_KEY")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let account_id = std::env::var("QUILLSPACE_WIX_ACCOUNT_ID")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let client = crate::services::wix_api::WixApiClient::new(api_key, account_id);
    
    match client.get_collection_item(default_site_id, "Books", &book_id).await {
        Ok(book) => Ok(Json(book)),
        Err(e) => {
            tracing::error!("Failed to get legacy Wix book {}: {}", book_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get Wix author info
pub async fn get_wix_author_info() -> Result<Json<serde_json::Value>, StatusCode> {
    let api_key = std::env::var("QUILLSPACE_WIX_API_KEY")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let account_id = std::env::var("QUILLSPACE_WIX_ACCOUNT_ID")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let client = crate::services::wix_api::WixApiClient::new(api_key, account_id);
    
    match client.get_collection_items("1e4e0091-f4d5-4a4c-a66a-4d09e7a5b4e9", "AuthorInfo").await {
        Ok(author) => Ok(Json(author)),
        Err(e) => {
            tracing::error!("Failed to get Wix author info: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Update Wix author info
pub async fn update_wix_author_info(
    Json(author_data): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let api_key = std::env::var("QUILLSPACE_WIX_API_KEY")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let account_id = std::env::var("QUILLSPACE_WIX_ACCOUNT_ID")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let client = crate::services::wix_api::WixApiClient::new(api_key, account_id);
    
    // Get existing AuthorInfo to update it
    match client.get_collection_items("1e4e0091-f4d5-4a4c-a66a-4d09e7a5b4e9", "AuthorInfo").await {
        Ok(existing_data) => {
            if let Some(items) = existing_data.get("dataItems").and_then(|v| v.as_array()) {
                if let Some(first_item) = items.first() {
                    if let Some(item_id) = first_item.get("id").and_then(|v| v.as_str()) {
                        return match client.update_collection_item("1e4e0091-f4d5-4a4c-a66a-4d09e7a5b4e9", "AuthorInfo", item_id, author_data).await {
                            Ok(author) => Ok(Json(author)),
                            Err(e) => {
                                tracing::error!("Failed to update Wix author info: {}", e);
                                Err(StatusCode::INTERNAL_SERVER_ERROR)
                            }
                        };
                    }
                }
            }
            // No existing author info, create new one
            match client.insert_collection_item("1e4e0091-f4d5-4a4c-a66a-4d09e7a5b4e9", "AuthorInfo", author_data).await {
                Ok(author) => Ok(Json(author)),
                Err(e) => {
                    tracing::error!("Failed to create Wix author info: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to get existing Wix author info: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Create new book in Wix with proper field types
pub async fn create_wix_book_with_proper_types(
    Json(book_data): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let api_key = std::env::var("QUILLSPACE_WIX_API_KEY")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let account_id = std::env::var("QUILLSPACE_WIX_ACCOUNT_ID")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let client = crate::services::wix_api::WixApiClient::new(api_key, account_id);
    let site_id = "1e4e0091-f4d5-4a4c-a66a-4d09e7a5b4e9";
    let collection_id = "Books";
    
    // First, ensure the priceAmount field exists with proper type (since price is already wrong type)
    match client.ensure_collection_field(site_id, collection_id, "bookPrice", "NUMBER", "Book Price").await {
        Ok(_) => {
            tracing::info!("PriceAmount field ensured as number type");
        }
        Err(e) => {
            tracing::warn!("Could not ensure priceAmount field type: {}", e);
            // Continue anyway, field might already exist
        }
    }
    
    // Create the book with the data
    match client.insert_collection_item(site_id, collection_id, book_data).await {
        Ok(book) => Ok(Json(book)),
        Err(e) => {
            tracing::error!("Failed to create Wix book with schema: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
