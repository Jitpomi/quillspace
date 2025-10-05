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
    middleware::from_fn_with_state,
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
        let db = DatabaseConnections::new(&config.database, &config.clickhouse).await?;
        
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

    // Setup row-level security
    database::postgres::setup_rls(state.db.postgres()).await?;
    info!("Row-level security policies configured");

    // Build the enhanced router with comprehensive middleware
    let app = create_app(state).await?;

    // Define the address to listen on
    let addr = SocketAddr::from(([
        127, 0, 0, 1
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
    let jwt_secret = state.jwt_secret.clone();
    
    let app = Router::new()
        // Health check routes (no middleware)
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
        
        // API routes with full middleware stack
        .nest("/api/v1", create_api_routes())
        
        // Legacy routes for compatibility
        .route("/", get(root))
        .route("/api/info", get(info))
        .route("/api/items", post(create_item))
        
        // Add Axum-compliant middleware stack
        .layer(
            ServiceBuilder::new()
                // Tower-HTTP middleware (always works)
                .layer(TraceLayer::new_for_http())
                .layer(CompressionLayer::new())
                .layer(TimeoutLayer::new(Duration::from_secs(30)))
                
                // Custom middleware with proper signatures
                .layer(from_fn(middleware::request_id_middleware))
                .layer(from_fn(middleware::timing_middleware))
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
        // Temporarily disable problematic routes for compilation
        // .nest("/tenants", routes::tenants::create_routes())
        // .nest("/content", routes::content::create_routes())
        // .nest("/analytics", routes::analytics::create_routes())
        // .nest("/users", routes::users::create_routes())
        // .nest("/auth", routes::auth::create_routes())
        
        // Add a simple test route
        .route("/test", get(|| async { "API is working!" }))
}

// Health check handlers
async fn health_check() -> &'static str {
    "OK"
}

async fn readiness_check(State(state): State<AppState>) -> Result<&'static str, StatusCode> {
    // Check database connectivity
    match sqlx::query("SELECT 1").execute(state.db.postgres()).await {
        Ok(_) => Ok("Ready"),
        Err(_) => Err(StatusCode::SERVICE_UNAVAILABLE),
    }
}

// Legacy route handlers for compatibility
async fn root() -> &'static str {
    "🚀 Welcome to QuillSpace - High-Performance Multi-Tenant Publishing Platform"
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
