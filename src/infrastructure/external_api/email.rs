/// Email service integration
/// Handles sending notifications via email

use reqwest::Client;
use serde_json::json;
use crate::shared::error::AppResult;

pub struct EmailService {
    client: Client,
    api_key: String,
    from_email: String,
}

impl EmailService {
    pub fn new(api_key: String, from_email: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            from_email,
        }
    }

    pub async fn send_email(&self, to: &str, subject: &str, body: &str) -> AppResult<()> {
        // Using a generic email service API (can be replaced with SendGrid, Mailgun, etc.)
        let url = "https://api.emailservice.com/v1/send";

        let payload = json!({
            "from": self.from_email,
            "to": to,
            "subject": subject,
            "html": body
        });

        let response = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await.unwrap();

        if response.status().is_success() {
            Ok(())
        } else {
            Err(crate::shared::error::AppError::ExternalService(
                format!("Email service error: {}", response.status())
            ))
        }
    }
}
