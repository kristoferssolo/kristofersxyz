//! Owner-session policy layered over `axum-login`.

use super::{AuthBackend, AuthError, AuthenticatedOwner, AxumAuthSession, Credentials};
use crate::{
    configuration::SessionPolicy,
    db::DbPool,
    domain::{OwnerId, Username},
    security_events::{SecurityEvent, SessionRejection},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use time::{Duration, OffsetDateTime};
use tokio::sync::Mutex;
use tower_sessions::session;

const ISSUED_AT_KEY: &str = "owner-issued-at";
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

    fn is_expired_at(self, lifetime: Duration, now: OffsetDateTime) -> bool {
        self.expires_at(lifetime)
            .is_none_or(|expires_at| now >= expires_at)
    }
}

enum StoredClaim<T> {
    Missing,
    Value(T),
    Corrupt,
}

/// A session whose portfolio-specific lifetime has not been checked yet.
pub struct Unverified;

/// A session with no valid Owner identity.
pub struct Anonymous;

/// A session whose Owner was loaded and verified by `axum-login`.
pub struct Authenticated {
    owner: AuthenticatedOwner,
}

/// An Owner session whose valid operations are determined by `State`.
pub struct OwnerSession<State> {
    inner: AxumAuthSession,
    state: State,
}

/// The two states produced by applying the portfolio's session policy.
pub enum SessionState {
    Anonymous(OwnerSession<Anonymous>),
    Authenticated(OwnerSession<Authenticated>),
}

#[derive(Debug, thiserror::Error)]
pub enum OwnerSessionError {
    #[error("failed to use the authentication backend")]
    Backend(#[from] AuthError),
    #[error("failed to update Owner sessions")]
    Database(#[from] sqlx::Error),
    #[error("failed to read or update the Owner session")]
    Session(#[from] session::Error),
    #[error("a persisted authenticated session has no id")]
    MissingSessionId,
}

impl From<axum_login::Error<AuthBackend>> for OwnerSessionError {
    fn from(error: axum_login::Error<AuthBackend>) -> Self {
        match error {
            axum_login::Error::Backend(error) => Self::Backend(error),
            axum_login::Error::Session(error) => Self::Session(error),
        }
    }
}

impl OwnerSession<Unverified> {
    #[must_use]
    pub const fn new(inner: AxumAuthSession) -> Self {
        Self {
            inner,
            state: Unverified,
        }
    }

    /// Applies the absolute lifetime after `axum-login` verifies the Owner.
    ///
    /// The unchanged issue timestamp is written back for authenticated requests
    /// so `tower-sessions` extends its inactivity deadline without extending the
    /// absolute deadline.
    ///
    /// # Errors
    ///
    /// Returns [`OwnerSessionError`] if the session store cannot be reached.
    #[tracing::instrument(name = "Resolve owner session", skip_all, err)]
    pub async fn resolve(self, policy: SessionPolicy) -> Result<SessionState, OwnerSessionError> {
        self.resolve_at(policy, OffsetDateTime::now_utc()).await
    }

    async fn resolve_at(
        self,
        policy: SessionPolicy,
        now: OffsetDateTime,
    ) -> Result<SessionState, OwnerSessionError> {
        let Some(owner) = self.inner.user.clone() else {
            if self.inner.session.is_empty().await {
                return Ok(self.anonymous());
            }
            return self.reject(SessionRejection::Unrecognized).await;
        };
        let issued_at = match self.claim::<SessionIssuedAt>(ISSUED_AT_KEY).await? {
            StoredClaim::Value(issued_at) => issued_at,
            StoredClaim::Missing => return self.reject(SessionRejection::PartialIdentity).await,
            StoredClaim::Corrupt => return self.reject(SessionRejection::Corrupt).await,
        };
        if issued_at.is_expired_at(policy.absolute_timeout(), now) {
            return self.reject(SessionRejection::AbsoluteExpired).await;
        }

        self.inner.session.insert(ISSUED_AT_KEY, issued_at).await?;
        tracing::debug!(
            event = "session_resolved",
            owner_id = %owner.id(),
            username = %owner.username(),
            "Resolved Owner session"
        );
        Ok(SessionState::Authenticated(OwnerSession {
            inner: self.inner,
            state: Authenticated { owner },
        }))
    }

    async fn claim<T>(&self, key: &str) -> Result<StoredClaim<T>, session::Error>
    where
        T: DeserializeOwned,
    {
        let Some(value) = self.inner.session.get_value(key).await? else {
            return Ok(StoredClaim::Missing);
        };
        Ok(serde_json::from_value(value).map_or(StoredClaim::Corrupt, StoredClaim::Value))
    }

    async fn reject(mut self, reason: SessionRejection) -> Result<SessionState, OwnerSessionError> {
        SecurityEvent::SessionRejected { reason }.record();
        if let Err(error) = self.inner.logout().await {
            SecurityEvent::SessionCleanupFailed.record();
            return Err(error.into());
        }
        Ok(self.anonymous())
    }

    fn anonymous(self) -> SessionState {
        SessionState::Anonymous(OwnerSession {
            inner: self.inner,
            state: Anonymous,
        })
    }
}

impl OwnerSession<Anonymous> {
    /// Verifies credentials through the configured `axum-login` backend.
    ///
    /// # Errors
    ///
    /// Returns [`OwnerSessionError`] if credential verification or session
    /// access fails.
    pub async fn authenticate(
        &self,
        credentials: Credentials,
    ) -> Result<Option<AuthenticatedOwner>, OwnerSessionError> {
        self.inner
            .authenticate(credentials)
            .await
            .map_err(Into::into)
    }

    /// Starts an Owner session and revokes the previous active session.
    ///
    /// `axum-login` rotates the session identifier before storing the Owner ID
    /// and authentication-version hash.
    ///
    /// # Errors
    ///
    /// Returns [`OwnerSessionError`] if the session store or SQLite cannot be
    /// updated.
    #[tracing::instrument(
        name = "Sign in owner session",
        skip_all,
        fields(owner_id = %owner.id(), username = %owner.username()),
        err,
    )]
    pub async fn sign_in(
        self,
        pool: &DbPool,
        owner: AuthenticatedOwner,
    ) -> Result<OwnerSession<Authenticated>, OwnerSessionError> {
        // Serializing this short section makes "newest login wins" reliable
        // within the single application process used by this deployment.
        let _guard = SIGN_INS.lock().await;
        let mut inner = self.inner;
        if let Err(error) = inner.login(&owner).await {
            record_session_start_failure(&owner);
            fail_closed(&mut inner).await;
            return Err(error.into());
        }
        if let Err(error) = inner
            .session
            .insert(ISSUED_AT_KEY, SessionIssuedAt::now())
            .await
        {
            record_session_start_failure(&owner);
            fail_closed(&mut inner).await;
            return Err(error.into());
        }
        if let Err(error) = inner.session.save().await {
            record_session_start_failure(&owner);
            fail_closed(&mut inner).await;
            return Err(error.into());
        }
        let Some(session_id) = inner.session.id() else {
            record_session_start_failure(&owner);
            fail_closed(&mut inner).await;
            return Err(OwnerSessionError::MissingSessionId);
        };
        let revoked_sessions = match sqlx::query!(
            r#"
DELETE FROM
    sessions
WHERE
    id <> ?1
        "#,
            session_id.to_string()
        )
        .execute(pool)
        .await
        {
            Ok(result) => result.rows_affected(),
            Err(error) => {
                record_session_start_failure(&owner);
                fail_closed(&mut inner).await;
                return Err(error.into());
            }
        };

        SecurityEvent::SessionStarted {
            owner_id: owner.id(),
            username: owner.username(),
            revoked_sessions,
        }
        .record();
        SecurityEvent::AuthenticationSucceeded {
            owner_id: owner.id(),
            username: owner.username(),
        }
        .record();
        Ok(OwnerSession {
            inner,
            state: Authenticated { owner },
        })
    }
}

impl OwnerSession<Authenticated> {
    #[must_use]
    pub const fn owner_id(&self) -> OwnerId {
        self.state.owner.id()
    }

