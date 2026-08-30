use sqlx::{
    SqlitePool,
    migrate::MigrateError,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::path::Path;

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

/// Opens a pool for one named database file.
///
/// Operational commands take the file to work on as an argument rather than
/// inheriting the configured database, so a missing file is an error instead of
/// a new empty database.
///
/// # Errors
///
/// Returns [`sqlx::Error`] when `path` does not exist or cannot be opened.
pub async fn connect_file(path: &Path) -> Result<DbPool, sqlx::Error> {
    DbPoolOptions::new()
        .connect_with(
            SqliteConnectOptions::new()
                .filename(path)
                .create_if_missing(false),
        )
        .await
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
