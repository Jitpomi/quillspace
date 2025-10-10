use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    auth::jwt_helpers::extract_auth_context_with_role,
    services::rls::{RlsService, SecurityTest, TenantSecurityStatus},
    types::{ApiResponse, TenantId, UserId},
    AppState,
};

/// Security management routes for admin users
pub fn security_router() -> Router<AppState> {
    Router::new()
        .route("/status", get(get_security_status))
        .route("/verify", get(verify_security))
        .route("/isolation", get(get_isolation_mode).post(set_isolation_mode))
        .route("/permissions", get(get_user_permissions))
}

/// Get comprehensive security status for the tenant
pub async fn get_security_status(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, StatusCode> {
    let auth_context = extract_auth_context_with_role(&headers, &state.jwt_manager)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let request_id = Uuid::new_v4();

    // Require admin permission for security status
    state.authorizer
        .require_permission(&auth_context.user_role, "security", "admin", &auth_context.tenant_id.to_string())
        .await
        .map_err(|_| StatusCode::FORBIDDEN)?;

    let rls_service = RlsService::new(state.db.postgres().clone());
    
    match rls_service.get_security_status(&auth_context.tenant_id).await {
        Ok(status) => {
            info!("Security status retrieved for tenant {}", auth_context.tenant_id);
            let response = ApiResponse::success(status, request_id);
            Ok((StatusCode::OK, Json(response)))
        }
        Err(e) => {
            error!("Failed to get security status: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Verify RLS security implementation
pub async fn verify_security(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, StatusCode> {
    let auth_context = extract_auth_context_with_role(&headers, &state.jwt_manager)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let request_id = Uuid::new_v4();

    // Require admin permission for security verification
    state.authorizer
        .require_permission(&auth_context.user_role, "security", "admin", &auth_context.tenant_id.to_string())
        .await
        .map_err(|_| StatusCode::FORBIDDEN)?;

    let rls_service = RlsService::new(state.db.postgres().clone());
    let user_id = UserId::from_uuid(auth_context.user_id);
    
    match rls_service.verify_security(&auth_context.tenant_id, &user_id).await {
        Ok(tests) => {
            info!("Security verification completed for tenant {}", auth_context.tenant_id);
            let response = ApiResponse::success(tests, request_id);
            Ok((StatusCode::OK, Json(response)))
        }
        Err(e) => {
            error!("Failed to verify security: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get current tenant isolation mode
pub async fn get_isolation_mode(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, StatusCode> {
    let auth_context = extract_auth_context_with_role(&headers, &state.jwt_manager)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let request_id = Uuid::new_v4();

    // Require admin permission to view isolation settings
    state.authorizer
        .require_permission(&auth_context.user_role, "tenants", "read", &auth_context.tenant_id.to_string())
        .await
        .map_err(|_| StatusCode::FORBIDDEN)?;

    let rls_service = RlsService::new(state.db.postgres().clone());
    
    match rls_service.get_tenant_isolation_mode(&auth_context.tenant_id).await {
        Ok(mode) => {
            let response_data = IsolationModeResponse { mode };
            let response = ApiResponse::success(response_data, request_id);
            Ok((StatusCode::OK, Json(response)))
        }
        Err(e) => {
            error!("Failed to get isolation mode: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Set tenant isolation mode (admin only)
pub async fn set_isolation_mode(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<SetIsolationModeRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let auth_context = extract_auth_context_with_role(&headers, &state.jwt_manager)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let request_id = Uuid::new_v4();

    // Require admin permission to configure tenant settings
    state.authorizer
        .require_permission(&auth_context.user_role, "tenants", "configure", &auth_context.tenant_id.to_string())
        .await
        .map_err(|_| StatusCode::FORBIDDEN)?;

    // Validate isolation mode
    if !["collaborative", "isolated", "role_based"].contains(&request.mode.as_str()) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let rls_service = RlsService::new(state.db.postgres().clone());
    let user_id = UserId::from_uuid(auth_context.user_id);
    
    match rls_service.set_tenant_isolation_mode(&auth_context.tenant_id, &user_id, &request.mode).await {
        Ok(result) => {
            info!("Isolation mode changed for tenant {}: {}", auth_context.tenant_id, result);
            let response_data = IsolationModeChangeResponse { 
                message: result,
                new_mode: request.mode 
            };
            let response = ApiResponse::success(response_data, request_id);
            Ok((StatusCode::OK, Json(response)))
        }
        Err(e) => {
            error!("Failed to set isolation mode: {}", e);
            if e.to_string().contains("Only tenant administrators") {
                Err(StatusCode::FORBIDDEN)
            } else if e.to_string().contains("Invalid isolation mode") {
                Err(StatusCode::BAD_REQUEST)
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

/// Get user permissions from Casbin
pub async fn get_user_permissions(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, StatusCode> {
    let auth_context = extract_auth_context_with_role(&headers, &state.jwt_manager)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let request_id = Uuid::new_v4();

    // Users can view their own permissions
    let permissions = state.authorizer.get_permissions_for_user(&auth_context.user_role).await;
    
    let response_data = UserPermissionsResponse {
        user_role: auth_context.user_role.to_string(),
        tenant_id: auth_context.tenant_id.to_string(),
        permissions: permissions.into_iter().map(|p| Permission {
            resource: p.get(1).cloned().unwrap_or_default(),
            action: p.get(2).cloned().unwrap_or_default(),
            tenant: p.get(3).cloned().unwrap_or_default(),
        }).collect(),
    };

    let response = ApiResponse::success(response_data, request_id);
    Ok((StatusCode::OK, Json(response)))
}

// Request/Response schemas
#[derive(Debug, Deserialize)]
pub struct SetIsolationModeRequest {
    pub mode: String,
}

#[derive(Debug, Serialize)]
pub struct IsolationModeResponse {
    pub mode: String,
}

#[derive(Debug, Serialize)]
pub struct IsolationModeChangeResponse {
    pub message: String,
    pub new_mode: String,
}

#[derive(Debug, Serialize)]
pub struct UserPermissionsResponse {
    pub user_role: String,
    pub tenant_id: String,
    pub permissions: Vec<Permission>,
}

#[derive(Debug, Serialize)]
pub struct Permission {
    pub resource: String,
    pub action: String,
    pub tenant: String,
}
