//! The owner session and its valid authentication transitions.

use tower_sessions::{Session, session};
use uuid::Uuid;

const USER_ID_KEY: &str = "user_id";
const USERNAME_KEY: &str = "username";

/// A session whose stored authentication data has not been checked yet.
pub struct Unverified;

/// A session with no complete owner identity.
pub struct Anonymous;

/// A session proven to contain an owner id and username.
pub struct Authenticated {
    _owner_id: Uuid,
    username: String,
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

impl AuthSession<Unverified> {
    #[must_use]
    pub const fn new(inner: Session) -> Self {
        Self {
            inner,
            state: Unverified,
        }
    }

    /// Reads both identity fields. A partial session is anonymous rather than
    /// authenticated, so guarded operations never receive incomplete state.
    pub async fn resolve(self) -> Result<SessionState, session::Error> {
        let owner_id = self.inner.get::<Uuid>(USER_ID_KEY).await?;
        let username = self.inner.get::<String>(USERNAME_KEY).await?;

        Ok(match (owner_id, username) {
            (Some(owner_id), Some(username)) => SessionState::Authenticated(AuthSession {
                inner: self.inner,
                state: Authenticated {
                    _owner_id: owner_id,
                    username,
                },
            }),
            _ => SessionState::Anonymous(AuthSession {
                inner: self.inner,
                state: Anonymous,
            }),
        })
    }
}

impl AuthSession<Anonymous> {
    /// Rotates the id before recording the authenticated owner.
    pub async fn sign_in(
        self,
        owner_id: Uuid,
        username: String,
    ) -> Result<AuthSession<Authenticated>, session::Error> {
        self.inner.cycle_id().await?;
        self.inner.insert(USER_ID_KEY, owner_id).await?;
        self.inner.insert(USERNAME_KEY, &username).await?;

        Ok(AuthSession {
            inner: self.inner,
            state: Authenticated {
                _owner_id: owner_id,
                username,
            },
        })
    }
}

impl AuthSession<Authenticated> {
    #[must_use]
    pub fn username(&self) -> &str {
        &self.state.username
    }

    /// Clears the identity, deletes the stored record, and returns an anonymous state.
    pub async fn sign_out(self) -> Result<AuthSession<Anonymous>, session::Error> {
        self.inner.flush().await?;
        Ok(AuthSession {
            inner: self.inner,
            state: Anonymous,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_ok, assert_some};
    use std::sync::Arc;
    use tower_sessions::MemoryStore;

    fn session() -> Session {
        Session::new(None, Arc::new(MemoryStore::default()), None)
    }

    fn anonymous(state: SessionState) -> Option<AuthSession<Anonymous>> {
        match state {
            SessionState::Anonymous(session) => Some(session),
            SessionState::Authenticated(_) => None,
        }
    }

    #[tokio::test]
    async fn an_empty_session_is_anonymous() {
        let state = assert_ok!(AuthSession::new(session()).resolve().await);
        assert!(matches!(state, SessionState::Anonymous(_)));
    }

    #[tokio::test]
    async fn a_partial_identity_is_not_authenticated() {
        let session = session();
        assert_ok!(session.insert(USER_ID_KEY, Uuid::new_v4()).await);

        let state = assert_ok!(AuthSession::new(session).resolve().await);
        assert!(matches!(state, SessionState::Anonymous(_)));
    }

    #[tokio::test]
    async fn sign_in_and_sign_out_follow_the_typed_transitions() {
        let state = assert_ok!(AuthSession::new(session()).resolve().await);
        let session = assert_some!(anonymous(state));
        let authenticated = assert_ok!(session.sign_in(Uuid::new_v4(), "owner".to_owned()).await);
        assert_eq!(authenticated.username(), "owner");

        let anonymous = assert_ok!(authenticated.sign_out().await);
        let state = assert_ok!(AuthSession::new(anonymous.inner).resolve().await);
        assert!(matches!(state, SessionState::Anonymous(_)));
    }
}
