use axum::{
    async_trait,
    extract::{FromRequestParts, Request},
    http::{request::Parts, StatusCode},
};
use crate::{
    auth::{
        casbin_auth::CasbinAuthorizer,
        jwt::JwtManager,
        jwt_helpers::AuthContext,
    },
    types::UserRole,
    AppState,
};
use uuid::Uuid;

/// Authentication context with Casbin authorization
#[derive(Clone)]
pub struct CasbinAuthContext {
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub user_role: UserRole,
    pub authorizer: CasbinAuthorizer,
}

impl CasbinAuthContext {
    /// Check if user has permission for a resource and action
    pub async fn check_permission(&self, resource: &str, action: &str) -> Result<bool, StatusCode> {
        self.authorizer
            .enforce(&self.user_role, resource, action, &self.tenant_id.to_string())
            .await
            .map_err(|e| {
                tracing::error!("Casbin authorization error: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })
    }

    /// Require permission, returning 403 if not authorized
    pub async fn require_permission(&self, resource: &str, action: &str) -> Result<(), StatusCode> {
        if self.check_permission(resource, action).await? {
            Ok(())
        } else {
            tracing::warn!(
                "Access denied: user {} (role: {:?}, tenant: {}) attempted {} on {}",
                self.user_id, self.user_role, self.tenant_id, action, resource
            );
            Err(StatusCode::FORBIDDEN)
        }
    }
}

#[async_trait]
impl FromRequestParts<AppState> for CasbinAuthContext {
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        // Extract JWT token from Authorization header
        let auth_header = parts
            .headers
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or(StatusCode::UNAUTHORIZED)?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(StatusCode::UNAUTHORIZED)?;

        // Verify JWT token
        let claims = state.jwt_manager.verify_token(token)
            .map_err(|e| {
                tracing::error!("JWT verification failed: {}", e);
                StatusCode::UNAUTHORIZED
            })?;

        // Parse user ID and tenant ID
        let user_id: Uuid = claims.sub.parse()
            .map_err(|e| {
                tracing::error!("Failed to parse user ID from JWT: {}", e);
                StatusCode::UNAUTHORIZED
            })?;

        let tenant_id: Uuid = claims.tenant_id.parse()
            .map_err(|e| {
                tracing::error!("Failed to parse tenant ID from JWT: {}", e);
                StatusCode::UNAUTHORIZED
            })?;

        // Parse user role
        let user_role = match claims.role.as_str() {
            "admin" => UserRole::Admin,
            "editor" => UserRole::Editor,
            "viewer" => UserRole::Viewer,
            _ => {
                tracing::error!("Unknown user role: {}", claims.role);
                return Err(StatusCode::UNAUTHORIZED);
            }
        };

        Ok(CasbinAuthContext {
            tenant_id,
            user_id,
            user_role,
            authorizer: state.authorizer.as_ref().clone(),
        })
    }
}
