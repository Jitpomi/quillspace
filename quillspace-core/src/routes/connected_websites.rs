use axum::{
    extract::{Path, Request, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put},
    Router,
};
use serde::Serialize;
use uuid::Uuid;
use crate::{
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
        .route("/wix/books", get(get_wix_books_simple))
        .route("/wix/books", post(create_wix_book))
        .route("/wix/books/with-schema", post(create_wix_book_with_proper_types))
        .route("/wix/books/:book_id", put(update_wix_book))
        .route("/wix/author", get(get_wix_author_info))
        .route("/wix/author", put(update_wix_author_info))
}

/// Get QuillSpace-built websites for the authenticated user
pub async fn get_user_websites(
    State(state): State<AppState>,
    request: Request,
) -> Result<Json<ConnectedWebsitesResponse>, StatusCode> {
    // Extract Authorization header
    let auth_header = request
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    // Extract token from "Bearer <token>"
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    // Decode and validate JWT token
    let claims = match state.jwt_manager.verify_token(token) {
        Ok(claims) => claims,
        Err(e) => {
            tracing::error!("JWT verification failed: {}", e);
            return Err(StatusCode::UNAUTHORIZED);
        }
    };
    
    // Get user ID from claims
    let user_id: Uuid = claims.sub.parse().map_err(|e| {
        tracing::error!("Failed to parse user ID from JWT: {}", e);
        StatusCode::UNAUTHORIZED
    })?;
    
    tracing::info!("Getting websites for user: {}", user_id);
    
    let service = ConnectedWebsitesService::new(state.db.clone());
    
    match service.get_user_websites(user_id).await {
        Ok(websites) => {
            tracing::info!("Found {} websites for user {}", websites.len(), user_id);
            Ok(Json(ConnectedWebsitesResponse { websites }))
        },
        Err(e) => {
            tracing::error!("Failed to get user websites: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get Wix books - SIMPLE VERSION
pub async fn get_wix_books_simple() -> Result<Json<serde_json::Value>, StatusCode> {
    let api_key = std::env::var("QUILLSPACE_WIX_API_KEY")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let account_id = std::env::var("QUILLSPACE_WIX_ACCOUNT_ID")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let client = crate::services::wix_api::WixApiClient::new(api_key, account_id);
    
    match client.get_collection_items("1e4e0091-f4d5-4a4c-a66a-4d09e7a5b4e9", "Books").await {
        Ok(books) => Ok(Json(books)),
        Err(e) => {
            tracing::error!("Failed to get Wix books: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Create new book in Wix
pub async fn create_wix_book(
    Json(book_data): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let api_key = std::env::var("QUILLSPACE_WIX_API_KEY")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let account_id = std::env::var("QUILLSPACE_WIX_ACCOUNT_ID")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let client = crate::services::wix_api::WixApiClient::new(api_key, account_id);
    
    match client.insert_collection_item("1e4e0091-f4d5-4a4c-a66a-4d09e7a5b4e9", "Books", book_data).await {
        Ok(book) => Ok(Json(book)),
        Err(e) => {
            tracing::error!("Failed to create Wix book: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Update book in Wix
pub async fn update_wix_book(
    Path(book_id): Path<String>,
    Json(book_data): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let api_key = std::env::var("QUILLSPACE_WIX_API_KEY")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let account_id = std::env::var("QUILLSPACE_WIX_ACCOUNT_ID")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let client = crate::services::wix_api::WixApiClient::new(api_key, account_id);
    
    match client.update_collection_item("1e4e0091-f4d5-4a4c-a66a-4d09e7a5b4e9", "Books", &book_id, book_data).await {
        Ok(book) => Ok(Json(book)),
        Err(e) => {
            tracing::error!("Failed to update Wix book: {}", e);
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
