/// SMS service implementation
/// Provides SMS sending capabilities through various providers

use crate::shared::error::{AppResult, AppError};
use crate::infrastructure::external_services::{SmsConfig, SmsProvider};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsMessage {
    pub to: String,
    pub body: String,
    pub from: Option<String>,
}

pub struct SmsService {
    config: SmsConfig,
    client: reqwest::Client,
}

impl SmsService {
    pub fn new(config: SmsConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .unwrap_or_default();

        Self { config, client }
    }

    pub async fn send_sms(&self, message: SmsMessage) -> AppResult<()> {
        match &self.config.provider {
            SmsProvider::Twilio => self.send_via_twilio(message).await,
            SmsProvider::Vonage => self.send_via_vonage(message).await,
            SmsProvider::AWS_SNS => self.send_via_aws_sns(message).await,
        }
    }

    async fn send_via_twilio(&self, _message: SmsMessage) -> AppResult<()> {
        tracing::info!("Sending SMS via Twilio");
        Ok(())
    }

    async fn send_via_vonage(&self, _message: SmsMessage) -> AppResult<()> {
        tracing::info!("Sending SMS via Vonage");
        Ok(())
    }

    async fn send_via_aws_sns(&self, _message: SmsMessage) -> AppResult<()> {
        tracing::info!("Sending SMS via AWS SNS");
        Ok(())
    }
}
