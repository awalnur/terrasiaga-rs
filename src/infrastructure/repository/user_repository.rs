/// User repository implementation with caching
/// Provides data access layer for user entities with Redis caching support

use async_trait::async_trait;
use diesel::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entities::User;
use crate::domain::value_objects::*;
use crate::domain::ports::repositories::UserRepository;
use crate::infrastructure::cache::{CacheService, CacheKeys};
use crate::shared::{AppResult, AppError};
use std::time::Duration;
use crate::infrastructure::database::DbPool;
use chrono::Utc;
use crate::infrastructure::database::schemas::users;
use crate::shared::error::DatabaseError;

/// Database model for users
#[derive(Queryable, Insertable, AsChangeset, Debug)]
#[diesel(table_name = users)]
pub struct UserModel {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub email: String,
    pub phone: Option<String>,
    pub role_id: Option<i32>,
    pub full_name: Option<String>,
    pub address: Option<String>,
    pub profile_photo_url: Option<String>,
    pub bio: Option<String>,
    pub date_of_birth: Option<chrono::NaiveDate>,
    pub gender: Option<String>,
    pub is_verified: Option<bool>,
    pub is_active: Option<bool>,
    pub last_login: Option<chrono::NaiveDateTime>,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

impl UserModel {
    /// Convert database model to domain entity
    pub fn to_domain(&self) -> AppResult<User> {
        let email = Email::new(self.email.clone())?;
        let username = Username::new(self.username.clone())?;
        let phone_number = match &self.phone {
            Some(phone) => Some(PhoneNumber::new(phone.clone())?),
            None => None,
        };

        // Basic role mapping fallback (improve by joining roles table if needed)
        let role = UserRole::Citizen;

        // Derive status from is_active/is_verified
        let status = match (self.is_active.unwrap_or(true), self.is_verified.unwrap_or(false)) {
            (true, true) => UserStatus::Active,
            (true, false) => UserStatus::Pending,
            (false, _) => UserStatus::Inactive,
        };

        let now = Utc::now();
        let created_at = self.created_at.map(|t| chrono::DateTime::<Utc>::from_utc(t, Utc)).unwrap_or(now);
        let updated_at = self.updated_at.map(|t| chrono::DateTime::<Utc>::from_utc(t, Utc)).unwrap_or(now);
        let last_login = self.last_login.map(|t| chrono::DateTime::<Utc>::from_utc(t, Utc));

        let mut user = User::new(
            email,
            username,
            self.full_name.clone().unwrap_or_default(),
            self.password_hash.clone(),
            role,
        )?;
        user.id = UserId::from_uuid(self.id);
        user.phone_number = phone_number;
        user.status = status;
        user.address = self.address.clone();
        user.created_at = created_at;
        user.updated_at = updated_at;
        user.last_login = last_login;
        user.version = 1;

        // Set profile fields
        user.profile.bio = self.bio.clone();
        user.profile.avatar_url = self.profile_photo_url.clone();

        Ok(user)
    }

    /// Convert domain entity to database model
    pub fn from_domain(user: &User) -> Self {
        let (is_active, is_verified) = match user.status() {
            UserStatus::Active => (Some(true), Some(true)),
            UserStatus::Pending => (Some(true), Some(false)),
            _ => (Some(false), Some(false)),
        };

        Self {
            id: user.id().value(),
            username: user.username().value().to_string(),
            password_hash: user.password_hash().to_string(),
            email: user.email().value().to_string(),
            phone: user.phone_number().map(|p| p.value().to_string()),
            role_id: None,
            full_name: Some(user.full_name().to_string()),
            address: user.address.clone(),
            profile_photo_url: user.profile.avatar_url.clone(),
            bio: user.profile.bio.clone(),
            date_of_birth: None,
            gender: None,
            is_verified,
            is_active,
            last_login: user.last_login.map(|t| t.naive_utc()),
            created_at: Some(user.created_at.naive_utc()),
            updated_at: Some(user.updated_at.naive_utc()),
        }
    }
}

/// PostgreSQL implementation of UserRepository with caching
pub struct PostgresUserRepository {
    pool: DbPool,
    cache: Arc<dyn CacheService>,
}

impl PostgresUserRepository {
    pub fn new(
        pool: DbPool,
        cache: Arc<dyn CacheService>,
    ) -> Self {
        Self { pool, cache }
    }

    /// Cache TTL for user data
    const USER_CACHE_TTL: Duration = Duration::from_secs(300); // 5 minutes

