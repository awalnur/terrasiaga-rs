/// Notification repository implementation
/// Provides data access for notification-related entities

use async_trait::async_trait;
use diesel::prelude::*;
use uuid::Uuid;
use chrono::{NaiveDateTime, Utc, DateTime};
use crate::shared::{AppResult, error::AppError, NotificationId, UserId};
use crate::infrastructure::database::DbPool;
use crate::domain::ports::repositories::NotificationRepository;
use crate::domain::entities::notification::{Notification, NotificationStatus, NotificationChannel, NotificationType, NotificationMetadata, DeliveryInfo, DeliveryAttempt};
use crate::shared::types::Priority;

use crate::infrastructure::database::schemas::notifications as db_notifications;
use crate::infrastructure::database::schemas::notifications::dsl::{notifications, id, user_id, channel, status, is_read, created_at, read_at};
use crate::shared::error::DatabaseError;

#[derive(Queryable, Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = db_notifications)]
struct NotificationModel {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub title: String,
    pub message: String,
    pub channel: String,
    pub status: Option<String>,
    pub is_read: Option<bool>,
    pub send_at: Option<NaiveDateTime>,
    pub read_at: Option<NaiveDateTime>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

pub struct PostgresNotificationRepository {
    pool: DbPool,
}

impl PostgresNotificationRepository {
    pub fn new(pool: DbPool) -> Self { Self { pool } }

    fn channel_to_str(ch: &NotificationChannel) -> &'static str {
        match ch {
            NotificationChannel::InApp => "push",
            NotificationChannel::Email => "email",
            NotificationChannel::SMS => "sms",
            NotificationChannel::WhatsApp => "whatsapp",
            NotificationChannel::Push => "push",
        }
    }

    fn str_to_channel(s: &str) -> NotificationChannel {
        match s.to_lowercase().as_str() {
            "email" => NotificationChannel::Email,
            "sms" => NotificationChannel::SMS,
            "whatsapp" => NotificationChannel::WhatsApp,
            "push" | "inapp" | "in_app" => NotificationChannel::Push,
            _ => NotificationChannel::Push,
        }
    }

    fn status_to_str(st: &NotificationStatus) -> String {
        match st {
            NotificationStatus::Pending => "pending",
            NotificationStatus::Sent => "sent",
            NotificationStatus::Delivered => "delivered",
            NotificationStatus::Read => "read",
            NotificationStatus::Failed => "failed",
            NotificationStatus::Expired => "expired",
        }.to_string()
    }

    fn str_to_status(s: Option<&str>) -> NotificationStatus {
        match s.unwrap_or("").to_lowercase().as_str() {
            "pending" => NotificationStatus::Pending,
            "sent" => NotificationStatus::Sent,
            "delivered" => NotificationStatus::Delivered,
            "read" => NotificationStatus::Read,
            "failed" => NotificationStatus::Failed,
            "expired" => NotificationStatus::Expired,
            _ => NotificationStatus::Pending,
        }
    }

    fn dt_opt_to_naive(dt: Option<DateTime<Utc>>) -> Option<NaiveDateTime> {
        dt.map(|d| d.naive_utc())
    }

    fn naive_opt_to_dt(dt: Option<NaiveDateTime>) -> DateTime<Utc> {
        // Fallback to now if not present
        match dt {
            Some(ndt) => DateTime::<Utc>::from_utc(ndt, Utc),
            None => Utc::now(),
        }
    }

    fn to_models(entity: &Notification) -> Vec<NotificationModel> {
        let _created_at = entity.audit.created_at.naive_utc();
        let updated_at = entity.audit.updated_at.naive_utc();
        if entity.channels.is_empty() {
            return vec![NotificationModel {
                id: entity.id.0,
                user_id: Some(entity.recipient_id.0),
                title: entity.title.clone(),
                message: entity.message.clone(),
                channel: "push".to_string(),
                status: Some(Self::status_to_str(&entity.status)),
                is_read: Some(matches!(entity.status, NotificationStatus::Read)),
                send_at: Self::dt_opt_to_naive(entity.delivery_info.sent_at),
                read_at: Self::dt_opt_to_naive(entity.delivery_info.read_at),
                created_at: Some(_created_at),
                updated_at: Some(updated_at),
            }];
        }
        entity.channels.iter().map(|ch| NotificationModel {
            id: Uuid::new_v4(), // create per-channel row id
            user_id: Some(entity.recipient_id.0),
            title: entity.title.clone(),
            message: entity.message.clone(),
            channel: Self::channel_to_str(ch).to_string(),
            status: Some(Self::status_to_str(&entity.status)),
            is_read: Some(matches!(entity.status, NotificationStatus::Read)),
            send_at: Self::dt_opt_to_naive(entity.delivery_info.sent_at),
            read_at: Self::dt_opt_to_naive(entity.delivery_info.read_at),
            created_at: Some(_created_at),
            updated_at: Some(updated_at),
        }).collect()
    }

