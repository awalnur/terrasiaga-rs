/// Advanced infrastructure macros for repository implementations
/// These macros provide more sophisticated patterns for database operations

use std::sync::Arc;
use async_trait::async_trait;

/// Macro untuk implementasi repository CRUD operations dengan retry dan error handling yang lebih baik
#[macro_export]
macro_rules! impl_repository_crud {
    ($repo_struct:ident, $entity:ty, $id_type:ty, $table:ident) => {
        #[async_trait::async_trait]
        impl $crate::domain::ports::Repository<$entity, $id_type> for $repo_struct {
            async fn find_by_id(&self, id: $id_type) -> $crate::shared::AppResult<Option<$entity>> {
                use diesel::prelude::*;
                use $crate::schema::$table::dsl::*;

                $crate::with_db_connection!(&self.pool, |conn| {
                    $table
                        .filter(id.eq(id))
                        .first::<$entity>(conn)
                        .optional()
                })
            }

            async fn save(&self, entity: &$entity) -> $crate::shared::AppResult<$entity> {
                use diesel::prelude::*;
                use $crate::schema::$table::dsl::*;

                let entity_clone = entity.clone();
                $crate::with_db_connection!(&self.pool, |conn| {
                    diesel::insert_into($table)
                        .values(&entity_clone)
                        .get_result::<$entity>(conn)
                })
            }

            async fn update(&self, entity: &$entity) -> $crate::shared::AppResult<$entity> {
                use diesel::prelude::*;
                use $crate::schema::$table::dsl::*;

                let entity_clone = entity.clone();
                let entity_id = entity.id();

                $crate::with_db_connection!(&self.pool, |conn| {
                    diesel::update($table.filter(id.eq(entity_id)))
                        .set(&entity_clone)
                        .get_result::<$entity>(conn)
                })
            }

            async fn delete(&self, entity_id: $id_type) -> $crate::shared::AppResult<bool> {
                use diesel::prelude::*;
                use $crate::schema::$table::dsl::*;

                $crate::with_db_connection!(&self.pool, |conn| {
                    let affected_rows = diesel::delete($table.filter(id.eq(entity_id)))
                        .execute(conn)?;
                    Ok(affected_rows > 0)
                })
            }

            async fn find_all(&self) -> $crate::shared::AppResult<Vec<$entity>> {
                use diesel::prelude::*;
                use $crate::schema::$table::dsl::*;

                $crate::with_db_connection!(&self.pool, |conn| {
                    $table.load::<$entity>(conn)
                })
            }

            async fn count(&self) -> $crate::shared::AppResult<i64> {
                use diesel::prelude::*;
                use $crate::schema::$table::dsl::*;

                $crate::with_db_connection!(&self.pool, |conn| {
                    $table.count().get_result::<i64>(conn)
                })
            }
        }
    };
}

/// Macro untuk implementasi repository dengan pagination
#[macro_export]
macro_rules! impl_repository_pagination {
    ($repo_struct:ident, $entity:ty, $table:ident) => {
        impl $repo_struct {
            pub async fn find_with_pagination(
                &self,
                page: i64,
                per_page: i64
            ) -> $crate::shared::AppResult<(Vec<$entity>, i64)> {
                use diesel::prelude::*;
                use $crate::schema::$table::dsl::*;

                let offset = (page - 1) * per_page;

                $crate::with_db_connection!(&self.pool, |conn| {
                    let items = $table
                        .limit(per_page)
                        .offset(offset)
                        .load::<$entity>(conn)?;

                    let total_count = $table.count().get_result::<i64>(conn)?;

                    Ok((items, total_count))
                })
            }
        }
    };
}

