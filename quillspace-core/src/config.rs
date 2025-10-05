use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub clickhouse: ClickHouseConfig,
    pub auth: AuthConfig,
    pub observability: ObservabilityConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: Option<usize>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ClickHouseConfig {
    pub url: String,
    pub database: String,
    pub username: String,
    pub password: String,
    pub compression: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expiration: i64,
    pub refresh_token_expiration: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ObservabilityConfig {
    pub metrics_enabled: bool,
    pub tracing_enabled: bool,
    pub prometheus_port: u16,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        let config = Config::builder()
            // Start with default values
            .add_source(File::with_name("config/default"))
            // Add environment-specific config
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))
            // Add local config (for development overrides)
            .add_source(File::with_name("config/local").required(false))
            // Override with environment variables
            .add_source(Environment::with_prefix("QUILLSPACE").separator("__"))
            .build()?;

        config.try_deserialize()
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 3000,
                workers: None,
            },
            database: DatabaseConfig {
                url: "postgresql://quillspace:dev_password@localhost:5432/quillspace_dev".to_string(),
                max_connections: 10,
                min_connections: 1,
                connect_timeout: 30,
            },
            clickhouse: ClickHouseConfig {
                url: "http://localhost:8123".to_string(),
                database: "quillspace_analytics".to_string(),
                username: "default".to_string(),
                password: "".to_string(),
                compression: "lz4".to_string(),
            },
            auth: AuthConfig {
                jwt_secret: "your-secret-key-change-in-production".to_string(),
                jwt_expiration: 3600, // 1 hour
                refresh_token_expiration: 86400 * 7, // 7 days
            },
            observability: ObservabilityConfig {
                metrics_enabled: true,
                tracing_enabled: true,
                prometheus_port: 9090,
            },
        }
    }
}
