/// Notification service implementation
/// Provides unified notification capabilities across multiple channels

use crate::shared::error::{AppResult, AppError};
use crate::infrastructure::external_services::{
    email::{EmailService, EmailMessage},
    sms::{SmsService, SmsMessage},
    whatsapp::{WhatsAppService, WhatsAppMessage, WhatsAppMessageType},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationMessage {
    pub title: String,
    pub body: String,
    pub recipients: Vec<NotificationRecipient>,
    pub channels: Vec<NotificationChannel>,
    pub priority: NotificationPriority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRecipient {
    pub user_id: uuid::Uuid,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationChannel {
    Email,
    SMS,
    WhatsApp,
    Push,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Critical,
}

pub struct NotificationService {
    email_service: Option<EmailService>,
    sms_service: Option<SmsService>,
    whatsapp_service: Option<WhatsAppService>,
}

impl NotificationService {
    pub fn new(
        email_service: Option<EmailService>,
        sms_service: Option<SmsService>,
        whatsapp_service: Option<WhatsAppService>,
    ) -> Self {
        Self {
            email_service,
            sms_service,
            whatsapp_service,
        }
    }

    pub async fn send_notification(&self, notification: NotificationMessage) -> AppResult<Vec<NotificationResult>> {
        let mut results = Vec::new();

        for channel in &notification.channels {
            match channel {
                NotificationChannel::Email => {
                    if let Some(email_service) = &self.email_service {
                        let result = self.send_email_notifications(email_service, &notification).await;
                        results.push(NotificationResult {
                            channel: channel.clone(),
                            success: result.is_ok(),
                            error: result.err().map(|e| e.to_string()),
                        });
                    }
                }
                NotificationChannel::SMS => {
                    if let Some(sms_service) = &self.sms_service {
                        let result = self.send_sms_notifications(sms_service, &notification).await;
                        results.push(NotificationResult {
                            channel: channel.clone(),
                            success: result.is_ok(),
                            error: result.err().map(|e| e.to_string()),
                        });
                    }
                }
                NotificationChannel::WhatsApp => {
                    if let Some(whatsapp_service) = &self.whatsapp_service {
                        let result = self.send_whatsapp_notifications(whatsapp_service, &notification).await;
                        results.push(NotificationResult {
                            channel: channel.clone(),
                            success: result.is_ok(),
                            error: result.err().map(|e| e.to_string()),
                        });
                    }
                }
                NotificationChannel::Push => {
                    // Push notification implementation would go here
                    results.push(NotificationResult {
                        channel: channel.clone(),
                        success: true,
                        error: None,
                    });
                }
            }
        }

        Ok(results)
    }

    async fn send_email_notifications(
        &self,
        email_service: &EmailService,
        notification: &NotificationMessage,
    ) -> AppResult<()> {
        let recipients: Vec<String> = notification
            .recipients
            .iter()
            .filter_map(|r| r.email.clone())
            .collect();

        if recipients.is_empty() {
            return Ok(());
        }

        let email_message = EmailMessage {
            to: recipients,
            subject: notification.title.clone(),
            body: notification.body.clone(),
            html_body: None,
            from_name: Some("Terra Siaga".to_string()),
        };

        email_service.send_email(email_message).await
    }

    async fn send_sms_notifications(
        &self,
        sms_service: &SmsService,
        notification: &NotificationMessage,
    ) -> AppResult<()> {
        for recipient in &notification.recipients {
            if let Some(phone) = &recipient.phone {
                let sms_message = SmsMessage {
                    to: phone.clone(),
                    body: format!("{}\n\n{}", notification.title, notification.body),
                    from: None,
                };

                sms_service.send_sms(sms_message).await?;
            }
        }
        Ok(())
    }

    async fn send_whatsapp_notifications(
        &self,
        whatsapp_service: &WhatsAppService,
        notification: &NotificationMessage,
    ) -> AppResult<()> {
        for recipient in &notification.recipients {
            if let Some(phone) = &recipient.phone {
                let whatsapp_message = WhatsAppMessage {
                    to: phone.clone(),
                    body: format!("{}\n\n{}", notification.title, notification.body),
                    message_type: WhatsAppMessageType::Text,
                };

                whatsapp_service.send_message(whatsapp_message).await?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationResult {
    pub channel: NotificationChannel,
    pub success: bool,
    pub error: Option<String>,
}
