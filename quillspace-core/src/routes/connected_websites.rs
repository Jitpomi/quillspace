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
    services::{
        connected_websites::{ConnectedWebsitesService, ConnectedWebsite},
        wix_data_types::WixDataTypeValidator,
    },
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
        .route("/websites/:website_id", get(get_single_website))
        .route("/wix/sites/:site_id/books", get(get_wix_books_simple))
        .route("/wix/sites/:site_id/books", post(create_wix_book))
        .route("/wix/sites/:site_id/books/with-schema", post(create_wix_book_with_proper_types))
        .route("/wix/sites/:site_id/books/:book_id", get(get_single_wix_book))
        .route("/wix/sites/:site_id/books/:book_id", put(update_wix_book))
        .route("/wix/sites/:site_id/books/:book_id", patch(patch_wix_book))
        .route("/wix/sites/:site_id/authors", get(get_wix_authors_for_site))
        .route("/wix/sites/:site_id/authors", post(create_wix_author_for_site))
        .route("/wix/sites/:site_id/authors/:author_id", put(update_wix_author))
        .route("/wix/sites/:site_id/authors/:author_id", patch(patch_wix_author))
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

/// Get a single connected website by website_id (supports UUID, wix_site_id, or external_site_id)
pub async fn get_single_website(
    Path(website_id): Path<String>,
    State(state): State<AppState>,
    auth: CasbinAuthContext,
) -> Result<Json<ConnectedWebsite>, StatusCode> {
    // Check Casbin permission for connected_websites read
    auth.require_permission("connected_websites", "read").await?;
    
    tracing::info!(
        "Getting website with id {} for user: {} (tenant: {}, role: {:?})", 
        website_id, auth.user_id, auth.tenant_id, auth.user_role
    );
    
    let service = ConnectedWebsitesService::new(state.db.clone());
    
    // Get all websites for user and filter by website_id
    match service.get_user_websites_with_tenant(auth.user_id, auth.tenant_id).await {
        Ok(websites) => {
            tracing::info!("Got {} websites, looking for website_id: {}", websites.len(), website_id);
            
            // Find website with matching ID - try multiple matching strategies
            if let Some(website) = websites.into_iter().find(|w| {
                // Try internal UUID id first
                if w.id.to_string() == website_id {
                    tracing::info!("Found match via internal UUID id");
                    return true;
                }
                
                // Try external_site_id (Wix site ID)
                if w.external_site_id == website_id {
                    tracing::info!("Found match via external_site_id");
                    return true;
                }
                
                // Try metadata.wix_site_id
                if let Some(metadata_id) = w.metadata.get("wix_site_id").and_then(|v| v.as_str()) {
                    if metadata_id == website_id {
                        tracing::info!("Found match via metadata.wix_site_id");
                        return true;
                    }
                }
                
                false
            }) {
                tracing::info!("Found website with id {} for user {}", website_id, auth.user_id);
                Ok(Json(website))
            } else {
                tracing::warn!("Website with id {} not found for user {}", website_id, auth.user_id);
                Err(StatusCode::NOT_FOUND)
            }
        },
        Err(e) => {
            tracing::error!("Failed to get websites for user {}: {}", auth.user_id, e);
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

    // Validate and format data according to Wix data type requirements
    let validated_data = if let Some(data_field) = book_data.get("data") {
        // Validate the nested data field
        match WixDataTypeValidator::validate_and_format_book_data(data_field) {
            Ok(validated) => {
                // Reconstruct the full payload with validated data
                let mut full_payload = book_data.clone();
                full_payload["data"] = validated;
                full_payload
            },
            Err(e) => {
                tracing::error!("Data validation failed for book creation: {}", e);
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    } else {
        // If no nested data field, validate the entire payload and wrap it
        match WixDataTypeValidator::validate_and_format_book_data(&book_data) {
            Ok(data) => {
                // Wrap the validated data in the proper Wix format
                serde_json::json!({
                    "data": data
                })
            },
            Err(e) => {
                tracing::error!("Data validation failed for book creation: {}", e);
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    };
    
    tracing::info!("Book data validated successfully for site {}", site_id);
    
    let client = crate::services::wix_api::WixApiClient::new(api_key, account_id);
    
    match client.insert_collection_item(&site_id, "Books", validated_data).await {
        Ok(book) => {
            tracing::info!("Book created for site {} by tenant {} with validated data types", site_id, auth.tenant_id);
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

    // Validate and format data according to Wix data type requirements
    let validated_data = if let Some(data_field) = book_data.get("data") {
        // Validate the nested data field
        match WixDataTypeValidator::validate_and_format_book_data(data_field) {
            Ok(validated) => {
                let mut full_payload = book_data.clone();
                full_payload["data"] = validated;
                full_payload
            },
            Err(e) => {
                tracing::error!("Data validation failed for book update: {}", e);
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    } else {
        match WixDataTypeValidator::validate_and_format_book_data(&book_data) {
            Ok(data) => data,
            Err(e) => {
                tracing::error!("Data validation failed for book update: {}", e);
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    };
    
    let client = crate::services::wix_api::WixApiClient::new(api_key, account_id);
    
    match client.update_collection_item(&site_id, "Books", &book_id, validated_data).await {
        Ok(book) => {
            tracing::info!("Book {} updated for site {} by tenant {} with validated data types", book_id, site_id, auth.tenant_id);
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

    // Validate and format patch data according to Wix data type requirements
    let validated_data = if let Some(data_field) = patch_data.get("data") {
        // Validate the nested data field
        match WixDataTypeValidator::validate_and_format_book_data(data_field) {
            Ok(validated) => {
                let mut full_payload = patch_data.clone();
                full_payload["data"] = validated;
                full_payload
            },
            Err(e) => {
                tracing::error!("Data validation failed for book patch: {}", e);
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    } else {
        match WixDataTypeValidator::validate_and_format_book_data(&patch_data) {
            Ok(data) => data,
            Err(e) => {
                tracing::error!("Data validation failed for book patch: {}", e);
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    };
    
    let client = crate::services::wix_api::WixApiClient::new(api_key, account_id);
    
    match client.patch_collection_item(&site_id, "Books", &book_id, validated_data).await {
        Ok(book) => {
            tracing::info!("Book {} patched for site {} by tenant {} with validated data types", book_id, site_id, auth.tenant_id);
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

/// Get Authors for a specific site with Casbin authorization
pub async fn get_wix_authors_for_site(
    Path(site_id): Path<String>,
    State(state): State<AppState>,
    auth: CasbinAuthContext,
) -> Result<Json<serde_json::Value>, StatusCode> {
    auth.require_permission("connected_websites", "read").await?;
    
    tracing::info!(
        "Getting authors for site {} by user: {} (tenant: {}, role: {:?})", 
        site_id, auth.user_id, auth.tenant_id, auth.user_role
    );
    
    // Verify site ownership
    let service = ConnectedWebsitesService::new(state.db.clone());
    let owns_site = service.verify_site_ownership(&site_id, auth.tenant_id).await
        .map_err(|e| {
            tracing::error!("Failed to verify site ownership: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    if !owns_site {
        tracing::warn!(
            "Tenant {} attempted to access authors for site {} - access denied (not site owner)", 
            auth.tenant_id, site_id
        );
        return Err(StatusCode::FORBIDDEN);
    }
    
    let api_key = std::env::var("QUILLSPACE_WIX_API_KEY")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let account_id = std::env::var("QUILLSPACE_WIX_ACCOUNT_ID")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let client = crate::services::wix_api::WixApiClient::new(api_key, account_id);
    
    match client.get_collection_items(&site_id, "Authors").await {
        Ok(authors) => {
            tracing::info!("Authors retrieved for site {} by tenant {}", site_id, auth.tenant_id);
            Ok(Json(authors))
        },
        Err(e) => {
            tracing::error!("Failed to get Wix authors for site {}: {}", site_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Create new author in Wix with proper validation
pub async fn create_wix_author_for_site(
    Path(site_id): Path<String>,
    State(state): State<AppState>,
    auth: CasbinAuthContext,
    Json(author_data): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    auth.require_permission("connected_websites", "create").await?;
    
    tracing::info!(
        "Creating author for site {} by user: {} (tenant: {}, role: {:?})", 
        site_id, auth.user_id, auth.tenant_id, auth.user_role
    );
    
    // Verify site ownership
    let service = ConnectedWebsitesService::new(state.db.clone());
    let owns_site = service.verify_site_ownership(&site_id, auth.tenant_id).await
        .map_err(|e| {
            tracing::error!("Failed to verify site ownership: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    if !owns_site {
        tracing::warn!(
            "Tenant {} attempted to create author for site {} - access denied (not site owner)", 
            auth.tenant_id, site_id
        );
        return Err(StatusCode::FORBIDDEN);
    }
    
    let api_key = std::env::var("QUILLSPACE_WIX_API_KEY")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let account_id = std::env::var("QUILLSPACE_WIX_ACCOUNT_ID")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Validate and format data according to Authors schema
    let validated_data = if let Some(data_field) = author_data.get("data") {
        match WixDataTypeValidator::validate_and_format_author_data(data_field) {
            Ok(validated) => {
                let mut full_payload = author_data.clone();
                full_payload["data"] = validated;
                full_payload
            },
            Err(e) => {
                tracing::error!("Data validation failed for author creation: {}", e);
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    } else {
        match WixDataTypeValidator::validate_and_format_author_data(&author_data) {
            Ok(data) => data,
            Err(e) => {
                tracing::error!("Data validation failed for author creation: {}", e);
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    };
    
    tracing::info!("Author data validated successfully for site {}", site_id);
    
    let client = crate::services::wix_api::WixApiClient::new(api_key, account_id);
    
    match client.insert_collection_item(&site_id, "Authors", validated_data).await {
        Ok(author) => {
            tracing::info!("Author created for site {} by tenant {} with validated data types", site_id, auth.tenant_id);
            Ok(Json(author))
        },
        Err(e) => {
            tracing::error!("Failed to create Wix author for site {}: {}", site_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Update author in Wix with Casbin authorization (owner-only)
pub async fn update_wix_author(
    Path((site_id, author_id)): Path<(String, String)>,
    State(state): State<AppState>,
    auth: CasbinAuthContext,
    Json(author_data): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    auth.require_permission("connected_websites", "update").await?;
    
    tracing::info!(
        "Updating author {} for site {} by user: {} (tenant: {}, role: {:?})", 
        author_id, site_id, auth.user_id, auth.tenant_id, auth.user_role
    );
    
    // Verify site ownership
    let service = ConnectedWebsitesService::new(state.db.clone());
    let owns_site = service.verify_site_ownership(&site_id, auth.tenant_id).await
        .map_err(|e| {
            tracing::error!("Failed to verify site ownership: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    if !owns_site {
        tracing::warn!(
            "Tenant {} attempted to update author {} for site {} - access denied (not site owner)", 
            auth.tenant_id, author_id, site_id
        );
        return Err(StatusCode::FORBIDDEN);
    }
    
    let api_key = std::env::var("QUILLSPACE_WIX_API_KEY")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let account_id = std::env::var("QUILLSPACE_WIX_ACCOUNT_ID")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Validate and format data according to Authors schema
    let validated_data = if let Some(data_field) = author_data.get("data") {
        match WixDataTypeValidator::validate_and_format_author_data(data_field) {
            Ok(validated) => {
                let mut full_payload = author_data.clone();
                full_payload["data"] = validated;
                full_payload
            },
            Err(e) => {
                tracing::error!("Data validation failed for author update: {}", e);
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    } else {
        match WixDataTypeValidator::validate_and_format_author_data(&author_data) {
            Ok(data) => data,
            Err(e) => {
                tracing::error!("Data validation failed for author update: {}", e);
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    };
    
    let client = crate::services::wix_api::WixApiClient::new(api_key, account_id);
    
    match client.update_collection_item(&site_id, "Authors", &author_id, validated_data).await {
        Ok(author) => {
            tracing::info!("Author {} updated for site {} by tenant {} with validated data types", author_id, site_id, auth.tenant_id);
            Ok(Json(author))
        },
        Err(e) => {
            tracing::error!("Failed to update Wix author {} for site {}: {}", author_id, site_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Partially update author in Wix (PATCH) with Casbin authorization (owner-only)
pub async fn patch_wix_author(
    Path((site_id, author_id)): Path<(String, String)>,
    State(state): State<AppState>,
    auth: CasbinAuthContext,
    Json(patch_data): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    auth.require_permission("connected_websites", "update").await?;
    
    tracing::info!(
        "Patching author {} for site {} by user: {} (tenant: {}, role: {:?})", 
        author_id, site_id, auth.user_id, auth.tenant_id, auth.user_role
    );
    
    // Verify site ownership
    let service = ConnectedWebsitesService::new(state.db.clone());
    let owns_site = service.verify_site_ownership(&site_id, auth.tenant_id).await
        .map_err(|e| {
            tracing::error!("Failed to verify site ownership: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    if !owns_site {
        tracing::warn!(
            "Tenant {} attempted to patch author {} for site {} - access denied (not site owner)", 
            auth.tenant_id, author_id, site_id
        );
        return Err(StatusCode::FORBIDDEN);
    }
    
    let api_key = std::env::var("QUILLSPACE_WIX_API_KEY")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let account_id = std::env::var("QUILLSPACE_WIX_ACCOUNT_ID")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Validate and format patch data according to Authors schema
    let validated_data = if let Some(data_field) = patch_data.get("data") {
        match WixDataTypeValidator::validate_and_format_author_data(data_field) {
            Ok(validated) => {
                let mut full_payload = patch_data.clone();
                full_payload["data"] = validated;
                full_payload
            },
            Err(e) => {
                tracing::error!("Data validation failed for author patch: {}", e);
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    } else {
        match WixDataTypeValidator::validate_and_format_author_data(&patch_data) {
            Ok(data) => data,
            Err(e) => {
                tracing::error!("Data validation failed for author patch: {}", e);
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    };
    
    let client = crate::services::wix_api::WixApiClient::new(api_key, account_id);
    
    match client.patch_collection_item(&site_id, "Authors", &author_id, validated_data).await {
        Ok(author) => {
            tracing::info!("Author {} patched for site {} by tenant {} with validated data types", author_id, site_id, auth.tenant_id);
            Ok(Json(author))
        },
        Err(e) => {
            tracing::error!("Failed to patch Wix author {} for site {}: {}", author_id, site_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
