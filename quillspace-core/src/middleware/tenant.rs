use crate::{
    types::{RequestContext, TenantId, UserId, UserRole},
    auth::JwtManager,
};
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use std::{sync::Arc, pin::Pin, future::Future};
use tracing::{debug, warn};
use uuid::Uuid;

/// Tenant context extraction middleware
pub async fn tenant_context_middleware(
    State(jwt_manager): State<Arc<JwtManager>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let headers = request.headers();
    
    // Try to extract tenant context from JWT token
    if let Some(context) = extract_tenant_from_jwt(headers, &jwt_manager).await? {
        request.extensions_mut().insert(context);
        Ok(next.run(request).await)
    } else {
        // Try to extract tenant from subdomain or header
        if let Some(context) = extract_tenant_from_request(headers).await? {
            request.extensions_mut().insert(context);
            Ok(next.run(request).await)
        } else {
            warn!("No tenant context found in request");
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// Extract tenant context from JWT token
async fn extract_tenant_from_jwt(
    headers: &HeaderMap,
    jwt_manager: &JwtManager,
) -> Result<Option<RequestContext>, StatusCode> {
    if let Some(auth_header) = headers.get("authorization") {
        let auth_str = auth_header.to_str().map_err(|_| StatusCode::BAD_REQUEST)?;
        
        if let Some(token) = auth_str.strip_prefix("Bearer ") {
            match jwt_manager.verify_token(token) {
                Ok(claims) => {
                    let tenant_id = Uuid::parse_str(&claims.tenant_id)
                        .map_err(|_| StatusCode::BAD_REQUEST)?;
                    let user_id = Uuid::parse_str(&claims.sub)
                        .map_err(|_| StatusCode::BAD_REQUEST)?;
                    
                    let role = match claims.role.as_str() {
                        "admin" => UserRole::Admin,
                        "editor" => UserRole::Editor,
                        "viewer" => UserRole::Viewer,
                        _ => return Err(StatusCode::BAD_REQUEST),
                    };
                    
                    debug!(
                        tenant_id = %tenant_id,
                        user_id = %user_id,
                        role = ?role,
                        "Extracted tenant context from JWT"
                    );
                    
                    let context = RequestContext::new(TenantId::from(tenant_id))
                        .with_user(UserId::from(user_id), role);
                    
                    return Ok(Some(context));
                }
                Err(e) => {
                    warn!("Invalid JWT token: {:?}", e);
                    return Err(StatusCode::UNAUTHORIZED);
                }
            }
        }
    }
    
    Ok(None)
}

/// Extract tenant context from request headers or subdomain
async fn extract_tenant_from_request(
    headers: &HeaderMap,
) -> Result<Option<RequestContext>, StatusCode> {
    // Try to get tenant ID from X-Tenant-ID header
    if let Some(tenant_header) = headers.get("x-tenant-id") {
        let tenant_str = tenant_header.to_str().map_err(|_| StatusCode::BAD_REQUEST)?;
        let tenant_id = Uuid::parse_str(tenant_str).map_err(|_| StatusCode::BAD_REQUEST)?;
        
        let context = RequestContext::new(TenantId::from(tenant_id));
        
        debug!(tenant_id = %tenant_id, "Extracted tenant context from header");
        return Ok(Some(context));
    }
    
    // Try to extract from Host header (subdomain-based routing)
    if let Some(host_header) = headers.get("host") {
        let host_str = host_header.to_str().map_err(|_| StatusCode::BAD_REQUEST)?;
        
        // Extract subdomain (e.g., "tenant1.quillspace.com" -> "tenant1")
        if let Some(subdomain) = extract_subdomain(host_str) {
            // In a real implementation, you'd look up the tenant ID by subdomain
            // For now, we'll generate a deterministic UUID from the subdomain
            let tenant_id = generate_tenant_id_from_subdomain(&subdomain);
            let context = RequestContext::new(TenantId::from(tenant_id));
            
            debug!(
                subdomain = %subdomain,
                tenant_id = %tenant_id,
                "Extracted tenant context from subdomain"
            );
            return Ok(Some(context));
        }
    }
    
    Ok(None)
}

/// Extract subdomain from host header
fn extract_subdomain(host: &str) -> Option<String> {
    let parts: Vec<&str> = host.split('.').collect();
    if parts.len() >= 3 {
        // Skip common prefixes like "www"
        let subdomain = parts[0];
        if subdomain != "www" && subdomain != "api" {
            return Some(subdomain.to_string());
        }
    }
    None
}

/// Generate a deterministic tenant ID from subdomain
/// In production, this should be a database lookup
fn generate_tenant_id_from_subdomain(subdomain: &str) -> Uuid {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    subdomain.hash(&mut hasher);
    let hash = hasher.finish();
    
    // Create a deterministic UUID from the hash
    let bytes = [
        (hash >> 56) as u8,
        (hash >> 48) as u8,
        (hash >> 40) as u8,
        (hash >> 32) as u8,
        (hash >> 24) as u8,
        (hash >> 16) as u8,
        (hash >> 8) as u8,
        hash as u8,
        0, 0, 0, 0, 0, 0, 0, 0,
    ];
    
    Uuid::from_bytes(bytes)
}

/// Middleware to require authentication
pub async fn require_auth_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Check if request context has authenticated user
    if let Some(context) = request.extensions().get::<RequestContext>() {
        if context.user_id.is_some() {
            Ok(next.run(request).await)
        } else {
            Err(StatusCode::UNAUTHORIZED)
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

/// Middleware to require specific role
pub fn require_role_middleware(required_role: UserRole) -> impl Clone + Fn(Request, Next) -> Pin<Box<dyn Future<Output = Result<Response, StatusCode>> + Send>> {
    move |request: Request, next: Next| {
        let required_role = required_role.clone();
        Box::pin(async move {
            if let Some(context) = request.extensions().get::<RequestContext>() {
                if let Some(user_role) = &context.user_role {
                    match (user_role, &required_role) {
                        (UserRole::Admin, _) => Ok(next.run(request).await), // Admin can access everything
                        (UserRole::Editor, UserRole::Editor | UserRole::Viewer) => Ok(next.run(request).await),
                        (UserRole::Viewer, UserRole::Viewer) => Ok(next.run(request).await),
                        _ => Err(StatusCode::FORBIDDEN),
                    }
                } else {
                    Err(StatusCode::UNAUTHORIZED)
                }
            } else {
                Err(StatusCode::UNAUTHORIZED)
            }
        })
    }
}

/// Extract request context from request extensions
pub fn get_request_context(request: &Request) -> Option<&RequestContext> {
    request.extensions().get::<RequestContext>()
}
