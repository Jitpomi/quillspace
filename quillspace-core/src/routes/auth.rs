use crate::{
    types::{ApiResponse, User, UserRole},
    auth::{JwtManager, Claims},
    AppState,
};
use axum::{
    extract::{Request, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};
use uuid::Uuid;

// Using Claims from auth::jwt module


/// Create authentication routes
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/refresh", post(refresh_token))
        .route("/logout", post(logout))
        .route("/me", get(get_current_user))
}

/// User login
async fn login(
    State(state): State<AppState>,
    Json(login_request): Json<LoginRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let request_id = Uuid::new_v4(); // Generate request ID
    info!("Login attempt for email: {}", login_request.email);

    // Authenticate user with email and password
    let client = get_db_client(&state).await?;

    let query = "SELECT * FROM authenticate_user($1)";
    
    match client.query_opt(query, &[&login_request.email]).await {
        Ok(Some(row)) => {
            let user = User::from_row(&row).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            // Verify password hash
            let password_hash: String = row.try_get("password_hash")
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            
            if !bcrypt::verify(&login_request.password, &password_hash)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
                error!("Login attempt with invalid password: {}", login_request.email);
                return Err(StatusCode::UNAUTHORIZED);
            }

            // Fetch tenant information
            let tenant_query = "SELECT * FROM tenants WHERE id = $1";
            let tenant_row = match client.query_one(tenant_query, &[&user.tenant_id]).await {
                Ok(row) => row,
                Err(e) => {
                    error!("Failed to fetch tenant: {}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            };

            let tenant = TenantInfo::from_row(&tenant_row).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            info!("Creating JWT token for user: {}", user.id);
            
            let token = generate_jwt_token(&state.jwt_manager, &user)?;
            
            info!(
                user_id = %user.id,
                tenant_id = %user.tenant_id,
                "User logged in successfully"
            );

            let response_data = LoginResponse {
                token,
                user: UserInfo::from_user(&user),
                tenant,
            };

            let response = ApiResponse::success(response_data, request_id);
            Ok(Json(response))
        }
        Ok(None) => {
            // Don't reveal whether user exists or not
            error!("Login attempt with invalid credentials: {}", login_request.email);
            Err(StatusCode::UNAUTHORIZED)
        }
        Err(e) => {
            error!("Database error during login: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Refresh JWT token
async fn refresh_token(
    State(state): State<AppState>,
    Json(refresh_request): Json<RefreshTokenRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let request_id = Uuid::new_v4(); // Generate request ID

    // Validate refresh token and issue new access token
    match state.jwt_manager.verify_token(&refresh_request.refresh_token) {
        Ok(old_claims) => {
            
            // Verify user still exists and is active
            let user_id = Uuid::parse_str(&old_claims.sub)
                .map_err(|_| StatusCode::BAD_REQUEST)?;

            let client = get_db_client(&state).await?;
            let user = fetch_user_by_id(&client, user_id).await?;

            // Generate new JWT token
            let token = generate_jwt_token(&state.jwt_manager, &user)?;
            
            let response_data = RefreshTokenResponse {
                access_token: token,
                token_type: "Bearer".to_string(),
                expires_in: state.config.auth.jwt_expiration,
            };

            let response = ApiResponse::success(response_data, request_id);
            Ok(Json(response))
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

/// User logout
async fn logout(
    State(state): State<AppState>,
    request: Request,
) -> Result<impl IntoResponse, StatusCode> {
    let request_id = Uuid::new_v4(); // Generate request ID

    // Extract Authorization header to get user info for logging
    let auth_header = request
        .headers()
        .get("authorization")
        .and_then(|h| match h.to_str() {
            Ok(s) => Some(s),
            Err(e) => {
                warn!("Invalid authorization header encoding - potential security issue: {}", e);
                None
            }
        });
    
    let user_id = if let Some(auth_header) = auth_header {
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            // Decode token to get user ID for logging
            match state.jwt_manager.verify_token(token) {
                Ok(claims) => {
                    match Uuid::parse_str(&claims.sub) {
                        Ok(id) => Some(id),
                        Err(_) => None,
                    }
                }
                Err(_) => None,
            }
        } else {
            None
        }
    } else {
        None
    };

    // Log the logout event
    if let Some(user_id) = user_id {
        info!(
            user_id = %user_id,
            request_id = %request_id,
            "User logged out successfully"
        );
    } else {
        info!(
            request_id = %request_id,
            "Logout request processed (token invalid or missing)"
        );
    }

    let response_data = LogoutResponse {
        message: "Logged out successfully".to_string(),
    };

    let response = ApiResponse::success(response_data, request_id);
    Ok(Json(response))
}

/// Get current user information from JWT token
async fn get_current_user(
    State(state): State<AppState>,
    request: Request,
) -> Result<impl IntoResponse, StatusCode> {
    let request_id = Uuid::new_v4();
    
    // Extract Authorization header
    let auth_header = request
        .headers()
        .get("authorization")
        .and_then(|h| match h.to_str() {
            Ok(s) => Some(s),
            Err(e) => {
                error!("Invalid authorization header encoding - rejecting request: {}", e);
                None
            }
        })
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    // Extract token from "Bearer <token>"
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    // Decode and validate JWT token
    let claims = match state.jwt_manager.verify_token(token) {
        Ok(claims) => claims,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };
    
    // Get database connection and fetch user
    let client = get_db_client(&state).await?;
    let user_id: Uuid = claims.sub.parse().map_err(|_| StatusCode::UNAUTHORIZED)?;
    let user = fetch_user_by_id(&client, user_id).await?;
    
    let response = ApiResponse::success(user, request_id);
    Ok(Json(response))
}

// Request/Response schemas
#[derive(Debug, Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct LoginResponse {
    token: String,
    user: UserInfo,
    tenant: TenantInfo,
}

#[derive(Debug, Serialize)]
struct UserInfo {
    id: Uuid,
    email: String,
    first_name: String,
    last_name: String,
    role: UserRole,
    tenant_id: Uuid,
}

impl UserInfo {
    /// Create UserInfo from User
    pub fn from_user(user: &User) -> Self {
        UserInfo {
            id: user.id,
            email: user.email.clone(),
            first_name: user.first_name.clone(),
            last_name: user.last_name.clone(),
            role: user.role.clone(),
            tenant_id: user.tenant_id,
        }
    }
}

/// Helper function to convert UserRole to string
fn role_to_string(role: &UserRole) -> &'static str {
    match role {
        UserRole::Admin => "admin",
        UserRole::Editor => "editor", 
        UserRole::Viewer => "viewer",
    }
}

/// Helper function to format full name
fn format_full_name(user: &User) -> String {
    format!("{} {}", user.first_name, user.last_name)
}

/// Helper function to get database connection
async fn get_db_client(state: &AppState) -> Result<deadpool_postgres::Client, StatusCode> {
    state.db.postgres().get().await.map_err(|e| {
        error!("Failed to get database connection: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

/// Helper function to fetch user by ID
async fn fetch_user_by_id(client: &deadpool_postgres::Client, user_id: Uuid) -> Result<User, StatusCode> {
    let query = "SELECT * FROM users WHERE id = $1 AND active = true";
    
    match client.query_opt(query, &[&user_id]).await {
        Ok(Some(row)) => {
            User::from_row(&row).map_err(|_| {
                error!("Failed to parse user from database row");
                StatusCode::INTERNAL_SERVER_ERROR
            })
        }
        Ok(None) => Err(StatusCode::UNAUTHORIZED),
        Err(e) => {
            error!("Database error during user lookup: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Helper function to generate JWT token
fn generate_jwt_token(jwt_manager: &JwtManager, user: &User) -> Result<String, StatusCode> {
    let role_str = role_to_string(&user.role);
    
    jwt_manager.generate_token(
        &user.id.to_string(),
        &user.email,
        &user.first_name,
        &user.last_name,
        role_str,
        &user.tenant_id.to_string()
    ).map_err(|e| {
        error!("Failed to generate JWT token: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[derive(Debug, Serialize)]
struct TenantInfo {
    id: Uuid,
    name: String,
    slug: String,
}

impl TenantInfo {
    /// Create TenantInfo from database row
    pub fn from_row(row: &tokio_postgres::Row) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(TenantInfo {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            slug: row.try_get("slug")?,
        })
    }
}

#[derive(Debug, Deserialize)]
struct RefreshTokenRequest {
    refresh_token: String,
}

#[derive(Debug, Serialize)]
struct RefreshTokenResponse {
    access_token: String,
    token_type: String,
    expires_in: i64,
}


#[derive(Debug, Serialize)]
struct LogoutResponse {
    message: String,
}
