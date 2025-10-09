pub mod postgres;
pub mod clickhouse;
pub mod rls_helper;
use anyhow::Result;
use deadpool_postgres::{Config, Pool, Runtime};
use std::sync::Arc;
use tokio_postgres::NoTls;

/// Database connections container
#[derive(Clone)]
pub struct DatabaseConnections {
    postgres: Arc<Pool>,
    clickhouse: Arc<clickhouse::AnalyticsService>,
}

impl DatabaseConnections {
    /// Create new database connections
    pub async fn new(postgres_url: &str, clickhouse_config: &crate::config::ClickHouseConfig) -> Result<Self> {
        tracing::info!("Creating database connection pool with URL: {}", postgres_url);
        let mut cfg = Config::new();
        cfg.url = Some(postgres_url.to_string());
        let postgres_pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;
        
        // Test database connection
        if let Err(e) = postgres_pool.get().await {
            tracing::error!("Database connection pool test failed: {}", e);
        }
        
        // Create ClickHouse client with proper authentication
        let clickhouse_client = clickhouse::Client::default()
            .with_url(&clickhouse_config.url)
            .with_user(&clickhouse_config.username)
            .with_password(&clickhouse_config.password)
            .with_database(&clickhouse_config.database);
        let clickhouse_service = clickhouse::AnalyticsService::new(clickhouse_client);
        
        Ok(Self {
            postgres: Arc::new(postgres_pool),
            clickhouse: Arc::new(clickhouse_service),
        })
    }

    /// Get PostgreSQL pool
    pub fn postgres(&self) -> &Pool {
        &self.postgres
    }

    /// Get ClickHouse service
    pub fn clickhouse(&self) -> &clickhouse::AnalyticsService {
        &self.clickhouse
    }
}
