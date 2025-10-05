use crate::{
    middleware::tenant::get_request_context,
    types::{ApiResponse, Tenant, UserRole},
    AppState,
};
use axum::{
    extract::{Path, Query, Request, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use uuid::Uuid;

/// Create tenant management routes
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_tenants).post(create_tenant))
        .route("/:tenant_id", get(get_tenant).put(update_tenant))
        .route("/:tenant_id/settings", get(get_tenant_settings).put(update_tenant_settings))
}

/// List tenants (admin only)
async fn list_tenants(
    State(state): State<AppState>,
    Query(params): Query<ListTenantsQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    // Placeholder tenant context (implement proper JWT auth later)
    let request_id = Uuid::new_v4();

    // For now, skip admin check (implement proper auth later)
    // In real implementation: check if user has Admin role

    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);

    let query = sqlx::query_as::<_, Tenant>(
        "SELECT * FROM tenants ORDER BY created_at DESC LIMIT $1 OFFSET $2"
    )
    .bind(limit as i64)
    .bind(offset as i64);

    match query.fetch_all(state.db.postgres()).await {
        Ok(tenants) => {
            let response = ApiResponse::success(tenants, request_id);
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to list tenants: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Create a new tenant
async fn create_tenant(
    State(state): State<AppState>,
    Json(tenant_request): Json<CreateTenantRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    // Placeholder tenant context (implement proper JWT auth later)
    let request_id = Uuid::new_v4();

    // For now, skip admin check (implement proper auth later)
    // In real implementation: check if user has Admin role

    let tenant_id = Uuid::new_v4();
    let now = chrono::Utc::now();

    let query = sqlx::query_as::<_, Tenant>(
        r#"
        INSERT INTO tenants (id, name, slug, settings, created_at, updated_at, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING *
        "#
    )
    .bind(tenant_id)
    .bind(&tenant_request.name)
    .bind(&tenant_request.slug)
    .bind(serde_json::json!({}))
    .bind(now)
    .bind(now)
    .bind(true);

    match query.fetch_one(state.db.postgres()).await {
        Ok(tenant) => {
            info!(
                tenant_id = %tenant_id,
                tenant_name = %tenant_request.name,
                "New tenant created"
            );
            
            let response = ApiResponse::success(tenant, request_id);
            Ok((StatusCode::CREATED, Json(response)))
        }
        Err(e) => {
            error!("Failed to create tenant: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get tenant details
async fn get_tenant(
    State(state): State<AppState>,
    Path(tenant_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    // Placeholder tenant context (implement proper JWT auth later)
    let request_id = Uuid::new_v4();

    // For now, skip tenant access check (implement proper auth later)
    // In real implementation: check if user can access this tenant

    let query = sqlx::query_as::<_, Tenant>("SELECT * FROM tenants WHERE id = $1")
        .bind(tenant_id);

    match query.fetch_optional(state.db.postgres()).await {
        Ok(Some(tenant)) => {
            let response = ApiResponse::success(tenant, request_id);
            Ok(Json(response))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get tenant: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Update tenant
async fn update_tenant(
    State(state): State<AppState>,
    Path(tenant_id): Path<Uuid>,
    Json(update_request): Json<UpdateTenantRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    // Placeholder tenant context (implement proper JWT auth later)
    let request_id = Uuid::new_v4();

    // For now, skip tenant access check (implement proper auth later)
    // In real implementation: check if user can update this tenant

    let now = chrono::Utc::now();

    let query = sqlx::query_as::<_, Tenant>(
        r#"
        UPDATE tenants 
        SET name = COALESCE($2, name),
            slug = COALESCE($3, slug),
            updated_at = $4
        WHERE id = $1
        RETURNING *
        "#
    )
    .bind(tenant_id)
    .bind(update_request.name.as_deref())
    .bind(update_request.slug.as_deref())
    .bind(now);

    match query.fetch_optional(state.db.postgres()).await {
        Ok(Some(tenant)) => {
            info!(tenant_id = %tenant_id, "Tenant updated");
            let response = ApiResponse::success(tenant, request_id);
            Ok(Json(response))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to update tenant: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get tenant settings
async fn get_tenant_settings(
    State(state): State<AppState>,
    Path(tenant_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    // Placeholder tenant context (implement proper JWT auth later)
    let request_id = Uuid::new_v4();

    // For now, skip tenant access check (implement proper auth later)
    // In real implementation: check if user can access this tenant's settings

    let query = sqlx::query!("SELECT settings FROM tenants WHERE id = $1", tenant_id);

    match query.fetch_optional(state.db.postgres()).await {
        Ok(Some(row)) => {
            let response = ApiResponse::success(row.settings, request_id);
            Ok(Json(response))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get tenant settings: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Update tenant settings
async fn update_tenant_settings(
    State(state): State<AppState>,
    Path(tenant_id): Path<Uuid>,
    Json(settings): Json<serde_json::Value>,
) -> Result<impl IntoResponse, StatusCode> {
    // Placeholder tenant context (implement proper JWT auth later)
    let request_id = Uuid::new_v4();

    // For now, skip tenant access check (implement proper auth later)
    // In real implementation: check if user can update this tenant's settings

    let now = chrono::Utc::now();

    let query = sqlx::query!(
        "UPDATE tenants SET settings = $2, updated_at = $3 WHERE id = $1",
        tenant_id,
        settings,
        now
    );

    match query.execute(state.db.postgres()).await {
        Ok(_) => {
            info!(tenant_id = %tenant_id, "Tenant settings updated");
            let response = ApiResponse::success(settings, request_id);
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to update tenant settings: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Request/Response types
#[derive(Debug, Deserialize)]
struct ListTenantsQuery {
    limit: Option<u32>,
    offset: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct CreateTenantRequest {
    name: String,
    slug: String,
}

#[derive(Debug, Deserialize)]
struct UpdateTenantRequest {
    name: Option<String>,
    slug: Option<String>,
}
