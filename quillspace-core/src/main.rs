mod config;
mod types;
mod database;
mod middleware;
mod routes;
mod services;

use axum::{
    extract::State,
    http::StatusCode,
    middleware::from_fn,
    response::IntoResponse,
    routing::{get, post},
    Json, Router, serve,
};
use config::AppConfig;
use database::DatabaseConnections;
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing::{info, warn};
use std::time::Duration;

// Enhanced application state with database connections
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub db: DatabaseConnections,
    pub jwt_secret: Arc<String>,
    pub request_count: Arc<Mutex<usize>>,
}

impl AppState {
    pub async fn new(config: AppConfig) -> anyhow::Result<Self> {
        let db = DatabaseConnections::new(&config.database.url, &config.clickhouse.url).await?;
        
        Ok(Self {
            jwt_secret: Arc::new(config.auth.jwt_secret.clone()),
            config: Arc::new(config),
            db,
            request_count: Arc::new(Mutex::new(0)),
        })
    }
}

// Response models
#[derive(Serialize)]
struct InfoResponse {
    app_name: String,
    version: String,
    request_count: usize,
}

#[derive(Deserialize)]
struct CreateItemRequest {
    name: String,
}

#[derive(Serialize)]
struct CreateItemResponse {
    id: u64,
    name: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration
    let config = AppConfig::from_env().unwrap_or_else(|_| {
        warn!("Failed to load config from environment, using defaults");
        AppConfig::default()
    });

    // Initialize tracing with environment filter
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting QuillSpace server with config: {:?}", config.server);

    // Skip metrics initialization for now to get basic server running
    // if config.observability.metrics_enabled {
    //     let recorder = metrics_exporter_prometheus::PrometheusBuilder::new()
    //         .build_recorder()?;
    //     metrics::set_global_recorder(recorder)?;
    //     info!("Prometheus metrics enabled on port {}", config.observability.prometheus_port);
    // }

    // Create enhanced app state with database connections
    let state = AppState::new(config.clone()).await?;
    info!("Database connections established");

    // Setup row-level security (temporarily disabled for debugging)
    // database::postgres::setup_rls(state.db.postgres()).await?;
    info!("Row-level security policies skipped for now");

    // Build the enhanced router with comprehensive middleware
    let app = create_app(state).await?;

    // Define the address to listen on (0.0.0.0 for Docker compatibility)
    let addr = SocketAddr::from(([
        0, 0, 0, 0
    ], config.server.port));
    
    info!("🚀 QuillSpace server listening on http://{}", addr);
    info!("📊 Metrics available on http://localhost:{}/metrics", config.observability.prometheus_port);
    info!("📚 API documentation available at http://{}/docs", addr);

    // Run the server
    let listener = TcpListener::bind(addr).await?;
    serve(listener, app).await?;
    
    Ok(())
}

/// Create the application router with all middleware and routes
async fn create_app(state: AppState) -> anyhow::Result<Router> {
    let _jwt_secret = state.jwt_secret.clone();
    
    let app = Router::new()
        // Health check routes (no middleware)
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
        
        // Legacy routes for compatibility
        .route("/", get(root))
        .route("/ping", get(ping))
        
        // API routes
        .nest("/api", create_api_routes())
        
        // Middleware stack (applied in reverse order)
        .layer(
            ServiceBuilder::new()
                .layer(from_fn(middleware::observability::metrics_middleware))
                .layer(from_fn(middleware::observability::cors_middleware))
                .layer(from_fn(middleware::observability::security_headers_middleware))
        )
        .with_state(state);

    Ok(app)
}

/// Create API routes with proper organization
fn create_api_routes() -> Router<AppState> {
    Router::new()
        // Enable API routes
        .nest("/tenants", routes::tenants::create_routes())
        .nest("/content", routes::content::create_routes())
        .nest("/analytics", routes::analytics::create_routes())
        .nest("/users", routes::users::create_routes())
        // Health and monitoring endpoints
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
        
        // Add a simple test route
        .route("/test", get(|| async { "API is working!" }))
}

async fn health_check() -> &'static str {
    "OK"
}

async fn readiness_check() -> &'static str {
    "Ready"
}

// Legacy route handlers for compatibility
async fn root() -> &'static str {
    "🚀 Welcome to QuillSpace - High-Performance Multi-Tenant Publishing Platform"
}

async fn ping() -> &'static str {
    "pong"
}

async fn info(State(state): State<AppState>) -> impl IntoResponse {
    // Increment request count
    let mut count = state.request_count.lock().await;
    *count += 1;

    // Create response
    let response = InfoResponse {
        app_name: "QuillSpace Core API".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        request_count: *count,
    };

    Json(response)
}

async fn create_item(
    State(state): State<AppState>,
    Json(request): Json<CreateItemRequest>,
) -> impl IntoResponse {
    // Increment request count
    let mut count = state.request_count.lock().await;
    *count += 1;

    // In a real app, you'd save to a database with tenant isolation
    let item_id = 42; // Placeholder

    // Create response
    let response = CreateItemResponse {
        id: item_id,
        name: request.name,
    };

    (StatusCode::CREATED, Json(response))
}