    #[must_use]
    pub const fn username(&self) -> &Username {
        self.state.owner.username()
    }

    /// Clears the identity and deletes the stored session record.
    ///
    /// # Errors
    ///
    /// Returns [`OwnerSessionError`] if the session store cannot be updated.
    #[tracing::instrument(
        name = "Sign out owner session",
        skip_all,
        fields(
            owner_id = %self.state.owner.id(),
            username = %self.state.owner.username(),
        ),
        err,
    )]
    pub async fn sign_out(mut self) -> Result<OwnerSession<Anonymous>, OwnerSessionError> {
        let owner = self.state.owner;
        if let Err(error) = self.inner.logout().await {
            SecurityEvent::SessionEndFailed {
                owner_id: owner.id(),
                username: owner.username(),
            }
            .record();
            return Err(error.into());
        }
        SecurityEvent::SessionEnded {
            owner_id: owner.id(),
            username: owner.username(),
        }
        .record();
        Ok(OwnerSession {
            inner: self.inner,
            state: Anonymous,
        })
    }
}

fn record_session_start_failure(owner: &AuthenticatedOwner) {
    SecurityEvent::SessionStartFailed {
        owner_id: owner.id(),
        username: owner.username(),
    }
    .record();
}

async fn fail_closed(session: &mut AxumAuthSession) {
    if session.logout().await.is_err() {
        SecurityEvent::SessionCleanupFailed.record();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::assert_some;

    #[test]
    fn the_absolute_lifetime_expires_at_the_policy_boundary() {
        let lifetime = Duration::hours(8);
        let issued_at = OffsetDateTime::now_utc();
        let claim = SessionIssuedAt(issued_at.unix_timestamp());
        let expires_at = assert_some!(issued_at.checked_add(lifetime));
        let just_before = assert_some!(expires_at.checked_sub(Duration::seconds(1)));

        assert!(!claim.is_expired_at(lifetime, just_before));
        assert!(claim.is_expired_at(lifetime, expires_at));
    }
}
