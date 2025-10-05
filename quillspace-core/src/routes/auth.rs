use crate::{
    types::{ApiResponse, User, UserRole},
    AppState,
};
use axum::{
    extract::{Request, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use uuid::Uuid;

/// JWT Claims structure
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,        // User ID
    tenant_id: String,  // Tenant ID
    role: String,       // User role
    exp: usize,         // Expiration time
    iat: usize,         // Issued at
}

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

    let query = "SELECT id, tenant_id, email, name, role::text as role, is_active, created_at, updated_at FROM users WHERE email = $1 AND is_active = true";
    
    match client.query_opt(query, &[&login_request.email]).await {
        Ok(Some(row)) => {
            // Construct User from database row
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

            // TODO: Verify password hash here
            // For demo, we'll assume password is correct

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
            let now = Utc::now();
            let exp_time = now + Duration::seconds(state.config.auth.jwt_expiration);

            let claims = Claims {
                sub: user.id.to_string(),
                tenant_id: user.tenant_id.to_string(),
                role: match user.role {
                    UserRole::Admin => "admin".to_string(),
                    UserRole::Editor => "editor".to_string(),
                    UserRole::Viewer => "viewer".to_string(),
                },
                exp: exp_time.timestamp() as usize,
                iat: now.timestamp() as usize,
            };

            match encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
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
    match jsonwebtoken::decode::<Claims>(
        &refresh_request.refresh_token,
        &jsonwebtoken::DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &jsonwebtoken::Validation::default(),
    ) {
        Ok(token_data) => {
            let old_claims = token_data.claims;
            
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

                    let now = Utc::now();
                    let exp_time = now + Duration::seconds(state.config.auth.jwt_expiration);

                    let new_claims = Claims {
                        sub: user.id.to_string(),
                        tenant_id: user.tenant_id.to_string(),
                        role: old_claims.role,
                        exp: exp_time.timestamp() as usize,
                        iat: now.timestamp() as usize,
                    };

                    match encode(
                        &Header::default(),
                        &new_claims,
                        &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
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
    State(_state): State<AppState>,
    Json(_logout_request): Json<LogoutRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let request_id = Uuid::new_v4(); // Generate request ID

    // In a real implementation, you would:
    // 1. Add the token to a blacklist
    // 2. Revoke refresh tokens
    // 3. Log the logout event
    // 4. Clear any server-side sessions

    info!("User logged out successfully");

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
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    // Extract token from "Bearer <token>"
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    // Decode and validate JWT token
    let claims = match jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &jsonwebtoken::Validation::default(),
    ) {
        Ok(token_data) => token_data.claims,
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

// Request/Response types
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

#[derive(Debug, Deserialize)]
struct LogoutRequest {
    token: String,
}

#[derive(Debug, Serialize)]
struct LogoutResponse {
    message: String,
}
