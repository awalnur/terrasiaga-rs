/// Notification use cases
/// Handles emergency alerts, mass notifications, and communication management

use async_trait::async_trait;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::application::use_cases::{UseCase, ValidatedUseCase};
use crate::domain::entities::{Notification, Disaster};
use crate::domain::value_objects::*;
use crate::domain::ports::repositories::{NotificationRepository, UserRepository, DisasterRepository};
use crate::domain::ports::services::{NotificationService, GeoService};
use crate::domain::events::{NotificationSentEvent, MassNotificationTriggeredEvent, EventPublisher};
use crate::shared::{AppResult, AppError};

/// Request to send emergency alert to users in affected area
#[derive(Debug, Clone)]
pub struct SendEmergencyAlertRequest {
    pub disaster_id: DisasterId,
    pub alert_type: String, // "evacuation", "shelter", "warning", "all_clear"
    pub severity: DisasterSeverity,
    pub affected_area_center: Coordinates,
    pub radius_km: f64,
    pub message: String,
    pub channels: Vec<String>, // "sms", "email", "push", "whatsapp"
    pub sent_by: UserId,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Response after sending emergency alert
#[derive(Debug, Clone)]
pub struct EmergencyAlertResponse {
    pub alert_id: Uuid,
    pub disaster_id: DisasterId,
    pub recipients_targeted: u32,
    pub notifications_sent: u32,
    pub failed_deliveries: u32,
    pub channels_used: Vec<String>,
    pub sent_at: DateTime<Utc>,
    pub estimated_delivery_time: u32, // seconds
}

/// Use case for sending emergency alerts
pub struct SendEmergencyAlertUseCase {
    notification_repository: Arc<dyn NotificationRepository>,
    user_repository: Arc<dyn UserRepository>,
    disaster_repository: Arc<dyn DisasterRepository>,
    notification_service: Arc<dyn NotificationService>,
    geo_service: Arc<dyn GeoService>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl SendEmergencyAlertUseCase {
    pub fn new(
        notification_repository: Arc<dyn NotificationRepository>,
        user_repository: Arc<dyn UserRepository>,
        disaster_repository: Arc<dyn DisasterRepository>,
        notification_service: Arc<dyn NotificationService>,
        geo_service: Arc<dyn GeoService>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            notification_repository,
            user_repository,
            disaster_repository,
            notification_service,
            geo_service,
            event_publisher,
        }
    }

    /// Get message template based on alert type and severity
    fn get_alert_template(&self, alert_type: &str, severity: &DisasterSeverity) -> String {
        let urgency = match severity {
            DisasterSeverity::Low => "PERHATIAN",
            DisasterSeverity::Medium => "PERINGATAN",
            DisasterSeverity::High => "BAHAYA",
            DisasterSeverity::Critical => "DARURAT",
        };

        match alert_type {
            "evacuation" => format!("[{}] EVAKUASI SEGERA! Tinggalkan area dan menuju ke tempat aman terdekat. Ikuti instruksi petugas.", urgency),
            "shelter" => format!("[{}] Cari tempat berlindung yang aman. Tetap di dalam ruangan dan hindari area berbahaya.", urgency),
            "warning" => format!("[{}] Waspada! Pantau perkembangan situasi dan bersiap untuk tindakan darurat jika diperlukan.", urgency),
            "all_clear" => format!("[{}] Situasi aman. Bahaya telah berlalu. Tetap waspada dan ikuti arahan petugas.", urgency),
            _ => format!("[{}] Peringatan darurat. Segera cek aplikasi Terra Siaga untuk informasi lengkap.", urgency),
        }
    }

