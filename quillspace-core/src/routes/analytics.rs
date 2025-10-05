use crate::{
    database::clickhouse::AnalyticsService,
    middleware::tenant::get_request_context,
    types::{ApiResponse, AnalyticsEvent, TenantId},
    AppState,
};
use axum::{
    extract::{Path, Query, Request, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, info};
use uuid::Uuid;

/// Create analytics routes
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/events", post(record_event))
        .route("/stats", get(get_tenant_stats))
        .route("/content/top", get(get_top_content))
        .route("/users/{user_id}/activity", get(get_user_activity))
}

/// Record an analytics event
async fn record_event(
    State(state): State<AppState>,
    Json(event_request): Json<RecordEventRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    // For now, we'll use a placeholder tenant context
    // In a real implementation, you'd extract this from JWT token in headers
    let tenant_id = TenantId::from_uuid(uuid::Uuid::new_v4()); // Placeholder
    let user_id = Some(uuid::Uuid::new_v4()); // Placeholder
    let request_id = Uuid::new_v4();

    let analytics = state.db.clickhouse();
    
    let event = AnalyticsEvent {
        event_id: Uuid::new_v4(),
        tenant_id: *tenant_id.as_uuid(),
        user_id,
        event_type: event_request.event_type,
        event_data: event_request.event_data,
        timestamp: Utc::now(),
        session_id: event_request.session_id,
        ip_address: event_request.ip_address,
        user_agent: event_request.user_agent,
    };

    match analytics.record_event(&event).await {
        Ok(_) => {
            info!(
                tenant_id = %tenant_id,
                event_type = %event.event_type,
                "Analytics event recorded"
            );
            
            let response = ApiResponse::success(
                RecordEventResponse {
                    event_id: event.event_id,
                    recorded_at: event.timestamp,
                },
                request_id,
            );
            Ok(Json(response))
        }
        Err(e) => {
            error!(
                tenant_id = %tenant_id,
                error = %e,
                "Failed to record analytics event"
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get tenant analytics statistics
async fn get_tenant_stats(
    State(state): State<AppState>,
    Query(params): Query<StatsQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    // Placeholder tenant context
    let tenant_id = TenantId::from_uuid(uuid::Uuid::new_v4());
    
    let analytics = state.db.clickhouse();
    let days = params.days.unwrap_or(7).min(365); // Cap at 1 year
    
    match analytics.get_tenant_stats(&tenant_id, days).await {
        Ok(stats) => {
            let response = ApiResponse::success(
                TenantStatsResponse {
                    tenant_id,
                    period_days: days,
                    stats,
                },
                Uuid::new_v4(), // Generate request ID
            );
            Ok(Json(response))
        }
        Err(e) => {
            error!(
                tenant_id = %tenant_id,
                error = %e,
                "Failed to get tenant statistics"
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get top content for tenant
async fn get_top_content(
    State(state): State<AppState>,
    Query(params): Query<TopContentQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    // Placeholder tenant context
    let tenant_id = TenantId::from_uuid(uuid::Uuid::new_v4());
    
    let analytics = state.db.clickhouse();
    let days = params.days.unwrap_or(7).min(365);
    let limit = params.limit.unwrap_or(10).min(100);
    
    match analytics.get_top_content(&tenant_id, days, limit).await {
        Ok(content) => {
            let response = ApiResponse::success(
                TopContentResponse {
                    tenant_id,
                    period_days: days,
                    content,
                },
                Uuid::new_v4(), // Generate request ID
            );
            Ok(Json(response))
        }
        Err(e) => {
            error!(
                tenant_id = %tenant_id,
                error = %e,
                "Failed to get top content"
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get user activity analytics
async fn get_user_activity(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Query(params): Query<UserActivityQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    // Placeholder tenant context
    let tenant_id = TenantId::from_uuid(uuid::Uuid::new_v4());
    
    // For now, skip authorization check (implement proper JWT auth later)
    
    let analytics = state.db.clickhouse();
    let days = params.days.unwrap_or(7).min(365);
    
    match analytics.get_user_activity(&tenant_id, &user_id, days).await {
        Ok(activity) => {
            let response = ApiResponse::success(
                UserActivityResponse {
                    tenant_id,
                    user_id,
                    period_days: days,
                    activity,
                },
                Uuid::new_v4(), // Generate request ID
            );
            Ok(Json(response))
        }
        Err(e) => {
            error!(
                tenant_id = %tenant_id,
                user_id = %user_id,
                error = %e,
                "Failed to get user activity"
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Request/Response types

#[derive(Debug, Deserialize)]
struct RecordEventRequest {
    event_type: String,
    event_data: serde_json::Value,
    session_id: Option<String>,
    ip_address: Option<String>,
    user_agent: Option<String>,
}

#[derive(Debug, Serialize)]
struct RecordEventResponse {
    event_id: Uuid,
    recorded_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct StatsQuery {
    days: Option<u32>,
}

#[derive(Debug, Serialize)]
struct TenantStatsResponse {
    tenant_id: TenantId,
    period_days: u32,
    stats: crate::database::clickhouse::TenantStats,
}

#[derive(Debug, Deserialize)]
struct TopContentQuery {
    days: Option<u32>,
    limit: Option<u32>,
}

#[derive(Debug, Serialize)]
struct TopContentResponse {
    tenant_id: TenantId,
    period_days: u32,
    content: Vec<crate::database::clickhouse::ContentStats>,
}

#[derive(Debug, Deserialize)]
struct UserActivityQuery {
    days: Option<u32>,
}

#[derive(Debug, Serialize)]
struct UserActivityResponse {
    tenant_id: TenantId,
    user_id: Uuid,
    period_days: u32,
    activity: Vec<crate::database::clickhouse::UserActivity>,
}
