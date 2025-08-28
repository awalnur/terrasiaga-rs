/// WhatsApp API integration
/// Handles sending notifications via WhatsApp Business API

use reqwest::Client;
use serde_json::json;
use crate::shared::error::AppResult;

pub struct WhatsAppService {
    client: Client,
    api_token: String,
    phone_number_id: String,
}

impl WhatsAppService {
    pub fn new(api_token: String, phone_number_id: String) -> Self {
        Self {
            client: Client::new(),
            api_token,
            phone_number_id,
        }
    }

    pub async fn send_message(&self, to: &str, message: &str) -> AppResult<()> {
        let url = format!(
            "https://graph.facebook.com/v17.0/{}/messages",
            self.phone_number_id
        );

        let payload = json!({
            "messaging_product": "whatsapp",
            "to": to,
            "type": "text",
            "text": {
                "body": message
            }
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await.unwrap();

        if response.status().is_success() {
            Ok(())
        } else {
            Err(crate::shared::error::AppError::ExternalService(
                format!("WhatsApp API error: {}", response.status())
            ))
        }
    }
}
