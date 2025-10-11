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

/// Helper function to convert database role string to UserRole enum
fn parse_user_role(role_str: &str) -> UserRole {
    match role_str {
        "admin" => UserRole::Admin,
        "editor" => UserRole::Editor,
        "viewer" => UserRole::Viewer,
        _ => UserRole::Viewer, // Default fallback
    }
}

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

    // In a real implementation, you would:
    // 1. Validate password hash
    // 2. Check account status
    // 3. Implement rate limiting
    // 4. Log security events

    // For demo purposes, we'll do a simple email lookup
    let client = match state.db.postgres().get().await {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let query = "SELECT * FROM authenticate_user($1)";
    
    match client.query_opt(query, &[&login_request.email]).await {
        Ok(Some(row)) => {
            // Construct User from database row
            let role_str: String = row.try_get("role").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let role = parse_user_role(&role_str);

            let first_name: String = row.try_get("first_name").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let last_name: String = row.try_get("last_name").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let full_name = format!("{} {}", first_name, last_name);

            let user = User {
                id: row.try_get("id").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
                tenant_id: row.try_get("tenant_id").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
                email: row.try_get("email").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
                name: full_name,
                role,
                is_active: row.try_get("active").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
                created_at: row.try_get("created_at").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
                updated_at: row.try_get("updated_at").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
            };

            // Verify password hash
            let password_hash: String = row.try_get("password_hash")
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            
            if !bcrypt::verify(&login_request.password, &password_hash)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
                error!("Login attempt with invalid password: {}", login_request.email);
                return Err(StatusCode::UNAUTHORIZED);
            }

            // Fetch tenant information
            let tenant_query = "SELECT id, name, slug FROM tenants WHERE id = $1";
            let tenant_row = match client.query_one(tenant_query, &[&user.tenant_id]).await {
                Ok(row) => row,
                Err(e) => {
                    error!("Failed to fetch tenant: {}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            };

            let tenant = TenantInfo {
                id: tenant_row.try_get("id").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
                name: tenant_row.try_get("name").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
                slug: tenant_row.try_get("slug").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
            };

            info!("Creating JWT token for user: {}", user.id);
            
            let role_str = match user.role {
                UserRole::Admin => "admin",
                UserRole::Editor => "editor", 
                UserRole::Viewer => "viewer",
            };

            match state.jwt_manager.generate_token(
                &user.id.to_string(),
                &user.email,
                &user.name,
                role_str,
                &user.tenant_id.to_string()
            ) {
                Ok(token) => {
                    info!(
                        user_id = %user.id,
                        tenant_id = %user.tenant_id,
                        "User logged in successfully"
                    );

                    let response_data = LoginResponse {
                        token,
                        user: UserInfo {
                            id: user.id,
                            email: user.email,
                            name: user.name,
                            role: user.role,
                            tenant_id: user.tenant_id,
                        },
                        tenant,
                    };

                    let response = ApiResponse::success(response_data, request_id);
                    Ok(Json(response))
                }
                Err(e) => {
                    error!("Failed to generate JWT token: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
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

    // In a real implementation, you would:
    // 1. Validate the refresh token
    // 2. Check if it's been revoked
    // 3. Generate new access token
    // 4. Optionally rotate refresh token

    // For demo purposes, we'll decode the existing token and issue a new one
    match state.jwt_manager.verify_token(&refresh_request.refresh_token) {
        Ok(old_claims) => {
            
            // Verify user still exists and is active
            let user_id = Uuid::parse_str(&old_claims.sub)
                .map_err(|_| StatusCode::BAD_REQUEST)?;

            let client = match state.db.postgres().get().await {
                Ok(client) => client,
                Err(e) => {
                    error!("Failed to get database connection: {}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            };

            let query = "SELECT id, tenant_id, email, name, role, is_active, created_at, updated_at FROM users WHERE id = $1 AND is_active = true";
            
            match client.query_opt(query, &[&user_id]).await {
                Ok(Some(row)) => {
                    // Construct User from database row
                    let role_str: String = row.get("role");
                    let role = parse_user_role(&role_str);

                    let user = User {
                        id: row.get("id"),
                        tenant_id: row.get("tenant_id"),
                        email: row.get("email"),
                        name: row.get("name"),
                        role,
                        is_active: row.get("is_active"),
                        created_at: row.get("created_at"),
                        updated_at: row.get("updated_at"),
                    };

                    // Generate new JWT token using our JWT manager
                    match state.jwt_manager.generate_token(
                        &user.id.to_string(),
                        &user.email,
                        &user.name,
                        &user.role.to_string(),
                        &user.tenant_id.to_string(),
                    ) {
                        Ok(token) => {
                            let response_data = RefreshTokenResponse {
                                access_token: token,
                                token_type: "Bearer".to_string(),
                                expires_in: state.config.auth.jwt_expiration,
                            };

                            let response = ApiResponse::success(response_data, request_id);
                            Ok(Json(response))
                        }
                        Err(e) => {
                            error!("Failed to generate new JWT token: {}", e);
                            Err(StatusCode::INTERNAL_SERVER_ERROR)
                        }
                    }
                }
                Ok(None) => Err(StatusCode::UNAUTHORIZED),
                Err(e) => {
                    error!("Database error during token refresh: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
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

    // In a real implementation, you would:
    // 1. Add the token to a blacklist/Redis cache
    // 2. Revoke refresh tokens from database
    // 3. Log the logout event with user context
    // 4. Clear any server-side sessions
    // 5. Invalidate related tokens

    // For now, we'll log the logout event
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
    
    // Get database connection
    let client = match state.db.postgres().get().await {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    // Fetch user from database using the user ID from JWT claims
    let user_id: Uuid = claims.sub.parse().map_err(|_| StatusCode::UNAUTHORIZED)?;
    let query = "SELECT id, tenant_id, email, name, role::text as role, is_active, created_at, updated_at FROM users WHERE id = $1 AND is_active = true";
    
    match client.query_opt(query, &[&user_id]).await {
        Ok(Some(row)) => {
            let role_str: String = row.try_get("role").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let role = parse_user_role(&role_str);
            
            let user = User {
                id: row.try_get("id").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
                tenant_id: row.try_get("tenant_id").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
                email: row.try_get("email").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
                name: row.try_get("name").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
                role,
                is_active: row.try_get("is_active").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
                created_at: row.try_get("created_at").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
                updated_at: row.try_get("updated_at").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
            };
            
            let response = ApiResponse::success(user, request_id);
            Ok(Json(response))
        }
        Ok(None) => Err(StatusCode::UNAUTHORIZED),
        Err(e) => {
            error!("Database error during user lookup: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
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
    name: String,
    role: UserRole,
    tenant_id: Uuid,
}

#[derive(Debug, Serialize)]
struct TenantInfo {
    id: Uuid,
    name: String,
    slug: String,
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

// LogoutRequest is no longer needed since we extract token from Authorization header
// Keeping for backward compatibility if needed
#[derive(Debug, Deserialize)]
struct LogoutRequest {
    token: Option<String>,
}

#[derive(Debug, Serialize)]
struct LogoutResponse {
    message: String,
}
