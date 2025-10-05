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
use chrono::{DateTime, Utc};
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

/// Create tenant management routes
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_tenants).post(create_tenant))
        .route("/{tenant_id}", get(get_tenant).put(update_tenant))
        .route("/{tenant_id}/settings", get(get_tenant_settings).put(update_tenant_settings))
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

    let query = "SELECT * FROM tenants ORDER BY created_at DESC LIMIT $1 OFFSET $2";
    let limit_i64 = limit as i64;
    let offset_i64 = offset as i64;
    let params_vec: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![&limit_i64, &offset_i64];

    match client.query(query, &params_vec).await {
        Ok(rows) => {
            let tenants: Result<Vec<Tenant>, _> = rows.iter().map(row_to_tenant).collect();
            let tenants = match tenants {
                Ok(tenants) => tenants,
                Err(e) => {
                    error!("Failed to parse tenant rows: {}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            };

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

    let settings = serde_json::json!({});
    let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![
        &tenant_id,
        &tenant_request.name,
        &tenant_request.slug,
        &settings,
        &now,
        &now,
        &true,
    ];

    match client.query_one(query, &params).await {
        Ok(row) => {
            let tenant = match row_to_tenant(&row) {
                Ok(tenant) => tenant,
                Err(e) => {
                    error!("Failed to parse tenant row: {}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            };

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

    // Get database connection
    let client = match state.db.postgres().get().await {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let query = "SELECT * FROM tenants WHERE id = $1";
    let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![&tenant_id];

    match client.query_opt(query, &params).await {
        Ok(Some(row)) => {
            let tenant = match row_to_tenant(&row) {
                Ok(tenant) => tenant,
                Err(e) => {
                    error!("Failed to parse tenant row: {}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            };

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

    // Get database connection
    let client = match state.db.postgres().get().await {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let query = r#"
        UPDATE tenants 
        SET name = COALESCE($2, name),
            slug = COALESCE($3, slug),
            updated_at = $4
        WHERE id = $1
        RETURNING *
        "#;

    let name_ref = update_request.name.as_deref();
    let slug_ref = update_request.slug.as_deref();
    
    let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![
        &tenant_id,
        &name_ref,
        &slug_ref,
        &now,
    ];

    match client.query_opt(query, &params).await {
        Ok(Some(row)) => {
            let tenant = match row_to_tenant(&row) {
                Ok(tenant) => tenant,
                Err(e) => {
                    error!("Failed to parse tenant row: {}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            };

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

    // Get database connection
    let client = match state.db.postgres().get().await {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let query = "SELECT settings FROM tenants WHERE id = $1";
    let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![&tenant_id];

    match client.query_opt(query, &params).await {
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

    // Get database connection
    let client = match state.db.postgres().get().await {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let query = "UPDATE tenants SET settings = $2, updated_at = $3 WHERE id = $1";
    let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![&tenant_id, &settings, &now];

    match client.execute(query, &params).await {
        Ok(rows_affected) => {
            if rows_affected > 0 {
                info!(tenant_id = %tenant_id, "Tenant settings updated");
                let response = ApiResponse::success(settings, request_id);
                Ok(Json(response))
            } else {
                Err(StatusCode::NOT_FOUND)
            }
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
