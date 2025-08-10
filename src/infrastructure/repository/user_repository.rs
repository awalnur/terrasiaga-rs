/// User repository implementation with caching
/// Provides data access layer for user entities with Redis caching support

use async_trait::async_trait;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entities::User;
use crate::domain::value_objects::*;
use crate::domain::ports::repositories::UserRepository;
use crate::infrastructure::cache::{CacheService, CacheKeys};
use crate::shared::{AppResult, AppError};
use std::time::Duration;

/// Database model for users
#[derive(Queryable, Insertable, AsChangeset, Debug)]
#[diesel(table_name = users)]
pub struct UserModel {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub full_name: String,
    pub phone_number: Option<String>,
    pub role: String,
    pub status: String,
    pub address_street: Option<String>,
    pub address_city: Option<String>,
    pub address_province: Option<String>,
    pub address_postal_code: Option<String>,
    pub address_country: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub last_login: Option<chrono::DateTime<chrono::Utc>>,
    pub version: i64,
}

table! {
    users (id) {
        id -> Uuid,
        email -> Varchar,
        password_hash -> Varchar,
        full_name -> Varchar,
        phone_number -> Nullable<Varchar>,
        role -> Varchar,
        status -> Varchar,
        address_street -> Nullable<Varchar>,
        address_city -> Nullable<Varchar>,
        address_province -> Nullable<Varchar>,
        address_postal_code -> Nullable<Varchar>,
        address_country -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        last_login -> Nullable<Timestamptz>,
        version -> BigInt,
    }
}

impl UserModel {
    /// Convert database model to domain entity
    pub fn to_domain(&self) -> AppResult<User> {
        let email = Email::new(self.email.clone())?;
        
        let phone_number = match &self.phone_number {
            Some(phone) => Some(PhoneNumber::new(phone.clone())?),
            None => None,
        };

        let role = match self.role.as_str() {
            "citizen" => UserRole::Citizen,
            "volunteer" => UserRole::Volunteer,
            "responder" => UserRole::Responder,
            "admin" => UserRole::Admin,
            "super_admin" => UserRole::SuperAdmin,
            _ => return Err(AppError::Internal(format!("Invalid user role: {}", self.role))),
        };

        let address = if let (Some(street), Some(city), Some(province)) = 
            (&self.address_street, &self.address_city, &self.address_province) {
            Some(Address::new(
                street.clone(),
                city.clone(),
                province.clone(),
                self.address_postal_code.clone(),
                self.address_country.clone(),
            )?)
        } else {
            None
        };

        let user = User::from_persistence(
            UserId::from_uuid(self.id),
            email,
            self.password_hash.clone(),
            self.full_name.clone(),
            phone_number,
            role,
            self.status.clone(),
            address,
            self.created_at,
            self.updated_at,
            self.last_login,
            self.version as u64,
        )?;

        Ok(user)
    }

    /// Convert domain entity to database model
    pub fn from_domain(user: &User) -> Self {
        let role_str = match user.role() {
            UserRole::Citizen => "citizen",
            UserRole::Volunteer => "volunteer",
            UserRole::Responder => "responder",
            UserRole::Admin => "admin",
            UserRole::SuperAdmin => "super_admin",
        };

        let (street, city, province, postal_code, country) = match user.address() {
            Some(addr) => (
                Some(addr.street.clone()),
                Some(addr.city.clone()),
                Some(addr.province.clone()),
                addr.postal_code.clone(),
                Some(addr.country.clone()),
            ),
            None => (None, None, None, None, None),
        };

        Self {
            id: user.id().value(),
            email: user.email().value().to_string(),
            password_hash: user.password_hash().to_string(),
            full_name: user.full_name().to_string(),
            phone_number: user.phone_number().map(|p| p.value().to_string()),
            role: role_str.to_string(),
            status: user.status().to_string(),
            address_street: street,
            address_city: city,
            address_province: province,
            address_postal_code: postal_code,
            address_country: country,
            created_at: user.created_at(),
            updated_at: user.updated_at(),
            last_login: user.last_login(),
            version: user.version() as i64,
        }
    }
}

/// PostgreSQL implementation of UserRepository with caching
pub struct PostgresUserRepository {
    pool: Arc<Pool<ConnectionManager<PgConnection>>>,
    cache: Arc<dyn CacheService>,
}

impl PostgresUserRepository {
    pub fn new(
        pool: Arc<Pool<ConnectionManager<PgConnection>>>,
        cache: Arc<dyn CacheService>,
    ) -> Self {
        Self { pool, cache }
    }

    /// Cache TTL for user data
    const USER_CACHE_TTL: Duration = Duration::from_secs(300); // 5 minutes

