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
    auth::jwt_helpers::{extract_auth_context, extract_auth_context_with_role},
    services::site::{CreateSiteRequest, SiteService, UpdateSiteRequest},
    types::ApiResponse,
    AppState,
};

/// Site list query parameters
#[derive(Debug, Deserialize)]
pub struct SiteListQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Site response for API (excludes sensitive data)
#[derive(Debug, Serialize)]
pub struct SiteResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub template_id: Option<Uuid>,
    pub custom_domain: Option<String>,
    pub subdomain: String,
    pub is_published: bool,
    pub build_status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Site detail response (includes theme config and SEO settings)
#[derive(Debug, Serialize)]
pub struct SiteDetailResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub template_id: Option<Uuid>,
    pub custom_domain: Option<String>,
    pub subdomain: String,
    pub is_published: bool,
    pub seo_settings: serde_json::Value,
    pub build_status: String,
    pub theme_config: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Subdomain availability check request
#[derive(Debug, Deserialize)]
pub struct SubdomainCheckQuery {
    pub subdomain: String,
}

pub fn sites_router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_sites).post(create_site))
        .route("/:site_id", get(get_site).put(update_site).delete(delete_site))
        .route("/:site_id/publish", post(publish_site))
        .route("/:site_id/unpublish", post(unpublish_site))
        .route("/check-subdomain", get(check_subdomain_availability))
}

