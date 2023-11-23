use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWebhookRequest {
    pub idempotency_key: String,
    pub subscription: WebhookSubscriptionRequestObject,
}

impl CreateWebhookRequest {
    pub fn new(notification_url: String, api_version: String) -> Self {
        Self {
            idempotency_key: uuid::Uuid::new_v4().to_string(),
            subscription: WebhookSubscriptionRequestObject {
                name: "Created Invoices Webhook".to_string(),
                event_types: vec!["invoice.created".to_string()],
                notification_url,
                api_version,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookSubscriptionRequestObject {
    pub name: String,
    pub event_types: Vec<String>,
    pub notification_url: String,
    pub api_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWebhookResponse {
    pub subscription: WebhookSubscriptionResponseObject,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookSubscriptionResponseObject {
    pub api_version: String,
    pub created_at: String,
    pub enabled: bool,
    pub event_types: Vec<String>,
    pub id: String,
    pub name: String,
    pub notification_url: String,
    pub signature_key: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventTypes {
    pub event_types: Vec<String>,
    pub metadata: Vec<EventMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub api_version_introduced: String,
    pub event_type: String,
    pub release_status: String,
}
