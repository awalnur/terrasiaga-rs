/// External notification service implementations
/// Handles SMS, email, push notifications, and WhatsApp integration

use async_trait::async_trait;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use reqwest::Client as HttpClient;

use crate::domain::entities::{User, Notification, Disaster};
use crate::domain::ports::services::NotificationService;
use crate::application::use_cases::EmergencyResponseDispatchResponse;
use crate::shared::{AppResult, AppError};

/// SMS service configuration
#[derive(Debug, Clone)]
pub struct SmsConfig {
    pub provider: String, // "twilio", "nexmo", "local_provider"
    pub api_key: String,
    pub api_secret: String,
    pub sender_id: String,
    pub base_url: String,
}

/// Email service configuration
#[derive(Debug, Clone)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
    pub from_email: String,
    pub from_name: String,
}

/// Push notification configuration
#[derive(Debug, Clone)]
pub struct PushConfig {
    pub firebase_server_key: String,
    pub firebase_sender_id: String,
    pub apns_key_id: String,
    pub apns_team_id: String,
    pub apns_bundle_id: String,
}

/// WhatsApp Business API configuration
#[derive(Debug, Clone)]
pub struct WhatsAppConfig {
    pub access_token: String,
    pub phone_number_id: String,
    pub business_account_id: String,
    pub webhook_verify_token: String,
}

/// Comprehensive notification service implementation
pub struct ExternalNotificationService {
    http_client: HttpClient,
    sms_config: SmsConfig,
    email_config: EmailConfig,
    push_config: PushConfig,
    whatsapp_config: WhatsAppConfig,
}

