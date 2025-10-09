mod config;
mod types;
mod database;
mod middleware;
mod routes;
mod services;
mod auth;

use axum::{
    extract::State,
    middleware::from_fn,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use crate::{
    auth::{JwtManager, CasbinAuthorizer},
    config::AppConfig,
    database::DatabaseConnections,
    services::TemplateEngine,
};
// Removed unused Deserialize import
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tracing::{info, warn};

// Enhanced application state with database connections
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub db: DatabaseConnections,
    pub jwt_secret: Arc<String>,
    pub jwt_manager: Arc<JwtManager>,
    pub authorizer: Arc<CasbinAuthorizer>,
    pub template_engine: Arc<TemplateEngine>,
    pub request_count: Arc<Mutex<usize>>,
}

impl AppState {
    pub async fn new(config: AppConfig) -> anyhow::Result<Self> {
        let db = DatabaseConnections::new(&config.database.url, &config.clickhouse).await?;
        let jwt_manager = JwtManager::new(&config.auth.jwt_secret, "quillspace");
        let authorizer = CasbinAuthorizer::new().await?;
        let template_engine = TemplateEngine::new(Arc::new(db.clone()))?;
        
        Ok(Self {
            jwt_secret: Arc::new(config.auth.jwt_secret.clone()),
            jwt_manager: Arc::new(jwt_manager),
            authorizer: Arc::new(authorizer),
            template_engine: Arc::new(template_engine),
            config: Arc::new(config),
            db,
            request_count: Arc::new(Mutex::new(0)),
        })
    }
}

// Response models
#[derive(serde::Serialize)]
struct InfoResponse {
    app_name: String,
    version: String,
    request_count: usize,
}

// Legacy types removed - using proper web builder APIs now

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

    // Initialize metrics if enabled
    if config.observability.metrics_enabled {
        let recorder = metrics_exporter_prometheus::PrometheusBuilder::new()
            .build_recorder();
        if let Err(e) = metrics::set_global_recorder(recorder) {
            warn!("Failed to set metrics recorder: {}", e);
        } else {
            info!("Prometheus metrics enabled on port {}", config.observability.prometheus_port);
        }
    }

    // Create enhanced app state with database connections
    let state = AppState::new(config.clone()).await?;
    info!("Database connections established");

    // Setup row-level security policies
    database::postgres::setup_rls(state.db.postgres()).await?;
    info!("Row-level security policies configured");

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
    axum::serve(listener, app).await?;
    
    Ok(())
}

/// Create the application router with all middleware and routes
async fn create_app(state: AppState) -> anyhow::Result<Router> {
    let _jwt_secret = state.jwt_secret.clone();
    
    // Create application routes
    info!("🔧 Registering routes...");
    let app = Router::new()
        // Health check routes
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
        
        // Basic routes
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
        .nest("/auth", routes::auth::create_routes())
        .nest("/tenants", routes::tenants::create_routes())
        .nest("/content", routes::content::create_routes())
        .nest("/analytics", routes::analytics::create_routes())
        .nest("/users", routes::users::create_routes())
        .nest("/templates", routes::templates::templates_router())
        // Web builder routes
        .nest("/sites", routes::sites::sites_router())
        .merge(routes::pages::pages_router())
        // Health and monitoring endpoints
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))

        // API status route
        .route("/status", get(|| async { "QuillSpace API is operational" }))
}

async fn health_check() -> &'static str {
    "OK"
}

async fn readiness_check() -> &'static str {
    "Ready"
}

// Root route handler
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

// Legacy create_item function removed - using proper web builder APIs
