use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    auth::jwt_helpers::extract_auth_context,
    services::page::{CreatePageRequest, PageService, PublishPageRequest, UpdatePageRequest},
    services::pages::{PageService as PuckPageService, SavePageDraftRequest, SwitchTemplateRequest},
    types::ApiResponse,
    AppState,
};

/// Page list query parameters
#[derive(Debug, Deserialize)]
pub struct PageListQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Page response for API
#[derive(Debug, Serialize)]
pub struct PageResponse {
    pub id: Uuid,
    pub site_id: Uuid,
    pub slug: String,
    pub title: String,
    pub meta_description: Option<String>,
    pub meta_keywords: Option<String>,
    pub is_published: bool,
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,
    pub sort_order: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Page detail response (includes Puck data)
#[derive(Debug, Serialize)]
pub struct PageDetailResponse {
    pub id: Uuid,
    pub site_id: Uuid,
    pub slug: String,
    pub title: String,
    pub meta_description: Option<String>,
    pub meta_keywords: Option<String>,
    pub puck_data: serde_json::Value,
    pub is_published: bool,
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,
    pub sort_order: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Page reorder request
#[derive(Debug, Deserialize)]
pub struct ReorderPagesRequest {
    pub page_orders: Vec<PageOrder>,
}

#[derive(Debug, Deserialize)]
pub struct PageOrder {
    pub page_id: Uuid,
    pub sort_order: i32,
}

pub fn pages_router() -> Router<AppState> {
    Router::new()
        .route("/sites/:site_id/pages", get(list_pages).post(create_page))
        .route("/sites/:site_id/pages/reorder", post(reorder_pages))
        .route("/pages/:page_id", get(get_page).put(update_page).delete(delete_page))
        .route("/pages/:page_id/publish", post(publish_page))
        .route("/pages/:page_id/unpublish", post(unpublish_page))
        // New Puck/MiniJinja endpoints
        .route("/pages/:page_id/draft", put(save_page_draft))
        .route("/pages/:page_id/template", put(switch_page_template))
        .route("/pages/:page_id/preview-link", post(generate_preview_link))
        .route("/preview/:token", get(render_preview_page))
}

/// List pages for a site
pub async fn list_pages(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(site_id): Path<Uuid>,
    Query(query): Query<PageListQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let (tenant_id, _user_id) = extract_auth_context(&headers, &state.jwt_manager)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let request_id = Uuid::new_v4();

    let limit = query.limit.unwrap_or(50).min(100);
    let offset = query.offset.unwrap_or(0);

    let page_service = PageService::new(state.db.postgres().clone());

    match page_service.list_pages(&tenant_id, site_id, limit, offset).await {
        Ok(pages) => {
            let response_pages: Vec<PageResponse> = pages
                .into_iter()
                .map(|p| PageResponse {
                    id: p.id,
                    site_id: p.site_id,
                    slug: p.slug,
                    title: p.title,
                    meta_description: p.meta_description,
                    meta_keywords: p.meta_keywords,
                    is_published: p.is_published,
                    published_at: p.published_at,
                    sort_order: p.sort_order,
                    created_at: p.created_at,
                    updated_at: p.updated_at,
                })
                .collect();

            let response = ApiResponse::success(response_pages, request_id);
            Ok((StatusCode::OK, Json(response)))
        }
        Err(e) => {
            error!("Failed to list pages: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get page by ID
pub async fn get_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(page_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let (tenant_id, _user_id) = extract_auth_context(&headers, &state.jwt_manager)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let request_id = Uuid::new_v4();

    let page_service = PageService::new(state.db.postgres().clone());

    match page_service.get_page(&tenant_id, page_id).await {
        Ok(Some(page)) => {
            let response_page = PageDetailResponse {
                id: page.id,
                site_id: page.site_id,
                slug: page.slug,
                title: page.title,
                meta_description: page.meta_description,
                meta_keywords: page.meta_keywords,
                puck_data: page.puck_data,
                is_published: page.is_published,
                published_at: page.published_at,
                sort_order: page.sort_order,
                created_at: page.created_at,
                updated_at: page.updated_at,
            };

            let response = ApiResponse::success(response_page, request_id);
            Ok((StatusCode::OK, Json(response)))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get page: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Create new page
pub async fn create_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(site_id): Path<Uuid>,
    Json(request): Json<CreatePageRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let (tenant_id, _user_id) = extract_auth_context(&headers, &state.jwt_manager)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let request_id = Uuid::new_v4();

    // Validate input
    if request.title.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    if request.slug.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let page_service = PageService::new(state.db.postgres().clone());

    match page_service.create_page(&tenant_id, site_id, request).await {
        Ok(page) => {
            info!("Created page {} for site {} tenant {}", page.id, site_id, tenant_id);

            let response_page = PageDetailResponse {
                id: page.id,
                site_id: page.site_id,
                slug: page.slug,
                title: page.title,
                meta_description: page.meta_description,
                meta_keywords: page.meta_keywords,
                puck_data: page.puck_data,
                is_published: page.is_published,
                published_at: page.published_at,
                sort_order: page.sort_order,
                created_at: page.created_at,
                updated_at: page.updated_at,
            };

            let response = ApiResponse::success(response_page, request_id);
            Ok((StatusCode::CREATED, Json(response)))
        }
        Err(e) => {
            error!("Failed to create page: {}", e);
            if e.to_string().contains("already exists") {
                Err(StatusCode::CONFLICT)
            } else if e.to_string().contains("not found") || e.to_string().contains("access denied") {
                Err(StatusCode::NOT_FOUND)
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

/// Update page
pub async fn update_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(page_id): Path<Uuid>,
    Json(request): Json<UpdatePageRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let (tenant_id, _user_id) = extract_auth_context(&headers, &state.jwt_manager)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let request_id = Uuid::new_v4();

    let page_service = PageService::new(state.db.postgres().clone());

    match page_service.update_page(&tenant_id, page_id, request).await {
        Ok(Some(page)) => {
            let response_page = PageDetailResponse {
                id: page.id,
                site_id: page.site_id,
                slug: page.slug,
                title: page.title,
                meta_description: page.meta_description,
                meta_keywords: page.meta_keywords,
                puck_data: page.puck_data,
                is_published: page.is_published,
                published_at: page.published_at,
                sort_order: page.sort_order,
                created_at: page.created_at,
                updated_at: page.updated_at,
            };

            let response = ApiResponse::success(response_page, request_id);
            Ok((StatusCode::OK, Json(response)))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to update page: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Delete page
pub async fn delete_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(page_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let (tenant_id, _user_id) = extract_auth_context(&headers, &state.jwt_manager)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let request_id = Uuid::new_v4();

    let page_service = PageService::new(state.db.postgres().clone());

    match page_service.delete_page(&tenant_id, page_id).await {
        Ok(true) => {
            info!("Deleted page {} for tenant {}", page_id, tenant_id);
            let response = ApiResponse::success((), request_id);
            Ok((StatusCode::OK, Json(response)))
        }
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to delete page: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Publish page
pub async fn publish_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(page_id): Path<Uuid>,
    Json(request): Json<PublishPageRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let (tenant_id, _user_id) = extract_auth_context(&headers, &state.jwt_manager)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let request_id = Uuid::new_v4();

    let page_service = PageService::new(state.db.postgres().clone());

    match page_service.publish_page(&tenant_id, page_id, request).await {
        Ok(Some(page)) => {
            info!("Published page {} for tenant {}", page_id, tenant_id);

            let response_page = PageDetailResponse {
                id: page.id,
                site_id: page.site_id,
                slug: page.slug,
                title: page.title,
                meta_description: page.meta_description,
                meta_keywords: page.meta_keywords,
                puck_data: page.puck_data,
                is_published: page.is_published,
                published_at: page.published_at,
                sort_order: page.sort_order,
                created_at: page.created_at,
                updated_at: page.updated_at,
            };

            let response = ApiResponse::success(response_page, request_id);
            Ok((StatusCode::OK, Json(response)))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to publish page: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Unpublish page
pub async fn unpublish_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(page_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let (tenant_id, _user_id) = extract_auth_context(&headers, &state.jwt_manager)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let request_id = Uuid::new_v4();

    let page_service = PageService::new(state.db.postgres().clone());

    match page_service.unpublish_page(&tenant_id, page_id).await {
        Ok(Some(page)) => {
            info!("Unpublished page {} for tenant {}", page_id, tenant_id);

            let response_page = PageDetailResponse {
                id: page.id,
                site_id: page.site_id,
                slug: page.slug,
                title: page.title,
                meta_description: page.meta_description,
                meta_keywords: page.meta_keywords,
                puck_data: page.puck_data,
                is_published: page.is_published,
                published_at: page.published_at,
                sort_order: page.sort_order,
                created_at: page.created_at,
                updated_at: page.updated_at,
            };

            let response = ApiResponse::success(response_page, request_id);
            Ok((StatusCode::OK, Json(response)))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to unpublish page: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Reorder pages within a site
pub async fn reorder_pages(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(site_id): Path<Uuid>,
    Json(request): Json<ReorderPagesRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let (tenant_id, _user_id) = extract_auth_context(&headers, &state.jwt_manager)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let request_id = Uuid::new_v4();

    let page_service = PageService::new(state.db.postgres().clone());

    let page_orders: Vec<(Uuid, i32)> = request
        .page_orders
        .into_iter()
        .map(|order| (order.page_id, order.sort_order))
        .collect();

    match page_service.reorder_pages(&tenant_id, site_id, page_orders).await {
        Ok(()) => {
            info!("Reordered pages for site {} tenant {}", site_id, tenant_id);
            let response = ApiResponse::success((), request_id);
            Ok((StatusCode::OK, Json(response)))
        }
        Err(e) => {
            error!("Failed to reorder pages: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Save page draft (Puck composition JSON)
pub async fn save_page_draft(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(page_id): Path<Uuid>,
    Json(request): Json<SavePageDraftRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let (tenant_id, _user_id) = extract_auth_context(&headers, &state.jwt_manager)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let request_id = Uuid::new_v4();

    // Create Puck page service with template cache
    let template_cache = state.template_cache.clone();
    let render_defaults = state.render_defaults.clone();
    let puck_service = PuckPageService::new(
        state.db.postgres().clone(),
        template_cache,
        render_defaults,
    );

    match puck_service.save_draft(page_id, tenant_id, request).await {
        Ok(page) => {
            info!("Saved draft for page {} tenant {}", page_id, tenant_id);

            let response_page = PageDetailResponse {
                id: page.id,
                site_id: page.site_id,
                slug: page.slug,
                title: page.title,
                meta_description: None, // TODO: Extract from composition
                meta_keywords: None,
                puck_data: serde_json::to_value(&page.draft_composition).unwrap_or_default(),
                is_published: page.is_published,
                published_at: None, // TODO: Add published_at to new Page struct
                sort_order: 0, // TODO: Add sort_order to new Page struct
                created_at: page.created_at,
                updated_at: page.updated_at,
            };

            let response = ApiResponse::success(response_page, request_id);
            Ok((StatusCode::OK, Json(response)))
        }
        Err(e) => {
            error!("Failed to save page draft: {}", e);
            match e {
                crate::services::pages::PageServiceError::PageNotFound(_) => Err(StatusCode::NOT_FOUND),
                crate::services::pages::PageServiceError::TemplateNotFound(_) => Err(StatusCode::BAD_REQUEST),
                _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
    }
}

/// Switch page template
pub async fn switch_page_template(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(page_id): Path<Uuid>,
    Json(request): Json<SwitchTemplateRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let (tenant_id, _user_id) = extract_auth_context(&headers, &state.jwt_manager)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let request_id = Uuid::new_v4();

    let template_cache = state.template_cache.clone();
    let render_defaults = state.render_defaults.clone();
    let puck_service = PuckPageService::new(
        state.db.postgres().clone(),
        template_cache,
        render_defaults,
    );

    match puck_service.switch_template(page_id, tenant_id, request).await {
        Ok(page) => {
            info!("Switched template for page {} tenant {}", page_id, tenant_id);

            let response_page = PageDetailResponse {
                id: page.id,
                site_id: page.site_id,
                slug: page.slug,
                title: page.title,
                meta_description: None,
                meta_keywords: None,
                puck_data: serde_json::to_value(&page.draft_composition).unwrap_or_default(),
                is_published: page.is_published,
                published_at: None,
                sort_order: 0,
                created_at: page.created_at,
                updated_at: page.updated_at,
            };

            let response = ApiResponse::success(response_page, request_id);
            Ok((StatusCode::OK, Json(response)))
        }
        Err(e) => {
            error!("Failed to switch page template: {}", e);
            match e {
                crate::services::pages::PageServiceError::PageNotFound(_) => Err(StatusCode::NOT_FOUND),
                crate::services::pages::PageServiceError::TemplateNotFound(_) => Err(StatusCode::BAD_REQUEST),
                _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
    }
}

/// Generate preview link for page
pub async fn generate_preview_link(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(page_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let (tenant_id, _user_id) = extract_auth_context(&headers, &state.jwt_manager)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let request_id = Uuid::new_v4();

    let template_cache = state.template_cache.clone();
    let render_defaults = state.render_defaults.clone();
    let puck_service = PuckPageService::new(
        state.db.postgres().clone(),
        template_cache,
        render_defaults,
    );

    let base_url = state.config.base_url.as_deref().unwrap_or("http://localhost:3000");

    match puck_service.generate_preview_link(page_id, tenant_id, base_url).await {
        Ok(preview_response) => {
            let response = ApiResponse::success(preview_response, request_id);
            Ok((StatusCode::OK, Json(response)))
        }
        Err(e) => {
            error!("Failed to generate preview link: {}", e);
            match e {
                crate::services::pages::PageServiceError::PageNotFound(_) => Err(StatusCode::NOT_FOUND),
                _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
    }
}

/// Render preview page from token
pub async fn render_preview_page(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let template_cache = state.template_cache.clone();
    let render_defaults = state.render_defaults.clone();
    let puck_service = PuckPageService::new(
        state.db.postgres().clone(),
        template_cache,
        render_defaults,
    );

    // Parse token to get tenant_id and page_id
    let (tenant_id, page_id, _expires_at) = match puck_service.parse_preview_token(&token) {
        Ok(parsed) => parsed,
        Err(e) => {
            error!("Invalid preview token: {}", e);
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    // Render preview HTML
    match puck_service.render_preview(page_id, tenant_id, None).await {
        Ok(html) => {
            let headers = [
                ("content-type", "text/html; charset=utf-8"),
                ("cache-control", "no-cache, no-store, must-revalidate"),
            ];
            Ok((StatusCode::OK, headers, html))
        }
        Err(e) => {
            error!("Failed to render preview: {}", e);
            match e {
                crate::services::pages::PageServiceError::PageNotFound(_) => Err(StatusCode::NOT_FOUND),
                _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
    }
}