impl ExternalNotificationService {
    pub fn new(
        sms_config: SmsConfig,
        email_config: EmailConfig,
        push_config: PushConfig,
        whatsapp_config: WhatsAppConfig,
    ) -> Self {
        let http_client = HttpClient::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            http_client,
            sms_config,
            email_config,
            push_config,
            whatsapp_config,
        }
    }

    /// Send SMS via Twilio or local provider
    async fn send_sms(&self, phone: &str, message: &str) -> AppResult<()> {
        match self.sms_config.provider.as_str() {
            "twilio" => self.send_twilio_sms(phone, message).await,
            "local_provider" => self.send_local_sms(phone, message).await,
            _ => Err(AppError::External("Unsupported SMS provider".to_string())),
        }
    }

    /// Send SMS via Twilio
    async fn send_twilio_sms(&self, phone: &str, message: &str) -> AppResult<()> {
        #[derive(Serialize)]
        struct TwilioSmsRequest {
            #[serde(rename = "To")]
            to: String,
            #[serde(rename = "From")]
            from: String,
            #[serde(rename = "Body")]
            body: String,
        }

        let request = TwilioSmsRequest {
            to: phone.to_string(),
            from: self.sms_config.sender_id.clone(),
            body: message.to_string(),
        };

        let url = format!("{}/2010-04-01/Accounts/{}/Messages.json",
                         self.sms_config.base_url, self.sms_config.api_key);

        let response = self.http_client
            .post(&url)
            .basic_auth(&self.sms_config.api_key, Some(&self.sms_config.api_secret))
            .form(&request)
            .send()
            .await
            .map_err(|e| AppError::External(format!("Twilio SMS request failed: {}", e)))?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(AppError::External(format!("Twilio SMS failed: {}", error_text)))
        }
    }

    /// Send SMS via local Indonesian provider
    async fn send_local_sms(&self, phone: &str, message: &str) -> AppResult<()> {
        #[derive(Serialize)]
        struct LocalSmsRequest {
            username: String,
            password: String,
            to: String,
            message: String,
            sender: String,
        }

        let request = LocalSmsRequest {
            username: self.sms_config.api_key.clone(),
            password: self.sms_config.api_secret.clone(),
            to: phone.to_string(),
            message: message.to_string(),
            sender: self.sms_config.sender_id.clone(),
        };

        let response = self.http_client
            .post(&self.sms_config.base_url)
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::External(format!("Local SMS request failed: {}", e)))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(AppError::External("Local SMS failed".to_string()))
        }
    }

    /// Send email via SMTP
    async fn send_email(&self, to_email: &str, subject: &str, content: &str) -> AppResult<()> {
        // In a real implementation, you'd use lettre or similar SMTP client
        // For now, we'll simulate email sending

        #[derive(Serialize)]
        struct EmailRequest {
            to: String,
            subject: String,
            html_content: String,
            from_email: String,
            from_name: String,
        }

        let request = EmailRequest {
            to: to_email.to_string(),
            subject: subject.to_string(),
            html_content: self.format_email_html(subject, content),
            from_email: self.email_config.from_email.clone(),
            from_name: self.email_config.from_name.clone(),
        };

        // Simulate email API call
        log::info!("Would send email to {} with subject: {}", to_email, subject);
        Ok(())
    }

    /// Format email content as HTML
    fn format_email_html(&self, subject: &str, content: &str) -> String {
        format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <meta charset="UTF-8">
                <title>{}</title>
                <style>
                    body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
                    .header {{ background-color: #d32f2f; color: white; padding: 20px; text-align: center; }}
                    .content {{ padding: 20px; }}
                    .footer {{ background-color: #f5f5f5; padding: 10px; text-align: center; font-size: 12px; }}
                    .emergency {{ background-color: #ffebee; border-left: 4px solid #d32f2f; padding: 10px; margin: 10px 0; }}
                </style>
            </head>
            <body>
                <div class="header">
                    <h1>Terra Siaga</h1>
                    <p>Sistem Manajemen Bencana</p>
                </div>
                <div class="content">
                    <h2>{}</h2>
                    <div class="emergency">
                        {}
                    </div>
                    <p>Tetap waspada dan ikuti instruksi dari petugas berwenang.</p>
                    <p>Download aplikasi Terra Siaga untuk informasi terkini.</p>
                </div>
                <div class="footer">
                    <p>Terra Siaga - Sistem Tanggap Darurat Indonesia</p>
                    <p>Jangan balas email ini. Hubungi 112 untuk keadaan darurat.</p>
                </div>
            </body>
            </html>
            "#,
            subject, subject, content
        )
    }

    /// Send push notification via Firebase
    async fn send_push_notification(&self, device_token: &str, title: &str, body: &str) -> AppResult<()> {
        #[derive(Serialize)]
        struct PushNotification {
            to: String,
            notification: PushData,
            data: PushDataPayload,
        }

        #[derive(Serialize)]
        struct PushData {
            title: String,
            body: String,
            icon: String,
            sound: String,
        }

        #[derive(Serialize)]
        struct PushDataPayload {
            notification_type: String,
            timestamp: String,
            action_url: String,
        }

        let notification = PushNotification {
            to: device_token.to_string(),
            notification: PushData {
                title: title.to_string(),
                body: body.to_string(),
                icon: "ic_emergency".to_string(),
                sound: "emergency_alert.wav".to_string(),
            },
            data: PushDataPayload {
                notification_type: "emergency_alert".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                action_url: "terrasiaga://disasters".to_string(),
            },
        };

        let response = self.http_client
            .post("https://fcm.googleapis.com/fcm/send")
            .header("Authorization", format!("key={}", self.push_config.firebase_server_key))
            .header("Content-Type", "application/json")
            .json(&notification)
            .send()
            .await
            .map_err(|e| AppError::External(format!("Push notification request failed: {}", e)))?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(AppError::External(format!("Push notification failed: {}", error_text)))
        }
    }

    /// Send WhatsApp message via Business API
    async fn send_whatsapp(&self, phone: &str, message: &str) -> AppResult<()> {
        #[derive(Serialize)]
        struct WhatsAppMessage {
            messaging_product: String,
            to: String,
            #[serde(rename = "type")]
            message_type: String,
            text: WhatsAppText,
        }

        #[derive(Serialize)]
        struct WhatsAppText {
            body: String,
        }

        let request = WhatsAppMessage {
            messaging_product: "whatsapp".to_string(),
            to: phone.to_string(),
            message_type: "text".to_string(),
            text: WhatsAppText {
                body: message.to_string(),
            },
        };

        let url = format!(
            "https://graph.facebook.com/v17.0/{}/messages",
            self.whatsapp_config.phone_number_id
        );

        let response = self.http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.whatsapp_config.access_token))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::External(format!("WhatsApp request failed: {}", e)))?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(AppError::External(format!("WhatsApp failed: {}", error_text)))
        }
    }

    /// Format emergency message for Indonesian context
    fn format_emergency_message(&self, disaster: &Disaster, message_type: &str) -> String {
        let location = format!("{}, {}",
                              disaster.address().city,
                              disaster.address().province);

        let severity = match disaster.severity() {
            crate::domain::value_objects::DisasterSeverity::Low => "RENDAH",
            crate::domain::value_objects::DisasterSeverity::Medium => "SEDANG",
            crate::domain::value_objects::DisasterSeverity::High => "TINGGI",
            crate::domain::value_objects::DisasterSeverity::Critical => "KRITIS",
        };

        match message_type {
            "emergency_alert" => format!(
                "🚨 PERINGATAN DARURAT 🚨\n\nJenis: {}\nTingkat: {}\nLokasi: {}\n\n{}\n\nHubungi 112 untuk bantuan darurat.\nUnduh Terra Siaga untuk info terkini.",
                disaster.disaster_type(),
                severity,
                location,
                disaster.description()
            ),
            "dispatch_notification" => format!(
                "🚑 TIM DARURAT DIKIRIM 🚑\n\nTim respons darurat telah dikirim ke:\nLokasi: {}\nJenis Bencana: {}\n\nHarap tetap tenang dan ikuti instruksi petugas.",
                location,
                disaster.disaster_type()
            ),
            _ => format!(
                "Terra Siaga: {}\nLokasi: {}\nInfo: {}",
                disaster.disaster_type(),
                location,
                disaster.description()
            ),
        }
    }
}

