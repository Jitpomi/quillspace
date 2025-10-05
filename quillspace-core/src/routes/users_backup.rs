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
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use uuid::Uuid;

/// Create user management routes
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_users).post(create_user))
        .route("/:user_id", get(get_user).put(update_user))
        .route("/:user_id/role", put(update_user_role))
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

    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);

    let query = sqlx::query_as::<_, User>(
        r#"
        SELECT * FROM users 
        WHERE tenant_id = $1 
        ORDER BY created_at DESC 
        LIMIT $2 OFFSET $3
        "#
    )
    .bind(*tenant_id.as_uuid())
    .bind(limit as i64)
    .bind(offset as i64);

    match query.fetch_all(state.db.postgres()).await {
        Ok(users) => {
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

    let query = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (id, tenant_id, email, name, role, created_at, updated_at, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING *
        "#
    )
    .bind(user_id)
    .bind(*tenant_id.as_uuid())
    .bind(&user_request.email)
    .bind(&user_request.name)
    .bind(user_request.role.clone())
    .bind(now)
    .bind(now)
    .bind(true);

    match query.fetch_one(state.db.postgres()).await {
        Ok(user) => {
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
async fn create_user(
    State(state): State<AppState>,
    Json(user_request): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    // Placeholder tenant context (implement proper JWT auth later)
    let tenant_id = crate::types::TenantId::from_uuid(Uuid::new_v4());
    let request_id = Uuid::new_v4();

    // For now, return a placeholder current user (implement proper JWT auth later)
    // In real implementation: extract user ID from JWT and fetch from database

    let placeholder_user_id = Uuid::new_v4();

    let query = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE id = $1 AND tenant_id = $2"
    )
    .bind(placeholder_user_id)
    .bind(*tenant_id.as_uuid());

    match query.fetch_optional(state.db.postgres()).await {
        Ok(Some(user)) => {
            let response = ApiResponse::success(user, request_id);
            Ok(Json(response))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get current user: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
    // In real implementation: check if user has Admin role

    let user_id = Uuid::new_v4();
    let now = chrono::Utc::now();

    let query = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (id, tenant_id, email, name, role, created_at, updated_at, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING *
        "#
    )
    .bind(user_id)
    .bind(*tenant_id.as_uuid())
    .bind(&user_request.email)
    .bind(&user_request.name)
    .bind(user_request.role.clone())
    .bind(now)
    .bind(now)
    .bind(true);

    match query.fetch_one(state.db.postgres()).await {
        Ok(user) => {
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

    let query = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE id = $1 AND tenant_id = $2"
    )
    .bind(user_id)
    .bind(*tenant_id.as_uuid());

    match query.fetch_optional(state.db.postgres()).await {
        Ok(Some(user)) => {
            let response = ApiResponse::success(user, request_id);
            Ok(Json(response))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get user: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
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

    let query = sqlx::query_as::<_, User>(
        r#"
        UPDATE users 
        SET name = COALESCE($3, name),
            email = COALESCE($4, email),
            updated_at = $5
        WHERE id = $1 AND tenant_id = $2
        RETURNING *
        "#
    )
    .bind(user_id)
    .bind(*tenant_id.as_uuid())
    .bind(update_request.name.as_deref())
    .bind(update_request.email.as_deref())
    .bind(now);

    match query.fetch_optional(state.db.postgres()).await {
        Ok(Some(user)) => {
            info!(user_id = %user_id, "User updated");
            let response = ApiResponse::success(user, request_id);
            Ok(Json(response))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to update user: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
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

    let query = sqlx::query_as::<_, User>(
        r#"
        UPDATE users 
        SET role = $3, updated_at = $4
        WHERE id = $1 AND tenant_id = $2
        RETURNING *
        "#
    )
    .bind(user_id)
    .bind(*tenant_id.as_uuid())
    .bind(role_request.role.clone())
    .bind(now);

    match query.fetch_optional(state.db.postgres()).await {
        Ok(Some(user)) => {
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
async fn update_user_role(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(role_request): Json<UpdateUserRoleRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    // Placeholder tenant context (implement proper JWT auth later)
    let tenant_id = crate::types::TenantId::from_uuid(Uuid::new_v4());
    let request_id = Uuid::new_v4();

    // For now, return a placeholder current user (implement proper JWT auth later)
    // In real implementation: extract user ID from JWT and fetch from database

    let placeholder_user_id = Uuid::new_v4();

    let query = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE id = $1 AND tenant_id = $2"
    )
    .bind(placeholder_user_id)
    .bind(*tenant_id.as_uuid());

    match query.fetch_optional(state.db.postgres()).await {
        Ok(Some(user)) => {
            let response = ApiResponse::success(user, request_id);
            Ok(Json(response))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get current user: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
    // In real implementation: check if user has Admin role

    let now = chrono::Utc::now();

    let query = sqlx::query_as::<_, User>(
        r#"
        UPDATE users 
        SET role = $3, updated_at = $4
        WHERE id = $1 AND tenant_id = $2
        RETURNING *
        "#
    )
    .bind(user_id)
    .bind(*tenant_id.as_uuid())
    .bind(role_request.role.clone())
    .bind(now);

    match query.fetch_optional(state.db.postgres()).await {
        Ok(Some(user)) => {

            let response = ApiResponse::success(user, request_id);
            Ok(Json(response))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to update user role: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
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

    let query = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE id = $1 AND tenant_id = $2"
    )
    .bind(placeholder_user_id)
    .bind(*tenant_id.as_uuid());

    match query.fetch_optional(state.db.postgres()).await {
        Ok(Some(user)) => {
            let response = ApiResponse::success(user, request_id);
            Ok(Json(response))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get current user: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
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
