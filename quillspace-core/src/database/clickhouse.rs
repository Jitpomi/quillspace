use crate::config::ClickHouseConfig;
use crate::types::{AnalyticsEvent, TenantId};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use uuid::Uuid;

pub use clickhouse::Client;

/// Create ClickHouse client with optimized settings
pub async fn create_client(config: &ClickHouseConfig) -> Result<Client> {
    info!("Connecting to ClickHouse database...");

    let client = Client::default()
        .with_url(&config.url)
        .with_database(&config.database)
        .with_user(&config.username)
        .with_password(&config.password)
        .with_compression(clickhouse::Compression::Lz4);

    // Test connection
    let result: u32 = client.query("SELECT 1").fetch_one().await?;
    if result != 1 {
        anyhow::bail!("ClickHouse connection test failed");
    }

    // Initialize analytics tables
    init_analytics_tables(&client).await?;

    info!("ClickHouse client created successfully");
    Ok(client)
}

/// Initialize ClickHouse tables for analytics
pub async fn init_analytics_tables(client: &Client) -> Result<()> {
    info!("Initializing ClickHouse analytics tables...");

    // Events table with multi-tenant support
    let create_events_table = r#"
        CREATE TABLE IF NOT EXISTS events (
            event_id UUID,
            tenant_id UUID,
            user_id Nullable(UUID),
            event_type String,
            event_data String,
            timestamp DateTime64(3),
            session_id Nullable(String),
            ip_address Nullable(String),
            user_agent Nullable(String),
            date Date MATERIALIZED toDate(timestamp)
        ) ENGINE = MergeTree()
        PARTITION BY (tenant_id, date)
        ORDER BY (tenant_id, event_type, timestamp)
        TTL date + INTERVAL 2 YEAR
        SETTINGS index_granularity = 8192
    "#;

    client.query(create_events_table).execute().await?;

    // Content analytics table
    let create_content_analytics_table = r#"
        CREATE TABLE IF NOT EXISTS content_analytics (
            content_id UUID,
            tenant_id UUID,
            action String,
            user_id Nullable(UUID),
            timestamp DateTime64(3),
            metadata String,
            date Date MATERIALIZED toDate(timestamp)
        ) ENGINE = MergeTree()
        PARTITION BY (tenant_id, date)
        ORDER BY (tenant_id, content_id, timestamp)
        TTL date + INTERVAL 1 YEAR
        SETTINGS index_granularity = 8192
    "#;

    client.query(create_content_analytics_table).execute().await?;

    // User activity aggregations (materialized view)
    let create_user_activity_mv = r#"
        CREATE MATERIALIZED VIEW IF NOT EXISTS user_activity_daily
        ENGINE = SummingMergeTree()
        PARTITION BY (tenant_id, date)
        ORDER BY (tenant_id, user_id, date, event_type)
        AS SELECT
            tenant_id,
            user_id,
            event_type,
            toDate(timestamp) as date,
            count() as event_count
        FROM events
        WHERE user_id IS NOT NULL
        GROUP BY tenant_id, user_id, event_type, date
    "#;

    if let Err(e) = client.query(create_user_activity_mv).execute().await {
        warn!("Failed to create user_activity_mv (may already exist): {}", e);
    }

    // Content performance aggregations
    let create_content_performance_mv = r#"
        CREATE MATERIALIZED VIEW IF NOT EXISTS content_performance_daily
        ENGINE = SummingMergeTree()
        PARTITION BY (tenant_id, date)
        ORDER BY (tenant_id, content_id, date, action)
        AS SELECT
            tenant_id,
            content_id,
            action,
            toDate(timestamp) as date,
            count() as action_count,
            uniq(user_id) as unique_users
        FROM content_analytics
        GROUP BY tenant_id, content_id, action, date
    "#;

    if let Err(e) = client.query(create_content_performance_mv).execute().await {
        warn!("Failed to create content_performance_daily (may already exist): {}", e);
    }

    info!("ClickHouse analytics tables initialized");
    Ok(())
}

/// ClickHouse analytics service
#[derive(Clone)]
pub struct AnalyticsService {
    client: Client,
}

impl std::fmt::Debug for AnalyticsService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnalyticsService")
            .field("client", &"<ClickHouse Client>")
            .finish()
    }
}

