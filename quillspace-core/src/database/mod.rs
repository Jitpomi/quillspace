pub mod postgres;
pub mod clickhouse;

use crate::config::{DatabaseConfig, ClickHouseConfig};
use anyhow::Result;
use sqlx::{PgPool, Pool, Postgres};
use std::sync::Arc;

/// Database connections container
#[derive(Clone)]
pub struct DatabaseConnections {
    pub postgres: Arc<PgPool>,
    pub clickhouse: Arc<clickhouse::Client>,
}

impl DatabaseConnections {
    pub async fn new(
        postgres_config: &DatabaseConfig,
        clickhouse_config: &ClickHouseConfig,
    ) -> Result<Self> {
        let postgres = postgres::create_pool(postgres_config).await?;
        let clickhouse = clickhouse::create_client(clickhouse_config).await?;

        Ok(Self {
            postgres: Arc::new(postgres),
            clickhouse: Arc::new(clickhouse),
        })
    }

    pub fn postgres(&self) -> &PgPool {
        &self.postgres
    }

    pub fn clickhouse(&self) -> &clickhouse::Client {
        &self.clickhouse
    }
}

/// Database transaction wrapper for multi-tenant operations
pub struct TenantTransaction<'a> {
    pub tx: sqlx::Transaction<'a, Postgres>,
    pub tenant_id: uuid::Uuid,
}

impl<'a> TenantTransaction<'a> {
    pub async fn begin(pool: &PgPool, tenant_id: uuid::Uuid) -> Result<Self> {
        let tx = pool.begin().await?;
        Ok(Self { tx, tenant_id })
    }

    pub async fn commit(self) -> Result<()> {
        self.tx.commit().await?;
        Ok(())
    }

    pub async fn rollback(self) -> Result<()> {
        self.tx.rollback().await?;
        Ok(())
    }
}
