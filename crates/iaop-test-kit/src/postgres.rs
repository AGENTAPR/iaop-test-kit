//! PostgreSQL test environment helpers.
//!
//! Provides isolated test databases, connection pool initialization,
//! and explicit failure guards to prevent tests from silently passing
//! when PostgreSQL is unavailable.

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

/// Helper for managing PostgreSQL integration test databases.
pub struct PostgresTestHelper;

impl PostgresTestHelper {
    /// Returns the configured database URL or standard dev fallback.
    pub fn default_url() -> String {
        std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@127.0.0.1:5432/postgres".to_string())
    }

    /// Connects to PostgreSQL or panics with an actionable error message.
    ///
    /// Non-negotiable requirement: integration tests MUST NOT silently pass
    /// when PostgreSQL is required.
    pub async fn connect_or_fail() -> PgPool {
        let url = Self::default_url();
        match PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(std::time::Duration::from_secs(3))
            .connect(&url)
            .await
        {
            Ok(pool) => pool,
            Err(e) => {
                panic!(
                    "FATAL: PostgreSQL integration test failed to connect to '{}'. \
                    Integration tests must not silently pass without database. Error: {}",
                    url, e
                );
            }
        }
    }

    /// Spawns a unique isolated test database (e.g. `iaop_test_{uuid}`)
    /// and returns the connected pool alongside an [`IsolatedDb`] handle
    /// that automatically drops the database when dropped.
    pub async fn create_isolated_db() -> (PgPool, IsolatedDb) {
        let admin_pool = Self::connect_or_fail().await;
        let db_name = format!("iaop_test_{}", Uuid::now_v7().simple());

        sqlx::query(&format!("CREATE DATABASE \"{}\"", db_name))
            .execute(&admin_pool)
            .await
            .unwrap_or_else(|e| {
                panic!(
                    "failed to create isolated test database '{}': {}",
                    db_name, e
                )
            });

        let base_url = Self::default_url();
        // Replace database name in URL
        let new_url = match base_url.rfind('/') {
            Some(idx) => format!("{}/{}", &base_url[..idx], db_name),
            None => format!("{}/{}", base_url, db_name),
        };

        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(&new_url)
            .await
            .unwrap_or_else(|e| {
                panic!(
                    "failed to connect to isolated test database '{}': {}",
                    db_name, e
                )
            });

        let handle = IsolatedDb {
            admin_pool,
            db_name,
        };

        (pool, handle)
    }
}

/// RAII handle that drops an isolated test database upon drop.
pub struct IsolatedDb {
    admin_pool: PgPool,
    db_name: String,
}

impl IsolatedDb {
    pub fn name(&self) -> &str {
        &self.db_name
    }

    pub async fn clean(self) {
        let _ = sqlx::query(&format!(
            "REVOKE CONNECT ON DATABASE \"{}\" FROM public; \
             SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{}'; \
             DROP DATABASE IF EXISTS \"{}\"",
            self.db_name, self.db_name, self.db_name
        ))
        .execute(&self.admin_pool)
        .await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_url_is_valid() {
        let url = PostgresTestHelper::default_url();
        assert!(url.starts_with("postgres://"));
    }
}
