use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::database::Database;
use anyhow::Result;

#[derive(Debug, Deserialize)]
pub struct CalendlyWebhookPayload {
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub event: String,
    pub payload: CalendlyEventPayload,
}

#[derive(Debug, Deserialize)]
pub struct CalendlyEventPayload {
    pub event_type: CalendlyEventType,
    pub scheduled_event: CalendlyScheduledEvent,
}

#[derive(Debug, Deserialize)]
pub struct CalendlyEventType {
    pub uuid: String,
    pub name: String,
    pub duration: i32,
}

#[derive(Debug, Deserialize)]
pub struct CalendlyScheduledEvent {
    pub uuid: String,
    pub name: String,
    pub status: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub event_guests: Vec<CalendlyGuest>,
    pub location: Option<CalendlyLocation>,
    pub invitees_counter: CalendlyInviteesCounter,
}

#[derive(Debug, Deserialize)]
pub struct CalendlyGuest {
    pub email: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CalendlyLocation {
    #[serde(rename = "type")]
    pub location_type: String,
    pub location: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CalendlyInviteesCounter {
    pub total: i32,
    pub active: i32,
    pub limit: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConsultationBooking {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub calendly_event_uuid: String,
    pub event_name: String,
    pub scheduled_at: DateTime<Utc>,
    pub status: BookingStatus,
    pub guest_email: String,
    pub project_brief: Option<serde_json::Value>,
    pub consultation_notes: Option<String>,
    pub proposal_sent: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BookingStatus {
    Scheduled,
    Completed,
    Cancelled,
    NoShow,
    Rescheduled,
}

pub struct CalendlyService {
    db: Database,
}

impl CalendlyService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Handle incoming Calendly webhook
    pub async fn handle_webhook(&self, payload: CalendlyWebhookPayload) -> Result<()> {
        match payload.event.as_str() {
            "invitee.created" => {
                self.handle_booking_created(payload.payload).await?;
            }
            "invitee.canceled" => {
                self.handle_booking_cancelled(payload.payload).await?;
            }
            _ => {
                tracing::info!("Unhandled Calendly webhook event: {}", payload.event);
            }
        }
        Ok(())
    }

    /// Handle new booking creation
    async fn handle_booking_created(&self, payload: CalendlyEventPayload) -> Result<()> {
        let guest_email = payload.scheduled_event.event_guests
            .first()
            .map(|g| g.email.clone())
            .unwrap_or_default();

        // Find user by email
        let user = self.find_user_by_email(&guest_email).await?;
        
        if let Some((user_id, tenant_id)) = user {
            let booking = ConsultationBooking {
                id: Uuid::new_v4(),
                tenant_id,
                user_id,
                calendly_event_uuid: payload.scheduled_event.uuid.clone(),
                event_name: payload.scheduled_event.name.clone(),
                scheduled_at: payload.scheduled_event.start_time,
                status: BookingStatus::Scheduled,
                guest_email,
                project_brief: None,
                consultation_notes: None,
                proposal_sent: false,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            self.create_booking(booking).await?;
            
            // Trigger post-booking workflow
            self.trigger_post_booking_workflow(&payload.scheduled_event.uuid).await?;
        }

        Ok(())
    }

    /// Handle booking cancellation
    async fn handle_booking_cancelled(&self, payload: CalendlyEventPayload) -> Result<()> {
        self.update_booking_status(
            &payload.scheduled_event.uuid,
            BookingStatus::Cancelled,
        ).await?;
        Ok(())
    }

    /// Find user by email address
    async fn find_user_by_email(&self, email: &str) -> Result<Option<(Uuid, Uuid)>> {
        let query = "
            SELECT u.id, u.tenant_id 
            FROM users u 
            WHERE u.email = $1 
            LIMIT 1
        ";
        
        let row = self.db.query_opt(query, &[&email]).await?;
        
        if let Some(row) = row {
            let user_id: Uuid = row.get(0);
            let tenant_id: Uuid = row.get(1);
            Ok(Some((user_id, tenant_id)))
        } else {
            Ok(None)
        }
    }

    /// Create new consultation booking
    async fn create_booking(&self, booking: ConsultationBooking) -> Result<()> {
        let query = "
            INSERT INTO consultation_bookings (
                id, tenant_id, user_id, calendly_event_uuid, event_name,
                scheduled_at, status, guest_email, project_brief,
                consultation_notes, proposal_sent, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        ";

        self.db.execute(query, &[
            &booking.id,
            &booking.tenant_id,
            &booking.user_id,
            &booking.calendly_event_uuid,
            &booking.event_name,
            &booking.scheduled_at,
            &booking.status,
            &booking.guest_email,
            &booking.project_brief,
            &booking.consultation_notes,
            &booking.proposal_sent,
            &booking.created_at,
            &booking.updated_at,
        ]).await?;

        Ok(())
    }

    /// Update booking status
    async fn update_booking_status(&self, calendly_uuid: &str, status: BookingStatus) -> Result<()> {
        let query = "
            UPDATE consultation_bookings 
            SET status = $1, updated_at = $2 
            WHERE calendly_event_uuid = $3
        ";

        self.db.execute(query, &[
            &status,
            &Utc::now(),
            &calendly_uuid,
        ]).await?;

        Ok(())
    }

    /// Get booking by Calendly UUID
    pub async fn get_booking_by_calendly_uuid(&self, calendly_uuid: &str) -> Result<Option<ConsultationBooking>> {
        let query = "
            SELECT id, tenant_id, user_id, calendly_event_uuid, event_name,
                   scheduled_at, status, guest_email, project_brief,
                   consultation_notes, proposal_sent, created_at, updated_at
            FROM consultation_bookings 
            WHERE calendly_event_uuid = $1
        ";

        let row = self.db.query_opt(query, &[&calendly_uuid]).await?;
        
        if let Some(row) = row {
            Ok(Some(ConsultationBooking {
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
            }))
        } else {
            Ok(None)
        }
    }

    /// Trigger post-booking workflow
    async fn trigger_post_booking_workflow(&self, calendly_uuid: &str) -> Result<()> {
        // This could trigger:
        // 1. Email notifications
        // 2. Project brief form creation
        // 3. Preparation materials sending
        // 4. Internal team notifications
        
        tracing::info!("Triggering post-booking workflow for event: {}", calendly_uuid);
        
        // TODO: Implement specific workflow steps
        // - Send welcome email with prep materials
        // - Create project brief form
        // - Notify internal team
        // - Set up consultation dashboard
        
        Ok(())
    }

    /// Update project brief
    pub async fn update_project_brief(&self, booking_id: Uuid, brief: serde_json::Value) -> Result<()> {
        let query = "
            UPDATE consultation_bookings 
            SET project_brief = $1, updated_at = $2 
            WHERE id = $3
        ";

        self.db.execute(query, &[
            &brief,
            &Utc::now(),
            &booking_id,
        ]).await?;

        Ok(())
    }

    /// Get user's bookings
    pub async fn get_user_bookings(&self, user_id: Uuid) -> Result<Vec<ConsultationBooking>> {
        let query = "
            SELECT id, tenant_id, user_id, calendly_event_uuid, event_name,
                   scheduled_at, status, guest_email, project_brief,
                   consultation_notes, proposal_sent, created_at, updated_at
            FROM consultation_bookings 
            WHERE user_id = $1
            ORDER BY scheduled_at DESC
        ";

        let rows = self.db.query(query, &[&user_id]).await?;
        let mut bookings = Vec::new();

        for row in rows {
            bookings.push(ConsultationBooking {
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

        Ok(bookings)
    }
}
