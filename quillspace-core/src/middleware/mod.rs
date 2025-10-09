pub mod auth;
pub mod tenant;
pub mod observability;
pub mod rate_limit;

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use tracing::{info, warn};
use uuid::Uuid;

/// Request ID middleware - adds unique request ID to all requests
pub async fn request_id_middleware(mut request: Request, next: Next) -> Result<Response, StatusCode> {
    let request_id = Uuid::new_v4();
    
    // Add request ID to headers for downstream services
    if let Ok(header_value) = request_id.to_string().parse() {
        request.headers_mut().insert("x-request-id", header_value);
    }

    // Store request ID in extensions for use in handlers
    request.extensions_mut().insert(request_id);

    let response = next.run(request).await;
    Ok(response)
}

/// Request timing middleware - logs request duration
pub async fn timing_middleware(request: Request, next: Next) -> Result<Response, StatusCode> {
    let start = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    
    let response = next.run(request).await;
    
    let duration = start.elapsed();
    let status = response.status();
    
    if status.is_server_error() {
        warn!(
            method = %method,
            uri = %uri,
            status = %status,
            duration_ms = duration.as_millis(),
            "Request completed with error"
        );
    } else {
        info!(
            method = %method,
            uri = %uri,
            status = %status,
            duration_ms = duration.as_millis(),
            "Request completed"
        );
    }
    
    Ok(response)
}

/// Health check middleware - bypasses other middleware for health endpoints
pub async fn health_check_middleware(request: Request, next: Next) -> Response {
    let path = request.uri().path();
    
    if path == "/health" || path == "/ready" {
        // Simple health check response
        if path == "/health" {
            return match Response::builder()
                .status(StatusCode::OK)
                .body("OK".into()) {
                Ok(response) => response,
                Err(_) => Response::new("OK".into()),
            };
        }
        
        if path == "/ready" {
            // Readiness check - could be enhanced with database connectivity checks
            return match Response::builder()
                .status(StatusCode::OK)
                .body("Ready".into()) {
                Ok(response) => response,
                Err(_) => Response::new("Ready".into()),
            };
        }
    }
    
    next.run(request).await
}
