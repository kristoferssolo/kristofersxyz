mod backup;
mod connection;
pub mod portfolio;
mod seed;

pub use backup::{BackupError, RestoreReport, back_up, prepare_restored};
pub use connection::{DbPool, DbPoolOptions, connect, connect_file, migrate};
pub use seed::seed_if_empty;

#[cfg(test)]
pub(crate) mod test_support {
    use super::{DbPool, DbPoolOptions, migrate};

    /// A migrated in-memory database for tests.
    pub async fn migrated_pool() -> DbPool {
        let pool = DbPoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("connect to an in-memory database");
        migrate(&pool).await.expect("run the migrations");
        pool
    }
}
