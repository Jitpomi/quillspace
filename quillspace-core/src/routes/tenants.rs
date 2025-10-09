use crate::{
    auth::jwt_helpers::{extract_auth_context, extract_auth_context_with_role},
    types::{ApiResponse, Tenant, UserRole},
    AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio_postgres::{Row, Error as PgError};
use tracing::{error, info};
use uuid::Uuid;

/// Helper function to convert a tokio-postgres Row to Tenant
fn row_to_tenant(row: &Row) -> Result<Tenant, PgError> {
    Ok(Tenant {
        id: row.try_get("id")?,
        name: row.try_get("name")?,
        slug: row.try_get("slug")?,
        settings: row.try_get("settings")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
        is_active: row.try_get("is_active")?,
    })
}

/// Query parameters for listing tenants
#[derive(Debug, Deserialize)]
pub struct ListTenantsQuery {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// Request body for creating a tenant
#[derive(Debug, Deserialize)]
pub struct CreateTenantRequest {
    pub name: String,
    pub slug: String,
    pub settings: Option<serde_json::Value>,
}

/// Request body for updating a tenant
#[derive(Debug, Deserialize)]
pub struct UpdateTenantRequest {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub settings: Option<serde_json::Value>,
}

/// Create tenant management routes
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_tenants).post(create_tenant))
        .route("/current", get(get_current_tenant))
        .route("/current/settings", get(get_current_tenant_settings).put(update_current_tenant_settings))
        .route("/:tenant_id", get(get_tenant).put(update_tenant))
        .route("/:tenant_id/settings", get(get_tenant_settings).put(update_tenant_settings))
}

