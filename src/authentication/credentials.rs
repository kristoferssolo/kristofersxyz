use super::password::{Password, PasswordHash, verify_password_hash};
use crate::{
    db::DbPool,
    domain::{OwnerId, Username},
};

/// A username and password submitted at login.
pub struct Credentials {
    pub username: Username,
    pub password: Password,
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("invalid username or password")]
    InvalidCredentials,
    #[error("failed to query stored credentials")]
    Database(#[from] sqlx::Error),
    #[error("stored user id is not a valid UUID")]
    MalformedUserId(#[from] uuid::Error),
    #[error("failed to configure the password hasher: {0}")]
    Params(argon2::Error),
    #[error("failed to hash or verify the password: {0}")]
    PasswordHash(argon2::password_hash::Error),
    #[error("the password hashing task failed to complete")]
    Join(#[from] tokio::task::JoinError),
}

/// Verifies credentials and returns the authenticated owner's id.
///
/// Unknown usernames still run Argon2 verification so they follow the same
/// expensive path as wrong passwords.
///
/// # Errors
///
/// Returns [`AuthError::InvalidCredentials`] for an unknown username or a
/// wrong password, or another variant if the database read or the hasher
/// fails.
pub async fn validate_credentials(
    credentials: Credentials,
    pool: &DbPool,
) -> Result<OwnerId, AuthError> {
    let stored = get_stored_credentials(&credentials.username, pool).await?;
    let (owner_id, expected_hash) = stored.map_or_else(
        || {
            (
                None,
                PasswordHash::from(
                    "$argon2id$v=19$m=15000,t=2,p=1$gZiV/M1gPc22ELAH/Jh1Hw$CWOrkoo7oJBQ/iyh7uJ0LO2aLEfrHwTWllSAxT0zRno".to_owned(),
                ),
            )
        },
        |(id, hash)| (Some(id), hash),
    );

    let password = credentials.password;
    tokio::task::spawn_blocking(move || verify_password_hash(&expected_hash, &password)).await??;

    owner_id.ok_or(AuthError::InvalidCredentials)
}

async fn get_stored_credentials(
    username: &Username,
    pool: &DbPool,
) -> Result<Option<(OwnerId, PasswordHash)>, AuthError> {
    let row = sqlx::query!(
        "SELECT user_id, password_hash FROM users WHERE username = ?1",
        username.as_str()
    )
    .fetch_optional(pool)
    .await?;

    row.map(|row| {
        Ok((
            OwnerId::try_from(row.user_id.as_str())?,
            PasswordHash::from(row.password_hash),
        ))
    })
    .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        authentication::compute_password_hash,
        db::{DbPoolOptions, migrate},
    };
    use claims::{assert_ok, assert_ok_eq};
    use secrecy::SecretString;

    /// A migrated in-memory database holding one user with a known password.
    async fn pool_with_user(username: &str, password: &str) -> (DbPool, OwnerId) {
        let pool = DbPoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("connect to an in-memory database");
        migrate(&pool).await.expect("run the migrations");

        let id = OwnerId::new();
        let password = assert_ok!(Password::new(SecretString::from(password.to_owned())));
        let hash = compute_password_hash(&password).expect("hash the password");
        sqlx::query!(
            "INSERT INTO users (user_id, username, password_hash) VALUES (?1, ?2, ?3)",
            id.to_string(),
            username,
            hash.expose_secret()
        )
        .execute(&pool)
        .await
        .expect("insert the user");
        (pool, id)
    }

    fn credentials(username: &str, password: &str) -> Credentials {
        Credentials {
            username: assert_ok!(Username::new(username.to_owned())),
            password: assert_ok!(Password::new(SecretString::from(password.to_owned()))),
        }
    }

    #[tokio::test]
    async fn the_right_password_returns_the_user_id() {
        let (pool, id) = pool_with_user("kristofers", "correct horse battery staple").await;
        assert_ok_eq!(
            validate_credentials(
                credentials("kristofers", "correct horse battery staple"),
                &pool
            )
            .await,
            id
        );
    }

    #[tokio::test]
    async fn a_wrong_password_is_rejected() {
        let (pool, _) = pool_with_user("kristofers", "correct horse battery staple").await;
        let result = validate_credentials(credentials("kristofers", "wrong"), &pool).await;
        assert!(matches!(result, Err(AuthError::InvalidCredentials)));
    }

    #[tokio::test]
    async fn an_unknown_username_is_rejected() {
        let (pool, _) = pool_with_user("kristofers", "correct horse battery staple").await;
        let result =
            validate_credentials(credentials("nobody", "correct horse battery staple"), &pool)
                .await;
        assert!(matches!(result, Err(AuthError::InvalidCredentials)));
    }
}