    /// Estimate delivery time based on channels and recipient count
    fn estimate_delivery_time(&self, channels: &[String], recipient_count: u32) -> u32 {
        let base_time = match channels.get(0).map(|s| s.as_str()) {
            Some("sms") => 30,     // SMS usually fastest
            Some("push") => 10,    // Push notifications very fast
            Some("email") => 60,   // Email can be slower
            Some("whatsapp") => 45, // WhatsApp API delivery
            _ => 30,
        };

        // Add time based on volume
        let volume_factor = (recipient_count / 1000).max(1) as u32;
        base_time + (volume_factor * 10)
    }
}

#[async_trait]
impl ValidatedUseCase<SendEmergencyAlertRequest, EmergencyAlertResponse> for SendEmergencyAlertUseCase {
    async fn validate(&self, request: &SendEmergencyAlertRequest) -> AppResult<()> {
        // Validate disaster exists
        self.disaster_repository
            .find_by_id(&request.disaster_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Disaster not found".to_string()))?;

        // Validate sender permissions
        let sender = self.user_repository
            .find_by_id(&request.sent_by)
            .await?
            .ok_or_else(|| AppError::NotFound("Sender not found".to_string()))?;

        if !sender.role().can_perform("send_emergency_alerts") {
            return Err(AppError::Forbidden("Insufficient permissions to send emergency alerts".to_string()));
        }

        // Validate alert type
        let valid_alert_types = ["evacuation", "shelter", "warning", "all_clear"];
        if !valid_alert_types.contains(&request.alert_type.as_str()) {
            return Err(AppError::Validation(format!(
                "Invalid alert type. Must be one of: {}",
                valid_alert_types.join(", ")
            )));
        }

        // Validate radius
        if request.radius_km <= 0.0 || request.radius_km > 100.0 {
            return Err(AppError::Validation("Radius must be between 0.1 and 100 km".to_string()));
        }

        // Validate channels
        let valid_channels = ["sms", "email", "push", "whatsapp"];
        for channel in &request.channels {
            if !valid_channels.contains(&channel.as_str()) {
                return Err(AppError::Validation(format!(
                    "Invalid channel '{}'. Must be one of: {}",
                    channel,
                    valid_channels.join(", ")
                )));
            }
        }

        if request.channels.is_empty() {
            return Err(AppError::Validation("At least one notification channel must be specified".to_string()));
        }

        // Validate message
        if request.message.trim().is_empty() {
            return Err(AppError::Validation("Message cannot be empty".to_string()));
        }

        if request.message.len() > 1000 {
            return Err(AppError::Validation("Message cannot exceed 1000 characters".to_string()));
        }

        Ok(())
    }
}

#[async_trait]
impl UseCase<SendEmergencyAlertRequest, EmergencyAlertResponse> for SendEmergencyAlertUseCase {
    async fn execute(&self, request: SendEmergencyAlertRequest) -> AppResult<EmergencyAlertResponse> {
        let alert_id = Uuid::new_v4();
        let sent_at = Utc::now();

        // Get users in affected area
        let affected_users = self.user_repository
            .find_users_in_radius(&request.affected_area_center, request.radius_km)
            .await?;

        let recipients_targeted = affected_users.len() as u32;

        // Prepare notification message
        let template_message = self.get_alert_template(&request.alert_type, &request.severity);
        let full_message = format!("{}\n\n{}", template_message, request.message);

        // Send notifications through all specified channels
        let mut notifications_sent = 0u32;
        let mut failed_deliveries = 0u32;

        for user in &affected_users {
            for channel in &request.channels {
                // Create notification record
                let notification_id = NotificationId::new();
                let notification = Notification::new(
                    notification_id.clone(),
                    user.id(),
                    "emergency_alert".to_string(),
                    request.alert_type.clone(),
                    full_message.clone(),
                    channel.clone(),
                    request.severity.priority_level(),
                    sent_at,
                    request.expires_at,
                )?;

                // Save notification
                match self.notification_repository.save(&notification).await {
                    Ok(_) => {
                        // Attempt to send via external service
                        match self.notification_service
                            .send_notification(&user, &notification, channel)
                            .await 
                        {
                            Ok(_) => {
                                notifications_sent += 1;

                                // Publish event
                                let event = NotificationSentEvent {
                                    event_id: Uuid::new_v4(),
                                    notification_id: notification_id.clone(),
                                    recipient_id: user.id(),
                                    notification_type: "emergency_alert".to_string(),
                                    channel: channel.clone(),
                                    content: full_message.clone(),
                                    occurred_at: sent_at,
                                    version: 1,
                                };
                                let _ = self.event_publisher.publish(&event).await;
                            }
                            Err(_) => {
                                failed_deliveries += 1;
                            }
                        }
                    }
                    Err(_) => {
                        failed_deliveries += 1;
                    }
                }
            }
        }

        // Publish mass notification event
        let mass_event = MassNotificationTriggeredEvent {
            event_id: Uuid::new_v4(),
            disaster_id: request.disaster_id.clone(),
            triggered_by: request.sent_by.clone(),
            affected_area_radius_km: request.radius_km,
            notification_type: request.alert_type.clone(),
            estimated_recipients: recipients_targeted,
            occurred_at: sent_at,
            version: 1,
        };

        self.event_publisher.publish(&mass_event).await?;

        let estimated_delivery_time = self.estimate_delivery_time(&request.channels, recipients_targeted);

        Ok(EmergencyAlertResponse {
            alert_id,
            disaster_id: request.disaster_id,
            recipients_targeted,
            notifications_sent,
            failed_deliveries,
            channels_used: request.channels,
            sent_at,
            estimated_delivery_time,
        })
    }
}

/// Request to send custom notification to specific users
#[derive(Debug, Clone)]
pub struct SendCustomNotificationRequest {
    pub recipient_ids: Vec<UserId>,
    pub notification_type: String,
    pub title: String,
    pub message: String,
    pub channels: Vec<String>,
    pub priority: u8, // 1-5 scale
    pub sent_by: UserId,
    pub expires_at: Option<DateTime<Utc>>,
    pub action_url: Option<String>, // Deep link for mobile apps
}

/// Use case for sending custom notifications
pub struct SendCustomNotificationUseCase {
    notification_repository: Arc<dyn NotificationRepository>,
    user_repository: Arc<dyn UserRepository>,
    notification_service: Arc<dyn NotificationService>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl SendCustomNotificationUseCase {
    pub fn new(
        notification_repository: Arc<dyn NotificationRepository>,
        user_repository: Arc<dyn UserRepository>,
        notification_service: Arc<dyn NotificationService>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            notification_repository,
            user_repository,
            notification_service,
            event_publisher,
        }
    }
}

#[async_trait]
impl ValidatedUseCase<SendCustomNotificationRequest, Vec<NotificationId>> for SendCustomNotificationUseCase {
    async fn validate(&self, request: &SendCustomNotificationRequest) -> AppResult<()> {
        // Validate sender permissions
        let sender = self.user_repository
            .find_by_id(&request.sent_by)
            .await?
            .ok_or_else(|| AppError::NotFound("Sender not found".to_string()))?;

        if !sender.role().can_perform("send_notifications") {
            return Err(AppError::Forbidden("Insufficient permissions to send notifications".to_string()));
        }

        // Validate recipients
        if request.recipient_ids.is_empty() {
            return Err(AppError::Validation("At least one recipient must be specified".to_string()));
        }

        if request.recipient_ids.len() > 10000 {
            return Err(AppError::Validation("Cannot send to more than 10,000 recipients at once".to_string()));
        }

        // Validate message content
        if request.title.trim().is_empty() {
            return Err(AppError::Validation("Title cannot be empty".to_string()));
        }

        if request.message.trim().is_empty() {
            return Err(AppError::Validation("Message cannot be empty".to_string()));
        }

        if request.title.len() > 100 {
            return Err(AppError::Validation("Title cannot exceed 100 characters".to_string()));
        }

        if request.message.len() > 2000 {
            return Err(AppError::Validation("Message cannot exceed 2000 characters".to_string()));
        }

        // Validate priority
        if request.priority == 0 || request.priority > 5 {
            return Err(AppError::Validation("Priority must be between 1 and 5".to_string()));
        }

        Ok(())
    }
}

#[async_trait]
impl UseCase<SendCustomNotificationRequest, Vec<NotificationId>> for SendCustomNotificationUseCase {
    async fn execute(&self, request: SendCustomNotificationRequest) -> AppResult<Vec<NotificationId>> {
        let mut sent_notifications = Vec::new();
        let sent_at = Utc::now();

        for recipient_id in &request.recipient_ids {
            // Get recipient
            let user = match self.user_repository.find_by_id(recipient_id).await? {
                Some(user) => user,
                None => continue, // Skip if user not found
            };

            for channel in &request.channels {
                // Create notification
                let notification_id = NotificationId::new();
                let notification = Notification::new(
                    notification_id.clone(),
                    user.id(),
                    request.notification_type.clone(),
                    request.title.clone(),
                    request.message.clone(),
                    channel.clone(),
                    request.priority,
                    sent_at,
                    request.expires_at,
                )?;

                // Save and send notification
                if let Ok(_) = self.notification_repository.save(&notification).await {
                    if let Ok(_) = self.notification_service
                        .send_notification(&user, &notification, channel)
                        .await 
                    {
                        sent_notifications.push(notification_id.clone());

                        // Publish event
                        let event = NotificationSentEvent {
                            event_id: Uuid::new_v4(),
                            notification_id: notification_id.clone(),
                            recipient_id: user.id(),
                            notification_type: request.notification_type.clone(),
                            channel: channel.clone(),
                            content: request.message.clone(),
                            occurred_at: sent_at,
                            version: 1,
                        };
                        let _ = self.event_publisher.publish(&event).await;
                    }
                }
            }
        }

        Ok(sent_notifications)
    }
}
