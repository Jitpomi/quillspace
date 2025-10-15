use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::{
    services::calendly::{CalendlyService, CalendlyWebhookPayload, ConsultationBooking},
    middleware::auth::AuthContext,
    AppState,
};

#[derive(Debug, Deserialize)]
pub struct ProjectBriefRequest {
    pub project_name: Option<String>,
    pub project_type: Option<String>,
    pub genre: Option<String>,
    pub target_audience: Option<String>,
    pub pages_needed: Option<Vec<String>>,
    pub features_required: Option<Vec<String>>,
    pub design_preferences: Option<serde_json::Value>,
    pub content_status: Option<String>,
    pub timeline: Option<String>,
    pub budget_range: Option<String>,
    pub existing_website: Option<String>,
    pub special_requirements: Option<String>,
    pub questions_for_team: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BookingResponse {
    pub booking: ConsultationBooking,
    pub project_brief_completed: bool,
    pub materials_available: bool,
}

#[derive(Debug, Serialize)]
pub struct ConsultationDashboard {
    pub upcoming_bookings: Vec<ConsultationBooking>,
    pub past_bookings: Vec<ConsultationBooking>,
    pub pending_briefs: Vec<PendingBrief>,
    pub preparation_materials: Vec<PreparationMaterial>,
}

#[derive(Debug, Serialize)]
pub struct PendingBrief {
    pub booking_id: Uuid,
    pub event_name: String,
    pub scheduled_at: chrono::DateTime<chrono::Utc>,
    pub brief_url: String,
}

#[derive(Debug, Serialize)]
pub struct PreparationMaterial {
    pub title: String,
    pub material_type: String,
    pub content: Option<String>,
    pub file_url: Option<String>,
}

pub fn consultation_routes() -> Router<AppState> {
    Router::new()
        .route("/webhooks/calendly", post(handle_calendly_webhook))
        .route("/consultations", get(get_user_consultations))
        .route("/consultations/:booking_id", get(get_consultation_details))
        .route("/consultations/:booking_id/brief", put(update_project_brief))
        .route("/consultations/dashboard", get(get_consultation_dashboard))
        .route("/consultations/:booking_id/materials", get(get_consultation_materials))
}

/// Handle incoming Calendly webhooks
pub async fn handle_calendly_webhook(
    State(state): State<AppState>,
    Json(payload): Json<CalendlyWebhookPayload>,
) -> Result<StatusCode, StatusCode> {
    let calendly_service = CalendlyService::new(state.db.clone());
    
    match calendly_service.handle_webhook(payload).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => {
            tracing::error!("Failed to handle Calendly webhook: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get user's consultation bookings
pub async fn get_user_consultations(
    auth: AuthContext,
    State(state): State<AppState>,
) -> Result<Json<Vec<ConsultationBooking>>, StatusCode> {
    let calendly_service = CalendlyService::new(state.db.clone());
    
    match calendly_service.get_user_bookings(auth.user_id).await {
        Ok(bookings) => Ok(Json(bookings)),
        Err(e) => {
            tracing::error!("Failed to get user bookings: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get specific consultation details
pub async fn get_consultation_details(
    auth: AuthContext,
    State(state): State<AppState>,
    Path(booking_id): Path<Uuid>,
) -> Result<Json<BookingResponse>, StatusCode> {
    let calendly_service = CalendlyService::new(state.db.clone());
    
    // Get booking details
    let query = "
        SELECT id, tenant_id, user_id, calendly_event_uuid, event_name,
               scheduled_at, status, guest_email, project_brief,
               consultation_notes, proposal_sent, created_at, updated_at
        FROM consultation_bookings 
        WHERE id = $1 AND user_id = $2
    ";
    
    let row = match state.db.query_opt(query, &[&booking_id, &auth.user_id]).await {
        Ok(Some(row)) => row,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Database error: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    let booking = ConsultationBooking {
        id: row.get(0),
        tenant_id: row.get(1),
        user_id: row.get(2),
        calendly_event_uuid: row.get(3),
        event_name: row.get(4),
        scheduled_at: row.get(5),
        status: row.get(6),
        guest_email: row.get(7),
        project_brief: row.get(8),
        consultation_notes: row.get(9),
        proposal_sent: row.get(10),
        created_at: row.get(11),
        updated_at: row.get(12),
    };
    
    // Check if project brief is completed
    let brief_query = "
        SELECT completed FROM project_brief_forms 
        WHERE booking_id = $1
    ";
    let project_brief_completed = state.db
        .query_opt(brief_query, &[&booking_id])
        .await
        .unwrap_or(None)
        .map(|row| row.get::<_, bool>(0))
        .unwrap_or(false);
    
    // Check if materials are available
    let materials_query = "
        SELECT COUNT(*) FROM consultation_materials 
        WHERE booking_id = $1
    ";
    let materials_count: i64 = state.db
        .query_one(materials_query, &[&booking_id])
        .await
        .map(|row| row.get(0))
        .unwrap_or(0);
    
    let response = BookingResponse {
        booking,
        project_brief_completed,
        materials_available: materials_count > 0,
    };
    
    Ok(Json(response))
}

/// Update project brief
pub async fn update_project_brief(
    auth: AuthContext,
    State(state): State<AppState>,
    Path(booking_id): Path<Uuid>,
    Json(brief): Json<ProjectBriefRequest>,
) -> Result<StatusCode, StatusCode> {
    // Verify booking belongs to user
    let booking_query = "
        SELECT id FROM consultation_bookings 
        WHERE id = $1 AND user_id = $2
    ";
    
    let booking_exists = state.db
        .query_opt(booking_query, &[&booking_id, &auth.user_id])
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .is_some();
    
    if !booking_exists {
        return Err(StatusCode::NOT_FOUND);
    }
    
    // Upsert project brief form
    let upsert_query = "
        INSERT INTO project_brief_forms (
            booking_id, project_name, project_type, genre, target_audience,
            pages_needed, features_required, design_preferences, content_status,
            timeline, budget_range, existing_website, special_requirements,
            questions_for_team, completed, completed_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
        ON CONFLICT (booking_id) 
        DO UPDATE SET
            project_name = EXCLUDED.project_name,
            project_type = EXCLUDED.project_type,
            genre = EXCLUDED.genre,
            target_audience = EXCLUDED.target_audience,
            pages_needed = EXCLUDED.pages_needed,
            features_required = EXCLUDED.features_required,
            design_preferences = EXCLUDED.design_preferences,
            content_status = EXCLUDED.content_status,
            timeline = EXCLUDED.timeline,
            budget_range = EXCLUDED.budget_range,
            existing_website = EXCLUDED.existing_website,
            special_requirements = EXCLUDED.special_requirements,
            questions_for_team = EXCLUDED.questions_for_team,
            completed = EXCLUDED.completed,
            completed_at = EXCLUDED.completed_at,
            updated_at = NOW()
    ";
    
    let now = chrono::Utc::now();
    let completed = true; // Mark as completed when submitted
    
    state.db.execute(upsert_query, &[
        &booking_id,
        &brief.project_name,
        &brief.project_type,
        &brief.genre,
        &brief.target_audience,
        &brief.pages_needed,
        &brief.features_required,
        &brief.design_preferences,
        &brief.content_status,
        &brief.timeline,
        &brief.budget_range,
        &brief.existing_website,
        &brief.special_requirements,
        &brief.questions_for_team,
        &completed,
        &now,
    ]).await.map_err(|e| {
        tracing::error!("Failed to update project brief: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok(StatusCode::OK)
}

/// Get consultation dashboard
pub async fn get_consultation_dashboard(
    auth: AuthContext,
    State(state): State<AppState>,
) -> Result<Json<ConsultationDashboard>, StatusCode> {
    let now = chrono::Utc::now();
    
    // Get upcoming bookings
    let upcoming_query = "
        SELECT id, tenant_id, user_id, calendly_event_uuid, event_name,
               scheduled_at, status, guest_email, project_brief,
               consultation_notes, proposal_sent, created_at, updated_at
        FROM consultation_bookings 
        WHERE user_id = $1 AND scheduled_at > $2 AND status = 'scheduled'
        ORDER BY scheduled_at ASC
    ";
    
    let upcoming_rows = state.db.query(upcoming_query, &[&auth.user_id, &now]).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let mut upcoming_bookings = Vec::new();
    for row in upcoming_rows {
        upcoming_bookings.push(ConsultationBooking {
            id: row.get(0),
            tenant_id: row.get(1),
            user_id: row.get(2),
            calendly_event_uuid: row.get(3),
            event_name: row.get(4),
            scheduled_at: row.get(5),
            status: row.get(6),
            guest_email: row.get(7),
            project_brief: row.get(8),
            consultation_notes: row.get(9),
            proposal_sent: row.get(10),
            created_at: row.get(11),
            updated_at: row.get(12),
        });
    }
    
    // Get past bookings
    let past_query = "
        SELECT id, tenant_id, user_id, calendly_event_uuid, event_name,
               scheduled_at, status, guest_email, project_brief,
               consultation_notes, proposal_sent, created_at, updated_at
        FROM consultation_bookings 
        WHERE user_id = $1 AND scheduled_at <= $2
        ORDER BY scheduled_at DESC
        LIMIT 10
    ";
    
    let past_rows = state.db.query(past_query, &[&auth.user_id, &now]).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let mut past_bookings = Vec::new();
    for row in past_rows {
        past_bookings.push(ConsultationBooking {
            id: row.get(0),
            tenant_id: row.get(1),
            user_id: row.get(2),
            calendly_event_uuid: row.get(3),
            event_name: row.get(4),
            scheduled_at: row.get(5),
            status: row.get(6),
            guest_email: row.get(7),
            project_brief: row.get(8),
            consultation_notes: row.get(9),
            proposal_sent: row.get(10),
            created_at: row.get(11),
            updated_at: row.get(12),
        });
    }
    
    // Get pending briefs
    let pending_briefs_query = "
        SELECT cb.id, cb.event_name, cb.scheduled_at
        FROM consultation_bookings cb
        LEFT JOIN project_brief_forms pbf ON cb.id = pbf.booking_id
        WHERE cb.user_id = $1 
        AND cb.scheduled_at > $2 
        AND (pbf.completed IS NULL OR pbf.completed = FALSE)
        ORDER BY cb.scheduled_at ASC
    ";
    
    let pending_rows = state.db.query(pending_briefs_query, &[&auth.user_id, &now]).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let mut pending_briefs = Vec::new();
    for row in pending_rows {
        let booking_id: Uuid = row.get(0);
        pending_briefs.push(PendingBrief {
            booking_id,
            event_name: row.get(1),
            scheduled_at: row.get(2),
            brief_url: format!("/consultations/{}/brief", booking_id),
        });
    }
    
    // Get preparation materials (mock data for now)
    let preparation_materials = vec![
        PreparationMaterial {
            title: "Website Design Process".to_string(),
            material_type: "guide".to_string(),
            content: Some("Learn about our proven website design process...".to_string()),
            file_url: None,
        },
        PreparationMaterial {
            title: "Portfolio Examples".to_string(),
            material_type: "portfolio".to_string(),
            content: None,
            file_url: Some("/portfolio".to_string()),
        },
    ];
    
    let dashboard = ConsultationDashboard {
        upcoming_bookings,
        past_bookings,
        pending_briefs,
        preparation_materials,
    };
    
    Ok(Json(dashboard))
}

/// Get consultation materials
pub async fn get_consultation_materials(
    auth: AuthContext,
    State(state): State<AppState>,
    Path(booking_id): Path<Uuid>,
) -> Result<Json<Vec<PreparationMaterial>>, StatusCode> {
    // Verify booking belongs to user
    let booking_query = "
        SELECT id FROM consultation_bookings 
        WHERE id = $1 AND user_id = $2
    ";
    
    let booking_exists = state.db
        .query_opt(booking_query, &[&booking_id, &auth.user_id])
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .is_some();
    
    if !booking_exists {
        return Err(StatusCode::NOT_FOUND);
    }
    
    // Get materials
    let materials_query = "
        SELECT title, material_type, content, file_url
        FROM consultation_materials 
        WHERE booking_id = $1
        ORDER BY created_at ASC
    ";
    
    let rows = state.db.query(materials_query, &[&booking_id]).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let mut materials = Vec::new();
    for row in rows {
        materials.push(PreparationMaterial {
            title: row.get(0),
            material_type: row.get(1),
            content: row.get(2),
            file_url: row.get(3),
        });
    }
    
    Ok(Json(materials))
}
