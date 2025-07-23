/// WhatsApp service implementation
/// Provides WhatsApp messaging capabilities through various providers

use crate::shared::error::{AppResult, AppError};
use crate::infrastructure::external_services::{WhatsAppConfig, WhatsAppProvider};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppMessage {
    pub to: String,
    pub body: String,
    pub message_type: WhatsAppMessageType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WhatsAppMessageType {
    Text,
    Template { template_name: String, parameters: Vec<String> },
}

pub struct WhatsAppService {
    config: WhatsAppConfig,
    client: reqwest::Client,
}

impl WhatsAppService {
    pub fn new(config: WhatsAppConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .unwrap_or_default();

        Self { config, client }
    }

    pub async fn send_message(&self, message: WhatsAppMessage) -> AppResult<()> {
        match &self.config.provider {
            WhatsAppProvider::Meta => self.send_via_meta(message).await,
            WhatsAppProvider::Twilio => self.send_via_twilio(message).await,
        }
    }

    async fn send_via_meta(&self, _message: WhatsAppMessage) -> AppResult<()> {
        tracing::info!("Sending WhatsApp message via Meta");
        Ok(())
    }

    async fn send_via_twilio(&self, _message: WhatsAppMessage) -> AppResult<()> {
        tracing::info!("Sending WhatsApp message via Twilio");
        Ok(())
    }
}
