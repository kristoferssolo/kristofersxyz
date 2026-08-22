pub mod portfolio;

use sqlx::{SqlitePool, migrate::MigrateError, sqlite::SqlitePoolOptions};

pub type DbPool = SqlitePool;
pub type DbPoolOptions = SqlitePoolOptions;

/// Opens a SQLite connection pool.
///
/// # Errors
///
/// Returns [`sqlx::Error`] when the pool cannot connect to `database_url`.
pub async fn connect(database_url: &str) -> Result<DbPool, sqlx::Error> {
    DbPoolOptions::new().connect(database_url).await
}

/// Applies every pending migration, embedded from `migrations/` at compile
/// time.
///
/// The deploy has no volume, so the database is empty on each boot; running the
/// migrations from code is what gives the schema without a manual step.
/// Idempotent, so an already-migrated database is left untouched.
///
/// # Errors
///
/// Returns [`MigrateError`] when a migration fails to apply.
pub async fn migrate(pool: &DbPool) -> Result<(), MigrateError> {
    sqlx::migrate!().run(pool).await
}
