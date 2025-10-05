use crate::{
    database::clickhouse::AnalyticsService as ClickHouseAnalyticsService,
    types::{AnalyticsEvent, TenantId},
};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, info};
use uuid::Uuid;

// Tinybird integration structures
#[derive(Debug, Serialize)]
struct TinybirdEvent {
    timestamp: DateTime<Utc>,
    tenant_id: String,
    user_id: Option<String>,
    event_type: String,
    event_data: serde_json::Value,
    session_id: Option<String>,
    ip_address: Option<String>,
    user_agent: Option<String>,
}

/// Analytics backend configuration
#[derive(Debug, Clone)]
pub enum AnalyticsBackend {
    ClickHouse(ClickHouseAnalyticsService),
    Tinybird { 
        api_url: String, 
        token: String,
        datasource: String,
    },
    Hybrid {
        clickhouse: ClickHouseAnalyticsService,
        tinybird_url: String,
        tinybird_token: String,
    },
}

/// High-level analytics service that orchestrates data collection and analysis
#[derive(Clone)]
pub struct AnalyticsService {
    backend: AnalyticsBackend,
    http_client: reqwest::Client,
}

impl AnalyticsService {
    pub fn new_clickhouse(clickhouse_service: ClickHouseAnalyticsService) -> Self {
        Self {
            backend: AnalyticsBackend::ClickHouse(clickhouse_service),
            http_client: reqwest::Client::new(),
        }
    }

    pub fn new_tinybird(api_url: String, token: String, datasource: String) -> Self {
        Self {
            backend: AnalyticsBackend::Tinybird { api_url, token, datasource },
            http_client: reqwest::Client::new(),
        }
    }

    pub fn new_hybrid(
        clickhouse_service: ClickHouseAnalyticsService,
        tinybird_url: String,
        tinybird_token: String,
    ) -> Self {
        Self {
            backend: AnalyticsBackend::Hybrid {
                clickhouse: clickhouse_service,
                tinybird_url,
                tinybird_token,
            },
            http_client: reqwest::Client::new(),
        }
    }

    /// Record a page view event
    pub async fn record_page_view(
        &self,
        tenant_id: &TenantId,
        user_id: Option<Uuid>,
        page_path: &str,
        session_id: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<()> {
        let event = AnalyticsEvent {
            event_id: Uuid::new_v4(),
            tenant_id: *tenant_id.as_uuid(),
            user_id,
            event_type: "page_view".to_string(),
            event_data: serde_json::json!({
                "page_path": page_path,
                "timestamp": Utc::now()
            }),
            timestamp: Utc::now(),
            session_id,
            ip_address,
            user_agent,
        };

        // Route to appropriate backend
        match &self.backend {
            AnalyticsBackend::ClickHouse(service) => {
                service.record_event(&event).await?;
            }
            AnalyticsBackend::Tinybird { api_url, token, datasource } => {
                self.send_to_tinybird(&event, api_url, token, datasource).await?;
            }
            AnalyticsBackend::Hybrid { clickhouse, tinybird_url, tinybird_token } => {
                // Send to both backends
                clickhouse.record_event(&event).await?;
                self.send_to_tinybird(&event, tinybird_url, tinybird_token, "events").await?;
            }
        }
        
        info!(
            tenant_id = %tenant_id,
            page_path = %page_path,
            "Page view recorded"
        );
        
        Ok(())
    }

    /// Send event to Tinybird via HTTP API
    async fn send_to_tinybird(
        &self,
        event: &AnalyticsEvent,
        api_url: &str,
        token: &str,
        datasource: &str,
    ) -> Result<()> {
        let tinybird_event = TinybirdEvent {
            timestamp: event.timestamp,
            tenant_id: event.tenant_id.to_string(),
            user_id: event.user_id.map(|id| id.to_string()),
            event_type: event.event_type.clone(),
            event_data: event.event_data.clone(),
            session_id: event.session_id.clone(),
            ip_address: event.ip_address.clone(),
            user_agent: event.user_agent.clone(),
        };

        let url = format!("{}/v0/events?name={}", api_url, datasource);
        
        let response = self.http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .json(&tinybird_event)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Tinybird API error: {}", response.status());
        }

        Ok(())
    }