    fn to_domain(model: &NotificationModel) -> Notification {
        Notification {
            id: NotificationId(model.id),
            recipient_id: UserId(model.user_id.unwrap_or(Uuid::nil())),
            title: model.title.clone(),
            message: model.message.clone(),
            notification_type: NotificationType::SystemNotification,
            priority: Priority::Normal,
            channels: vec![Self::str_to_channel(&model.channel)],
            status: Self::str_to_status(model.status.as_deref()),
            metadata: NotificationMetadata {
                source_entity_type: None,
                source_entity_id: None,
                action_url: None,
                expires_at: None,
                is_actionable: false,
                tags: vec![],
            },
            delivery_info: DeliveryInfo {
                scheduled_at: None,
                sent_at: model.send_at.map(|t| DateTime::<Utc>::from_utc(t, Utc)),
                delivered_at: None,
                read_at: model.read_at.map(|t| DateTime::<Utc>::from_utc(t, Utc)),
                failed_at: None,
                failure_reason: None,
                retry_count: 0,
                delivery_attempts: vec![],
            },
            audit: crate::shared::types::AuditFields {
                created_at: Self::naive_opt_to_dt(model.created_at),
                updated_at: Self::naive_opt_to_dt(model.updated_at),
                created_by: None,
                updated_by: None,
                version: 1,
            },
        }
    }
}

#[async_trait]
impl NotificationRepository for PostgresNotificationRepository {
    async fn find_by_id(&self, nid: &NotificationId) -> AppResult<Option<Notification>> {
        let mut conn = self.pool.get().map_err(|e|AppError::Database(DatabaseError::ConnectionPool(e)))?;
        let res: Option<NotificationModel> = notifications
            .filter(crate::infrastructure::database::schemas::notifications::dsl::id.eq(nid.0))
            .first::<NotificationModel>(&mut conn)
            .optional()
            .map_err(|e|AppError::Database(DatabaseError::Diesel(e)))?;
        Ok(res.map(|m| Self::to_domain(&m)))
    }

    async fn save(&self, entity: &Notification) -> AppResult<Notification> {
        let mut conn = self.pool.get().map_err(|e|AppError::Database(DatabaseError::ConnectionPool(e)))?;
        let rows = Self::to_models(entity);
        for row in rows {
            diesel::insert_into(notifications)
                .values(&row)
                .execute(&mut conn)
                .map_err(|e|AppError::Database(DatabaseError::Diesel(e)))?;
        }
        Ok(entity.clone())
    }

    async fn update(&self, entity: &Notification) -> AppResult<Notification> {
        // For simplicity, update by id for the first channel only
        let mut conn = self.pool.get().map_err(|e|AppError::Database(DatabaseError::ConnectionPool(e)))?;
        let model = Self::to_models(entity).into_iter().next().unwrap();
        diesel::update(notifications.filter(crate::infrastructure::database::schemas::notifications::dsl::id.eq(model.id)))
            .set(&model)
            .execute(&mut conn)
            .map_err(|e|AppError::Database(DatabaseError::Diesel(e)))?;
        Ok(entity.clone())
    }

    async fn delete(&self, nid: &NotificationId) -> AppResult<bool> {
        let mut conn = self.pool.get().map_err(|e|AppError::Database(DatabaseError::ConnectionPool(e)))?;
        let count = diesel::delete(notifications.filter(id.eq(nid.0)))
            .execute(&mut conn)
            .map_err(|e|AppError::Database(DatabaseError::Diesel(e)))?;
        Ok(count > 0)
    }

    async fn find_all(&self) -> AppResult<Vec<Notification>> {
        let mut conn = self.pool.get().map_err(|e|AppError::Database(DatabaseError::ConnectionPool(e)))?;
        let rows: Vec<NotificationModel> = notifications
            .order(created_at.desc())
            .load(&mut conn)
            .map_err(|e|AppError::Database(DatabaseError::Diesel(e)))?;
        Ok(rows.iter().map(Self::to_domain).collect())
    }

    async fn find_by_recipient(&self, recipient_id: &UserId) -> AppResult<Vec<Notification>> {
        let mut conn = self.pool.get().map_err(|e|AppError::Database(DatabaseError::ConnectionPool(e)))?;
        let rows: Vec<NotificationModel> = notifications
            .filter(user_id.eq(recipient_id.0))
            .order(created_at.desc())
            .load(&mut conn)
            .map_err(|e|AppError::Database(DatabaseError::Diesel(e)))?;
        Ok(rows.iter().map(Self::to_domain).collect())
    }

    async fn find_unread(&self, recipient_id_val: &UserId) -> AppResult<Vec<Notification>> {
        let mut conn = self.pool.get().map_err(|e|AppError::Database(DatabaseError::ConnectionPool(e)))?;
        let rows: Vec<NotificationModel> = notifications
            .filter(user_id.eq(recipient_id_val.0))
            .filter(is_read.eq(false).or(is_read.is_null()))
            .order(created_at.desc())
            .load(&mut conn)
            .map_err(|e|AppError::Database(DatabaseError::Diesel(e)))?;
        Ok(rows.iter().map(Self::to_domain).collect())
    }