    /// Invalidate user cache
    async fn invalidate_user_cache(&self, user_id: &UserId, email: &Email) -> AppResult<()> {
        let user_key = CacheKeys::user(&user_id.value());
        let email_key = CacheKeys::user_by_email(email.value());
        
        let _ = self.cache.delete(&user_key).await;
        let _ = self.cache.delete(&email_key).await;
        
        Ok(())
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn find_by_id(&self, uid: &UserId) -> AppResult<Option<User>> {
        let cache_key = CacheKeys::user(&uid.value());

        // Try cache first (typed via JSON string)
        if let Some(json_str) = self.cache.get_string(&cache_key).await? {
            if let Ok(user) = serde_json::from_str::<User>(&json_str) {
                return Ok(Some(user));
            }
        }

        // Query database
        use crate::infrastructure::database::schemas::users::dsl::*;

        let mut conn = self.pool.get()
            .map_err(|e|AppError::Database(DatabaseError::ConnectionPool(e)))?;

        let user_model: Option<UserModel> = users
            .filter(id.eq(uid.value()))
            .first(&mut conn)
            .optional()
            .map_err(|e|AppError::Database(DatabaseError::Diesel(e)))?;

        match user_model {
            Some(model) => {
                let user = model.to_domain()?;
                
                // Cache the result
                let _ = self.cache.set_string(&cache_key, serde_json::to_string(&user).unwrap_or_default(), Some(Self::USER_CACHE_TTL)).await;

                Ok(Some(user))
            }
            None => Ok(None),
        }
    }

    async fn find_by_email(&self, email_val: &Email) -> AppResult<Option<User>> {

        let cache_key = CacheKeys::user_by_email(email_val.value());
        
        // Try cache first
        if let Some(json_str) = self.cache.get_string(&cache_key).await? {
            if let Ok(user) = serde_json::from_str::<User>(&json_str) {
                return Ok(Some(user));
            }
        }

        // Query database
        use crate::infrastructure::database::schemas::users::dsl::*;

        let mut conn = self.pool.get()
            .map_err(|e|AppError::Database(DatabaseError::ConnectionPool(e)))?;

        let user_model: Option<UserModel> = users
            .filter(email.eq(email_val.value()))
            .first(&mut conn)
            .optional()
            .map_err(|e|AppError::Database(DatabaseError::Diesel(e)))?;

        match user_model {
            Some(model) => {
                let user = model.to_domain()?;
                
                // Cache the result
                let _ = self.cache.set_string(&cache_key, serde_json::to_string(&user).unwrap_or_default(), Some(Self::USER_CACHE_TTL)).await;

                Ok(Some(user))
            }
            None => Ok(None),
        }
    }

    async fn save(&self, user: &User) -> AppResult<User> {
        use crate::infrastructure::database::schemas::users::dsl::*;

        let mut conn = self.pool.get()
            .map_err(|e|AppError::Database(DatabaseError::ConnectionPool(e)))?;

        let user_model = UserModel::from_domain(user);

        // Upsert by id
        let saved_model = diesel::insert_into(users)
            .values(&user_model)
            .on_conflict(id)
            .do_update()
            .set((
                username.eq(&user_model.username),
                password_hash.eq(&user_model.password_hash),
                email.eq(&user_model.email),
                phone.eq(&user_model.phone),
                full_name.eq(&user_model.full_name),
                address.eq(&user_model.address),
                profile_photo_url.eq(&user_model.profile_photo_url),
                bio.eq(&user_model.bio),
                is_verified.eq(&user_model.is_verified),
                is_active.eq(&user_model.is_active),
                last_login.eq(&user_model.last_login),
                updated_at.eq(Some(chrono::Utc::now().naive_utc())),
            ))
            .get_result::<UserModel>(&mut conn)
            .map_err(|e|AppError::Database(DatabaseError::Diesel(e)))?;

        let saved_user = saved_model.to_domain()?;
        
        // Invalidate cache
        self.invalidate_user_cache(&saved_user.id(), &saved_user.email()).await?;
        
        Ok(saved_user)
    }

    async fn update(&self, user: &User) -> AppResult<User> {
        use crate::infrastructure::database::schemas::users::dsl::*;
        let mut conn = self.pool.get()
            .map_err(|e|AppError::Database(DatabaseError::ConnectionPool(e)))?;
        let user_model = UserModel::from_domain(user);
        let updated_model = diesel::update(users.filter(id.eq(user_model.id)))
            .set((
                username.eq(&user_model.username),
                password_hash.eq(&user_model.password_hash),
                email.eq(&user_model.email),
                phone.eq(&user_model.phone),
                full_name.eq(&user_model.full_name),
                address.eq(&user_model.address),
                profile_photo_url.eq(&user_model.profile_photo_url),
                bio.eq(&user_model.bio),
                is_verified.eq(&user_model.is_verified),
                is_active.eq(&user_model.is_active),
                last_login.eq(&user_model.last_login),
                updated_at.eq(Some(chrono::Utc::now().naive_utc())),
            ))
            .get_result::<UserModel>(&mut conn)
            .map_err(|e|AppError::Database(DatabaseError::Diesel(e)))?;
        let updated = updated_model.to_domain()?;
        self.invalidate_user_cache(updated.id(), updated.email()).await?;
        Ok(updated)
    }

    async fn delete(&self, uid: &UserId) -> AppResult<bool> {
        use crate::infrastructure::database::schemas::users::dsl::*;

        // Get user first for cache invalidation
        let user = match self.find_by_id(uid).await? {
            Some(user) => user,
            None => return Ok(false),
        };

        let mut conn = self.pool.get()
            .map_err(|e|AppError::Database(DatabaseError::ConnectionPool(e)))?;

        let deleted_count = diesel::delete(users.filter(id.eq(uid.value())))
            .execute(&mut conn)
            .map_err(|e|AppError::Database(DatabaseError::Diesel(e)))?;

        if deleted_count > 0 {
            // Invalidate cache
            self.invalidate_user_cache(&user.id(), &user.email()).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn find_all(&self) -> AppResult<Vec<User>> {
        use crate::infrastructure::database::schemas::users::dsl::*;
        let mut conn = self.pool.get()
            .map_err(|e|AppError::Database(DatabaseError::ConnectionPool(e)))?;
        let user_models: Vec<UserModel> = users
            .load(&mut conn)
            .map_err(|e|AppError::Database(DatabaseError::Diesel(e)))?;
        let mut result = Vec::new();
        for m in user_models { if let Ok(u) = m.to_domain() { result.push(u); } }
        Ok(result)
    }

    async fn find_users_in_radius(
        &self,
        _center: &Coordinates,
        _radius_km: f64,
    ) -> AppResult<Vec<User>> {
        // Placeholder: return active users without geo filtering (no location table)
        use crate::infrastructure::database::schemas::users::dsl::*;
        let mut conn = self.pool.get()
            .map_err(|e|AppError::Database(DatabaseError::ConnectionPool(e)))?;
        let user_models: Vec<UserModel> = users
            .filter(is_active.eq(true))
            .load(&mut conn)
            .map_err(|e|AppError::Database(DatabaseError::Diesel(e)))?;
        let mut nearby_users = Vec::new();
        for model in user_models {
            if let Ok(user) = model.to_domain() {
                nearby_users.push(user);
            }
        }
        Ok(nearby_users)
    }

    async fn find_by_role(&self, _role: &UserRole) -> AppResult<Vec<User>> {
        // TODO: implement join with roles/user_roles to filter by role. For now, return active users.
        use crate::infrastructure::database::schemas::users::dsl::*;
        let mut conn = self.pool.get().map_err(|e|AppError::Database(DatabaseError::ConnectionPool(e)))?;
        let rows: Vec<UserModel> = users.filter(is_active.eq(true)).load(&mut conn).map_err(|e|AppError::Database(DatabaseError::Diesel(e)))?;
        Ok(rows.into_iter().filter_map(|m| m.to_domain().ok()).collect())
    }

    async fn count_by_status(&self, status_val: &str) -> AppResult<u64> {
        use crate::infrastructure::database::schemas::users::dsl::*;
        let cache_key = format!("user_count:status:{}", status_val);
        if let Some(s) = self.cache.get_string(&cache_key).await? {
            if let Ok(cached) = s.parse::<u64>() { return Ok(cached); }
        }
        let mut conn = self.pool.get()
            .map_err(|e|AppError::Database(DatabaseError::ConnectionPool(e)))?;
        let query = match status_val.to_lowercase().as_str() {
            "active" => users.filter(is_active.eq(true)).into_boxed(),
            "inactive" => users.filter(is_active.eq(false)).into_boxed(),
            "pending" => users.filter(is_verified.eq(false)).into_boxed(),
            _ => users.into_boxed(),
        };
        let count_i64: i64 = query.count().get_result(&mut conn).map_err(|e|AppError::Database(DatabaseError::Diesel(e)))?;
        let count_u64 = count_i64 as u64;
        let _ = self.cache.set_string(&cache_key, count_u64.to_string(), Some(Duration::from_secs(60))).await;
        Ok(count_u64)
    }

    async fn find_by_username(&self, username_val: &str) -> AppResult<Option<User>> {
        use crate::infrastructure::database::schemas::users::dsl::*;
        let mut conn = self.pool.get().map_err(|e|AppError::Database(DatabaseError::ConnectionPool(e)))?;
        let model: Option<UserModel> = users
            .filter(username.eq(username_val))
            .first::<UserModel>(&mut conn)
            .optional()
            .map_err(|e|AppError::Database(DatabaseError::Diesel(e)))?;
        Ok(model.and_then(|m| m.to_domain().ok()))
    }

    async fn find_active_responders(&self) -> AppResult<Vec<User>> {
        // TODO: implement join with roles to filter responders; currently returns active users only
        use crate::infrastructure::database::schemas::users::dsl::*;
        let mut conn = self.pool.get().map_err(|e|AppError::Database(DatabaseError::ConnectionPool(e)))?;
        let models: Vec<UserModel> = users.filter(is_active.eq(true)).load(&mut conn).map_err(|e|AppError::Database(DatabaseError::Diesel(e)))?;
        Ok(models.into_iter().filter_map(|m| m.to_domain().ok()).collect())
    }

    async fn update_last_login(&self, user_id_val: &crate::shared::UserId) -> AppResult<bool> {
        use crate::infrastructure::database::schemas::users::dsl::*;
        let mut conn = self.pool.get()
            .map_err(|e|AppError::Database(DatabaseError::ConnectionPool(e)))?;
        let now = Utc::now().naive_utc();
        let updated = diesel::update(users.filter(id.eq(user_id_val.value())))
            .set((last_login.eq(Some(now)), updated_at.eq(Some(now))))
            .execute(&mut conn)
            .map_err(|e|AppError::Database(DatabaseError::Diesel(e)))?;
        if updated > 0 {
            let _ = self.cache.delete(&CacheKeys::user(&user_id_val.value())).await;
        }
        Ok(updated > 0)
    }

    async fn verify_email(&self, user_id_val: &crate::shared::UserId) -> AppResult<bool> {
        use crate::infrastructure::database::schemas::users::dsl::*;
        let mut conn = self.pool.get()
            .map_err(|e|AppError::Database(DatabaseError::ConnectionPool(e)))?;
        let now = Utc::now().naive_utc();
        let updated = diesel::update(users.filter(id.eq(user_id_val.value())))
            .set((is_verified.eq(true), is_active.eq(true), updated_at.eq(Some(now))))
            .execute(&mut conn)
            .map_err(|e|AppError::Database(DatabaseError::Diesel(e)))?;
        if updated > 0 {
            let _ = self.cache.delete(&CacheKeys::user(&user_id_val.value())).await;
        }
        Ok(updated > 0)
    }

    async fn update_password(&self, user_id_val: &crate::shared::UserId, new_password_hash: &str) -> AppResult<bool> {
        use crate::infrastructure::database::schemas::users::dsl::*;
        let mut conn = self.pool.get()
            .map_err(|e|AppError::Database(DatabaseError::ConnectionPool(e)))?;
        let now = Utc::now().naive_utc();
        let updated = diesel::update(users.filter(id.eq(user_id_val.value())))
            .set((password_hash.eq(new_password_hash), updated_at.eq(Some(now))))
            .execute(&mut conn)
            .map_err(|e|AppError::Database(DatabaseError::Diesel(e)))?;
        if updated > 0 {
            // Invalidate cache keys best-effort: user key; email unknown here
            let _ = self.cache.delete(&CacheKeys::user(&user_id_val.value())).await;
        }
        Ok(updated > 0)
    }

    async fn count_by_role(&self, _role_val: &UserRole) -> AppResult<u64> {
        // TODO: implement using roles tables; for now count active users
        use crate::infrastructure::database::schemas::users::dsl::*;
        let mut conn = self.pool.get().map_err(|e|AppError::Database(DatabaseError::ConnectionPool(e)))?;
        let count_i64: i64 = users.filter(is_active.eq(true)).count().get_result(&mut conn).map_err(|e|AppError::Database(DatabaseError::Diesel(e)))?;
        Ok(count_i64 as u64)
    }
}
