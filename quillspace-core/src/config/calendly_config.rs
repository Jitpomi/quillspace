use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendlyConfig {
    pub webhook_url: String,
    pub webhook_secret: String,
    pub api_token: String,
    pub organization_uri: String,
    pub event_type_uris: Vec<String>,
    pub redirect_url: String,
}

impl CalendlyConfig {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            webhook_url: env::var("CALENDLY_WEBHOOK_URL")
                .unwrap_or_else(|_| "https://api.quillspace.io/webhooks/calendly".to_string()),
            webhook_secret: env::var("CALENDLY_WEBHOOK_SECRET")
                .expect("CALENDLY_WEBHOOK_SECRET must be set"),
            api_token: env::var("CALENDLY_API_TOKEN")
                .expect("CALENDLY_API_TOKEN must be set"),
            organization_uri: env::var("CALENDLY_ORGANIZATION_URI")
                .expect("CALENDLY_ORGANIZATION_URI must be set"),
            event_type_uris: env::var("CALENDLY_EVENT_TYPE_URIS")
                .unwrap_or_default()
                .split(',')
                .filter(|s| !s.is_empty())
                .map(|s| s.trim().to_string())
                .collect(),
            redirect_url: env::var("CALENDLY_REDIRECT_URL")
                .unwrap_or_else(|_| "https://app.quillspace.io/consultation-booked".to_string()),
        })
    }

    /// Get webhook setup instructions for production
    pub fn get_setup_instructions(&self) -> WebhookSetupInstructions {
        WebhookSetupInstructions {
            webhook_url: self.webhook_url.clone(),
            events_to_subscribe: vec![
                "invitee.created".to_string(),
                "invitee.canceled".to_string(),
            ],
            signing_key_note: "Use the CALENDLY_WEBHOOK_SECRET environment variable".to_string(),
            redirect_configuration: RedirectConfig {
                success_url: format!("{}?event={{event_uuid}}&invitee={{invitee_uuid}}", self.redirect_url),
                cancel_url: format!("{}/cancelled", self.redirect_url),
            },
        }
    }
}

#[derive(Debug, Serialize)]
pub struct WebhookSetupInstructions {
    pub webhook_url: String,
    pub events_to_subscribe: Vec<String>,
    pub signing_key_note: String,
    pub redirect_configuration: RedirectConfig,
}

#[derive(Debug, Serialize)]
pub struct RedirectConfig {
    pub success_url: String,
    pub cancel_url: String,
}

/// Production webhook verification
pub fn verify_webhook_signature(
    payload: &str,
    signature: &str,
    secret: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    
    type HmacSha256 = Hmac<Sha256>;
    
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())?;
    mac.update(payload.as_bytes());
    
    let expected_signature = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));
    
    Ok(expected_signature == signature)
}

/// Calendly API client for production operations
pub struct CalendlyApiClient {
    config: CalendlyConfig,
    client: reqwest::Client,
}

impl CalendlyApiClient {
    pub fn new(config: CalendlyConfig) -> Self {
        let client = reqwest::Client::new();
        Self { config, client }
    }

    /// Set up webhook subscription
    pub async fn setup_webhook_subscription(&self) -> Result<WebhookSubscription, Box<dyn std::error::Error>> {
        let webhook_data = serde_json::json!({
            "url": self.config.webhook_url,
            "events": [
                "invitee.created",
                "invitee.canceled"
            ],
            "organization": self.config.organization_uri,
            "scope": "organization"
        });

        let response = self.client
            .post("https://api.calendly.com/webhook_subscriptions")
            .header("Authorization", format!("Bearer {}", self.config.api_token))
            .header("Content-Type", "application/json")
            .json(&webhook_data)
            .send()
            .await?;

        if response.status().is_success() {
            let subscription: WebhookSubscription = response.json().await?;
            Ok(subscription)
        } else {
            let error_text = response.text().await?;
            Err(format!("Failed to create webhook subscription: {}", error_text).into())
        }
    }

    /// List existing webhook subscriptions
    pub async fn list_webhook_subscriptions(&self) -> Result<Vec<WebhookSubscription>, Box<dyn std::error::Error>> {
        let response = self.client
            .get("https://api.calendly.com/webhook_subscriptions")
            .header("Authorization", format!("Bearer {}", self.config.api_token))
            .query(&[("organization", &self.config.organization_uri)])
            .send()
            .await?;

        if response.status().is_success() {
            let data: WebhookListResponse = response.json().await?;
            Ok(data.collection)
        } else {
            let error_text = response.text().await?;
            Err(format!("Failed to list webhook subscriptions: {}", error_text).into())
        }
    }

