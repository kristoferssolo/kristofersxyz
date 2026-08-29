//! A SQLite-backed [`SessionStore`] over the application's own connection pool.
//!
//! `tower-sessions-sqlx-store` pins sqlx 0.8. This implementation uses the
//! application's sqlx 0.9 pool and migrations. Logout deletes the server-side
//! row, which invalidates the cookie.

use crate::db::DbPool;
use async_trait::async_trait;
use time::OffsetDateTime;
use tower_sessions::{
    session::{Id, Record},
    session_store::{self, ExpiredDeletion, SessionStore},
};

/// Stores the id, JSON record, and Unix expiry timestamp in one row.
#[derive(Clone, Debug)]
pub struct SqliteSessionStore {
    pool: DbPool,
}

impl SqliteSessionStore {
    #[must_use]
    pub const fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Deletes expired records and returns how many rows were removed.
    ///
    /// # Errors
    ///
    /// Returns [`sqlx::Error`] when SQLite cannot perform the deletion.
    #[tracing::instrument(name = "Purge expired sessions", skip_all, err)]
    pub async fn purge_expired(&self) -> Result<u64, sqlx::Error> {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        let result = sqlx::query!("DELETE FROM sessions WHERE expiry_date <= ?1", now)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }

    /// Checks for an id collision before insertion.
    async fn id_taken(&self, id: &Id) -> session_store::Result<bool> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM sessions WHERE id = ?1",
            id.to_string()
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|error| backend(&error))?;
        Ok(count > 0)
    }

    async fn upsert(&self, record: &Record) -> session_store::Result<()> {
        let data = serde_json::to_string(record)
            .map_err(|error| session_store::Error::Encode(error.to_string()))?;
        sqlx::query!(
            "INSERT INTO sessions (id, data, expiry_date) VALUES (?1, ?2, ?3)
             ON CONFLICT(id) DO UPDATE SET data = excluded.data, expiry_date = excluded.expiry_date",
            record.id.to_string(),
            data,
            record.expiry_date.unix_timestamp()
        )
        .execute(&self.pool)
        .await
        .map_err(|error| backend(&error))?;
        Ok(())
    }
}

#[async_trait]
impl ExpiredDeletion for SqliteSessionStore {
    async fn delete_expired(&self) -> session_store::Result<()> {
        self.purge_expired()
            .await
            .map_err(|error| backend(&error))?;
        Ok(())
    }
}

#[async_trait]
impl SessionStore for SqliteSessionStore {
    async fn create(&self, record: &mut Record) -> session_store::Result<()> {
        while self.id_taken(&record.id).await? {
            record.id = Id::default();
        }
        self.upsert(record).await
    }

    async fn save(&self, record: &Record) -> session_store::Result<()> {
        self.upsert(record).await
    }

    async fn load(&self, session_id: &Id) -> session_store::Result<Option<Record>> {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        let row = sqlx::query!(
            "SELECT data FROM sessions WHERE id = ?1 AND expiry_date > ?2",
            session_id.to_string(),
            now
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| backend(&error))?;

        row.map(|row| {
            serde_json::from_str::<Record>(&row.data)
                .map_err(|error| session_store::Error::Decode(error.to_string()))
        })
        .transpose()
    }

    async fn delete(&self, session_id: &Id) -> session_store::Result<()> {
        sqlx::query!("DELETE FROM sessions WHERE id = ?1", session_id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|error| backend(&error))?;
        Ok(())
    }
}

/// Converts a sqlx failure into the session store's backend error.
fn backend(error: &sqlx::Error) -> session_store::Error {
    session_store::Error::Backend(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_support::migrated_pool;
    use std::collections::HashMap;
    use time::Duration;

    async fn store() -> SqliteSessionStore {
        SqliteSessionStore::new(migrated_pool().await)
    }

    fn record(offset: Duration) -> Record {
        let mut data = HashMap::new();
        data.insert("user_id".to_owned(), serde_json::json!("owner"));
        Record {
            id: Id::default(),
            data,
            expiry_date: OffsetDateTime::now_utc()
                .checked_add(offset)
                .unwrap_or_else(|| panic!("session expiry fits in OffsetDateTime")),
        }
    }

    #[tokio::test]
    async fn create_then_load_round_trips_the_record() {
        let store = store().await;
        let mut record = record(Duration::hours(1));
        store.create(&mut record).await.expect("create the session");

        let loaded = store.load(&record.id).await.expect("load the session");
        assert_eq!(loaded, Some(record));
    }

    #[tokio::test]
    async fn save_overwrites_the_record() {
        let store = store().await;
        let mut record = record(Duration::hours(1));
        store.create(&mut record).await.expect("create the session");

        record
            .data
            .insert("user_id".to_owned(), serde_json::json!("someone else"));
        store.save(&record).await.expect("save the session");

        let loaded = store.load(&record.id).await.expect("load the session");
        assert_eq!(loaded, Some(record));
    }

    #[tokio::test]
    async fn an_expired_session_does_not_load() {
        let store = store().await;
        let mut record = record(-Duration::hours(1));
        store.create(&mut record).await.expect("create the session");

        assert_eq!(store.load(&record.id).await.expect("load"), None);
    }

    #[tokio::test]
    async fn purge_expired_deletes_only_expired_sessions() {
        let store = store().await;
        let mut expired = record(-Duration::hours(1));
        let mut active = record(Duration::hours(1));
        store.create(&mut expired).await.expect("create expired");
        store.create(&mut active).await.expect("create active");

        assert_eq!(store.purge_expired().await.expect("purge expired"), 1);
        assert_eq!(store.load(&expired.id).await.expect("load expired"), None);
        assert_eq!(
            store.load(&active.id).await.expect("load active"),
            Some(active)
        );
    }

    #[tokio::test]
    async fn delete_removes_the_session() {
        let store = store().await;
        let mut record = record(Duration::hours(1));
        store.create(&mut record).await.expect("create the session");
        store.delete(&record.id).await.expect("delete the session");

        assert_eq!(store.load(&record.id).await.expect("load"), None);
    }
}