    /// Invalidate user cache
    async fn invalidate_user_cache(&self, user_id: &UserId, email: &Email) -> AppResult<()> {
        let user_key = CacheKeys::user(&user_id.value().to_string());
        let email_key = CacheKeys::user_by_email(email.value());
        
        let _ = self.cache.delete(&user_key).await;
        let _ = self.cache.delete(&email_key).await;
        
        Ok(())
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn find_by_id(&self, id: &UserId) -> AppResult<Option<User>> {
        let cache_key = CacheKeys::user(&id.value().to_string());
        
        // Try cache first
        if let Ok(Some(user)) = self.cache.get::<User>(&cache_key).await {
            return Ok(Some(user));
        }

        // Query database
        use self::users::dsl::*;
        
        let mut conn = self.pool.get()
            .map_err(|e| AppError::Database(e.to_string()))?;

        let user_model: Option<UserModel> = users
            .filter(id.eq(id.value()))
            .first(&mut conn)
            .optional()
            .map_err(|e| AppError::Database(e.to_string()))?;

        match user_model {
            Some(model) => {
                let user = model.to_domain()?;
                
                // Cache the result
                let _ = self.cache.set(&cache_key, &user, Some(Self::USER_CACHE_TTL)).await;
                
                Ok(Some(user))
            }
            None => Ok(None),
        }
    }

    async fn find_by_email(&self, email: &Email) -> AppResult<Option<User>> {
        let cache_key = CacheKeys::user_by_email(email.value());
        
        // Try cache first
        if let Ok(Some(user)) = self.cache.get::<User>(&cache_key).await {
            return Ok(Some(user));
        }

        // Query database
        use self::users::dsl::*;
        
        let mut conn = self.pool.get()
            .map_err(|e| AppError::Database(e.to_string()))?;

        let user_model: Option<UserModel> = users
            .filter(email.eq(email.value()))
            .first(&mut conn)
            .optional()
            .map_err(|e| AppError::Database(e.to_string()))?;

        match user_model {
            Some(model) => {
                let user = model.to_domain()?;
                
                // Cache the result
                let _ = self.cache.set(&cache_key, &user, Some(Self::USER_CACHE_TTL)).await;
                
                Ok(Some(user))
            }
            None => Ok(None),
        }
    }

    async fn save(&self, user: &User) -> AppResult<User> {
        use self::users::dsl::*;
        
        let mut conn = self.pool.get()
            .map_err(|e| AppError::Database(e.to_string()))?;

        let user_model = UserModel::from_domain(user);

        // Use upsert (INSERT ... ON CONFLICT UPDATE)
        let saved_model = diesel::insert_into(users)
            .values(&user_model)
            .on_conflict(id)
            .do_update()
            .set((
                email.eq(&user_model.email),
                password_hash.eq(&user_model.password_hash),
                full_name.eq(&user_model.full_name),
                phone_number.eq(&user_model.phone_number),
                role.eq(&user_model.role),
                status.eq(&user_model.status),
                address_street.eq(&user_model.address_street),
                address_city.eq(&user_model.address_city),
                address_province.eq(&user_model.address_province),
                address_postal_code.eq(&user_model.address_postal_code),
                address_country.eq(&user_model.address_country),
                updated_at.eq(chrono::Utc::now()),
                last_login.eq(&user_model.last_login),
                version.eq(&user_model.version),
            ))
            .get_result::<UserModel>(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

        let saved_user = saved_model.to_domain()?;
        
        // Invalidate cache
        self.invalidate_user_cache(&saved_user.id(), &saved_user.email()).await?;
        
        Ok(saved_user)
    }

    async fn delete(&self, id: &UserId) -> AppResult<bool> {
        use self::users::dsl::*;
        
        // Get user first for cache invalidation
        let user = match self.find_by_id(id).await? {
            Some(user) => user,
            None => return Ok(false),
        };

        let mut conn = self.pool.get()
            .map_err(|e| AppError::Database(e.to_string()))?;

        let deleted_count = diesel::delete(users.filter(id.eq(id.value())))
            .execute(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

        if deleted_count > 0 {
            // Invalidate cache
            self.invalidate_user_cache(&user.id(), &user.email()).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn find_users_in_radius(
        &self,
        center: &Coordinates,
        radius_km: f64,
    ) -> AppResult<Vec<User>> {
        // This is a simplified implementation
        // In production, you'd use PostGIS for efficient geospatial queries
        use self::users::dsl::*;
        
        let mut conn = self.pool.get()
            .map_err(|e| AppError::Database(e.to_string()))?;

        // For now, get all active users and filter in memory
        // In production, use PostGIS ST_DWithin function
        let user_models: Vec<UserModel> = users
            .filter(status.eq("active"))
            .load(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut nearby_users = Vec::new();
        
        for model in user_models {
            // In a real implementation, you'd have user location data
            // For now, we'll return all active users as a placeholder
            if let Ok(user) = model.to_domain() {
                nearby_users.push(user);
            }
        }

        Ok(nearby_users)
    }

    async fn find_by_role(&self, role: &UserRole) -> AppResult<Vec<User>> {
        use self::users::dsl::*;
        
        let role_str = match role {
            UserRole::Citizen => "citizen",
            UserRole::Volunteer => "volunteer",
            UserRole::Responder => "responder",
            UserRole::Admin => "admin",
            UserRole::SuperAdmin => "super_admin",
        };

        let mut conn = self.pool.get()
            .map_err(|e| AppError::Database(e.to_string()))?;

        let user_models: Vec<UserModel> = users
            .filter(role.eq(role_str))
            .filter(status.eq("active"))
            .load(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut result_users = Vec::new();
        for model in user_models {
            if let Ok(user) = model.to_domain() {
                result_users.push(user);
            }
        }

        Ok(result_users)
    }

    async fn count_by_status(&self, status: &str) -> AppResult<u64> {
        use self::users::dsl::*;
        
        let cache_key = format!("user_count:status:{}", status);
        
        // Try cache first (short TTL for counts)
        if let Ok(Some(count)) = self.cache.get::<u64>(&cache_key).await {
            return Ok(count);
        }

        let mut conn = self.pool.get()
            .map_err(|e| AppError::Database(e.to_string()))?;

        let count: i64 = users
            .filter(status.eq(status))
            .count()
            .get_result(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

        let count_u64 = count as u64;
        
        // Cache for 1 minute
        let _ = self.cache.set(&cache_key, &count_u64, Some(Duration::from_secs(60))).await;

        Ok(count_u64)
    }
}