impl AnalyticsService {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// Get access to the underlying ClickHouse client for advanced queries
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Record an analytics event
    pub async fn record_event(&self, event: &AnalyticsEvent) -> Result<()> {
        // Use direct query instead of insert builder to avoid serialization issues
        let query = r#"
            INSERT INTO events (
                event_id, tenant_id, user_id, event_type, event_data, 
                timestamp, session_id, ip_address, user_agent
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#;
        
        self.client
            .query(query)
            .bind(event.event_id)
            .bind(event.tenant_id)
            .bind(event.user_id)
            .bind(&event.event_type)
            .bind(serde_json::to_string(&event.event_data)?)
            .bind(event.timestamp.timestamp_millis() as f64 / 1000.0)
            .bind(event.session_id.as_deref())
            .bind(event.ip_address.as_deref())
            .bind(event.user_agent.as_deref())
            .execute()
            .await?;
            
        Ok(())
    }

    /// Record content analytics
    pub async fn record_content_action(
        &self,
        tenant_id: Uuid,
        content_id: Uuid,
        action: &str,
        user_id: Option<Uuid>,
        metadata: serde_json::Value,
    ) -> Result<()> {
        // Use direct query to avoid serialization issues
        let query = r#"
            INSERT INTO content_analytics (
                content_id, tenant_id, action, user_id, timestamp, metadata
            ) VALUES (?, ?, ?, ?, ?, ?)
        "#;
        
        self.client
            .query(query)
            .bind(content_id)
            .bind(tenant_id)
            .bind(action)
            .bind(user_id)
            .bind(Utc::now().timestamp_millis() as f64 / 1000.0)
            .bind(serde_json::to_string(&metadata)?)
            .execute()
            .await?;
            
        Ok(())
    }

    /// Get tenant event statistics
    pub async fn get_tenant_stats(
        &self,
        tenant_id: &TenantId,
        days: u32,
    ) -> Result<TenantStats> {
        let query = r#"
            SELECT
                count() as total_events,
                uniq(user_id) as unique_users,
                uniq(session_id) as unique_sessions,
                countIf(event_type = 'page_view') as page_views,
                countIf(event_type = 'content_create') as content_created,
                countIf(event_type = 'content_publish') as content_published
            FROM events
            WHERE tenant_id = ? AND timestamp >= now() - INTERVAL ? DAY
        "#;

        let stats = self.client
            .query(query)
            .bind(tenant_id.as_uuid())
            .bind(days)
            .fetch_one::<TenantStatsRow>()
            .await?;

        Ok(TenantStats {
            total_events: stats.total_events,
            unique_users: stats.unique_users,
            unique_sessions: stats.unique_sessions,
            page_views: stats.page_views,
            content_created: stats.content_created,
            content_published: stats.content_published,
        })
    }

    /// Get top content by views
    pub async fn get_top_content(
        &self,
        tenant_id: &TenantId,
        days: u32,
        limit: u32,
    ) -> Result<Vec<ContentStats>> {
        let query = r#"
            SELECT
                content_id,
                countIf(action = 'view') as views,
                countIf(action = 'like') as likes,
                countIf(action = 'share') as shares,
                uniq(user_id) as unique_viewers
            FROM content_analytics
            WHERE tenant_id = ? AND timestamp >= now() - INTERVAL ? DAY
            GROUP BY content_id
            ORDER BY views DESC
            LIMIT ?
        "#;

        let results = self.client
            .query(query)
            .bind(tenant_id.as_uuid())
            .bind(days)
            .bind(limit)
            .fetch_all::<ContentStatsRow>()
            .await?;

        Ok(results.into_iter().map(|row| ContentStats {
            content_id: row.content_id,
            views: row.views,
            likes: row.likes,
            shares: row.shares,
            unique_viewers: row.unique_viewers,
        }).collect())
    }

    /// Get user activity timeline
    pub async fn get_user_activity(
        &self,
        tenant_id: &TenantId,
        user_id: &Uuid,
        days: u32,
    ) -> Result<Vec<UserActivity>> {
        let query = r#"
            SELECT
                toDate(timestamp) as date,
                event_type,
                count() as event_count
            FROM events
            WHERE tenant_id = ? AND user_id = ? AND timestamp >= now() - INTERVAL ? DAY
            GROUP BY date, event_type
            ORDER BY date DESC, event_type
        "#;

        let results = self.client
            .query(query)
            .bind(tenant_id.as_uuid())
            .bind(user_id)
            .bind(days)
            .fetch_all::<UserActivityRow>()
            .await?;

        Ok(results.into_iter().map(|row| UserActivity {
            date: row.date,
            event_type: row.event_type,
            event_count: row.event_count,
        }).collect())
    }
}

// ClickHouse row structures
#[derive(clickhouse::Row, Serialize, Deserialize)]
struct EventRow {
    event_id: Uuid,
    tenant_id: Uuid,
    user_id: Option<Uuid>,
    event_type: String,
    event_data: String,
    timestamp: DateTime<Utc>,
    session_id: Option<String>,
    ip_address: Option<String>,
    user_agent: Option<String>,
}

#[derive(clickhouse::Row, Serialize)]
struct ContentAnalyticsRow {
    content_id: Uuid,
    tenant_id: Uuid,
    action: String,
    user_id: Option<Uuid>,
    timestamp: DateTime<Utc>,
    metadata: String,
}

#[derive(clickhouse::Row, Deserialize)]
struct TenantStatsRow {
    total_events: u64,
    unique_users: u64,
    unique_sessions: u64,
    page_views: u64,
    content_created: u64,
    content_published: u64,
}

#[derive(clickhouse::Row, Deserialize)]
struct ContentStatsRow {
    content_id: Uuid,
    views: u64,
    likes: u64,
    shares: u64,
    unique_viewers: u64,
}

#[derive(clickhouse::Row, Deserialize)]
struct UserActivityRow {
    date: chrono::NaiveDate,
    event_type: String,
    event_count: u64,
}

// Response structures
#[derive(Debug, Serialize)]
pub struct TenantStats {
    pub total_events: u64,
    pub unique_users: u64,
    pub unique_sessions: u64,
    pub page_views: u64,
    pub content_created: u64,
    pub content_published: u64,
}

#[derive(Debug, Serialize)]
pub struct ContentStats {
    pub content_id: Uuid,
    pub views: u64,
    pub likes: u64,
    pub shares: u64,
    pub unique_viewers: u64,
}

#[derive(Debug, Serialize)]
pub struct UserActivity {
    pub date: chrono::NaiveDate,
    pub event_type: String,
    pub event_count: u64,
}
