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

/// Loads the bundled seed into an empty database, embedded at compile time so
/// no file has to ship. Paired with the volumeless deploy: a fresh boot starts
/// empty, and this makes the site self-sufficient without a manual seed step.
///
/// The guard is what keeps it safe once a persistent volume and the CMS arrive:
/// when content already exists it does nothing, so edits are never clobbered.
///
/// # Errors
///
/// Returns [`sqlx::Error`] if the emptiness check or the seed fails.
pub async fn seed_if_empty(pool: &DbPool) -> Result<(), sqlx::Error> {
    let existing: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM site")
        .fetch_one(pool)
        .await?;
    if existing > 0 {
        return Ok(());
    }

    sqlx::raw_sql(include_str!("../../seeds/portfolio.sql"))
        .execute(pool)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn migrated_pool() -> DbPool {
        let pool = DbPoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("connect to an in-memory database");
        migrate(&pool).await.expect("run the migrations");
        pool
    }

    #[tokio::test]
    async fn seed_if_empty_populates_a_fresh_database() {
        let pool = migrated_pool().await;
        seed_if_empty(&pool).await.expect("seed the empty database");

        let projects: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM project")
            .fetch_one(&pool)
            .await
            .expect("count projects");
        assert_eq!(projects, 3);
    }

    /// The CMS-safety property: re-running never overwrites existing content.
    #[tokio::test]
    async fn seed_if_empty_leaves_existing_content_alone() {
        let pool = migrated_pool().await;
        seed_if_empty(&pool).await.expect("first seed");

        sqlx::query("UPDATE site SET title = 'edited' WHERE id = 1")
            .execute(&pool)
            .await
            .expect("edit the seeded content");
        seed_if_empty(&pool).await.expect("second seed is a no-op");

        let title: String = sqlx::query_scalar("SELECT title FROM site WHERE id = 1")
            .fetch_one(&pool)
            .await
            .expect("read the title back");
        assert_eq!(title, "edited");
    }
}
