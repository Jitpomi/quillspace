use crate::{
    auth::jwt_helpers::{extract_auth_context, extract_auth_context_with_role},
    types::{ApiResponse, User, UserRole},
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

/// Helper function to convert a tokio-postgres Row to User
fn row_to_user(row: &Row) -> Result<User, PgError> {
    Ok(User {
        id: row.try_get("id")?,
        tenant_id: row.try_get("tenant_id")?,
        email: row.try_get("email")?,
        name: row.try_get("name")?,
        role: match row.try_get::<_, String>("role")?.as_str() {
            "admin" => UserRole::Admin,
            "editor" => UserRole::Editor,
            "viewer" => UserRole::Viewer,
            _ => UserRole::Viewer, // Default fallback
        },
        is_active: row.try_get("is_active")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

/// Convert UserRole to string for database storage
fn user_role_to_string(role: &UserRole) -> &'static str {
    match role {
        UserRole::Admin => "admin",
        UserRole::Editor => "editor",
        UserRole::Viewer => "viewer",
    }
}

/// Query parameters for listing users
#[derive(Debug, Deserialize)]
pub struct ListUsersQuery {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub role: Option<String>,
}

/// Request body for creating a user
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub name: String,
    pub role: UserRole,
}

/// Request body for updating a user
#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub role: Option<UserRole>,
    pub is_active: Option<bool>,
}

/// Create user management routes
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_users).post(create_user))
        .route("/current", get(get_current_user))
        .route("/:user_id", get(get_user).put(update_user))
        .route("/:user_id/deactivate", put(deactivate_user))
}

