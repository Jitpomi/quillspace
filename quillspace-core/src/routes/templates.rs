use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    auth::jwt_helpers::extract_auth_context,
    services::template_engine::{Template, TemplateEngine, SiteContext, PageContext},
    types::ApiResponse,
    AppState,
};

/// Template creation request
#[derive(Debug, Deserialize)]
pub struct CreateTemplateRequest {
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub html_source: String,
    pub default_schema: Value,
}

/// Template update request
#[derive(Debug, Deserialize)]
pub struct UpdateTemplateRequest {
    pub description: Option<String>,
    pub html_source: Option<String>,
    pub default_schema: Option<Value>,
}

/// Template list query parameters
#[derive(Debug, Deserialize)]
pub struct TemplateListQuery {
    pub category: Option<String>,
    pub include_public: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Template response for API
#[derive(Debug, Serialize)]
pub struct TemplateResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub preview_image_url: Option<String>,
    pub is_public: bool,
    pub version: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    // Note: html_source is excluded from API response for security
}

/// Template detail response (includes HTML source for authorized users)
#[derive(Debug, Serialize)]
pub struct TemplateDetailResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub html_source: String,
    pub default_schema: Value,
    pub preview_image_url: Option<String>,
    pub is_public: bool,
    pub version: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub fn templates_router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_templates).post(create_template))
        .route("/:template_id", get(get_template).put(update_template).delete(delete_template))
        .route("/:template_id/render", post(render_template))
        .route("/render-puck", post(render_puck_page))
        .route("/generate-static", post(generate_static_html))
}