/// List tenants (admin only)
async fn list_tenants(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<ListTenantsQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let request_id = Uuid::new_v4();

    // Verify admin authorization for tenant operations
    let auth_context = extract_auth_context_with_role(&headers, &state.jwt_manager)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    if auth_context.user_role != UserRole::Admin {
        return Err(StatusCode::FORBIDDEN);
    }
    
    let limit: u32 = params.limit.unwrap_or(20).min(100);
    let offset: u32 = params.offset.unwrap_or(0);

    // Get database connection
    let client = match state.db.postgres().get().await {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let query = "SELECT * FROM tenants WHERE is_active = true ORDER BY created_at DESC LIMIT $1 OFFSET $2";
    
    match client.query(query, &[&(limit as i64), &(offset as i64)]).await {
        Ok(rows) => {
            let tenants: Result<Vec<Tenant>, _> = rows.iter().map(row_to_tenant).collect();
            match tenants {
                Ok(tenants) => {
                    let response = ApiResponse::success(tenants, request_id);
                    Ok(Json(response))
                }
                Err(e) => {
                    error!("Failed to parse tenant rows: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Err(e) => {
            error!("Failed to list tenants: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Create a new tenant (admin only)
async fn create_tenant(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateTenantRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let request_id = Uuid::new_v4();

    // Verify admin authorization for tenant creation
    let auth_context = extract_auth_context_with_role(&headers, &state.jwt_manager)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    if auth_context.user_role != UserRole::Admin {
        return Err(StatusCode::FORBIDDEN);
    }

    let tenant_id = Uuid::new_v4();
    let now = chrono::Utc::now();
    let settings = request.settings.unwrap_or_else(|| serde_json::json!({}));

    // Get database connection
    let client = match state.db.postgres().get().await {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let query = r#"
        INSERT INTO tenants (id, name, slug, settings, created_at, updated_at, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING *
        "#;

    match client.query_one(
        query,
        &[&tenant_id, &request.name, &request.slug, &settings, &now, &now, &true],
    ).await {
        Ok(row) => {
            match row_to_tenant(&row) {
                Ok(tenant) => {
                    info!("Created tenant {} with ID {}", tenant.name, tenant.id);
                    let response = ApiResponse::success(tenant, request_id);
                    Ok((StatusCode::CREATED, Json(response)))
                }
                Err(e) => {
                    error!("Failed to parse created tenant: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Err(e) => {
            error!("Failed to create tenant: {}", e);
            if e.to_string().contains("duplicate key") {
                Err(StatusCode::CONFLICT)
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

/// Get current user's tenant
async fn get_current_tenant(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, StatusCode> {
    let request_id = Uuid::new_v4();

    let auth_context = extract_auth_context_with_role(&headers, &state.jwt_manager)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    get_tenant_by_id(state, *auth_context.tenant_id.as_uuid(), request_id).await
}

/// Get tenant by ID
async fn get_tenant(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(tenant_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let request_id = Uuid::new_v4();

    // Verify user authorization for tenant access
    let auth_context = extract_auth_context_with_role(&headers, &state.jwt_manager)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    // Admin can access any tenant, others can only access their own
    if auth_context.user_role != UserRole::Admin && auth_context.tenant_id.as_uuid() != &tenant_id {
        return Err(StatusCode::FORBIDDEN);
    }

    get_tenant_by_id(state, tenant_id, request_id).await
}

/// Internal helper to get tenant by ID
async fn get_tenant_by_id(
    state: AppState,
    tenant_id: Uuid,
    request_id: Uuid,
) -> Result<Json<ApiResponse<Tenant>>, StatusCode> {
    // Get database connection
    let client = match state.db.postgres().get().await {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let query = "SELECT * FROM tenants WHERE id = $1 AND is_active = true";
    
    match client.query_opt(query, &[&tenant_id]).await {
        Ok(Some(row)) => {
            match row_to_tenant(&row) {
                Ok(tenant) => {
                    let response = ApiResponse::success(tenant, request_id);
                    Ok(Json(response))
                }
                Err(e) => {
                    error!("Failed to parse tenant row: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
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
    headers: HeaderMap,
    Path(tenant_id): Path<Uuid>,
    Json(request): Json<UpdateTenantRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let request_id = Uuid::new_v4();

    // Verify user authorization for tenant updates
    let auth_context = extract_auth_context_with_role(&headers, &state.jwt_manager)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    // Admin can update any tenant, others can only update their own
    if auth_context.user_role != UserRole::Admin && auth_context.tenant_id.as_uuid() != &tenant_id {
        return Err(StatusCode::FORBIDDEN);
    }

    let now = chrono::Utc::now();

    // Get database connection
    let client = match state.db.postgres().get().await {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Build dynamic update query
    let mut set_clauses = Vec::new();
    let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![&tenant_id];
    let mut param_count = 1;

    if let Some(name) = &request.name {
        param_count += 1;
        set_clauses.push(format!("name = ${}", param_count));
        params.push(name);
    }

    if let Some(slug) = &request.slug {
        param_count += 1;
        set_clauses.push(format!("slug = ${}", param_count));
        params.push(slug);
    }

    if let Some(settings) = &request.settings {
        param_count += 1;
        set_clauses.push(format!("settings = ${}", param_count));
        params.push(settings);
    }

    if set_clauses.is_empty() {
        return get_tenant_by_id(state, tenant_id, request_id).await;
    }

    param_count += 1;
    set_clauses.push(format!("updated_at = ${}", param_count));
    params.push(&now);

    let query = format!(
        "UPDATE tenants SET {} WHERE id = $1 AND is_active = true RETURNING *",
        set_clauses.join(", ")
    );

    match client.query_opt(&query, &params).await {
        Ok(Some(row)) => {
            match row_to_tenant(&row) {
                Ok(tenant) => {
                    let response = ApiResponse::success(tenant, request_id);
                    Ok(Json(response))
                }
                Err(e) => {
                    error!("Failed to parse updated tenant: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to update tenant: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get current tenant settings
async fn get_current_tenant_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, StatusCode> {
    let request_id = Uuid::new_v4();

    let auth_context = extract_auth_context_with_role(&headers, &state.jwt_manager)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    get_tenant_settings_by_id(state, *auth_context.tenant_id.as_uuid(), request_id).await
}

/// Get tenant settings
async fn get_tenant_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(tenant_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let request_id = Uuid::new_v4();

    // Verify user authorization for tenant settings access
    let auth_context = extract_auth_context_with_role(&headers, &state.jwt_manager)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    // Admin can access any tenant settings, others can only access their own
    if auth_context.user_role != UserRole::Admin && auth_context.tenant_id.as_uuid() != &tenant_id {
        return Err(StatusCode::FORBIDDEN);
    }

    get_tenant_settings_by_id(state, tenant_id, request_id).await
}

/// Internal helper to get tenant settings by ID
async fn get_tenant_settings_by_id(
    state: AppState,
    tenant_id: Uuid,
    request_id: Uuid,
) -> Result<impl IntoResponse, StatusCode> {
    // Get database connection
    let client = match state.db.postgres().get().await {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let query = "SELECT settings FROM tenants WHERE id = $1 AND is_active = true";
    
    match client.query_opt(query, &[&tenant_id]).await {
        Ok(Some(row)) => {
            let settings: serde_json::Value = row.get("settings");
            let response = ApiResponse::success(settings, request_id);
            Ok(Json(response))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get tenant settings: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Update current tenant settings
async fn update_current_tenant_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(settings): Json<serde_json::Value>,
) -> Result<impl IntoResponse, StatusCode> {
    let request_id = Uuid::new_v4();

    let auth_context = extract_auth_context_with_role(&headers, &state.jwt_manager)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Only admin or editor can update tenant settings
    if !matches!(auth_context.user_role, UserRole::Admin | UserRole::Editor) {
        return Err(StatusCode::FORBIDDEN);
    }

    update_tenant_settings_by_id(state, *auth_context.tenant_id.as_uuid(), settings, request_id).await
}

/// Update tenant settings
async fn update_tenant_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(tenant_id): Path<Uuid>,
    Json(settings): Json<serde_json::Value>,
) -> Result<impl IntoResponse, StatusCode> {
    let request_id = Uuid::new_v4();

    // Verify user authorization for tenant settings updates
    let auth_context = extract_auth_context_with_role(&headers, &state.jwt_manager)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    // Admin can update any tenant settings, editors can only update their own
    if auth_context.user_role == UserRole::Admin {
        // Admin can update any tenant
    } else if auth_context.user_role == UserRole::Editor && auth_context.tenant_id.as_uuid() == &tenant_id {
        // Editor can update their own tenant
    } else {
        return Err(StatusCode::FORBIDDEN);
    }

    update_tenant_settings_by_id(state, tenant_id, settings, request_id).await
}

/// Internal helper to update tenant settings by ID
async fn update_tenant_settings_by_id(
    state: AppState,
    tenant_id: Uuid,
    settings: serde_json::Value,
    request_id: Uuid,
) -> Result<impl IntoResponse, StatusCode> {
    let now = chrono::Utc::now();

    // Get database connection
    let client = match state.db.postgres().get().await {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let query = "UPDATE tenants SET settings = $2, updated_at = $3 WHERE id = $1 AND is_active = true RETURNING settings";

    match client.query_opt(query, &[&tenant_id, &settings, &now]).await {
        Ok(Some(row)) => {
            let updated_settings: serde_json::Value = row.get("settings");
            let response = ApiResponse::success(updated_settings, request_id);
            Ok(Json(response))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to update tenant settings: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
