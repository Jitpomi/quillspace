use crate::{
    database::clickhouse::AnalyticsService,
    middleware::tenant::get_request_context,
    types::{ApiResponse, Content, ContentStatus, PaginatedResponse, PaginationParams, UserRole},
    AppState,
};
use axum::{
    extract::{Path, Query, Request, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use uuid::Uuid;

/// Create content management routes
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_content).post(create_content))
        .route("/:content_id", get(get_content).put(update_content).delete(delete_content))
        .route("/:content_id/publish", post(publish_content))
        .route("/:content_id/archive", post(archive_content))
        .route("/:content_id/analytics", get(get_content_analytics))
}

/// List content with tenant isolation
async fn list_content(
    State(state): State<AppState>,
    Query(params): Query<ListContentQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    // Placeholder tenant context (implement proper JWT auth later)
    let tenant_id = crate::types::TenantId::from_uuid(Uuid::new_v4());
    let request_id = Uuid::new_v4();

    let limit = params.pagination.limit.unwrap_or(20).min(100);
    let offset = ((params.pagination.page.unwrap_or(1) - 1) * limit) as i64;

    // Build query with filters
    let mut query_builder = sqlx::QueryBuilder::new(
        "SELECT * FROM content WHERE tenant_id = "
    );
    query_builder.push_bind(*tenant_id.as_uuid());

    if let Some(status) = params.status {
        query_builder.push(" AND status = ");
        query_builder.push_bind(status);
    }

    if let Some(author_id) = params.author_id {
        query_builder.push(" AND author_id = ");
        query_builder.push_bind(author_id);
    }

    query_builder.push(" ORDER BY created_at DESC LIMIT ");
    query_builder.push_bind(limit as i64);
    query_builder.push(" OFFSET ");
    query_builder.push_bind(offset);

    let query = query_builder.build_query_as::<Content>();

    // Get total count for pagination
    let count_query = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM content WHERE tenant_id = $1",
        *tenant_id.as_uuid()
    );

    match tokio::try_join!(
        query.fetch_all(state.db.postgres()),
        count_query.fetch_one(state.db.postgres())
    ) {
        Ok((content, total_count)) => {
            let total = total_count.unwrap_or(0) as u64;
            let page = params.pagination.page.unwrap_or(1);
            let total_pages = ((total + limit as u64 - 1) / limit as u64) as u32;

            let paginated = PaginatedResponse {
                items: content,
                total,
                page,
                limit,
                total_pages,
            };

            let response = ApiResponse::success(paginated, request_id);
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to list content: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Create new content
async fn create_content(
    State(state): State<AppState>,
    Json(content_request): Json<CreateContentRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    // Placeholder tenant context (implement proper JWT auth later)
    let tenant_id = crate::types::TenantId::from_uuid(Uuid::new_v4());
    let user_id = Uuid::new_v4(); // Placeholder user ID
    let request_id = Uuid::new_v4();

    // For now, skip permission check (implement proper auth later)
    // In real implementation: check if user has Editor or Admin role

    let content_id = Uuid::new_v4();
    let author_id = user_id; // Use placeholder user ID
    let now = chrono::Utc::now();

    let query = sqlx::query_as::<_, Content>(
        r#"
        INSERT INTO content (id, tenant_id, title, slug, body, status, author_id, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING *
        "#
    )
    .bind(content_id)
    .bind(*tenant_id.as_uuid())
    .bind(&content_request.title)
    .bind(&content_request.slug)
    .bind(&content_request.body)
    .bind(ContentStatus::Draft)
    .bind(author_id)
    .bind(now)
    .bind(now);

    match query.fetch_one(state.db.postgres()).await {
        Ok(content) => {
            // Record analytics event
            let analytics = AnalyticsService::new(state.db.clickhouse().clone());
            let _ = analytics.record_content_action(
                *tenant_id.as_uuid(),
                content_id,
                "create",
                Some(author_id),
                serde_json::json!({ "title": content_request.title }),
            ).await;

            info!(
                content_id = %content_id,
                tenant_id = %tenant_id,
                "Content created"
            );

            let response = ApiResponse::success(content, request_id);
            Ok((StatusCode::CREATED, Json(response)))
        }
        Err(e) => {
            error!("Failed to create content: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get content by ID
async fn get_content(
    State(state): State<AppState>,
    Path(content_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    // Placeholder tenant context (implement proper JWT auth later)
    let tenant_id = crate::types::TenantId::from_uuid(Uuid::new_v4());
    let request_id = Uuid::new_v4();

    let query = sqlx::query_as::<_, Content>(
        "SELECT * FROM content WHERE id = $1 AND tenant_id = $2"
    )
    .bind(content_id)
    .bind(*tenant_id.as_uuid());

    match query.fetch_optional(state.db.postgres()).await {
        Ok(Some(content)) => {
            // Record view analytics
            let analytics = AnalyticsService::new(state.db.clickhouse().clone());
            let _ = analytics.record_content_action(
                *tenant_id.as_uuid(),
                content_id,
                "view",
                Some(Uuid::new_v4()), // Placeholder user ID
                serde_json::json!({}),
            ).await;

            let response = ApiResponse::success(content, request_id);
            Ok(Json(response))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get content: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Update content
async fn update_content(
    State(state): State<AppState>,
    Path(content_id): Path<Uuid>,
    Json(update_request): Json<UpdateContentRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    // Placeholder tenant context (implement proper JWT auth later)
    let tenant_id = crate::types::TenantId::from_uuid(Uuid::new_v4());
    let request_id = Uuid::new_v4();

    // For now, skip permission check (implement proper auth later)
    // In real implementation: check if user has Editor or Admin role

    let now = chrono::Utc::now();

    let query = sqlx::query_as::<_, Content>(
        r#"
        UPDATE content 
        SET title = COALESCE($3, title),
            slug = COALESCE($4, slug),
            body = COALESCE($5, body),
            updated_at = $6
        WHERE id = $1 AND tenant_id = $2
        RETURNING *
        "#
    )
    .bind(content_id)
    .bind(*tenant_id.as_uuid())
    .bind(update_request.title.as_deref())
    .bind(update_request.slug.as_deref())
    .bind(update_request.body.as_deref())
    .bind(now);

    match query.fetch_optional(state.db.postgres()).await {
        Ok(Some(content)) => {
            // Record analytics event
            let analytics = AnalyticsService::new(state.db.clickhouse().clone());
            let _ = analytics.record_content_action(
                *tenant_id.as_uuid(),
                content_id,
                "update",
                Some(Uuid::new_v4()), // Placeholder user ID
                serde_json::json!({}),
            ).await;

            info!(content_id = %content_id, "Content updated");
            let response = ApiResponse::success(content, request_id);
            Ok(Json(response))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to update content: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Publish content
async fn publish_content(
    State(state): State<AppState>,
    Path(content_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    // Placeholder tenant context (implement proper JWT auth later)
    let tenant_id = crate::types::TenantId::from_uuid(Uuid::new_v4());
    let request_id = Uuid::new_v4();

    // For now, skip permission check (implement proper auth later)
    // In real implementation: check if user has Editor or Admin role

    let now = chrono::Utc::now();

    let query = sqlx::query_as::<_, Content>(
        r#"
        UPDATE content 
        SET status = $3, published_at = $4, updated_at = $5
        WHERE id = $1 AND tenant_id = $2
        RETURNING *
        "#
    )
    .bind(content_id)
    .bind(*tenant_id.as_uuid())
    .bind(ContentStatus::Published)
    .bind(now)
    .bind(now);

    match query.fetch_optional(state.db.postgres()).await {
        Ok(Some(content)) => {
            // Record analytics event
            let analytics = AnalyticsService::new(state.db.clickhouse().clone());
            let _ = analytics.record_content_action(
                *tenant_id.as_uuid(),
                content_id,
                "publish",
                Some(Uuid::new_v4()), // Placeholder user ID
                serde_json::json!({}),
            ).await;

            info!(content_id = %content_id, "Content published");
            let response = ApiResponse::success(content, request_id);
            Ok(Json(response))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to publish content: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Archive content
async fn archive_content(
    State(state): State<AppState>,
    Path(content_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    // Placeholder tenant context (implement proper JWT auth later)
    let tenant_id = crate::types::TenantId::from_uuid(Uuid::new_v4());
    let request_id = Uuid::new_v4();

    // For now, skip permission check (implement proper auth later)
    // In real implementation: check if user has Editor or Admin role

    let now = chrono::Utc::now();

    let query = sqlx::query_as::<_, Content>(
        r#"
        UPDATE content 
        SET status = $3, updated_at = $4
        WHERE id = $1 AND tenant_id = $2
        RETURNING *
        "#
    )
    .bind(content_id)
    .bind(*tenant_id.as_uuid())
    .bind(ContentStatus::Archived)
    .bind(now);

    match query.fetch_optional(state.db.postgres()).await {
        Ok(Some(content)) => {
            let response = ApiResponse::success(content, request_id);
            Ok(Json(response))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to archive content: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Delete content
async fn delete_content(
    State(state): State<AppState>,
    Path(content_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    // Placeholder tenant context (implement proper JWT auth later)
    let tenant_id = crate::types::TenantId::from_uuid(Uuid::new_v4());

    // For now, skip admin check (implement proper auth later)
    // In real implementation: check if user has Admin role

    let query = sqlx::query!(
        "DELETE FROM content WHERE id = $1 AND tenant_id = $2",
        content_id,
        *tenant_id.as_uuid()
    );

    match query.execute(state.db.postgres()).await {
        Ok(result) => {
            if result.rows_affected() > 0 {
                info!(content_id = %content_id, "Content deleted");
                Ok(StatusCode::NO_CONTENT)
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        }
        Err(e) => {
            error!("Failed to delete content: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get content analytics
async fn get_content_analytics(
    State(state): State<AppState>,
    Path(content_id): Path<Uuid>,
    Query(params): Query<ContentAnalyticsQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    // Placeholder tenant context (implement proper JWT auth later)
    let tenant_id = crate::types::TenantId::from_uuid(Uuid::new_v4());
    let request_id = Uuid::new_v4();

    let analytics = AnalyticsService::new(state.db.clickhouse().clone());
    let days = params.days.unwrap_or(30);

    // Get content performance data from ClickHouse
    let query = r#"
        SELECT
            action,
            count() as action_count,
            uniq(user_id) as unique_users
        FROM content_analytics
        WHERE tenant_id = ? AND content_id = ? AND timestamp >= now() - INTERVAL ? DAY
        GROUP BY action
        ORDER BY action_count DESC
    "#;

    match state.db.clickhouse()
        .query(query)
        .bind(*tenant_id.as_uuid())
        .bind(content_id)
        .bind(days)
        .fetch_all::<ContentAnalyticsRow>()
        .await
    {
        Ok(results) => {
            let analytics_data: Vec<ContentAnalyticsData> = results
                .into_iter()
                .map(|row| ContentAnalyticsData {
                    action: row.action,
                    count: row.action_count,
                    unique_users: row.unique_users,
                })
                .collect();

            let response = ApiResponse::success(analytics_data, request_id);
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to get content analytics: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Request/Response types
#[derive(Debug, Deserialize)]
struct ListContentQuery {
    #[serde(flatten)]
    pagination: PaginationParams,
    status: Option<ContentStatus>,
    author_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
struct CreateContentRequest {
    title: String,
    slug: String,
    body: String,
}

#[derive(Debug, Deserialize)]
struct UpdateContentRequest {
    title: Option<String>,
    slug: Option<String>,
    body: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ContentAnalyticsQuery {
    days: Option<u32>,
}

#[derive(clickhouse::Row, serde::Deserialize)]
struct ContentAnalyticsRow {
    action: String,
    action_count: u64,
    unique_users: u64,
}

#[derive(Debug, Serialize)]
struct ContentAnalyticsData {
    action: String,
    count: u64,
    unique_users: u64,
}
