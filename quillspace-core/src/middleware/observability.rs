use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use tracing::{info, warn};

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
    let origin = request.headers().get("origin").cloned();
    
    let mut response = next.run(request).await;
    
    let headers = response.headers_mut();
    
    // Set CORS headers
    if let Some(origin) = origin {
        headers.insert("access-control-allow-origin", origin);
    } else {
        if let Ok(header_value) = "*".parse() {
            headers.insert("access-control-allow-origin", header_value);
        }
    }
    
    if let Ok(methods) = "GET, POST, PUT, DELETE, OPTIONS".parse() {
        headers.insert("access-control-allow-methods", methods);
    }
    
    if let Ok(headers_value) = "content-type, authorization, x-tenant-id, x-request-id".parse() {
        headers.insert("access-control-allow-headers", headers_value);
    }
    
    if let Ok(max_age) = "86400".parse() {
        headers.insert("access-control-max-age", max_age);
    }
    
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
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_string())
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