/// List sites for the authenticated tenant
pub async fn list_sites(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<SiteListQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let auth_context = extract_auth_context_with_role(&headers, &state.jwt_manager)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let request_id = Uuid::new_v4();

    // TODO: Re-enable Casbin authorization after testing
    // state.authorizer
    //     .require_permission(&auth_context.user_role, Resource::Sites.as_str(), Action::Read.as_str(), &auth_context.tenant_id.to_string())
    //     .await?;

    let limit = query.limit.unwrap_or(20).min(100);
    let offset = query.offset.unwrap_or(0);

    let site_service = SiteService::new(state.db.postgres().clone());

    match site_service.list_sites(&auth_context.tenant_id, limit, offset).await {
        Ok(sites) => {
            let response_sites: Vec<SiteResponse> = sites
                .into_iter()
                .map(|s| SiteResponse {
                    id: s.id,
                    name: s.name,
                    description: s.description,
                    template_id: s.template_id,
                    custom_domain: s.custom_domain,
                    subdomain: s.subdomain,
                    is_published: s.is_published,
                    build_status: s.build_status,
                    created_at: s.created_at,
                    updated_at: s.updated_at,
                })
                .collect();

            let response = ApiResponse::success(response_sites, request_id);
            Ok((StatusCode::OK, Json(response)))
        }
        Err(e) => {
            error!("Failed to list sites: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get site by ID
pub async fn get_site(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(site_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let (tenant_id, _user_id) = extract_auth_context(&headers, &state.jwt_manager)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let request_id = Uuid::new_v4();

    let site_service = SiteService::new(state.db.postgres().clone());

    match site_service.get_site(&tenant_id, site_id).await {
        Ok(Some(site)) => {
            let response_site = SiteDetailResponse {
                id: site.id,
                name: site.name,
                description: site.description,
                template_id: site.template_id,
                custom_domain: site.custom_domain,
                subdomain: site.subdomain,
                is_published: site.is_published,
                seo_settings: site.seo_settings,
                build_status: site.build_status,
                theme_config: site.theme_config,
                created_at: site.created_at,
                updated_at: site.updated_at,
            };

            let response = ApiResponse::success(response_site, request_id);
            Ok((StatusCode::OK, Json(response)))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get site: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Create new site
pub async fn create_site(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateSiteRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let (tenant_id, _user_id) = extract_auth_context(&headers, &state.jwt_manager)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let request_id = Uuid::new_v4();

    // Validate input
    if request.name.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let site_service = SiteService::new(state.db.postgres().clone());

    match site_service.create_site(&tenant_id, request).await {
        Ok(site) => {
            info!("Created site {} for tenant {}", site.id, tenant_id);

            let response_site = SiteDetailResponse {
                id: site.id,
                name: site.name,
                description: site.description,
                template_id: site.template_id,
                custom_domain: site.custom_domain,
                subdomain: site.subdomain,
                is_published: site.is_published,
                seo_settings: site.seo_settings,
                build_status: site.build_status,
                theme_config: site.theme_config,
                created_at: site.created_at,
                updated_at: site.updated_at,
            };

            let response = ApiResponse::success(response_site, request_id);
            Ok((StatusCode::CREATED, Json(response)))
        }
        Err(e) => {
            error!("Failed to create site: {}", e);
            if e.to_string().contains("already taken") || e.to_string().contains("reserved") {
                Err(StatusCode::CONFLICT)
            } else if e.to_string().contains("invalid") || e.to_string().contains("cannot") {
                Err(StatusCode::BAD_REQUEST)
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

/// Update site
pub async fn update_site(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(site_id): Path<Uuid>,
    Json(request): Json<UpdateSiteRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let (tenant_id, _user_id) = extract_auth_context(&headers, &state.jwt_manager)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let request_id = Uuid::new_v4();

    let site_service = SiteService::new(state.db.postgres().clone());

    match site_service.update_site(&tenant_id, site_id, request).await {
        Ok(Some(site)) => {
            let response_site = SiteDetailResponse {
                id: site.id,
                name: site.name,
                description: site.description,
                template_id: site.template_id,
                custom_domain: site.custom_domain,
                subdomain: site.subdomain,
                is_published: site.is_published,
                seo_settings: site.seo_settings,
                build_status: site.build_status,
                theme_config: site.theme_config,
                created_at: site.created_at,
                updated_at: site.updated_at,
            };

            let response = ApiResponse::success(response_site, request_id);
            Ok((StatusCode::OK, Json(response)))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to update site: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Delete site
pub async fn delete_site(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(site_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let (tenant_id, _user_id) = extract_auth_context(&headers, &state.jwt_manager)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let request_id = Uuid::new_v4();

    let site_service = SiteService::new(state.db.postgres().clone());

    match site_service.delete_site(&tenant_id, site_id).await {
        Ok(true) => {
            info!("Deleted site {} for tenant {}", site_id, tenant_id);
            let response = ApiResponse::success((), request_id);
            Ok((StatusCode::OK, Json(response)))
        }
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to delete site: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Publish site
pub async fn publish_site(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(site_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let (tenant_id, _user_id) = extract_auth_context(&headers, &state.jwt_manager)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let request_id = Uuid::new_v4();

    let site_service = SiteService::new(state.db.postgres().clone());

    match site_service.publish_site(&tenant_id, site_id).await {
        Ok(Some(site)) => {
            info!("Published site {} for tenant {}", site_id, tenant_id);

            let response_site = SiteDetailResponse {
                id: site.id,
                name: site.name,
                description: site.description,
                template_id: site.template_id,
                custom_domain: site.custom_domain,
                subdomain: site.subdomain,
                is_published: site.is_published,
                seo_settings: site.seo_settings,
                build_status: site.build_status,
                theme_config: site.theme_config,
                created_at: site.created_at,
                updated_at: site.updated_at,
            };

            let response = ApiResponse::success(response_site, request_id);
            Ok((StatusCode::OK, Json(response)))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to publish site: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Unpublish site
pub async fn unpublish_site(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(site_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let (tenant_id, _user_id) = extract_auth_context(&headers, &state.jwt_manager)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let request_id = Uuid::new_v4();

    let site_service = SiteService::new(state.db.postgres().clone());

    match site_service.unpublish_site(&tenant_id, site_id).await {
        Ok(Some(site)) => {
            info!("Unpublished site {} for tenant {}", site_id, tenant_id);

            let response_site = SiteDetailResponse {
                id: site.id,
                name: site.name,
                description: site.description,
                template_id: site.template_id,
                custom_domain: site.custom_domain,
                subdomain: site.subdomain,
                is_published: site.is_published,
                seo_settings: site.seo_settings,
                build_status: site.build_status,
                theme_config: site.theme_config,
                created_at: site.created_at,
                updated_at: site.updated_at,
            };

            let response = ApiResponse::success(response_site, request_id);
            Ok((StatusCode::OK, Json(response)))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to unpublish site: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Check subdomain availability
pub async fn check_subdomain_availability(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<SubdomainCheckQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let (_tenant_id, _user_id) = extract_auth_context(&headers, &state.jwt_manager)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let request_id = Uuid::new_v4();

    let site_service = SiteService::new(state.db.postgres().clone());

    match site_service.is_subdomain_available(&query.subdomain).await {
        Ok(is_available) => {
            let response = ApiResponse::success(
                serde_json::json!({
                    "subdomain": query.subdomain,
                    "available": is_available
                }),
                request_id,
            );
            Ok((StatusCode::OK, Json(response)))
        }
        Err(e) => {
            error!("Failed to check subdomain availability: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
