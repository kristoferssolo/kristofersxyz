use super::{DbPool, DbPoolOptions, migrate, seed_if_empty};

/// SQLite gives each connection its own in-memory database, so tests use one
/// connection for migration, seeding, and queries.
pub(super) async fn migrated_pool() -> DbPool {
    let pool = DbPoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("connect to an in-memory database");
    migrate(&pool).await.expect("run the migrations");
    pool
}

pub(super) async fn seeded_pool() -> DbPool {
    let pool = migrated_pool().await;
    seed_if_empty(&pool).await.expect("seed the database");
    pool
}
