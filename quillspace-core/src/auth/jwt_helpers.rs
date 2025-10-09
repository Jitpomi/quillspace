use tracing::{debug, error};
use crate::types::{TenantId, UserRole};
use axum::http::{HeaderMap, StatusCode};
use uuid::Uuid;

/// Complete authentication context
pub struct AuthContext {
    pub tenant_id: TenantId,
    pub user_id: Uuid,
    pub user_role: UserRole,
}

/// Extract complete authentication context from JWT token
pub fn extract_auth_context_with_role(headers: &HeaderMap, jwt_manager: &crate::auth::jwt::JwtManager) -> Result<AuthContext, StatusCode> {
    let auth_header = headers
        .get("authorization")
        .and_then(|h| match h.to_str() {
            Ok(s) => Some(s),
            Err(e) => {
                error!("Invalid authorization header encoding - potential security issue: {}", e);
                None
            }
        })
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    let claims = jwt_manager.verify_token(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    let tenant_id = TenantId::from_uuid(
        Uuid::parse_str(&claims.tenant_id).map_err(|_| StatusCode::UNAUTHORIZED)?
    );
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::UNAUTHORIZED)?;
    let user_role = crate::auth::extract_user_role_from_jwt(&claims)?;
    
    Ok(AuthContext {
        tenant_id,
        user_id,
        user_role,
    })
}

/// Extract tenant and user context from JWT token - UNIVERSAL HELPER (backward compatibility)
pub fn extract_auth_context(headers: &HeaderMap, jwt_manager: &crate::auth::jwt::JwtManager) -> Result<(TenantId, Uuid), StatusCode> {
    let auth_header = headers
        .get("authorization")
        .and_then(|h| match h.to_str() {
            Ok(s) => Some(s),
            Err(e) => {
                error!("Invalid authorization header encoding - potential security issue: {}", e);
                None
            }
        })
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    let claims = jwt_manager.verify_token(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    let tenant_id = TenantId::from_uuid(
        Uuid::parse_str(&claims.tenant_id).map_err(|_| StatusCode::UNAUTHORIZED)?
    );
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    Ok((tenant_id, user_id))
}

/// Extract only tenant context from JWT token
pub fn extract_tenant_context(headers: &HeaderMap, jwt_manager: &crate::auth::jwt::JwtManager) -> Result<TenantId, StatusCode> {
    let (tenant_id, _) = extract_auth_context(headers, jwt_manager)?;
    Ok(tenant_id)
}
