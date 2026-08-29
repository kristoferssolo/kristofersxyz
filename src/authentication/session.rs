//! The owner session and its valid authentication transitions.

use crate::{
    configuration::SessionPolicy,
    db::DbPool,
    domain::{OwnerId, Username},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use time::{Duration, OffsetDateTime};
use tokio::sync::Mutex;
use tower_sessions::{Session, session};

const OWNER_ID_KEY: &str = "owner_id";
const LEGACY_USER_ID_KEY: &str = "user_id";
const LEGACY_USERNAME_KEY: &str = "username";
const ISSUED_AT_KEY: &str = "issued_at";
static SIGN_INS: Mutex<()> = Mutex::const_new(());

#[derive(Clone, Copy, Deserialize, Serialize)]
struct SessionIssuedAt(i64);

impl SessionIssuedAt {
    fn now() -> Self {
        Self(OffsetDateTime::now_utc().unix_timestamp())
    }

    fn expires_at(self, lifetime: Duration) -> Option<OffsetDateTime> {
        OffsetDateTime::from_unix_timestamp(self.0)
            .ok()?
            .checked_add(lifetime)
    }
}

enum StoredClaim<T> {
    Missing,
    Value(T),
    Corrupt,
}

/// A session whose stored authentication data has not been checked yet.
pub struct Unverified;

/// A session with no valid owner identity.
pub struct Anonymous;

/// A session whose owner still exists in the database.
pub struct Authenticated {
    owner_id: OwnerId,
    username: Username,
}

/// A session whose state is carried in `State`.
pub struct AuthSession<State> {
    inner: Session,
    state: State,
}

/// The two states produced by checking a session's stored identity.
pub enum SessionState {
    Anonymous(AuthSession<Anonymous>),
    Authenticated(AuthSession<Authenticated>),
}

#[derive(Debug, thiserror::Error)]
pub enum AuthSessionError {
    #[error("failed to query the owner session")]
    Database(#[from] sqlx::Error),
    #[error("failed to read or update the owner session")]
    Session(#[from] session::Error),
    #[error("a persisted authenticated session has no id")]
    MissingSessionId,
}

impl AuthSession<Unverified> {
    #[must_use]
    pub const fn new(inner: Session) -> Self {
        Self {
            inner,
            state: Unverified,
        }
    }

    /// Resolves the stored owner ID against the current database row.
    ///
    /// Empty sessions remain anonymous. Partial, malformed, or orphaned
    /// sessions are flushed so the same invalid state is not accepted again.
    ///
    /// # Errors
    ///
    /// Returns [`AuthSessionError`] if SQLite or the session store cannot be
    /// reached.
    #[tracing::instrument(name = "Resolve owner session", skip_all, err)]
    pub async fn resolve(
        self,
        pool: &DbPool,
        policy: SessionPolicy,
    ) -> Result<SessionState, AuthSessionError> {
        self.resolve_at(pool, policy, OffsetDateTime::now_utc())
            .await
    }

    async fn resolve_at(
        self,
        pool: &DbPool,
        policy: SessionPolicy,
        now: OffsetDateTime,
    ) -> Result<SessionState, AuthSessionError> {
        let owner_id = match self.read_owner_id().await? {
            StoredClaim::Value(owner_id) => owner_id,
            StoredClaim::Missing if self.inner.is_empty().await => return Ok(self.anonymous()),
            StoredClaim::Missing => return self.reject("session_partial_identity").await,
            StoredClaim::Corrupt => return self.reject("session_corrupt").await,
        };
        let issued_at = match self.claim::<SessionIssuedAt>(ISSUED_AT_KEY).await? {
            StoredClaim::Value(issued_at) => issued_at,
            StoredClaim::Missing => return self.reject("session_partial_identity").await,
            StoredClaim::Corrupt => return self.reject("session_corrupt").await,
        };
        let Some(expires_at) = issued_at.expires_at(policy.absolute_timeout()) else {
            return self.reject("session_corrupt").await;
        };
        if now >= expires_at {
            return self.reject("session_absolute_expired").await;
        }

        let row = sqlx::query!(
            "SELECT username FROM users WHERE user_id = ?1",
            owner_id.to_string()
        )
        .fetch_optional(pool)
        .await?;
        let Some(row) = row else {
            return self.reject("session_owner_missing").await;
        };
        let Ok(username) = Username::new(row.username) else {
            return self.reject("session_owner_corrupt").await;
        };

        if self.inner.get_value(LEGACY_USERNAME_KEY).await?.is_some() {
            self.inner.remove_value(LEGACY_USERNAME_KEY).await?;
        }
        tracing::debug!(
            event = "session_resolved",
            owner_id = %owner_id,
            username = %username,
            "Resolved owner session"
        );
        Ok(SessionState::Authenticated(AuthSession {
            inner: self.inner,
            state: Authenticated { owner_id, username },
        }))
    }

    async fn read_owner_id(&self) -> Result<StoredClaim<OwnerId>, session::Error> {
        match self.claim(OWNER_ID_KEY).await? {
            StoredClaim::Missing => {}
            claim => return Ok(claim),
        }

        let legacy_owner_id = self.claim(LEGACY_USER_ID_KEY).await?;
        if let StoredClaim::Value(owner_id) = legacy_owner_id {
            self.inner.insert(OWNER_ID_KEY, owner_id).await?;
            self.inner.remove_value(LEGACY_USER_ID_KEY).await?;
        }
        Ok(legacy_owner_id)
    }

    async fn claim<T>(&self, key: &str) -> Result<StoredClaim<T>, session::Error>
    where
        T: DeserializeOwned,
    {
        let Some(value) = self.inner.get_value(key).await? else {
            return Ok(StoredClaim::Missing);
        };
        Ok(serde_json::from_value(value).map_or(StoredClaim::Corrupt, StoredClaim::Value))
    }

    async fn reject(self, event: &'static str) -> Result<SessionState, AuthSessionError> {
        tracing::warn!(event, "Rejected invalid owner session");
        self.inner.flush().await?;
        Ok(self.anonymous())
    }

    fn anonymous(self) -> SessionState {
        SessionState::Anonymous(AuthSession {
            inner: self.inner,
            state: Anonymous,
        })
    }
}

impl AuthSession<Anonymous> {
    /// Rotates the ID, records only the stable owner ID, and revokes the
    /// previous owner session.
    ///
    /// # Errors
    ///
    /// Returns [`AuthSessionError`] if SQLite or the session store cannot be
    /// updated.
    #[tracing::instrument(
        name = "Sign in owner session",
        skip_all,
        fields(owner_id = %owner_id, username = %username),
        err,
    )]
    pub async fn sign_in(
        self,
        pool: &DbPool,
        owner_id: OwnerId,
        username: Username,
    ) -> Result<AuthSession<Authenticated>, AuthSessionError> {
        // Serializing this short section makes "newest login wins" reliable
        // within the single application process used by this deployment.
        let _guard = SIGN_INS.lock().await;
        self.inner.cycle_id().await?;
        self.inner.insert(OWNER_ID_KEY, owner_id).await?;
        self.inner
            .insert(ISSUED_AT_KEY, SessionIssuedAt::now())
            .await?;
        self.inner.save().await?;
        let session_id = self.inner.id().ok_or(AuthSessionError::MissingSessionId)?;
        sqlx::query!(
            "DELETE FROM sessions WHERE id <> ?1",
            session_id.to_string()
        )
        .execute(pool)
        .await?;

        tracing::info!(
            event = "session_started",
            owner_id = %owner_id,
            username = %username,
            "Started owner session"
        );
        Ok(AuthSession {
            inner: self.inner,
            state: Authenticated { owner_id, username },
        })
    }
}

impl AuthSession<Authenticated> {
    #[must_use]
    pub const fn username(&self) -> &Username {
        &self.state.username
    }

    /// Clears the identity, deletes the stored record, and returns an
    /// anonymous state.
    ///
    /// # Errors
    ///
    /// Returns [`session::Error`] if the session store cannot be updated.
    #[tracing::instrument(
        name = "Sign out owner session",
        skip_all,
        fields(
            owner_id = %self.state.owner_id,
            username = %self.state.username,
        ),
        err,
    )]
    pub async fn sign_out(self) -> Result<AuthSession<Anonymous>, session::Error> {
        self.inner.flush().await?;
        tracing::info!(
            event = "session_ended",
            owner_id = %self.state.owner_id,
            username = %self.state.username,
            "Ended owner session"
        );
        Ok(AuthSession {
            inner: self.inner,
            state: Anonymous,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{configuration::DeploymentMode, db::test_support::migrated_pool};
    use claims::{assert_ok, assert_some};
    use std::sync::Arc;
    use tower_sessions::{MemoryStore, SessionStore};

    async fn pool_with_owner(owner_id: OwnerId, username: &str) -> DbPool {
        let pool = migrated_pool().await;
        sqlx::query!(
            "INSERT INTO users (user_id, username, password_hash) VALUES (?1, ?2, 'hash')",
            owner_id.to_string(),
            username,
        )
        .execute(&pool)
        .await
        .expect("insert the owner");
        pool
    }

    fn session(store: Arc<MemoryStore>) -> Session {
        Session::new(None, store, None)
    }

    fn policy() -> SessionPolicy {
        DeploymentMode::Local.session_policy()
    }

    async fn insert_identity(session: &Session, owner_id: OwnerId, issued_at: OffsetDateTime) {
        assert_ok!(session.insert(OWNER_ID_KEY, owner_id).await);
        assert_ok!(
            session
                .insert(ISSUED_AT_KEY, SessionIssuedAt(issued_at.unix_timestamp()))
                .await
        );
        assert_ok!(session.save().await);
    }

    fn anonymous(state: SessionState) -> Option<AuthSession<Anonymous>> {
        match state {
            SessionState::Anonymous(session) => Some(session),
            SessionState::Authenticated(_) => None,
        }
    }

    #[tokio::test]
    async fn an_empty_session_is_anonymous() {
        let owner_id = OwnerId::new();
        let pool = pool_with_owner(owner_id, "owner").await;
        let state = assert_ok!(
            AuthSession::new(session(Arc::new(MemoryStore::default())))
                .resolve(&pool, policy())
                .await
        );
        assert!(matches!(state, SessionState::Anonymous(_)));
    }

    #[tokio::test]
    async fn a_partial_identity_is_flushed() {
        let owner_id = OwnerId::new();
        let pool = pool_with_owner(owner_id, "owner").await;
        let session = session(Arc::new(MemoryStore::default()));
        assert_ok!(session.insert(LEGACY_USERNAME_KEY, "owner").await);
        assert_ok!(session.save().await);

        let state = assert_ok!(
            AuthSession::new(session.clone())
                .resolve(&pool, policy())
                .await
        );
        assert!(matches!(state, SessionState::Anonymous(_)));
        assert!(session.id().is_none());
        assert!(session.is_empty().await);
    }

    #[tokio::test]
    async fn the_current_database_username_is_authoritative() {
        let owner_id = OwnerId::new();
        let pool = pool_with_owner(owner_id, "owner").await;
        let store = Arc::new(MemoryStore::default());
        let session = session(store);
        insert_identity(&session, owner_id, OffsetDateTime::now_utc()).await;
        assert_ok!(session.insert(LEGACY_USERNAME_KEY, "stale name").await);
        assert_ok!(session.save().await);

        let state = assert_ok!(
            AuthSession::new(session.clone())
                .resolve(&pool, policy())
                .await
        );
        let authenticated = assert_some!(match state {
            SessionState::Authenticated(session) => Some(session),
            SessionState::Anonymous(_) => None,
        });
        assert_eq!(authenticated.username().as_str(), "owner");
        assert_eq!(
            assert_ok!(session.get::<String>(LEGACY_USERNAME_KEY).await),
            None
        );
    }

    #[tokio::test]
    async fn a_deleted_owner_invalidates_the_session() {
        let owner_id = OwnerId::new();
        let pool = pool_with_owner(owner_id, "owner").await;
        let store = Arc::new(MemoryStore::default());
        let session = session(store.clone());
        insert_identity(&session, owner_id, OffsetDateTime::now_utc()).await;
        let session_id = assert_some!(session.id());
        sqlx::query!("DELETE FROM users WHERE user_id = ?1", owner_id.to_string())
            .execute(&pool)
            .await
            .expect("delete the owner");

        let state = assert_ok!(
            AuthSession::new(session.clone())
                .resolve(&pool, policy())
                .await
        );
        assert!(matches!(state, SessionState::Anonymous(_)));
        assert_eq!(assert_ok!(store.load(&session_id).await), None);
    }

    #[tokio::test]
    async fn the_absolute_lifetime_expires_at_the_policy_boundary() {
        let owner_id = OwnerId::new();
        let pool = pool_with_owner(owner_id, "owner").await;
        let policy = policy();
        let issued_at = OffsetDateTime::now_utc();
        let expires_at = assert_some!(issued_at.checked_add(policy.absolute_timeout()));
        let just_before = assert_some!(expires_at.checked_sub(Duration::seconds(1)));

        let active = session(Arc::new(MemoryStore::default()));
        insert_identity(&active, owner_id, issued_at).await;
        let active_state = assert_ok!(
            AuthSession::new(active)
                .resolve_at(&pool, policy, just_before)
                .await
        );
        assert!(matches!(active_state, SessionState::Authenticated(_)));

        let expired = session(Arc::new(MemoryStore::default()));
        insert_identity(&expired, owner_id, issued_at).await;
        let expired_state = assert_ok!(
            AuthSession::new(expired.clone())
                .resolve_at(&pool, policy, expires_at)
                .await
        );
        assert!(matches!(expired_state, SessionState::Anonymous(_)));
        assert!(expired.id().is_none());
    }

    #[tokio::test]
    async fn sign_in_and_sign_out_follow_the_typed_transitions() {
        let owner_id = OwnerId::new();
        let pool = pool_with_owner(owner_id, "owner").await;
        let store = Arc::new(MemoryStore::default());
        let state = assert_ok!(
            AuthSession::new(session(store))
                .resolve(&pool, policy())
                .await
        );
        let session = assert_some!(anonymous(state));
        let username = assert_ok!(Username::new("owner".to_owned()));
        let authenticated = assert_ok!(session.sign_in(&pool, owner_id, username.clone()).await);
        assert_eq!(authenticated.state.owner_id, owner_id);
        assert_eq!(authenticated.username(), &username);

        let anonymous = assert_ok!(authenticated.sign_out().await);
        let state = assert_ok!(
            AuthSession::new(anonymous.inner)
                .resolve(&pool, policy())
                .await
        );
        assert!(matches!(state, SessionState::Anonymous(_)));
    }
}
