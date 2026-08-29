use super::{AuthError, Credentials, validate_credentials};
use crate::{
    db::DbPool,
    domain::{OwnerId, SessionVersion, Username},
};
use axum_login::{AuthUser, AuthnBackend};
use std::fmt;

/// The Owner data loaded for an authenticated request.
#[derive(Clone)]
pub struct AuthenticatedOwner {
    id: OwnerId,
    username: Username,
    session_version: SessionVersion,
}

impl AuthenticatedOwner {
    #[must_use]
    pub const fn id(&self) -> OwnerId {
        self.id
    }

    #[must_use]
    pub const fn username(&self) -> &Username {
        &self.username
    }
}

impl fmt::Debug for AuthenticatedOwner {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AuthenticatedOwner")
            .field("id", &self.id)
            .field("username", &self.username)
            .finish_non_exhaustive()
    }
}

impl AuthUser for AuthenticatedOwner {
    type Id = OwnerId;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.session_version.as_bytes()
    }
}

/// Loads and authenticates the sole portfolio Owner for `axum-login`.
#[derive(Clone)]
pub struct AuthBackend {
    pool: DbPool,
}

impl AuthBackend {
    #[must_use]
    pub const fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl fmt::Debug for AuthBackend {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AuthBackend")
            .finish_non_exhaustive()
    }
}

impl AuthnBackend for AuthBackend {
    type User = AuthenticatedOwner;
    type Credentials = Credentials;
    type Error = AuthError;

    async fn authenticate(
        &self,
        credentials: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        match validate_credentials(credentials, &self.pool).await {
            Ok(owner_id) => self.get_user(&owner_id).await,
            Err(AuthError::InvalidCredentials) => Ok(None),
            Err(error) => Err(error),
        }
    }

    #[tracing::instrument(
        name = "Load authenticated Owner",
        skip_all,
        fields(owner_id = %owner_id),
        err,
    )]
    async fn get_user(&self, owner_id: &OwnerId) -> Result<Option<Self::User>, Self::Error> {
        let row = sqlx::query!(
            "SELECT username, session_version FROM users WHERE user_id = ?1",
            owner_id.to_string()
        )
        .fetch_optional(&self.pool)
        .await?;

        row.map(|row| {
            Ok(AuthenticatedOwner {
                id: *owner_id,
                username: Username::new(row.username)?,
                session_version: SessionVersion::try_from(row.session_version.as_str())
                    .map_err(|_| AuthError::MalformedSessionVersion)?,
            })
        })
        .transpose()
    }
}

pub type AxumAuthSession = axum_login::AuthSession<AuthBackend>;