    /// Update Calendly event type with redirect URL
    pub async fn update_event_type_redirect(&self, event_type_uri: &str) -> Result<(), Box<dyn std::error::Error>> {
        let redirect_data = serde_json::json!({
            "custom_questions": [],
            "redirect_url": format!("{}?event={{event_uuid}}&invitee={{invitee_uuid}}", self.config.redirect_url)
        });

        let response = self.client
            .patch(&format!("https://api.calendly.com/event_types/{}", event_type_uri))
            .header("Authorization", format!("Bearer {}", self.config.api_token))
            .header("Content-Type", "application/json")
            .json(&redirect_data)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Failed to update event type redirect: {}", error_text).into());
        }

        Ok(())
    }

    /// Get event details from Calendly API
    pub async fn get_event_details(&self, event_uuid: &str) -> Result<CalendlyEvent, Box<dyn std::error::Error>> {
        let response = self.client
            .get(&format!("https://api.calendly.com/scheduled_events/{}", event_uuid))
            .header("Authorization", format!("Bearer {}", self.config.api_token))
            .send()
            .await?;

        if response.status().is_success() {
            let event_response: EventResponse = response.json().await?;
            Ok(event_response.resource)
        } else {
            let error_text = response.text().await?;
            Err(format!("Failed to get event details: {}", error_text).into())
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookSubscription {
    pub uri: String,
    pub callback_url: String,
    pub created_at: String,
    pub updated_at: String,
    pub retry_started_at: Option<String>,
    pub state: String,
    pub events: Vec<String>,
    pub scope: String,
    pub organization: String,
}

#[derive(Debug, Deserialize)]
struct WebhookListResponse {
    collection: Vec<WebhookSubscription>,
}

#[derive(Debug, Deserialize)]
struct EventResponse {
    resource: CalendlyEvent,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CalendlyEvent {
    pub uri: String,
    pub name: String,
    pub status: String,
    pub start_time: String,
    pub end_time: String,
    pub event_type: String,
    pub location: Option<CalendlyLocation>,
    pub invitees_counter: CalendlyInviteesCounter,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CalendlyLocation {
    #[serde(rename = "type")]
    pub location_type: String,
    pub location: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CalendlyInviteesCounter {
    pub total: i32,
    pub active: i32,
    pub limit: i32,
}

/// Production deployment checklist
pub fn get_production_checklist() -> ProductionChecklist {
    ProductionChecklist {
        environment_variables: vec![
            EnvVar {
                name: "CALENDLY_WEBHOOK_SECRET".to_string(),
                description: "Secret key for webhook signature verification".to_string(),
                required: true,
                example: Some("your_webhook_secret_here".to_string()),
            },
            EnvVar {
                name: "CALENDLY_API_TOKEN".to_string(),
                description: "Calendly API personal access token".to_string(),
                required: true,
                example: Some("your_api_token_here".to_string()),
            },
            EnvVar {
                name: "CALENDLY_ORGANIZATION_URI".to_string(),
                description: "Your Calendly organization URI".to_string(),
                required: true,
                example: Some("https://api.calendly.com/organizations/AAAAAAAAAAAAAAAA".to_string()),
            },
            EnvVar {
                name: "CALENDLY_WEBHOOK_URL".to_string(),
                description: "Public URL for webhook endpoint".to_string(),
                required: false,
                example: Some("https://api.quillspace.io/webhooks/calendly".to_string()),
            },
            EnvVar {
                name: "CALENDLY_REDIRECT_URL".to_string(),
                description: "URL to redirect users after booking".to_string(),
                required: false,
                example: Some("https://app.quillspace.io/consultation-booked".to_string()),
            },
        ],
        setup_steps: vec![
            "1. Create Calendly Personal Access Token in your account settings".to_string(),
            "2. Get your organization URI from Calendly API".to_string(),
            "3. Set up environment variables in production".to_string(),
            "4. Deploy webhook endpoint to production".to_string(),
            "5. Run webhook subscription setup".to_string(),
            "6. Update Calendly event types with redirect URLs".to_string(),
            "7. Test webhook delivery with a test booking".to_string(),
        ],
        testing_checklist: vec![
            "✓ Webhook signature verification works".to_string(),
            "✓ Booking creation triggers email sequence".to_string(),
            "✓ User redirection works after booking".to_string(),
            "✓ Project brief form is accessible".to_string(),
            "✓ Consultation dashboard shows booking".to_string(),
            "✓ Email automation sequences are triggered".to_string(),
        ],
    }
}

#[derive(Debug, Serialize)]
pub struct ProductionChecklist {
    pub environment_variables: Vec<EnvVar>,
    pub setup_steps: Vec<String>,
    pub testing_checklist: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct EnvVar {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub example: Option<String>,
}
