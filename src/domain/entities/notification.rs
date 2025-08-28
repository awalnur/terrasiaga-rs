/// Notification domain entity
/// Represents notifications sent to users with delivery tracking and business rules

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::shared::{NotificationId, UserId, AppResult, AppError, AuditFields, Priority};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: NotificationId,
    pub recipient_id: UserId,
    pub title: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub priority: Priority,
    pub channels: Vec<NotificationChannel>,
    pub status: NotificationStatus,
    pub metadata: NotificationMetadata,
    pub delivery_info: DeliveryInfo,
    pub audit: AuditFields,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NotificationType {
    DisasterAlert,
    EmergencyResponse,
    StatusUpdate,
    SystemNotification,
    VerificationRequest,
    AssignmentNotification,
    ReminderNotification,
    WeatherAlert,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NotificationChannel {
    InApp,
    Email,
    SMS,
    WhatsApp,
    Push,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NotificationStatus {
    Pending,
    Sent,
    Delivered,
    Read,
    Failed,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationMetadata {
    pub source_entity_type: Option<String>, // "disaster", "user", "system"
    pub source_entity_id: Option<String>,
    pub action_url: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_actionable: bool,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryInfo {
    pub scheduled_at: Option<DateTime<Utc>>,
    pub sent_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub read_at: Option<DateTime<Utc>>,
    pub failed_at: Option<DateTime<Utc>>,
    pub failure_reason: Option<String>,
    pub retry_count: u8,
    pub delivery_attempts: Vec<DeliveryAttempt>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryAttempt {
    pub channel: NotificationChannel,
    pub attempted_at: DateTime<Utc>,
    pub success: bool,
    pub error_message: Option<String>,
    pub response_details: Option<String>,
}

impl Notification {
    /// Create a new notification
    pub fn new(
        recipient_id: UserId,
        title: String,
        message: String,
        notification_type: NotificationType,
        priority: Priority,
        channels: Vec<NotificationChannel>,
    ) -> AppResult<Self> {
        // Business rules validation
        if title.trim().is_empty() {
            return Err(AppError::Validation("Notification title cannot be empty".to_string()));
        }

        if message.trim().is_empty() {
            return Err(AppError::Validation("Notification message cannot be empty".to_string()));
        }

        if channels.is_empty() {
            return Err(AppError::Validation("At least one notification channel must be specified".to_string()));
        }

        let now = Utc::now();

        Ok(Self {
            id: NotificationId(uuid::Uuid::new_v4()),
            recipient_id,
            title: title.trim().to_string(),
            message: message.trim().to_string(),
            notification_type,
            priority,
            channels,
            status: NotificationStatus::Pending,
            metadata: NotificationMetadata {
                source_entity_type: None,
                source_entity_id: None,
                action_url: None,
                expires_at: None,
                is_actionable: false,
                tags: Vec::new(),
            },
            delivery_info: DeliveryInfo {
                scheduled_at: None,
                sent_at: None,
                delivered_at: None,
                read_at: None,
                failed_at: None,
                failure_reason: None,
                retry_count: 0,
                delivery_attempts: Vec::new(),
            },
            audit: AuditFields {

                created_at: now,
                updated_at: now,
                created_by: None,
                updated_by: None,
                version: 0,
            },
        })
    }

    /// Schedule notification for later delivery
    pub fn schedule(&mut self, scheduled_at: DateTime<Utc>) -> AppResult<()> {
        match self.status {
            NotificationStatus::Pending => {
                if scheduled_at <= Utc::now() {
                    return Err(AppError::Validation("Scheduled time must be in the future".to_string()));
                }

                self.delivery_info.scheduled_at = Some(scheduled_at);
                self.audit.updated_at = Utc::now();
                Ok(())
            }
            _ => Err(AppError::BusinessRuleViolation(
                "Only pending notifications can be scheduled".to_string()
            )),
        }
    }

    /// Mark notification as sent
    pub fn mark_as_sent(&mut self) -> AppResult<()> {
        match self.status {
            NotificationStatus::Pending => {
                self.status = NotificationStatus::Sent;
                self.delivery_info.sent_at = Some(Utc::now());
                self.audit.updated_at = Utc::now();
                Ok(())
            }
            _ => Err(AppError::BusinessRuleViolation(
                "Only pending notifications can be marked as sent".to_string()
            )),
        }
    }

    /// Mark notification as delivered
    pub fn mark_as_delivered(&mut self) -> AppResult<()> {
        match self.status {
            NotificationStatus::Sent => {
                self.status = NotificationStatus::Delivered;
                self.delivery_info.delivered_at = Some(Utc::now());
                self.audit.updated_at = Utc::now();
                Ok(())
            }
            _ => Err(AppError::BusinessRuleViolation(
                "Only sent notifications can be marked as delivered".to_string()
            )),
        }
    }

    /// Mark notification as read
    pub fn mark_as_read(&mut self) -> AppResult<()> {
        match self.status {
            NotificationStatus::Delivered | NotificationStatus::Sent => {
                self.status = NotificationStatus::Read;
                self.delivery_info.read_at = Some(Utc::now());
                self.audit.updated_at = Utc::now();
                Ok(())
            }
            NotificationStatus::Read => Ok(()), // Already read, no error
            _ => Err(AppError::BusinessRuleViolation(
                "Only delivered notifications can be marked as read".to_string()
            )),
        }
    }

    /// Mark notification as failed with reason
    pub fn mark_as_failed(&mut self, reason: String) -> AppResult<()> {
        self.status = NotificationStatus::Failed;
        self.delivery_info.failed_at = Some(Utc::now());
        self.delivery_info.failure_reason = Some(reason);
        self.audit.updated_at = Utc::now();
        Ok(())
    }

    /// Add delivery attempt
    pub fn add_delivery_attempt(&mut self, attempt: DeliveryAttempt) -> AppResult<()> {
        self.delivery_info.delivery_attempts.push(attempt);
        self.delivery_info.retry_count = self.delivery_info.delivery_attempts.len() as u8;
        self.audit.updated_at = Utc::now();
        Ok(())
    }

    /// Check if notification has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.metadata.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Check if notification should be retried
    pub fn should_retry(&self) -> bool {
        matches!(self.status, NotificationStatus::Failed)
            && self.delivery_info.retry_count < 3
            && !self.is_expired()
    }

    /// Get delivery success rate for this notification
    pub fn delivery_success_rate(&self) -> f64 {
        if self.delivery_info.delivery_attempts.is_empty() {
            return 0.0;
        }

        let successful_attempts = self.delivery_info.delivery_attempts
            .iter()
            .filter(|attempt| attempt.success)
            .count();

        successful_attempts as f64 / self.delivery_info.delivery_attempts.len() as f64
    }

    /// Set metadata for the notification
    pub fn set_metadata(&mut self, metadata: NotificationMetadata) -> AppResult<()> {
        self.metadata = metadata;
        self.audit.updated_at = Utc::now();
        Ok(())
    }
}
