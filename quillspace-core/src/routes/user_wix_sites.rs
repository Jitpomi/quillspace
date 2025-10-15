use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::{
    services::user_wix_sites::{UserWixSitesService, UserWixSite},
    auth::jwt_helpers::AuthContext,
    AppState,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserWixSite {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub wix_site_id: String,
    pub site_name: String,
    pub wix_url: String,
    pub custom_domain: Option<String>,
    pub project_status: String,
    pub service_type: String,
    pub client_can_edit: bool,
    pub client_can_publish: bool,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserWixSiteRequest {
    pub user_email: Option<String>,  // Either email or user_id
    pub user_id: Option<Uuid>,       // Either email or user_id
    pub wix_site_id: String,
    pub site_name: String,
    pub wix_url: Option<String>,
    pub custom_domain: Option<String>,
    pub project_status: Option<String>,
    pub service_type: Option<String>,
    pub client_can_edit: Option<bool>,
    pub client_can_publish: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserWixSiteRequest {
    pub site_name: Option<String>,
    pub wix_url: Option<String>,
    pub custom_domain: Option<String>,
    pub project_status: Option<String>,
    pub client_can_edit: Option<bool>,
    pub client_can_publish: Option<bool>,
}

pub fn user_wix_sites_routes() -> Router<AppState> {
    Router::new()
        .route("/admin/user-wix-sites", get(list_user_wix_sites))
        .route("/admin/user-wix-sites", post(create_user_wix_site))
        .route("/admin/user-wix-sites/:id", get(get_user_wix_site))
        .route("/admin/user-wix-sites/:id", put(update_user_wix_site))
        .route("/admin/user-wix-sites/:id", delete(delete_user_wix_site))
        .route("/admin/users/:user_id/wix-sites", get(get_user_wix_sites))
}

/// List all user-wix-site mappings (admin only)
pub async fn list_user_wix_sites(
    auth: AuthContext,
    State(state): State<AppState>,
) -> Result<Json<Vec<UserWixSite>>, StatusCode> {
    // Check admin permission
    if !is_admin(&auth) {
        return Err(StatusCode::FORBIDDEN);
    }

    let query = "
        SELECT id, tenant_id, user_id, wix_site_id, site_name, wix_url,
               custom_domain, project_status, service_type, client_can_edit,
               client_can_publish, metadata, created_at, updated_at
        FROM user_wix_sites
        ORDER BY created_at DESC
    ";

    match state.db.query(query, &[]).await {
        Ok(rows) => {
            let sites = rows.into_iter().map(|row| UserWixSite {
                id: row.get(0),
                tenant_id: row.get(1),
                user_id: row.get(2),
                wix_site_id: row.get(3),
                site_name: row.get(4),
                wix_url: row.get(5),
                custom_domain: row.get(6),
                project_status: row.get(7),
                service_type: row.get(8),
                client_can_edit: row.get(9),
                client_can_publish: row.get(10),
                metadata: row.get(11),
                created_at: row.get(12),
                updated_at: row.get(13),
            }).collect();
            
            Ok(Json(sites))
        }
        Err(e) => {
            tracing::error!("Failed to list user wix sites: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Create a new user-wix-site mapping
pub async fn create_user_wix_site(
    auth: AuthContext,
    State(state): State<AppState>,
    Json(request): Json<CreateUserWixSiteRequest>,
) -> Result<Json<UserWixSite>, StatusCode> {
    // Check admin permission
    if !is_admin(&auth) {
        return Err(StatusCode::FORBIDDEN);
    }

    // Find user by email or use provided user_id
    let user_id = if let Some(email) = &request.user_email {
        // Look up user by email
        let user_query = "SELECT id FROM users WHERE email ILIKE $1 LIMIT 1";
        match state.db.query_one(user_query, &[&format!("%{}%", email)]).await {
            Ok(row) => row.get::<_, Uuid>(0),
            Err(_) => return Err(StatusCode::BAD_REQUEST), // User not found
        }
    } else if let Some(user_id) = request.user_id {
        user_id
    } else {
        return Err(StatusCode::BAD_REQUEST); // Neither email nor user_id provided
    };

    let now = chrono::Utc::now();

    let query = "
        INSERT INTO user_wix_sites (
            wix_site_id, tenant_id, user_id, display_name, custom_domain, 
            project_status, service_type, client_can_edit, client_can_publish, 
            metadata, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        RETURNING wix_site_id, tenant_id, user_id, display_name, custom_domain,
                  project_status, service_type, client_can_edit, client_can_publish,
                  metadata, created_at, updated_at
    ";

    let metadata = serde_json::json!({
        "created_by": "admin",
        "created_via": "api"
    });

    match state.db.query_one(query, &[
        &request.wix_site_id,
        &auth.tenant_id,
        &user_id,
        &request.site_name,
        &request.custom_domain,
        &request.project_status.unwrap_or_else(|| "active".to_string()),
        &request.service_type.unwrap_or_else(|| "build_and_manage".to_string()),
        &request.client_can_edit.unwrap_or(true),
        &request.client_can_publish.unwrap_or(true),
        &metadata,
        &now,
        &now,
    ]).await {
        Ok(row) => {
            let site = UserWixSite {
                id: row.get(0),
                tenant_id: row.get(1),
                user_id: row.get(2),
                wix_site_id: row.get(3),
                site_name: row.get(4),
                wix_url: row.get(5),
                custom_domain: row.get(6),
                project_status: row.get(7),
                service_type: row.get(8),
                client_can_edit: row.get(9),
                client_can_publish: row.get(10),
                metadata: row.get(11),
                created_at: row.get(12),
                updated_at: row.get(13),
            };
            
            Ok(Json(site))
        }
        Err(e) => {
            tracing::error!("Failed to create user wix site: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get a specific user-wix-site mapping
pub async fn get_user_wix_site(
    auth: AuthContext,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<UserWixSite>, StatusCode> {
    if !is_admin(&auth) {
        return Err(StatusCode::FORBIDDEN);
    }

    let query = "
        SELECT id, tenant_id, user_id, wix_site_id, site_name, wix_url,
               custom_domain, project_status, service_type, client_can_edit,
               client_can_publish, metadata, created_at, updated_at
        FROM user_wix_sites
        WHERE id = $1
    ";

    match state.db.query_one(query, &[&id]).await {
        Ok(row) => {
            let site = UserWixSite {
                id: row.get(0),
                tenant_id: row.get(1),
                user_id: row.get(2),
                wix_site_id: row.get(3),
                site_name: row.get(4),
                wix_url: row.get(5),
                custom_domain: row.get(6),
                project_status: row.get(7),
                service_type: row.get(8),
                client_can_edit: row.get(9),
                client_can_publish: row.get(10),
                metadata: row.get(11),
                created_at: row.get(12),
                updated_at: row.get(13),
            };
            
            Ok(Json(site))
        }
        Err(_) => Err(StatusCode::NOT_FOUND)
    }
}

/// Update a user-wix-site mapping
pub async fn update_user_wix_site(
    auth: AuthContext,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateUserWixSiteRequest>,
) -> Result<Json<UserWixSite>, StatusCode> {
    if !is_admin(&auth) {
        return Err(StatusCode::FORBIDDEN);
    }

    let mut updates = Vec::new();
    let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![&id];
    let mut param_count = 1;

    if let Some(site_name) = &request.site_name {
        param_count += 1;
        updates.push(format!("site_name = ${}", param_count));
        params.push(site_name);
    }

    if let Some(wix_url) = &request.wix_url {
        param_count += 1;
        updates.push(format!("wix_url = ${}", param_count));
        params.push(wix_url);
    }

    if let Some(custom_domain) = &request.custom_domain {
        param_count += 1;
        updates.push(format!("custom_domain = ${}", param_count));
        params.push(custom_domain);
    }

    if let Some(project_status) = &request.project_status {
        param_count += 1;
        updates.push(format!("project_status = ${}", param_count));
        params.push(project_status);
    }

    if let Some(client_can_edit) = &request.client_can_edit {
        param_count += 1;
        updates.push(format!("client_can_edit = ${}", param_count));
        params.push(client_can_edit);
    }

    if let Some(client_can_publish) = &request.client_can_publish {
        param_count += 1;
        updates.push(format!("client_can_publish = ${}", param_count));
        params.push(client_can_publish);
    }

    if updates.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    param_count += 1;
    updates.push(format!("updated_at = ${}", param_count));
    let now = chrono::Utc::now();
    params.push(&now);

    let query = format!(
        "UPDATE user_wix_sites SET {} WHERE id = $1 
         RETURNING id, tenant_id, user_id, wix_site_id, site_name, wix_url,
                   custom_domain, project_status, service_type, client_can_edit,
                   client_can_publish, metadata, created_at, updated_at",
        updates.join(", ")
    );

    match state.db.query_one(&query, &params).await {
        Ok(row) => {
            let site = UserWixSite {
                id: row.get(0),
                tenant_id: row.get(1),
                user_id: row.get(2),
                wix_site_id: row.get(3),
                site_name: row.get(4),
                wix_url: row.get(5),
                custom_domain: row.get(6),
                project_status: row.get(7),
                service_type: row.get(8),
                client_can_edit: row.get(9),
                client_can_publish: row.get(10),
                metadata: row.get(11),
                created_at: row.get(12),
                updated_at: row.get(13),
            };
            
            Ok(Json(site))
        }
        Err(e) => {
            tracing::error!("Failed to update user wix site: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Delete a user-wix-site mapping
pub async fn delete_user_wix_site(
    auth: AuthContext,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    if !is_admin(&auth) {
        return Err(StatusCode::FORBIDDEN);
    }

    let query = "DELETE FROM user_wix_sites WHERE id = $1";

    match state.db.execute(query, &[&id]).await {
        Ok(rows_affected) => {
            if rows_affected > 0 {
                Ok(StatusCode::NO_CONTENT)
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        }
        Err(e) => {
            tracing::error!("Failed to delete user wix site: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get all Wix sites for a specific user
pub async fn get_user_wix_sites(
    auth: AuthContext,
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<Vec<UserWixSite>>, StatusCode> {
    if !is_admin(&auth) {
        return Err(StatusCode::FORBIDDEN);
    }

    let query = "
        SELECT id, tenant_id, user_id, wix_site_id, site_name, wix_url,
               custom_domain, project_status, service_type, client_can_edit,
               client_can_publish, metadata, created_at, updated_at
        FROM user_wix_sites
        WHERE user_id = $1
        ORDER BY created_at DESC
    ";

    match state.db.query(query, &[&user_id]).await {
        Ok(rows) => {
            let sites = rows.into_iter().map(|row| UserWixSite {
                id: row.get(0),
                tenant_id: row.get(1),
                user_id: row.get(2),
                wix_site_id: row.get(3),
                site_name: row.get(4),
                wix_url: row.get(5),
                custom_domain: row.get(6),
                project_status: row.get(7),
                service_type: row.get(8),
                client_can_edit: row.get(9),
                client_can_publish: row.get(10),
                metadata: row.get(11),
                created_at: row.get(12),
                updated_at: row.get(13),
            }).collect();
            
            Ok(Json(sites))
        }
        Err(e) => {
            tracing::error!("Failed to get user wix sites: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Helper function to check admin permissions
fn is_admin(auth: &AuthContext) -> bool {
    // Implement your admin check logic here
    // This could check user role, permissions, etc.
    true // Placeholder - implement proper admin check
}
