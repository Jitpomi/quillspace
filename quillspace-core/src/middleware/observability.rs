use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use tracing::{error, info, warn};

/// Metrics middleware - simplified version for compilation
pub async fn metrics_middleware(request: Request, next: Next) -> Result<Response, StatusCode> {
    let start = Instant::now();
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    
    let response = next.run(request).await;
    
    let duration = start.elapsed();
    info!(
        method = %method,
        path = %path,
        status = %response.status(),
        duration_ms = duration.as_millis(),
        "Request processed"
    );
    
    Ok(response)
}

/// Tracing middleware - adds structured logging spans
pub async fn tracing_middleware(request: Request, next: Next) -> Result<Response, StatusCode> {
    let method = request.method().clone();
    let uri = request.uri().clone();
    
    info!(method = %method, uri = %uri, "Processing request");
    
    let response = next.run(request).await;
    
    info!(
        method = %method,
        uri = %uri,
        status = %response.status(),
        "Request completed"
    );
    
    Ok(response)
}

/// CORS middleware for cross-origin requests
pub async fn cors_middleware(request: Request, next: Next) -> Result<Response, StatusCode> {
    use axum::http::{Method, HeaderValue};
    
    // Handle preflight requests
    if request.method() == Method::OPTIONS {
        let mut response = match Response::builder()
            .status(StatusCode::OK)
            .body("".into()) {
            Ok(response) => response,
            Err(e) => {
                error!("Failed to build CORS response: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };
            
        let headers = response.headers_mut();
        
        // Set CORS headers for preflight
        headers.insert("access-control-allow-origin", HeaderValue::from_static("*"));
        headers.insert("access-control-allow-methods", HeaderValue::from_static("GET, POST, PUT, DELETE, OPTIONS"));
        headers.insert("access-control-allow-headers", HeaderValue::from_static("content-type, authorization, x-tenant-id, x-request-id"));
        headers.insert("access-control-max-age", HeaderValue::from_static("86400"));
        headers.insert("access-control-allow-credentials", HeaderValue::from_static("true"));
        
        return Ok(response);
    }
    
    let origin = request.headers().get("origin").cloned();
    
    let mut response = next.run(request).await;
    
    let headers = response.headers_mut();
    
    // Set CORS headers for actual requests
    if let Some(origin) = origin {
        headers.insert("access-control-allow-origin", origin);
    } else {
        headers.insert("access-control-allow-origin", HeaderValue::from_static("*"));
    }
    
    headers.insert("access-control-allow-methods", HeaderValue::from_static("GET, POST, PUT, DELETE, OPTIONS"));
    headers.insert("access-control-allow-headers", HeaderValue::from_static("content-type, authorization, x-tenant-id, x-request-id"));
    headers.insert("access-control-allow-credentials", HeaderValue::from_static("true"));
    
    Ok(response)
}

/// Security headers middleware
pub async fn security_headers_middleware(request: Request, next: Next) -> Result<Response, StatusCode> {
    let mut response = next.run(request).await;
    
    let headers = response.headers_mut();
    
    // Security headers - with proper error handling
    if let Ok(value) = "nosniff".parse() {
        headers.insert("x-content-type-options", value);
    }
    if let Ok(value) = "DENY".parse() {
        headers.insert("x-frame-options", value);
    }
    if let Ok(value) = "1; mode=block".parse() {
        headers.insert("x-xss-protection", value);
    }
    if let Ok(value) = "max-age=31536000; includeSubDomains".parse() {
        headers.insert("strict-transport-security", value);
    }
    if let Ok(value) = "default-src 'self'".parse() {
        headers.insert("content-security-policy", value);
    }
    
    Ok(response)
}

/// Extract header value as string
fn extract_header(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| match value.to_str() {
            Ok(s) => Some(s.to_string()),
            Err(e) => {
                warn!("Invalid header '{}' encoding: {}", name, e);
                None
            }
        })
}

/// Normalize path for metrics (remove IDs and dynamic segments)
fn normalize_path(path: &str) -> String {
    let segments: Vec<&str> = path.split('/').collect();
    let mut normalized = Vec::new();
    
    for segment in segments {
        if segment.is_empty() {
            continue;
        }
        
        // Replace UUIDs and numeric IDs with placeholders
        if is_uuid(segment) {
            normalized.push("{id}");
        } else if segment.chars().all(|c| c.is_ascii_digit()) {
            normalized.push("{id}");
        } else {
            normalized.push(segment);
        }
    }
    
    format!("/{}", normalized.join("/"))
}

/// Check if a string is a UUID
fn is_uuid(s: &str) -> bool {
    s.len() == 36 && s.chars().enumerate().all(|(i, c)| {
        match i {
            8 | 13 | 18 | 23 => c == '-',
            _ => c.is_ascii_hexdigit(),
        }
    })
}