/// Macro untuk implementasi repository dengan search functionality
#[macro_export]
macro_rules! impl_repository_search {
    ($repo_struct:ident, $entity:ty, $table:ident, $search_field:ident) => {
        impl $repo_struct {
            pub async fn search(
                &self,
                query: &str,
                limit: Option<i64>
            ) -> $crate::shared::AppResult<Vec<$entity>> {
                use diesel::prelude::*;
                use $crate::schema::$table::dsl::*;

                let search_pattern = format!("%{}%", query);
                let search_limit = limit.unwrap_or(100);

                $crate::with_db_connection!(&self.pool, |conn| {
                    $table
                        .filter($search_field.ilike(&search_pattern))
                        .limit(search_limit)
                        .load::<$entity>(conn)
                })
            }
        }
    };
    ($repo_struct:ident, $entity:ty, $table:ident, $search_field:ident, $($additional_field:ident),+) => {
        impl $repo_struct {
            pub async fn search(
                &self,
                query: &str,
                limit: Option<i64>
            ) -> $crate::shared::AppResult<Vec<$entity>> {
                use diesel::prelude::*;
                use $crate::schema::$table::dsl::*;

                let search_pattern = format!("%{}%", query);
                let search_limit = limit.unwrap_or(100);

                $crate::with_db_connection!(&self.pool, |conn| {
                    $table
                        .filter(
                            $search_field.ilike(&search_pattern)
                            $(.or($additional_field.ilike(&search_pattern)))+
                        )
                        .limit(search_limit)
                        .load::<$entity>(conn)
                })
            }
        }
    };
}

/// Macro untuk implementasi repository dengan soft delete
#[macro_export]
macro_rules! impl_repository_soft_delete {
    ($repo_struct:ident, $entity:ty, $id_type:ty, $table:ident) => {
        impl $repo_struct {
            pub async fn soft_delete(&self, entity_id: $id_type) -> $crate::shared::AppResult<bool> {
                use diesel::prelude::*;
                use $crate::schema::$table::dsl::*;
                use chrono::Utc;

                $crate::with_db_connection!(&self.pool, |conn| {
                    let affected_rows = diesel::update($table.filter(id.eq(entity_id)))
                        .set(deleted_at.eq(Some(Utc::now().naive_utc())))
                        .execute(conn)?;
                    Ok(affected_rows > 0)
                })
            }

            pub async fn restore(&self, entity_id: $id_type) -> $crate::shared::AppResult<bool> {
                use diesel::prelude::*;
                use $crate::schema::$table::dsl::*;

                $crate::with_db_connection!(&self.pool, |conn| {
                    let affected_rows = diesel::update($table.filter(id.eq(entity_id)))
                        .set(deleted_at.eq(None::<chrono::NaiveDateTime>))
                        .execute(conn)?;
                    Ok(affected_rows > 0)
                })
            }

            pub async fn find_active(&self) -> $crate::shared::AppResult<Vec<$entity>> {
                use diesel::prelude::*;
                use $crate::schema::$table::dsl::*;

                $crate::with_db_connection!(&self.pool, |conn| {
                    $table
                        .filter(deleted_at.is_null())
                        .load::<$entity>(conn)
                })
            }

            pub async fn find_deleted(&self) -> $crate::shared::AppResult<Vec<$entity>> {
                use diesel::prelude::*;
                use $crate::schema::$table::dsl::*;

                $crate::with_db_connection!(&self.pool, |conn| {
                    $table
                        .filter(deleted_at.is_not_null())
                        .load::<$entity>(conn)
                })
            }
        }
    };
}

/// Macro untuk implementasi repository dengan caching
#[macro_export]
macro_rules! impl_repository_cached {
    ($repo_struct:ident, $entity:ty, $id_type:ty, cache_ttl: $ttl:expr) => {
        impl $repo_struct {
            pub async fn find_by_id_cached(
                &self,
                id: $id_type,
                cache: &Arc<$crate::infrastructure::cache::CacheService>
            ) -> $crate::shared::AppResult<Option<$entity>> {
                let cache_key = format!("{}:{}", stringify!($entity), id);

                // Try to get from cache first
                if let Ok(Some(cached_data)) = cache.get::<$entity>(&cache_key).await {
                    tracing::debug!("Cache hit for key: {}", cache_key);
                    return Ok(Some(cached_data));
                }

                // If not in cache, get from database
                if let Some(entity) = self.find_by_id(id).await? {
                    // Store in cache for future requests
                    let _ = cache.set(&cache_key, &entity, Some(std::time::Duration::from_secs($ttl))).await;
                    tracing::debug!("Cached entity with key: {}", cache_key);
                    Ok(Some(entity))
                } else {
                    Ok(None)
                }
            }

            pub async fn invalidate_cache(
                &self,
                id: $id_type,
                cache: &Arc<$crate::infrastructure::cache::CacheService>
            ) -> $crate::shared::AppResult<()> {
                let cache_key = format!("{}:{}", stringify!($entity), id);
                cache.delete(&cache_key).await.map_err(|e| {
                    $crate::shared::error::AppError::Cache {
                        message: format!("Failed to invalidate cache for key {}: {}", cache_key, e),
                        source: Some(Box::new(e)),
                    }
                })?;
                tracing::debug!("Invalidated cache for key: {}", cache_key);
                Ok(())
            }
        }
    };
}