/// List templates
pub async fn list_templates(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<TemplateListQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let (tenant_id, _user_id) = extract_auth_context(&headers, &state.jwt_manager)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let request_id = Uuid::new_v4();
    
    info!("LIST_TEMPLATES: tenant_id={}, query={:?}", tenant_id, query);

    let limit = query.limit.unwrap_or(20).min(100);
    let offset = query.offset.unwrap_or(0);
    let include_public = query.include_public.unwrap_or(true);

    match state.template_engine.list_templates(
        tenant_id.into(),
        include_public,
        query.category.as_deref(),
        limit,
        offset,
    ).await {
        Ok(templates) => {
            let response_templates: Vec<TemplateResponse> = templates
                .into_iter()
                .map(|t| TemplateResponse {
                    id: t.id,
                    name: t.name,
                    description: t.description,
                    category: t.category,
                    preview_image_url: t.preview_image_url,
                    is_public: t.is_public,
                    version: t.version,
                    created_at: t.created_at,
                    updated_at: t.updated_at,
                })
                .collect();

            let response = ApiResponse::success(response_templates, request_id);
            Ok((StatusCode::OK, Json(response)))
        }
        Err(e) => {
            error!("Failed to list templates: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get template by ID
pub async fn get_template(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(template_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let (tenant_id, _user_id) = extract_auth_context(&headers, &state.jwt_manager)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let request_id = Uuid::new_v4();

    // First try to get by ID
    let query = "
        SELECT id, tenant_id, name, description, category, html_source, 
               default_schema, preview_image_url, is_public, version,
               created_at, updated_at
        FROM templates 
        WHERE id = $1 AND (tenant_id = $2 OR is_public = true)
    ";

    let client = match state.db.postgres().get().await {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    match client.query_opt(query, &[&template_id, tenant_id.as_uuid()]).await {
        Ok(Some(row)) => {
            let template = Template {
                id: row.get("id"),
                tenant_id: row.get("tenant_id"),
                name: row.get("name"),
                description: row.get("description"),
                category: row.get("category"),
                html_source: row.get("html_source"),
                default_schema: row.get("default_schema"),
                preview_image_url: row.get("preview_image_url"),
                is_public: row.get("is_public"),
                version: row.get("version"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };

            let response_template = TemplateDetailResponse {
                id: template.id,
                name: template.name,
                description: template.description,
                category: template.category,
                html_source: template.html_source,
                default_schema: template.default_schema,
                preview_image_url: template.preview_image_url,
                is_public: template.is_public,
                version: template.version,
                created_at: template.created_at,
                updated_at: template.updated_at,
            };

            let response = ApiResponse::success(response_template, request_id);
            Ok((StatusCode::OK, Json(response)))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get template: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Create new template
pub async fn create_template(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateTemplateRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let (tenant_id, _user_id) = extract_auth_context(&headers, &state.jwt_manager)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let request_id = Uuid::new_v4();

    // Validate input
    if request.name.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    if request.html_source.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    match state.template_engine.create_template(
        tenant_id.into(),
        &request.name,
        request.description.as_deref(),
        &request.category,
        &request.html_source,
        &request.default_schema,
    ).await {
        Ok(template) => {
            let response_template = TemplateDetailResponse {
                id: template.id,
                name: template.name,
                description: template.description,
                category: template.category,
                html_source: template.html_source,
                default_schema: template.default_schema,
                preview_image_url: template.preview_image_url,
                is_public: template.is_public,
                version: template.version,
                created_at: template.created_at,
                updated_at: template.updated_at,
            };

            let response = ApiResponse::success(response_template, request_id);
            Ok((StatusCode::CREATED, Json(response)))
        }
        Err(e) => {
            error!("Failed to create template: {}", e);
            if e.to_string().contains("syntax error") {
                Err(StatusCode::BAD_REQUEST)
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

/// Update template
pub async fn update_template(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(template_id): Path<Uuid>,
    Json(request): Json<UpdateTemplateRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let (tenant_id, _user_id) = extract_auth_context(&headers, &state.jwt_manager)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let request_id = Uuid::new_v4();

    match state.template_engine.update_template(
        template_id,
        tenant_id.into(),
        request.html_source.as_deref(),
        request.description.as_deref(),
        request.default_schema.as_ref(),
    ).await {
        Ok(template) => {
            let response_template = TemplateDetailResponse {
                id: template.id,
                name: template.name,
                description: template.description,
                category: template.category,
                html_source: template.html_source,
                default_schema: template.default_schema,
                preview_image_url: template.preview_image_url,
                is_public: template.is_public,
                version: template.version,
                created_at: template.created_at,
                updated_at: template.updated_at,
            };

            let response = ApiResponse::success(response_template, request_id);
            Ok((StatusCode::OK, Json(response)))
        }
        Err(e) => {
            error!("Failed to update template: {}", e);
            if e.to_string().contains("not found") || e.to_string().contains("access denied") {
                Err(StatusCode::NOT_FOUND)
            } else if e.to_string().contains("syntax error") {
                Err(StatusCode::BAD_REQUEST)
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

/// Delete template
pub async fn delete_template(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(template_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let (tenant_id, _user_id) = extract_auth_context(&headers, &state.jwt_manager)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let request_id = Uuid::new_v4();

    match state.template_engine.delete_template(template_id, tenant_id.into()).await {
        Ok(_) => {
            let response = ApiResponse::success((), request_id);
            Ok((StatusCode::OK, Json(response)))
        }
        Err(e) => {
            error!("Failed to delete template: {}", e);
            if e.to_string().contains("not found") || e.to_string().contains("access denied") {
                Err(StatusCode::NOT_FOUND)
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

/// Render template with context (for preview)
#[derive(Debug, Deserialize)]
pub struct RenderTemplateRequest {
    pub site_context: serde_json::Value,
    pub page_context: serde_json::Value,
    pub puck_content: String,
}

pub async fn render_template(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(template_id): Path<Uuid>,
    Json(request): Json<RenderTemplateRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let (tenant_id, _user_id) = extract_auth_context(&headers, &state.jwt_manager)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let request_id = Uuid::new_v4();

    // Get template first
    let query = "
        SELECT name FROM templates 
        WHERE id = $1 AND (tenant_id = $2 OR is_public = true)
    ";

    let client = match state.db.postgres().get().await {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    let template_name = match client.query_opt(query, &[&template_id, tenant_id.as_uuid()]).await {
        Ok(Some(row)) => row.get::<_, String>("name"),
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get template for rendering: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Create a simple context for rendering
    let context = serde_json::json!({
        "site": request.site_context,
        "page": request.page_context,
        "puck_content": request.puck_content,
        "user": null
    });

    // Create proper TemplateContext
    use crate::services::template_engine::{TemplateContext, SiteContext, PageContext};
    
    let template_context = TemplateContext {
        site: SiteContext {
            id: Uuid::new_v4(), // This should come from actual site data
            name: "Site Name".to_string(),
            description: Some("Site Description".to_string()),
            subdomain: "subdomain".to_string(),
            custom_domain: None,
            seo_settings: serde_json::json!({}),
        },
        page: PageContext {
            id: Uuid::new_v4(), // This should come from actual page data
            slug: "page-slug".to_string(),
            title: "Page Title".to_string(),
            meta_description: None,
            meta_keywords: None,
            is_published: true,
            published_at: Some(chrono::Utc::now()),
        },
        puck_data: Some(serde_json::json!({})),
        puck_content: request.puck_content,
        user: None,
    };

    // Render the template using the template engine
    match state.template_engine.render_template(&template_name, tenant_id.into(), &template_context).await {
        Ok(rendered_html) => {
            let response = ApiResponse::success(serde_json::json!({
                "template_name": template_name,
                "rendered_html": rendered_html,
                "context": context
            }), request_id);
            Ok((StatusCode::OK, Json(response)))
        }
        Err(e) => {
            error!("Failed to render template: {}", e);
            if e.to_string().contains("not found") {
                Err(StatusCode::NOT_FOUND)
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

/// Request for rendering Puck page
#[derive(Debug, Deserialize)]
pub struct RenderPuckPageRequest {
    pub puck_data: Value,
    pub site: SiteContext,
    pub page: PageContext,
}

/// Request for generating static HTML
#[derive(Debug, Deserialize)]
pub struct GenerateStaticHtmlRequest {
    pub puck_data: Value,
    pub site: SiteContext,
    pub page: PageContext,
}

/// Render Puck page using template engine
pub async fn render_puck_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<RenderPuckPageRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let (tenant_id, _user_id) = extract_auth_context(&headers, &state.jwt_manager)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let request_id = uuid::Uuid::new_v4();
    info!("Rendering Puck page for tenant: {}", tenant_id);

    // Render the Puck page using the template engine
    match state.template_engine.render_puck_page(
        &request.puck_data,
        &request.site,
        &request.page,
        tenant_id.into(),
    ).await {
        Ok(rendered_html) => {
            let response = ApiResponse::success(serde_json::json!({
                "rendered_html": rendered_html,
                "site": request.site,
                "page": request.page
            }), request_id);
            Ok((StatusCode::OK, Json(response)))
        }
        Err(e) => {
            error!("Failed to render Puck page: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Generate static HTML from Puck data
pub async fn generate_static_html(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<GenerateStaticHtmlRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let (_tenant_id, _user_id) = extract_auth_context(&headers, &state.jwt_manager)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    info!("Generating static HTML for site: {}", request.site.name);

    // Generate static HTML using the template engine
    match state.template_engine.generate_static_html(
        &request.puck_data,
        &request.site,
        &request.page,
    ).await {
        Ok(static_html) => {
            // Return HTML directly for static serving
            Ok((StatusCode::OK, Html(static_html)))
        }
        Err(e) => {
            error!("Failed to generate static HTML: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
