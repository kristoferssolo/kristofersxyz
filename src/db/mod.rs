use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

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
