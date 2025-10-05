use crate::{
    middleware::tenant::get_request_context,
    types::{ApiResponse, User, UserRole},
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

/// Helper function to convert a tokio-postgres Row to User
fn row_to_user(row: &Row) -> Result<User, PgError> {
    let role_str: String = row.try_get("role")?;
    let role = match role_str.as_str() {
        "Admin" => UserRole::Admin,
        "Editor" => UserRole::Editor,
        "Viewer" => UserRole::Viewer,
        _ => UserRole::Viewer, // Default fallback
    };

    Ok(User {
        id: row.try_get("id")?,
        tenant_id: row.try_get("tenant_id")?,
        email: row.try_get("email")?,
        name: row.try_get("name")?,
        role,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
        is_active: row.try_get("is_active")?,
    })
}

/// Helper function to convert UserRole to string for database
fn user_role_to_string(role: &UserRole) -> &'static str {
    match role {
        UserRole::Admin => "Admin",
        UserRole::Editor => "Editor",
        UserRole::Viewer => "Viewer",
    }
}

/// Create user management routes
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_users).post(create_user))
        .route("/{user_id}", get(get_user).put(update_user))
        .route("/{user_id}/role", put(update_user_role))
        .route("/me", get(get_current_user))
}

/// List users in tenant
async fn list_users(
    State(state): State<AppState>,
    Query(params): Query<ListUsersQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    // Placeholder tenant context (implement proper JWT auth later)
    let tenant_id = crate::types::TenantId::from_uuid(Uuid::new_v4());
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

    let query = r#"
        SELECT * FROM users 
        WHERE tenant_id = $1 
        ORDER BY created_at DESC 
        LIMIT $2 OFFSET $3
        "#;

    let limit_i64 = limit as i64;
    let offset_i64 = offset as i64;
    let params_vec: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![
        tenant_id.as_uuid(),
        &limit_i64,
        &offset_i64,
    ];

    match client.query(query, &params_vec).await {
        Ok(rows) => {
            let users: Result<Vec<User>, _> = rows.iter().map(row_to_user).collect();
            let users = match users {
                Ok(users) => users,
                Err(e) => {
                    error!("Failed to parse user rows: {}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            };

            let response = ApiResponse::success(users, request_id);
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to list users: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Create a new user
async fn create_user(
    State(state): State<AppState>,
    Json(user_request): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    // Placeholder tenant context (implement proper JWT auth later)
    let tenant_id = crate::types::TenantId::from_uuid(Uuid::new_v4());
    let request_id = Uuid::new_v4();

    // For now, skip admin check (implement proper auth later)
    // In real implementation: check if user has Admin role

    let user_id = Uuid::new_v4();
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
        INSERT INTO users (id, tenant_id, email, name, role, created_at, updated_at, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING *
        "#;

    let role_str = user_role_to_string(&user_request.role);
    let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![
        &user_id,
        tenant_id.as_uuid(),
        &user_request.email,
        &user_request.name,
        &role_str,
        &now,
        &now,
        &true,
    ];

    match client.query_one(query, &params).await {
        Ok(row) => {
            let user = match row_to_user(&row) {
                Ok(user) => user,
                Err(e) => {
                    error!("Failed to parse user row: {}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            };

            info!(
                user_id = %user_id,
                tenant_id = %tenant_id,
                email = %user_request.email,
                "New user created"
            );
            
            let response = ApiResponse::success(user, request_id);
            Ok((StatusCode::CREATED, Json(response)))
        }
        Err(e) => {
            error!("Failed to create user: {}", e);
            if e.to_string().contains("unique constraint") {
                Err(StatusCode::CONFLICT)
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

/// Get user by ID
async fn get_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    // Placeholder tenant context (implement proper JWT auth later)
    let tenant_id = crate::types::TenantId::from_uuid(Uuid::new_v4());
    let request_id = Uuid::new_v4();

    // For now, skip user access check (implement proper auth later)
    // In real implementation: check if user can view this user profile

    // Get database connection
    let client = match state.db.postgres().get().await {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let query = "SELECT * FROM users WHERE id = $1 AND tenant_id = $2";
    let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![&user_id, tenant_id.as_uuid()];

    match client.query_opt(query, &params).await {
        Ok(Some(row)) => {
            let user = match row_to_user(&row) {
                Ok(user) => user,
                Err(e) => {
                    error!("Failed to parse user row: {}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            };

            let response = ApiResponse::success(user, request_id);
            Ok(Json(response))
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
    Path(user_id): Path<Uuid>,
    Json(update_request): Json<UpdateUserRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    // Placeholder tenant context (implement proper JWT auth later)
    let tenant_id = crate::types::TenantId::from_uuid(Uuid::new_v4());
    let request_id = Uuid::new_v4();

    // For now, skip user access check (implement proper auth later)
    // In real implementation: check if user can update this user profile

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
        UPDATE users 
        SET name = COALESCE($3, name),
            email = COALESCE($4, email),
            updated_at = $5
        WHERE id = $1 AND tenant_id = $2
        RETURNING *
        "#;

    let name_ref = update_request.name.as_deref();
    let email_ref = update_request.email.as_deref();
    
    let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![
        &user_id,
        tenant_id.as_uuid(),
        &name_ref,
        &email_ref,
        &now,
    ];

    match client.query_opt(query, &params).await {
        Ok(Some(row)) => {
            let user = match row_to_user(&row) {
                Ok(user) => user,
                Err(e) => {
                    error!("Failed to parse user row: {}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            };

            let response = ApiResponse::success(user, request_id);
            Ok(Json(response))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to update user: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Update user role (admin only)
async fn update_user_role(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(role_request): Json<UpdateUserRoleRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    // Placeholder tenant context (implement proper JWT auth later)
    let tenant_id = crate::types::TenantId::from_uuid(Uuid::new_v4());
    let request_id = Uuid::new_v4();

    // For now, skip admin check (implement proper auth later)
    // In real implementation: check if user has Admin role

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
        UPDATE users 
        SET role = $3, updated_at = $4
        WHERE id = $1 AND tenant_id = $2
        RETURNING *
        "#;

    let role_str = user_role_to_string(&role_request.role);
    let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![
        &user_id,
        tenant_id.as_uuid(),
        &role_str,
        &now,
    ];

    match client.query_opt(query, &params).await {
        Ok(Some(row)) => {
            let user = match row_to_user(&row) {
                Ok(user) => user,
                Err(e) => {
                    error!("Failed to parse user row: {}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            };

            let response = ApiResponse::success(user, request_id);
            Ok(Json(response))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to update user role: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get current user profile
async fn get_current_user(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    // Placeholder tenant context (implement proper JWT auth later)
    let tenant_id = crate::types::TenantId::from_uuid(Uuid::new_v4());
    let request_id = Uuid::new_v4();

    // For now, return a placeholder current user (implement proper JWT auth later)
    // In real implementation: extract user ID from JWT and fetch from database

    let placeholder_user_id = Uuid::new_v4();

    // Get database connection
    let client = match state.db.postgres().get().await {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let query = "SELECT * FROM users WHERE id = $1 AND tenant_id = $2";
    let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![&placeholder_user_id, tenant_id.as_uuid()];

    match client.query_opt(query, &params).await {
        Ok(Some(row)) => {
            let user = match row_to_user(&row) {
                Ok(user) => user,
                Err(e) => {
                    error!("Failed to parse user row: {}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            };

            let response = ApiResponse::success(user, request_id);
            Ok(Json(response))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get current user: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Request/Response types
#[derive(Debug, Deserialize)]
struct ListUsersQuery {
    limit: Option<u32>,
    offset: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct CreateUserRequest {
    email: String,
    name: String,
    role: UserRole,
}

#[derive(Debug, Deserialize)]
struct UpdateUserRequest {
    name: Option<String>,
    email: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UpdateUserRoleRequest {
    role: UserRole,
}