/// List users in tenant (admin only)
async fn list_users(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<ListUsersQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let request_id = Uuid::new_v4();

    // Verify admin authorization for user management
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

    // Set RLS context
    if let Err(e) = client.execute(
        "SELECT set_config('rls.tenant_id', $1, true)",
        &[&auth_context.tenant_id.to_string()],
    ).await {
        error!("Failed to set RLS context: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    // Build query with optional role filter
    let (query, role_param) = if let Some(role) = &params.role {
        (
            "SELECT * FROM users WHERE role = $3 AND is_active = true ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            Some(role.as_str()),
        )
    } else {
        (
            "SELECT * FROM users WHERE is_active = true ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            None,
        )
    };

    let result = if let Some(role) = role_param {
        client.query(query, &[&(limit as i64), &(offset as i64), &role]).await
    } else {
        client.query(query, &[&(limit as i64), &(offset as i64)]).await
    };

    match result {
        Ok(rows) => {
            let users: Result<Vec<User>, _> = rows.iter().map(row_to_user).collect();
            match users {
                Ok(users) => {
                    let response = ApiResponse::success(users, request_id);
                    Ok(Json(response))
                }
                Err(e) => {
                    error!("Failed to parse user rows: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Err(e) => {
            error!("Failed to list users: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Create a new user (admin only)
async fn create_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let request_id = Uuid::new_v4();

    // Verify admin authorization for user creation
    let auth_context = extract_auth_context_with_role(&headers, &state.jwt_manager)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    if auth_context.user_role != UserRole::Admin {
        return Err(StatusCode::FORBIDDEN);
    }

    let user_id = Uuid::new_v4();
    let now = chrono::Utc::now();
    let role_str = user_role_to_string(&request.role);

    // Get database connection
    let client = match state.db.postgres().get().await {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Set RLS context
    if let Err(e) = client.execute(
        "SELECT set_config('rls.tenant_id', $1, true)",
        &[&auth_context.tenant_id.to_string()],
    ).await {
        error!("Failed to set RLS context: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let query = r#"
        INSERT INTO users (id, tenant_id, email, name, role, is_active, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING *
        "#;

    match client.query_one(
        query,
        &[
            &user_id,
            auth_context.tenant_id.as_uuid(),
            &request.email,
            &request.name,
            &role_str,
            &true,
            &now,
            &now,
        ],
    ).await {
        Ok(row) => {
            match row_to_user(&row) {
                Ok(user) => {
                    info!("Created user {} with ID {}", user.email, user.id);
                    let response = ApiResponse::success(user, request_id);
                    Ok((StatusCode::CREATED, Json(response)))
                }
                Err(e) => {
                    error!("Failed to parse created user: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Err(e) => {
            error!("Failed to create user: {}", e);
            if e.to_string().contains("duplicate key") {
                Err(StatusCode::CONFLICT)
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

/// Get current user profile
async fn get_current_user(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, StatusCode> {
    let request_id = Uuid::new_v4();

    let auth_context = extract_auth_context_with_role(&headers, &state.jwt_manager)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    get_user_by_id(state, auth_context.user_id, auth_context.tenant_id, request_id).await
}

/// Get user by ID
async fn get_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let request_id = Uuid::new_v4();

    // Verify user authorization
    let auth_context = extract_auth_context_with_role(&headers, &state.jwt_manager)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    // Admin can view any user in tenant, others can only view themselves
    if auth_context.user_role != UserRole::Admin && auth_context.user_id != user_id {
        return Err(StatusCode::FORBIDDEN);
    }

    get_user_by_id(state, user_id, auth_context.tenant_id, request_id).await
}

/// Internal helper to get user by ID
async fn get_user_by_id(
    state: AppState,
    user_id: Uuid,
    tenant_id: crate::types::TenantId,
    request_id: Uuid,
) -> Result<Json<ApiResponse<User>>, StatusCode> {
    // Get database connection
    let client = match state.db.postgres().get().await {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Set RLS context
    if let Err(e) = client.execute(
        "SELECT set_config('rls.tenant_id', $1, true)",
        &[&tenant_id.to_string()],
    ).await {
        error!("Failed to set RLS context: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let query = "SELECT * FROM users WHERE id = $1 AND is_active = true";
    
    match client.query_opt(query, &[&user_id]).await {
        Ok(Some(row)) => {
            match row_to_user(&row) {
                Ok(user) => {
                    let response = ApiResponse::success(user, request_id);
                    Ok(Json(response))
                }
                Err(e) => {
                    error!("Failed to parse user row: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get user: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Update user
async fn update_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(user_id): Path<Uuid>,
    Json(request): Json<UpdateUserRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let request_id = Uuid::new_v4();

    // Verify user authorization
    let auth_context = extract_auth_context_with_role(&headers, &state.jwt_manager)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    // Admin can update any user, others can only update themselves (limited fields)
    let can_update_role = auth_context.user_role == UserRole::Admin;
    let can_update_user = auth_context.user_role == UserRole::Admin || auth_context.user_id == user_id;
    
    if !can_update_user {
        return Err(StatusCode::FORBIDDEN);
    }

    // Non-admin users cannot change roles or activation status
    if !can_update_role && (request.role.is_some() || request.is_active.is_some()) {
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

    // Set RLS context
    if let Err(e) = client.execute(
        "SELECT set_config('rls.tenant_id', $1, true)",
        &[&auth_context.tenant_id.to_string()],
    ).await {
        error!("Failed to set RLS context: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    // Build dynamic update query
    let mut set_clauses = Vec::new();
    let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![&user_id];
    let mut param_count = 1;

    if let Some(name) = &request.name {
        param_count += 1;
        set_clauses.push(format!("name = ${}", param_count));
        params.push(name);
    }

    let role_str_ref;
    if let Some(role) = &request.role {
        role_str_ref = user_role_to_string(role);
        param_count += 1;
        set_clauses.push(format!("role = ${}", param_count));
        params.push(&role_str_ref);
    }

    if let Some(is_active) = &request.is_active {
        param_count += 1;
        set_clauses.push(format!("is_active = ${}", param_count));
        params.push(is_active);
    }

    if set_clauses.is_empty() {
        return get_user_by_id(state, user_id, auth_context.tenant_id, request_id).await;
    }

    param_count += 1;
    set_clauses.push(format!("updated_at = ${}", param_count));
    params.push(&now);

    let query = format!(
        "UPDATE users SET {} WHERE id = $1 AND is_active = true RETURNING *",
        set_clauses.join(", ")
    );

    match client.query_opt(&query, &params).await {
        Ok(Some(row)) => {
            match row_to_user(&row) {
                Ok(user) => {
                    let response = ApiResponse::success(user, request_id);
                    Ok(Json(response))
                }
                Err(e) => {
                    error!("Failed to parse updated user: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to update user: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Deactivate user (admin only)
async fn deactivate_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let request_id = Uuid::new_v4();

    // Verify admin authorization for user deactivation
    let auth_context = extract_auth_context_with_role(&headers, &state.jwt_manager)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    if auth_context.user_role != UserRole::Admin {
        return Err(StatusCode::FORBIDDEN);
    }

    // Prevent admin from deactivating themselves
    if auth_context.user_id == user_id {
        return Err(StatusCode::BAD_REQUEST);
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

    // Set RLS context
    if let Err(e) = client.execute(
        "SELECT set_config('rls.tenant_id', $1, true)",
        &[&auth_context.tenant_id.to_string()],
    ).await {
        error!("Failed to set RLS context: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let query = "UPDATE users SET is_active = false, updated_at = $2 WHERE id = $1 AND is_active = true RETURNING *";

    match client.query_opt(query, &[&user_id, &now]).await {
        Ok(Some(row)) => {
            match row_to_user(&row) {
                Ok(user) => {
                    info!("Deactivated user {} ({})", user.email, user.id);
                    let response = ApiResponse::success(user, request_id);
                    Ok(Json(response))
                }
                Err(e) => {
                    error!("Failed to parse deactivated user: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to deactivate user: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
