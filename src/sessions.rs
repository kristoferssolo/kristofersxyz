//! A SQLite-backed [`SessionStore`] over the application's own connection pool.
//!
//! `tower-sessions-sqlx-store` pins sqlx 0.8, which would pull a second sqlx
//! into the build alongside this crate's 0.9. Implementing the store here keeps
//! one sqlx, one pool, and one migration story, while still giving real
//! server-side sessions: a logout deletes the row, so a stolen cookie stops
//! working immediately.

use crate::db::DbPool;
use async_trait::async_trait;
use time::OffsetDateTime;
use tower_sessions::{
    session::{Id, Record},
    session_store::{self, SessionStore},
};

/// Stores each session as one row: the id, the record as JSON, and the expiry
/// as a unix timestamp so expired rows can be filtered and swept in SQL.
#[derive(Clone, Debug)]
pub struct SqliteSessionStore {
    pool: DbPool,
}

impl SqliteSessionStore {
    #[must_use]
    pub const fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Whether a row already holds this id, used to resolve the astronomically
    /// unlikely id collision on creation.
    async fn id_taken(&self, id: &Id) -> session_store::Result<bool> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sessions WHERE id = ?1")
            .bind(id.to_string())
            .fetch_one(&self.pool)
            .await
            .map_err(backend)?;
        Ok(count > 0)
    }

    async fn upsert(&self, record: &Record) -> session_store::Result<()> {
        let data = serde_json::to_string(record)
            .map_err(|error| session_store::Error::Encode(error.to_string()))?;
        sqlx::query(
            "INSERT INTO sessions (id, data, expiry_date) VALUES (?1, ?2, ?3)
             ON CONFLICT(id) DO UPDATE SET data = excluded.data, expiry_date = excluded.expiry_date",
        )
        .bind(record.id.to_string())
        .bind(data)
        .bind(record.expiry_date.unix_timestamp())
        .execute(&self.pool)
        .await
        .map_err(backend)?;
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
        let row: Option<(String,)> =
            sqlx::query_as("SELECT data FROM sessions WHERE id = ?1 AND expiry_date > ?2")
                .bind(session_id.to_string())
                .bind(now)
                .fetch_optional(&self.pool)
                .await
                .map_err(backend)?;

        row.map(|(data,)| {
            serde_json::from_str::<Record>(&data)
                .map_err(|error| session_store::Error::Decode(error.to_string()))
        })
        .transpose()
    }

    async fn delete(&self, session_id: &Id) -> session_store::Result<()> {
        sqlx::query("DELETE FROM sessions WHERE id = ?1")
            .bind(session_id.to_string())
            .execute(&self.pool)
            .await
            .map_err(backend)?;
        Ok(())
    }
}

/// A database failure the store cannot recover from, in the store's own error
/// vocabulary.
fn backend(error: sqlx::Error) -> session_store::Error {
    session_store::Error::Backend(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{DbPoolOptions, migrate};
    use std::collections::HashMap;
    use time::Duration;

    async fn store() -> SqliteSessionStore {
        let pool = DbPoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("connect to an in-memory database");
        migrate(&pool).await.expect("run the migrations");
        SqliteSessionStore::new(pool)
    }

    fn record(offset: Duration) -> Record {
        let mut data = HashMap::new();
        data.insert("user_id".to_owned(), serde_json::json!("owner"));
        Record {
            id: Id::default(),
            data,
            expiry_date: OffsetDateTime::now_utc() + offset,
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
    async fn delete_removes_the_session() {
        let store = store().await;
        let mut record = record(Duration::hours(1));
        store.create(&mut record).await.expect("create the session");
        store.delete(&record.id).await.expect("delete the session");

        assert_eq!(store.load(&record.id).await.expect("load"), None);
    }
}
