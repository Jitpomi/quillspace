use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use crate::database::Database;
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailTemplate {
    pub id: Uuid,
    pub name: String,
    pub subject: String,
    pub html_content: String,
    pub text_content: String,
    pub template_type: EmailType,
    pub variables: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmailType {
    BookingConfirmation,
    PreConsultationReminder,
    PostConsultationFollowup,
    ProjectBriefReminder,
    ProposalSent,
    ProposalAccepted,
    ProjectKickoff,
    ProjectUpdate,
    ProjectCompletion,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailJob {
    pub id: Uuid,
    pub booking_id: Uuid,
    pub email_type: EmailType,
    pub recipient_email: String,
    pub template_variables: serde_json::Value,
    pub scheduled_for: DateTime<Utc>,
    pub sent_at: Option<DateTime<Utc>>,
    pub status: EmailStatus,
    pub retry_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EmailStatus {
    Pending,
    Sent,
    Failed,
    Cancelled,
}

pub struct EmailAutomationService {
    db: Database,
}

impl EmailAutomationService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Trigger email sequence for new booking
    pub async fn trigger_booking_sequence(&self, booking_id: Uuid) -> Result<()> {
        let booking = self.get_booking_details(booking_id).await?;
        
        // 1. Immediate confirmation email
        self.schedule_email(
            booking_id,
            EmailType::BookingConfirmation,
            &booking.guest_email,
            serde_json::json!({
                "event_name": booking.event_name,
                "scheduled_at": booking.scheduled_at,
                "consultation_url": format!("/consultations/{}", booking_id),
                "brief_url": format!("/consultations/{}/brief", booking_id)
            }),
            Utc::now(),
        ).await?;

        // 2. Project brief reminder (2 hours after booking)
        self.schedule_email(
            booking_id,
            EmailType::ProjectBriefReminder,
            &booking.guest_email,
            serde_json::json!({
                "event_name": booking.event_name,
                "scheduled_at": booking.scheduled_at,
                "brief_url": format!("/consultations/{}/brief", booking_id)
            }),
            Utc::now() + Duration::hours(2),
        ).await?;

        // 3. Pre-consultation reminder (24 hours before)
        self.schedule_email(
            booking_id,
            EmailType::PreConsultationReminder,
            &booking.guest_email,
            serde_json::json!({
                "event_name": booking.event_name,
                "scheduled_at": booking.scheduled_at,
                "preparation_checklist": self.get_preparation_checklist(),
                "zoom_link": "TBD" // Would come from Calendly
            }),
            booking.scheduled_at - Duration::hours(24),
        ).await?;

        Ok(())
    }

    /// Schedule individual email
    async fn schedule_email(
        &self,
        booking_id: Uuid,
        email_type: EmailType,
        recipient: &str,
        variables: serde_json::Value,
        scheduled_for: DateTime<Utc>,
    ) -> Result<()> {
        let email_job = EmailJob {
            id: Uuid::new_v4(),
            booking_id,
            email_type,
            recipient_email: recipient.to_string(),
            template_variables: variables,
            scheduled_for,
            sent_at: None,
            status: EmailStatus::Pending,
            retry_count: 0,
            created_at: Utc::now(),
        };

        let query = "
            INSERT INTO email_jobs (
                id, booking_id, email_type, recipient_email, template_variables,
                scheduled_for, sent_at, status, retry_count, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        ";

        self.db.execute(query, &[
            &email_job.id,
            &email_job.booking_id,
            &email_job.email_type,
            &email_job.recipient_email,
            &email_job.template_variables,
            &email_job.scheduled_for,
            &email_job.sent_at,
            &email_job.status,
            &email_job.retry_count,
            &email_job.created_at,
        ]).await?;

        Ok(())
    }

    /// Process pending emails (called by cron job)
    pub async fn process_pending_emails(&self) -> Result<()> {
        let query = "
            SELECT id, booking_id, email_type, recipient_email, template_variables,
                   scheduled_for, sent_at, status, retry_count, created_at
            FROM email_jobs 
            WHERE status = 'pending' 
            AND scheduled_for <= NOW()
            AND retry_count < 3
            ORDER BY scheduled_for ASC
            LIMIT 50
        ";

        let rows = self.db.query(query, &[]).await?;

        for row in rows {
            let email_job = EmailJob {
                id: row.get(0),
                booking_id: row.get(1),
                email_type: row.get(2),
                recipient_email: row.get(3),
                template_variables: row.get(4),
                scheduled_for: row.get(5),
                sent_at: row.get(6),
                status: row.get(7),
                retry_count: row.get(8),
                created_at: row.get(9),
            };

            match self.send_email(&email_job).await {
                Ok(_) => {
                    self.mark_email_sent(email_job.id).await?;
                }
                Err(e) => {
                    tracing::error!("Failed to send email {}: {}", email_job.id, e);
                    self.increment_retry_count(email_job.id).await?;
                }
            }
        }

        Ok(())
    }

    /// Send individual email
    async fn send_email(&self, job: &EmailJob) -> Result<()> {
        let template = self.get_email_template(&job.email_type).await?;
        
        // Replace template variables
        let subject = self.replace_variables(&template.subject, &job.template_variables);
        let html_content = self.replace_variables(&template.html_content, &job.template_variables);
        let text_content = self.replace_variables(&template.text_content, &job.template_variables);

        // TODO: Integrate with actual email service (SendGrid, AWS SES, etc.)
        tracing::info!(
            "Sending email: {} to {} with subject: {}",
            job.email_type,
            job.recipient_email,
            subject
        );

        // Mock email sending for now
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        Ok(())
    }

    /// Get email template by type
    async fn get_email_template(&self, email_type: &EmailType) -> Result<EmailTemplate> {
        // For now, return hardcoded templates
        // In production, these would be stored in database
        Ok(match email_type {
            EmailType::BookingConfirmation => EmailTemplate {
                id: Uuid::new_v4(),
                name: "Booking Confirmation".to_string(),
                subject: "🎉 Your QuillSpace consultation is confirmed!".to_string(),
                html_content: self.get_booking_confirmation_template(),
                text_content: "Your consultation has been confirmed...".to_string(),
                template_type: EmailType::BookingConfirmation,
                variables: vec!["event_name".to_string(), "scheduled_at".to_string()],
            },
            EmailType::ProjectBriefReminder => EmailTemplate {
                id: Uuid::new_v4(),
                name: "Project Brief Reminder".to_string(),
                subject: "📝 Complete your project brief for maximum consultation value".to_string(),
                html_content: self.get_brief_reminder_template(),
                text_content: "Please complete your project brief...".to_string(),
                template_type: EmailType::ProjectBriefReminder,
                variables: vec!["brief_url".to_string()],
            },
            EmailType::PreConsultationReminder => EmailTemplate {
                id: Uuid::new_v4(),
                name: "Pre-consultation Reminder".to_string(),
                subject: "⏰ Your QuillSpace consultation is tomorrow!".to_string(),
                html_content: self.get_pre_consultation_template(),
                text_content: "Your consultation is tomorrow...".to_string(),
                template_type: EmailType::PreConsultationReminder,
                variables: vec!["event_name".to_string(), "preparation_checklist".to_string()],
            },
            _ => {
                return Err(anyhow::anyhow!("Template not found for email type: {:?}", email_type));
            }
        })
    }

    /// Replace template variables
    fn replace_variables(&self, template: &str, variables: &serde_json::Value) -> String {
        let mut result = template.to_string();
        
        if let Some(obj) = variables.as_object() {
            for (key, value) in obj {
                let placeholder = format!("{{{{{}}}}}", key);
                let replacement = match value {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Number(n) => n.to_string(),
                    _ => value.to_string(),
                };
                result = result.replace(&placeholder, &replacement);
            }
        }
        
        result
    }

    /// Mark email as sent
    async fn mark_email_sent(&self, email_id: Uuid) -> Result<()> {
        let query = "
            UPDATE email_jobs 
            SET status = 'sent', sent_at = NOW() 
            WHERE id = $1
        ";
        self.db.execute(query, &[&email_id]).await?;
        Ok(())
    }

    /// Increment retry count
    async fn increment_retry_count(&self, email_id: Uuid) -> Result<()> {
        let query = "
            UPDATE email_jobs 
            SET retry_count = retry_count + 1,
                status = CASE WHEN retry_count >= 2 THEN 'failed' ELSE 'pending' END
            WHERE id = $1
        ";
        self.db.execute(query, &[&email_id]).await?;
        Ok(())
    }

    /// Get booking details
    async fn get_booking_details(&self, booking_id: Uuid) -> Result<BookingDetails> {
        let query = "
            SELECT event_name, scheduled_at, guest_email
            FROM consultation_bookings 
            WHERE id = $1
        ";
        
        let row = self.db.query_one(query, &[&booking_id]).await?;
        
        Ok(BookingDetails {
            event_name: row.get(0),
            scheduled_at: row.get(1),
            guest_email: row.get(2),
        })
    }

    /// Get preparation checklist
    fn get_preparation_checklist(&self) -> Vec<String> {
        vec![
            "Author bio and headshot".to_string(),
            "Book covers and descriptions".to_string(),
            "Existing website URL (if any)".to_string(),
            "Social media handles".to_string(),
            "Preferred color schemes or design inspiration".to_string(),
            "List of must-have website features".to_string(),
            "Content you want to include".to_string(),
            "Questions about the design process".to_string(),
        ]
    }

    // Email templates (in production, these would be in database with rich editor)
    fn get_booking_confirmation_template(&self) -> String {
        r#"
        <div style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto;">
            <div style="background: linear-gradient(135deg, #9CAF88, #7A9B6E); padding: 40px 20px; text-align: center;">
                <h1 style="color: white; margin: 0; font-size: 28px;">🎉 Consultation Confirmed!</h1>
                <p style="color: white; margin: 10px 0 0 0; opacity: 0.9;">We're excited to help bring your author website to life</p>
            </div>
            
            <div style="padding: 30px 20px; background: white;">
                <h2 style="color: #333; margin-bottom: 20px;">What's Next?</h2>
                
                <div style="background: #f8f9fa; padding: 20px; border-radius: 8px; margin-bottom: 20px;">
                    <h3 style="color: #9CAF88; margin-top: 0;">📝 Complete Your Project Brief</h3>
                    <p>Help us prepare for our consultation by filling out your project requirements.</p>
                    <a href="{{brief_url}}" style="background: #9CAF88; color: white; padding: 12px 24px; text-decoration: none; border-radius: 6px; display: inline-block;">Complete Brief →</a>
                </div>
                
                <div style="border-left: 4px solid #9CAF88; padding-left: 20px; margin: 20px 0;">
                    <h3 style="margin-top: 0;">Your Consultation Details</h3>
                    <p><strong>Event:</strong> {{event_name}}</p>
                    <p><strong>Date & Time:</strong> {{scheduled_at}}</p>
                </div>
                
                <p>We'll send you a reminder 24 hours before our consultation with preparation materials.</p>
                
                <hr style="border: none; border-top: 1px solid #eee; margin: 30px 0;">
                
                <p style="color: #666; font-size: 14px;">
                    Questions? Reply to this email or visit your 
                    <a href="{{consultation_url}}" style="color: #9CAF88;">consultation dashboard</a>.
                </p>
            </div>
        </div>
        "#.to_string()
    }

    fn get_brief_reminder_template(&self) -> String {
        r#"
        <div style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto;">
            <div style="background: #FFF3CD; border: 1px solid #FFEAA7; padding: 20px; border-radius: 8px; margin-bottom: 20px;">
                <h2 style="color: #856404; margin-top: 0;">📝 Don't forget your project brief!</h2>
                <p style="color: #856404;">Completing your project brief helps us prepare a more valuable consultation for you.</p>
            </div>
            
            <div style="padding: 20px; background: white;">
                <p>Hi there!</p>
                
                <p>We noticed you haven't completed your project brief yet. Taking just 5-10 minutes to fill this out will help us:</p>
                
                <ul>
                    <li>Understand your specific needs and goals</li>
                    <li>Prepare relevant examples and recommendations</li>
                    <li>Make the most of our consultation time together</li>
                </ul>
                
                <div style="text-align: center; margin: 30px 0;">
                    <a href="{{brief_url}}" style="background: #9CAF88; color: white; padding: 15px 30px; text-decoration: none; border-radius: 6px; font-size: 16px;">Complete Project Brief →</a>
                </div>
                
                <p style="color: #666; font-size: 14px;">
                    This will only take a few minutes and will make our consultation much more valuable for you.
                </p>
            </div>
        </div>
        "#.to_string()
    }

    fn get_pre_consultation_template(&self) -> String {
        r#"
        <div style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto;">
            <div style="background: #9CAF88; padding: 30px 20px; text-align: center;">
                <h1 style="color: white; margin: 0;">⏰ Your consultation is tomorrow!</h1>
                <p style="color: white; margin: 10px 0 0 0; opacity: 0.9;">{{event_name}}</p>
            </div>
            
            <div style="padding: 30px 20px; background: white;">
                <h2>We're looking forward to our conversation!</h2>
                
                <p>Here's a quick checklist to help you prepare:</p>
                
                <div style="background: #f8f9fa; padding: 20px; border-radius: 8px;">
                    <h3 style="color: #9CAF88; margin-top: 0;">Preparation Checklist:</h3>
                    {{preparation_checklist}}
                </div>
                
                <div style="background: #E3F2FD; padding: 20px; border-radius: 8px; margin: 20px 0;">
                    <h3 style="color: #1976D2; margin-top: 0;">What to Expect:</h3>
                    <ul style="margin: 0;">
                        <li><strong>Discovery (15 min):</strong> We'll discuss your goals and vision</li>
                        <li><strong>Design Review (10 min):</strong> Show examples that fit your style</li>
                        <li><strong>Next Steps (5 min):</strong> Outline timeline and investment</li>
                    </ul>
                </div>
                
                <p><strong>Meeting Link:</strong> {{zoom_link}}</p>
                
                <p>See you tomorrow!</p>
            </div>
        </div>
        "#.to_string()
    }
}

#[derive(Debug)]
struct BookingDetails {
    event_name: String,
    scheduled_at: DateTime<Utc>,
    guest_email: String,
}