    async fn mark_as_read(&self, id_val: &NotificationId) -> AppResult<bool> {
        let mut conn = self.pool.get().map_err(|e|AppError::Database(DatabaseError::ConnectionPool(e)))?;
        let now = Utc::now().naive_utc();
        let count = diesel::update(notifications.filter(id.eq(id_val.0)))
            .set((is_read.eq(true), read_at.eq(Some(now)), status.eq(Some("read".to_string()))))
            .execute(&mut conn)
            .map_err(|e|AppError::Database(DatabaseError::Diesel(e)))?;
        Ok(count > 0)
    }

    async fn mark_all_as_read(&self, recipient_id_val: &UserId) -> AppResult<u64> {
        let mut conn = self.pool.get().map_err(|e|AppError::Database(DatabaseError::ConnectionPool(e)))?;
        let now = Utc::now().naive_utc();
        let count = diesel::update(
            notifications.filter(user_id.eq(recipient_id_val.0).and(is_read.eq(false).or(is_read.is_null())))
        )
        .set((is_read.eq(true), read_at.eq(Some(now)), status.eq(Some("read".to_string()))))
        .execute(&mut conn)
        .map_err(|e|AppError::Database(DatabaseError::Diesel(e)))? as u64;
        Ok(count)
    }

    async fn delete_old_notifications(&self, days: u32) -> AppResult<u64> {
        let mut conn = self.pool.get().map_err(|e|AppError::Database(DatabaseError::ConnectionPool(e)))?;
        let cutoff = (Utc::now() - chrono::Duration::days(days as i64)).naive_utc();
        let count = diesel::delete(
            notifications.filter(created_at.lt(cutoff))
        ).execute(&mut conn)
        .map_err(|e|AppError::Database(DatabaseError::Diesel(e)))? as u64;
        Ok(count)
    }

    async fn count_unread(&self, recipient_id_val: &UserId) -> AppResult<u64> {
        let mut conn = self.pool.get().map_err(|e|AppError::Database(DatabaseError::ConnectionPool(e)))?;
        let cnt: i64 = notifications
            .filter(user_id.eq(recipient_id_val.0))
            .filter(is_read.eq(false).or(is_read.is_null()))
            .count()
            .get_result(&mut conn)
            .map_err(|e|AppError::Database(DatabaseError::Diesel(e)))?;
        Ok(cnt as u64)
    }

    async fn find_by_status(&self, status_val: NotificationStatus) -> AppResult<Vec<Notification>> {
        let mut conn = self.pool.get().map_err(|e|AppError::Database(DatabaseError::ConnectionPool(e)))?;
        let rows: Vec<NotificationModel> = notifications
            .filter(status.eq(Some(Self::status_to_str(&status_val))))
            .order(created_at.desc())
            .load(&mut conn)
            .map_err(|e|AppError::Database(DatabaseError::Diesel(e)))?;
        Ok(rows.iter().map(Self::to_domain).collect())
    }

    async fn save_notification(&self, notification: &Notification) -> AppResult<Notification> {
        self.save(notification).await
    }

    async fn find_by_user(&self, user_id_val: &UserId, limit_opt: Option<u32>) -> AppResult<Vec<Notification>> {
        let mut conn = self.pool.get().map_err(|e|AppError::Database(DatabaseError::ConnectionPool(e)))?;
        let mut query = notifications.into_boxed();
        query = query.filter(user_id.eq(user_id_val.0)).order(created_at.desc());
        let rows: Vec<NotificationModel> = match limit_opt {
            Some(l) => query.limit(l as i64).load(&mut conn).map_err(|e|AppError::Database(DatabaseError::Diesel(e)))?,
            None => query.load(&mut conn).map_err(|e|AppError::Database(DatabaseError::Diesel(e)))?,
        };
        Ok(rows.iter().map(Self::to_domain).collect())
    }

    async fn find_unread_by_recipient(&self, recipient_id_val: UserId) -> AppResult<Vec<Notification>> {
        self.find_unread(&recipient_id_val).await
    }

    async fn find_by_channel(&self, channel_val: NotificationChannel) -> AppResult<Vec<Notification>> {
        let mut conn = self.pool.get().map_err(|e|AppError::Database(DatabaseError::ConnectionPool(e)))?;
        let rows: Vec<NotificationModel> = notifications
            .filter(channel.eq(Self::channel_to_str(&channel_val)))
            .order(created_at.desc())
            .load(&mut conn)
            .map_err(|e|AppError::Database(DatabaseError::Diesel(e)))?;
        Ok(rows.iter().map(Self::to_domain).collect())
    }
}
