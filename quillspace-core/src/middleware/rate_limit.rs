use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, warn};

/// Simple in-memory rate limiter
/// In production, use Redis or a proper rate limiting service
#[derive(Clone)]
pub struct RateLimiter {
    requests: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    max_requests: usize,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window,
        }
    }

    pub fn check_rate_limit(&self, key: &str) -> bool {
        let mut requests = self.requests.lock().unwrap();
        let now = Instant::now();
        
        // Get or create request history for this key
        let request_times = requests.entry(key.to_string()).or_insert_with(Vec::new);
        
        // Remove old requests outside the window
        request_times.retain(|&time| now.duration_since(time) < self.window);
        
        // Check if we're under the limit
        if request_times.len() < self.max_requests {
            request_times.push(now);
            true
        } else {
            false
        }
    }
}

/// Rate limiting middleware
pub async fn rate_limit_middleware(request: Request, next: Next) -> Result<Response, StatusCode> {
    // Create a simple rate limiter (100 requests per minute)
    static RATE_LIMITER: std::sync::OnceLock<RateLimiter> = std::sync::OnceLock::new();
    let limiter = RATE_LIMITER.get_or_init(|| RateLimiter::new(100, Duration::from_secs(60)));
    
    // Use IP address as the key (in production, consider user ID or API key)
    let client_ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");
    
    if limiter.check_rate_limit(client_ip) {
        debug!("Rate limit check passed for {}", client_ip);
        Ok(next.run(request).await)
    } else {
        warn!("Rate limit exceeded for {}", client_ip);
        Err(StatusCode::TOO_MANY_REQUESTS)
    }
}

/// Per-tenant rate limiting middleware
pub async fn tenant_rate_limit_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract tenant ID from headers or context
    let tenant_id = request
        .headers()
        .get("x-tenant-id")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("default");
    
    // Create tenant-specific rate limiter (1000 requests per minute per tenant)
    static TENANT_LIMITER: std::sync::OnceLock<RateLimiter> = std::sync::OnceLock::new();
    let limiter = TENANT_LIMITER.get_or_init(|| RateLimiter::new(1000, Duration::from_secs(60)));
    
    if limiter.check_rate_limit(tenant_id) {
        debug!("Tenant rate limit check passed for {}", tenant_id);
        Ok(next.run(request).await)
    } else {
        warn!("Tenant rate limit exceeded for {}", tenant_id);
        Err(StatusCode::TOO_MANY_REQUESTS)
    }
}