/// Macro untuk batch operations
#[macro_export]
macro_rules! impl_repository_batch {
    ($repo_struct:ident, $entity:ty, $table:ident) => {
        impl $repo_struct {
            pub async fn batch_insert(&self, entities: Vec<$entity>) -> $crate::shared::AppResult<Vec<$entity>> {
                use diesel::prelude::*;
                use $crate::schema::$table::dsl::*;

                if entities.is_empty() {
                    return Ok(vec![]);
                }

                $crate::with_db_connection!(&self.pool, |conn| {
                    diesel::insert_into($table)
                        .values(&entities)
                        .get_results::<$entity>(conn)
                })
            }

            pub async fn batch_update(&self, entities: Vec<$entity>) -> $crate::shared::AppResult<usize> {
                if entities.is_empty() {
                    return Ok(0);
                }

                let mut total_updated = 0;

                for entity in entities {
                    if self.update(&entity).await.is_ok() {
                        total_updated += 1;
                    }
                }

                Ok(total_updated)
            }
        }
    };
}

/// Macro untuk transaction support
#[macro_export]
macro_rules! impl_repository_transaction {
    ($repo_struct:ident) => {
        impl $repo_struct {
            pub async fn with_transaction<F, R>(&self, f: F) -> $crate::shared::AppResult<R>
            where
                F: FnOnce(&mut $crate::infrastructure::database::DbConnection) -> $crate::shared::AppResult<R> + Send + 'static,
                R: Send + 'static,
            {
                use diesel::prelude::*;

                $crate::with_db_connection!(&self.pool, |conn| {
                    conn.transaction(|conn| f(conn))
                })
            }
        }
    };
}

/// Complete repository implementation macro
#[macro_export]
macro_rules! impl_complete_repository {
    (
        $repo_struct:ident,
        $entity:ty,
        $id_type:ty,
        $table:ident,
        options: {
            pagination: $pagination:expr,
            search_fields: [$($search_field:ident),*],
            soft_delete: $soft_delete:expr,
            caching: { enabled: $cache_enabled:expr, ttl: $cache_ttl:expr },
            batch_operations: $batch_ops:expr
        }
    ) => {
        // Base implementation
        $crate::impl_repository_base!($repo_struct, $crate::infrastructure::database::DbPool);
        $crate::impl_repository_crud!($repo_struct, $entity, $id_type, $table);

        // Optional implementations
        $(
            #[cfg(feature = "pagination")]
            $(if $pagination {
                $crate::impl_repository_pagination!($repo_struct, $entity, $table);
            })?
        )?

        $(
            #[cfg(feature = "search")]
            $(if !vec![$($search_field),*].is_empty() {
                $crate::impl_repository_search!($repo_struct, $entity, $table, $($search_field),*);
            })?
        )?

        $(
            #[cfg(feature = "soft-delete")]
            $(if $soft_delete {
                $crate::impl_repository_soft_delete!($repo_struct, $entity, $id_type, $table);
            })?
        )?

        $(
            #[cfg(feature = "caching")]
            $(if $cache_enabled {
                $crate::impl_repository_cached!($repo_struct, $entity, $id_type, cache_ttl: $cache_ttl);
            })?
        )?

        $(
            #[cfg(feature = "batch-operations")]
            $(if $batch_ops {
                $crate::impl_repository_batch!($repo_struct, $entity, $table);
            })?
        )?

        // Always include transaction support
        $crate::impl_repository_transaction!($repo_struct);
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macro_compilation() {
        // Test that macros compile correctly
        // In a real test, you would test with actual database operations
    }
}