    /// Record a content interaction event
    pub async fn record_content_interaction(
        &self,
        tenant_id: &TenantId,
        user_id: Option<Uuid>,
        content_id: Uuid,
        interaction_type: &str,
        metadata: serde_json::Value,
    ) -> Result<()> {
        let event = AnalyticsEvent {
            event_id: Uuid::new_v4(),
            tenant_id: *tenant_id.as_uuid(),
            user_id,
            event_type: format!("content_{}", interaction_type),
            event_data: serde_json::json!({
                "content_id": content_id,
                "interaction_type": interaction_type,
                "metadata": metadata
            }),
            timestamp: Utc::now(),
            session_id: None,
            ip_address: None,
            user_agent: None,
        };

        // Route to appropriate backend
        match &self.backend {
            AnalyticsBackend::ClickHouse(service) => {
                service.record_event(&event).await?;
                // Also record in content analytics table
                service.record_content_action(
                    *tenant_id.as_uuid(),
                    content_id,
                    interaction_type,
                    user_id,
                    metadata,
                ).await?;
            }
            AnalyticsBackend::Tinybird { api_url, token, datasource } => {
                self.send_to_tinybird(&event, api_url, token, datasource).await?;
            }
            AnalyticsBackend::Hybrid { clickhouse, tinybird_url, tinybird_token } => {
                // Send to both backends
                clickhouse.record_event(&event).await?;
                clickhouse.record_content_action(
                    *tenant_id.as_uuid(),
                    content_id,
                    interaction_type,
                    user_id,
                    metadata,
                ).await?;
                self.send_to_tinybird(&event, tinybird_url, tinybird_token, "events").await?;
            }
        }

        info!(
            tenant_id = %tenant_id,
            content_id = %content_id,
            interaction_type = %interaction_type,
            "Content interaction recorded"
        );
        
        Ok(())
    }

    /// Record a user action event
    pub async fn record_user_action(
        &self,
        tenant_id: &TenantId,
        user_id: Uuid,
        action: &str,
        metadata: serde_json::Value,
    ) -> Result<()> {
        let event = AnalyticsEvent {
            event_id: Uuid::new_v4(),
            tenant_id: *tenant_id.as_uuid(),
            user_id: Some(user_id),
            event_type: format!("user_{}", action),
            event_data: serde_json::json!({
                "action": action,
                "metadata": metadata
            }),
            timestamp: Utc::now(),
            session_id: None,
            ip_address: None,
            user_agent: None,
        };

        // Route to appropriate backend
        match &self.backend {
            AnalyticsBackend::ClickHouse(service) => {
                service.record_event(&event).await?;
            }
            AnalyticsBackend::Tinybird { api_url, token, datasource } => {
                self.send_to_tinybird(&event, api_url, token, datasource).await?;
            }
            AnalyticsBackend::Hybrid { clickhouse, tinybird_url, tinybird_token } => {
                // Send to both backends
                clickhouse.record_event(&event).await?;
                self.send_to_tinybird(&event, tinybird_url, tinybird_token, "events").await?;
            }
        }
        
        info!(
            tenant_id = %tenant_id,
            user_id = %user_id,
            action = %action,
            "User action recorded"
        );
        
        Ok(())
    }

    /// Get comprehensive dashboard data
    pub async fn get_dashboard_data(
        &self,
        tenant_id: &TenantId,
        days: u32,
    ) -> Result<DashboardData> {
        // Only ClickHouse backend supports dashboard data for now
        let (stats, top_content) = match &self.backend {
            AnalyticsBackend::ClickHouse(service) => {
                let stats = service.get_tenant_stats(tenant_id, days).await?;
                let top_content = service.get_top_content(tenant_id, days, 10).await?;
                (stats, top_content)
            }
            AnalyticsBackend::Hybrid { clickhouse, .. } => {
                let stats = clickhouse.get_tenant_stats(tenant_id, days).await?;
                let top_content = clickhouse.get_top_content(tenant_id, days, 10).await?;
                (stats, top_content)
            }
            AnalyticsBackend::Tinybird { .. } => {
                // For Tinybird-only, we'd need to implement API calls to get stats
                // For now, return empty data
                anyhow::bail!("Dashboard data not yet implemented for Tinybird-only backend");
            }
        };
        
        // Get additional metrics
        let daily_stats = self.get_daily_stats(tenant_id, days).await?;
        let user_engagement = self.get_user_engagement_metrics(tenant_id, days).await?;

        Ok(DashboardData {
            overview: OverviewStats {
                total_events: stats.total_events,
                unique_users: stats.unique_users,
                unique_sessions: stats.unique_sessions,
                page_views: stats.page_views,
                content_created: stats.content_created,
                content_published: stats.content_published,
            },
            top_content,
            daily_stats,
            user_engagement,
        })
    }

    /// Get daily statistics for charting
    async fn get_daily_stats(
        &self,
        tenant_id: &TenantId,
        days: u32,
    ) -> Result<Vec<DailyStats>> {
        let query = r#"
            SELECT
                toDate(timestamp) as date,
                count() as total_events,
                uniq(user_id) as unique_users,
                uniq(session_id) as unique_sessions,
                countIf(event_type = 'page_view') as page_views
            FROM events
            WHERE tenant_id = ? AND timestamp >= now() - INTERVAL ? DAY
            GROUP BY date
            ORDER BY date
        "#;

        let client = match &self.backend {
            AnalyticsBackend::ClickHouse(service) => service.client(),
            AnalyticsBackend::Hybrid { clickhouse, .. } => clickhouse.client(),
            AnalyticsBackend::Tinybird { .. } => {
                anyhow::bail!("Daily stats not available for Tinybird-only backend");
            }
        };

        let results = client
            .query(query)
            .bind(tenant_id.as_uuid())
            .bind(days)
            .fetch_all::<DailyStatsRow>()
            .await?;

        Ok(results.into_iter().map(|row| DailyStats {
            date: row.date,
            total_events: row.total_events,
            unique_users: row.unique_users,
            unique_sessions: row.unique_sessions,
            page_views: row.page_views,
        }).collect())
    }

