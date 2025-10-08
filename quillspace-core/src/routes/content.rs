use crate::{
    auth::{jwt_helpers::{extract_auth_context, extract_auth_context_with_role}, Resource, Action},
    types::{ApiResponse, Content, ContentStatus, PaginatedResponse, PaginationParams},
    AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio_postgres::{Row, Error as PgError};
use tracing::{error, info};
use uuid::Uuid;


/// Helper function to convert a tokio-postgres Row to Content
fn row_to_content(row: &Row) -> Result<Content, PgError> {
    Ok(Content {
        id: row.try_get("id")?,
        tenant_id: row.try_get("tenant_id")?,
        title: row.try_get("title")?,
        slug: row.try_get("slug")?,
        body: row.try_get("body")?,
        status: row.try_get("status")?,
        author_id: row.try_get("author_id")?,
        published_at: row.try_get("published_at")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

/// Helper function to convert ContentStatus to string for database
fn content_status_to_string(status: &ContentStatus) -> &'static str {
    match status {
        ContentStatus::Draft => "draft",
        ContentStatus::Published => "published",
        ContentStatus::Archived => "archived",
    }
}

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
    headers: HeaderMap,
    Query(params): Query<ListContentQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let (tenant_id, _user_id) = extract_auth_context(&headers, &state.jwt_manager)?;
    let request_id = Uuid::new_v4();

    let limit: u32 = params.pagination.limit.unwrap_or(20).min(100);
    let offset: i64 = ((params.pagination.page.unwrap_or(1) - 1) * limit) as i64;

    // Get database connection
    let client = match state.db.postgres().get().await {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Build query with filters
    let mut query = "SELECT * FROM content WHERE tenant_id = $1".to_string();
    let mut params_vec: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![tenant_id.as_uuid()];
    let mut param_index = 2;

    // Declare status_str outside the conditional block so it lives long enough
    let status_str;
    if let Some(status) = &params.status {
        query.push_str(&format!(" AND status = ${}", param_index));
        status_str = content_status_to_string(status);
        params_vec.push(&status_str);
        param_index += 1;
    }

    if let Some(author_id) = &params.author_id {
        query.push_str(&format!(" AND author_id = ${}", param_index));
        params_vec.push(author_id);
        param_index += 1;
    }

    query.push_str(&format!(" ORDER BY created_at DESC LIMIT ${} OFFSET ${}", param_index, param_index + 1));
    let limit_i64 = limit as i64;
    params_vec.push(&limit_i64);
    params_vec.push(&offset);

    // Get total count for pagination
    let count_query = "SELECT COUNT(*) FROM content WHERE tenant_id = $1";
    let count_params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![tenant_id.as_uuid()];

    let content_rows = match client.query(query.as_str(), &params_vec).await {
        Ok(rows) => rows,
        Err(e) => {
            error!("Failed to query content: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    let count_row = match client.query_one(count_query, &count_params).await {
        Ok(row) => row,
        Err(e) => {
            error!("Failed to query content count: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    let content: Result<Vec<Content>, _> = content_rows.iter().map(row_to_content).collect();
    let content = match content {
        Ok(content) => content,
        Err(e) => {
            error!("Failed to parse content rows: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let total: u64 = count_row.get::<_, i64>(0) as u64;
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

/// Create new content
async fn create_content(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(content_request): Json<CreateContentRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let auth_context = extract_auth_context_with_role(&headers, &state.jwt_manager)?;
    let request_id = Uuid::new_v4();

    // Require write permission to create content using Casbin
    state.authorizer.require_permission(&auth_context.user_role, Resource::Content.as_str(), Action::Write.as_str()).await?;

    let content_id = Uuid::new_v4();
    let author_id = auth_context.user_id; // Use actual user ID from JWT
    let now = chrono::Utc::now();

    // Get database connection
    let client = match state.db.postgres().get().await {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Use the status from the request, defaulting to Draft if not provided
    let status = content_request.status.unwrap_or(ContentStatus::Draft);
    
    let query = r#"
        INSERT INTO content (id, tenant_id, title, slug, body, status, author_id, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING *
        "#;
    
    let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![
        &content_id,
        auth_context.tenant_id.as_uuid(),
        &content_request.title,
        &content_request.slug,
        &content_request.body,
        &status,
        &author_id,
        &now,
        &now,
    ];

    match client.query_one(query, &params).await {
        Ok(row) => {
            let content = match row_to_content(&row) {
                Ok(content) => content,
                Err(e) => {
                    error!("Failed to parse content row: {}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            };

            // Record analytics event
            let _ = state.db.clickhouse().record_content_action(
                *auth_context.tenant_id.as_uuid(),
                content_id,
                "create",
                Some(author_id),
                serde_json::json!({ "title": content_request.title }),
            ).await;

            info!(
                content_id = %content_id,
                tenant_id = %auth_context.tenant_id,
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
    headers: HeaderMap,
    Path(content_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let (tenant_id, _user_id) = extract_auth_context(&headers, &state.jwt_manager)?;
    let request_id = Uuid::new_v4();

    // Get database connection
    let client = match state.db.postgres().get().await {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let query = "SELECT * FROM content WHERE id = $1 AND tenant_id = $2";
    let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![&content_id, tenant_id.as_uuid()];

    match client.query_opt(query, &params).await {
        Ok(Some(row)) => {
            let content = match row_to_content(&row) {
                Ok(content) => content,
                Err(e) => {
                    error!("Failed to parse content row: {}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            };

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

/// Delete content
async fn delete_content(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(content_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let (tenant_id, _user_id) = extract_auth_context(&headers, &state.jwt_manager)?;
    let request_id = Uuid::new_v4();

    // Get database connection
    let client = match state.db.postgres().get().await {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let query = "SELECT * FROM content WHERE id = $1 AND tenant_id = $2";
    let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![&content_id, tenant_id.as_uuid()];

    match client.query_opt(query, &params).await {
        Ok(Some(row)) => {
            let content = match row_to_content(&row) {
                Ok(content) => content,
                Err(e) => {
                    error!("Failed to parse content row: {}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            };

            // Record view analytics
            let _ = state.db.clickhouse().record_content_action(
                *tenant_id.as_uuid(),
                content_id,
                "view",
                Some(_user_id), // Use actual user ID from JWT
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
    headers: HeaderMap,
    Path(content_id): Path<Uuid>,
    Json(update_request): Json<UpdateContentRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let (tenant_id, _user_id) = extract_auth_context(&headers, &state.jwt_manager)?;
    let request_id = Uuid::new_v4();

    // For now, skip permission check (implement proper auth later)
    // In real implementation: check if user has Editor or Admin role

    let now = chrono::Utc::now();

    // Get database connection
    let client = match state.db.postgres().get().await {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let query = r#"
        UPDATE content 
        SET title = COALESCE($3, title),
            slug = COALESCE($4, slug),
            body = COALESCE($5, body),
            updated_at = $6
        WHERE id = $1 AND tenant_id = $2
        RETURNING *
        "#;

    let title_ref = update_request.title.as_deref();
    let slug_ref = update_request.slug.as_deref();
    let body_ref = update_request.body.as_deref();
    
    let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![
        &content_id,
        tenant_id.as_uuid(),
        &title_ref,
        &slug_ref,
        &body_ref,
        &now,
    ];

    match client.query_opt(query, &params).await {
        Ok(Some(row)) => {
            let content = match row_to_content(&row) {
                Ok(content) => content,
                Err(e) => {
                    error!("Failed to parse content row: {}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            };

            // Record analytics event
            let _ = state.db.clickhouse().record_content_action(
                *tenant_id.as_uuid(),
                content_id,
                "update",
                Some(_user_id), // Use actual user ID from JWT
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
    headers: HeaderMap,
    Path(content_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let (tenant_id, _user_id) = extract_auth_context(&headers, &state.jwt_manager)?;
    let request_id = Uuid::new_v4();

    // For now, skip permission check (implement proper auth later)
    // In real implementation: check if user has Editor or Admin role

    let now = chrono::Utc::now();

    // Get database connection
    let client = match state.db.postgres().get().await {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let query = r#"
        UPDATE content 
        SET status = $3, published_at = $4, updated_at = $5
        WHERE id = $1 AND tenant_id = $2
        RETURNING *
        "#;

    let status_str = content_status_to_string(&ContentStatus::Published);
    
    let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![
        &content_id,
        tenant_id.as_uuid(),
        &status_str,
        &now,
        &now,
    ];

    match client.query_opt(query, &params).await {
        Ok(Some(row)) => {
            let content = match row_to_content(&row) {
                Ok(content) => content,
                Err(e) => {
                    error!("Failed to parse content row: {}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            };

            // Record analytics event
            let _ = state.db.clickhouse().record_content_action(
                *tenant_id.as_uuid(),
                content_id,
                "publish",
                Some(_user_id), // Use actual user ID from JWT
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
    headers: HeaderMap,
    Path(content_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let (tenant_id, _user_id) = extract_auth_context(&headers, &state.jwt_manager)?;
    let request_id = Uuid::new_v4();

    // For now, skip permission check (implement proper auth later)
    // In real implementation: check if user has Editor or Admin role

    let now = chrono::Utc::now();

    // Get database connection
    let client = match state.db.postgres().get().await {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let query = r#"
        UPDATE content 
        SET status = $3, updated_at = $4
        WHERE id = $1 AND tenant_id = $2
        RETURNING *
        "#;

    let status_str = content_status_to_string(&ContentStatus::Archived);
    
    let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![
        &content_id,
        tenant_id.as_uuid(),
        &status_str,
        &now,
    ];

    match client.query_opt(query, &params).await {
        Ok(Some(row)) => {
            let content = match row_to_content(&row) {
                Ok(content) => content,
                Err(e) => {
                    error!("Failed to parse content row: {}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            };

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

/// Get content analytics
async fn get_content_analytics(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(content_id): Path<Uuid>,
    Query(params): Query<ContentAnalyticsQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let (tenant_id, _user_id) = extract_auth_context(&headers, &state.jwt_manager)?;
    let request_id = Uuid::new_v4();

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
        .client()
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
    status: Option<ContentStatus>,
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
