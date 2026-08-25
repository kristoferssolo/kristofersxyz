use crate::db::DbPool;
use argon2::{
    Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version,
    password_hash::SaltString,
};
use secrecy::{ExposeSecret, SecretString};
use uuid::Uuid;

/// A username and password submitted at login.
pub struct Credentials {
    pub username: String,
    pub password: SecretString,
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

/// Verifies a username and password against the stored hash, returning the
/// authenticated user's id. Unknown usernames still run Argon2 verification
/// so they follow the same expensive path as wrong passwords.
///
/// # Errors
///
/// Returns [`AuthError::InvalidCredentials`] for an unknown username or a
/// wrong password, or another variant if the database read or the hasher
/// fails.
pub async fn validate_credentials(
    credentials: Credentials,
    pool: &DbPool,
) -> Result<Uuid, AuthError> {
    let mut user_id = None;
    let mut expected_hash = SecretString::from(
        "$argon2id$v=19$m=15000,t=2,p=1$gZiV/M1gPc22ELAH/Jh1Hw$CWOrkoo7oJBQ/iyh7uJ0LO2aLEfrHwTWllSAxT0zRno".to_owned(),
    );

    if let Some((stored_id, stored_hash)) =
        get_stored_credentials(&credentials.username, pool).await?
    {
        user_id = Some(stored_id);
        expected_hash = stored_hash;
    }

    let password = credentials.password;
    tokio::task::spawn_blocking(move || verify_password_hash(&expected_hash, &password)).await??;

    user_id.ok_or(AuthError::InvalidCredentials)
}

/// Hashes a password with Argon2id and a fresh random salt.
///
/// # Errors
///
/// Returns an [`AuthError`] if the hasher cannot be configured or the hash
/// cannot be computed.
pub fn compute_password_hash(password: &SecretString) -> Result<SecretString, AuthError> {
    let salt = SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
    let params = Params::new(15_000, 2, 1, None).map_err(AuthError::Params)?;
    let hash = Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
        .hash_password(password.expose_secret().as_bytes(), &salt)
        .map_err(AuthError::PasswordHash)?
        .to_string();
    Ok(SecretString::from(hash))
}

/// Verifies a candidate against the parameters encoded in a PHC hash.
fn verify_password_hash(
    expected: &SecretString,
    candidate: &SecretString,
) -> Result<(), AuthError> {
    let expected = PasswordHash::new(expected.expose_secret()).map_err(AuthError::PasswordHash)?;
    Argon2::default()
        .verify_password(candidate.expose_secret().as_bytes(), &expected)
        .map_err(|error| match error {
            argon2::password_hash::Error::Password => AuthError::InvalidCredentials,
            other => AuthError::PasswordHash(other),
        })
}

async fn get_stored_credentials(
    username: &str,
    pool: &DbPool,
) -> Result<Option<(Uuid, SecretString)>, AuthError> {
    let row = sqlx::query!(
        "SELECT user_id, password_hash FROM users WHERE username = ?1",
        username
    )
    .fetch_optional(pool)
    .await?;

    row.map(|row| {
        Ok((
            Uuid::parse_str(&row.user_id)?,
            SecretString::from(row.password_hash),
        ))
    })
    .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{DbPoolOptions, migrate};
    use claims::assert_ok_eq;

    /// A migrated in-memory database holding one user with a known password.
    async fn pool_with_user(username: &str, password: &str) -> (DbPool, Uuid) {
        let pool = DbPoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("connect to an in-memory database");
        migrate(&pool).await.expect("run the migrations");

        let id = Uuid::new_v4();
        let hash = compute_password_hash(&SecretString::from(password.to_owned()))
            .expect("hash the password");
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
            username: username.to_owned(),
            password: SecretString::from(password.to_owned()),
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