    /// Get user engagement metrics
    async fn get_user_engagement_metrics(
        &self,
        tenant_id: &TenantId,
        days: u32,
    ) -> Result<UserEngagementMetrics> {
        let query = r#"
            SELECT
                avg(session_events) as avg_session_length,
                quantile(0.5)(session_events) as median_session_length,
                uniq(user_id) as active_users,
                count() / uniq(user_id) as events_per_user
            FROM (
                SELECT
                    user_id,
                    session_id,
                    count() as session_events
                FROM events
                WHERE tenant_id = ? AND timestamp >= now() - INTERVAL ? DAY
                    AND user_id IS NOT NULL AND session_id IS NOT NULL
                GROUP BY user_id, session_id
            )
        "#;

        let result = match &self.backend {
            AnalyticsBackend::ClickHouse(service) => service.client(),
            AnalyticsBackend::Hybrid { clickhouse, .. } => clickhouse.client(),
            AnalyticsBackend::Tinybird { .. } => {
                anyhow::bail!("ClickHouse client not available for Tinybird-only backend");
            }
        }
            .query(query)
            .bind(tenant_id.as_uuid())
            .bind(days)
            .fetch_one::<UserEngagementRow>()
            .await?;

        Ok(UserEngagementMetrics {
            average_session_length: result.avg_session_length,
            median_session_length: result.median_session_length,
            active_users: result.active_users,
            events_per_user: result.events_per_user,
        })
    }

    /// Generate analytics report
    pub async fn generate_report(
        &self,
        tenant_id: &TenantId,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<AnalyticsReport> {
        let query = r#"
            SELECT
                event_type,
                count() as event_count,
                uniq(user_id) as unique_users,
                uniq(session_id) as unique_sessions
            FROM events
            WHERE tenant_id = ? 
                AND timestamp >= ? 
                AND timestamp <= ?
            GROUP BY event_type
            ORDER BY event_count DESC
        "#;

        let results = match &self.backend {
            AnalyticsBackend::ClickHouse(service) => service.client(),
            AnalyticsBackend::Hybrid { clickhouse, .. } => clickhouse.client(),
            AnalyticsBackend::Tinybird { .. } => {
                anyhow::bail!("ClickHouse client not available for Tinybird-only backend");
            }
        }
            .query(query)
            .bind(tenant_id.as_uuid())
            .bind(start_date)
            .bind(end_date)
            .fetch_all::<EventTypeStatsRow>()
            .await?;

        let event_breakdown: HashMap<String, EventTypeStats> = results
            .into_iter()
            .map(|row| (
                row.event_type.clone(),
                EventTypeStats {
                    event_type: row.event_type,
                    count: row.event_count,
                    unique_users: row.unique_users,
                    unique_sessions: row.unique_sessions,
                }
            ))
            .collect();

        Ok(AnalyticsReport {
            tenant_id: *tenant_id.as_uuid(),
            start_date,
            end_date,
            event_breakdown,
            generated_at: Utc::now(),
        })
    }
}

// Data structures
#[derive(Debug, Serialize)]
pub struct DashboardData {
    pub overview: OverviewStats,
    pub top_content: Vec<crate::database::clickhouse::ContentStats>,
    pub daily_stats: Vec<DailyStats>,
    pub user_engagement: UserEngagementMetrics,
}

#[derive(Debug, Serialize)]
pub struct OverviewStats {
    pub total_events: u64,
    pub unique_users: u64,
    pub unique_sessions: u64,
    pub page_views: u64,
    pub content_created: u64,
    pub content_published: u64,
}

#[derive(Debug, Serialize)]
pub struct DailyStats {
    pub date: chrono::NaiveDate,
    pub total_events: u64,
    pub unique_users: u64,
    pub unique_sessions: u64,
    pub page_views: u64,
}

#[derive(Debug, Serialize)]
pub struct UserEngagementMetrics {
    pub average_session_length: f64,
    pub median_session_length: f64,
    pub active_users: u64,
    pub events_per_user: f64,
}

#[derive(Debug, Serialize)]
pub struct AnalyticsReport {
    pub tenant_id: Uuid,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub event_breakdown: HashMap<String, EventTypeStats>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct EventTypeStats {
    pub event_type: String,
    pub count: u64,
    pub unique_users: u64,
    pub unique_sessions: u64,
}

// ClickHouse row structures
#[derive(clickhouse::Row, Deserialize)]
struct DailyStatsRow {
    date: chrono::NaiveDate,
    total_events: u64,
    unique_users: u64,
    unique_sessions: u64,
    page_views: u64,
}

#[derive(clickhouse::Row, Deserialize)]
struct UserEngagementRow {
    avg_session_length: f64,
    median_session_length: f64,
    active_users: u64,
    events_per_user: f64,
}

#[derive(clickhouse::Row, Deserialize)]
struct EventTypeStatsRow {
    event_type: String,
    event_count: u64,
    unique_users: u64,
    unique_sessions: u64,
}