#[async_trait]
impl NotificationService for ExternalNotificationService {
    async fn send_notification(
        &self,
        user: &User,
        notification: &Notification,
        channel: &str,
    ) -> AppResult<()> {
        match channel {
            "sms" => {
                if let Some(phone) = user.phone_number() {
                    self.send_sms(phone.value(), notification.content()).await
                } else {
                    Err(AppError::Validation("User has no phone number for SMS".to_string()))
                }
            }
            "email" => {
                self.send_email(
                    user.email().value(),
                    notification.title(),
                    notification.content(),
                ).await
            }
            "push" => {
                // In real implementation, get device tokens from user preferences
                let mock_device_token = format!("device_token_{}", user.id().value());
                self.send_push_notification(
                    &mock_device_token,
                    notification.title(),
                    notification.content(),
                ).await
            }
            "whatsapp" => {
                if let Some(phone) = user.phone_number() {
                    self.send_whatsapp(phone.value(), notification.content()).await
                } else {
                    Err(AppError::Validation("User has no phone number for WhatsApp".to_string()))
                }
            }
            _ => Err(AppError::Validation(format!("Unsupported notification channel: {}", channel))),
        }
    }

    async fn send_emergency_alert(&self, disaster: &Disaster, radius_km: f64) -> AppResult<()> {
        // This would typically get users in radius and send to all
        // For now, we'll log the emergency alert
        let message = self.format_emergency_message(disaster, "emergency_alert");

        log::warn!(
            "EMERGENCY ALERT: {} at {} (radius: {}km)\nMessage: {}",
            disaster.disaster_type(),
            format!("{}, {}", disaster.address().city, disaster.address().province),
            radius_km,
            message
        );

        // In real implementation, query users in radius and send notifications
        Ok(())
    }

    async fn notify_emergency_dispatch(
        &self,
        disaster: &Disaster,
        response: &EmergencyResponseDispatchResponse,
    ) -> AppResult<()> {
        let message = self.format_emergency_message(disaster, "dispatch_notification");

        log::info!(
            "DISPATCH NOTIFICATION: {} team dispatched for {} (ETA: {})",
            response.team_type,
            disaster.disaster_type(),
            response.estimated_arrival.format("%H:%M")
        );

        // In real implementation, notify relevant stakeholders
        Ok(())
    }

    async fn send_welcome_notification(&self, user: &User) -> AppResult<()> {
        let welcome_message = format!(
            "Selamat datang di Terra Siaga, {}!\n\nAnda telah berhasil mendaftar sebagai {}. Aplikasi ini akan membantu Anda mendapatkan informasi darurat dan peringatan bencana.\n\nTetap waspada, tetap aman!",
            user.full_name(),
            match user.role() {
                crate::domain::value_objects::UserRole::Citizen => "Warga",
                crate::domain::value_objects::UserRole::Volunteer => "Relawan",
                crate::domain::value_objects::UserRole::Responder => "Petugas Respons",
                crate::domain::value_objects::UserRole::Admin => "Administrator",
                crate::domain::value_objects::UserRole::SuperAdmin => "Super Administrator",
            }
        );

        // Send welcome email
        self.send_email(
            user.email().value(),
            "Selamat Datang di Terra Siaga",
            &welcome_message,
        ).await?;

        // Send welcome SMS if phone number available
        if let Some(phone) = user.phone_number() {
            let sms_message = format!(
                "Selamat datang di Terra Siaga, {}! Akun Anda telah aktif. Unduh aplikasi mobile untuk notifikasi darurat. Info: https://terrasiaga.id",
                user.full_name()
            );
            let _ = self.send_sms(phone.value(), &sms_message).await;
        }

        Ok(())
    }
}
