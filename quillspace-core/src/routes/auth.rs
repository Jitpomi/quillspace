use crate::{
    types::{ApiResponse, User, UserRole},
    AppState,
};
use axum::{
    extract::{Request, State},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
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
}

/// User login
async fn login(
    State(state): State<AppState>,
    Json(login_request): Json<LoginRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let request_id = Uuid::new_v4(); // Generate request ID

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

    let query = "SELECT id, tenant_id, email, name, role, is_active, created_at, updated_at FROM users WHERE email = $1 AND is_active = true";
    
    match client.query_opt(query, &[&login_request.email]).await {
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

            // TODO: Verify password hash here
            // For demo, we'll assume password is correct

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
                        access_token: token,
                        token_type: "Bearer".to_string(),
                        expires_in: state.config.auth.jwt_expiration,
                        user: UserInfo {
                            id: user.id,
                            email: user.email,
                            name: user.name,
                            role: user.role,
                            tenant_id: user.tenant_id,
                        },
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

// Request/Response types
#[derive(Debug, Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct LoginResponse {
    access_token: String,
    token_type: String,
    expires_in: i64,
    user: UserInfo,
}

#[derive(Debug, Serialize)]
struct UserInfo {
    id: Uuid,
    email: String,
    name: String,
    role: UserRole,
    tenant_id: Uuid,
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
