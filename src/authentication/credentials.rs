use super::password::{Password, PasswordHash, PasswordHashError, verify_password_hash};
use crate::{
    db::DbPool,
    domain::{OwnerId, Username, UsernameError},
};
use tokio::sync::{Semaphore, SemaphorePermit};

const MAX_CONCURRENT_PASSWORD_TASKS: usize = 2;
static PASSWORD_TASKS: Semaphore = Semaphore::const_new(MAX_CONCURRENT_PASSWORD_TASKS);

// Generated with the current policy. The policy test below prevents this
// account-enumeration defense from silently falling behind real hashes.
const DUMMY_PASSWORD_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$HPdFuefGL1IJvSCaYT3kjw$EvhqrUVFb63ZwH7epIEHKqrd5mYQU0z5hXot94G1gVo";

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
    #[error("stored username is invalid")]
    MalformedUsername(#[from] UsernameError),
    #[error("stored session version is not a valid UUID")]
    MalformedSessionVersion,
    #[error(transparent)]
    PasswordHash(#[from] PasswordHashError),
    #[error("the password hashing task failed to complete")]
    Join(#[from] tokio::task::JoinError),
    #[error("password verification is unavailable")]
    PasswordTasksUnavailable,
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
#[tracing::instrument(
    name = "Validate owner credentials",
    skip_all,
    fields(
        username = %credentials.username,
        owner_id = tracing::field::Empty,
    ),
    err,
)]
pub async fn validate_credentials(
    credentials: Credentials,
    pool: &DbPool,
) -> Result<OwnerId, AuthError> {
    let stored = get_stored_credentials(&credentials.username, pool).await?;
    let (owner_id, expected_hash) = stored.map_or_else(
        || PasswordHash::try_from(DUMMY_PASSWORD_HASH.to_owned()).map(|hash| (None, hash)),
        |(id, hash)| Ok((Some(id), hash)),
    )?;

    let previous_hash = expected_hash.expose_secret().to_owned();
    let password = credentials.password;
    let span = tracing::Span::current();
    let _permit = reserve_password_task(&PASSWORD_TASKS)?;
    let replacement = tokio::task::spawn_blocking(move || {
        span.in_scope(|| verify_password_hash(&expected_hash, &password))
    })
    .await??;

    let owner_id = owner_id.ok_or(AuthError::InvalidCredentials)?;
    if let Some(replacement) = replacement {
        sqlx::query!(
            r#"
            UPDATE
                users
            SET
                password_hash = ?1
            WHERE
                user_id = ?2
                AND password_hash = ?3
        "#,
            replacement.expose_secret(),
            owner_id.to_string(),
            previous_hash,
        )
        .execute(pool)
        .await?;
    }
    tracing::Span::current().record("owner_id", tracing::field::display(owner_id));
    Ok(owner_id)
}

fn reserve_password_task(semaphore: &Semaphore) -> Result<SemaphorePermit<'_>, AuthError> {
    semaphore
        .try_acquire()
        .map_err(|_| AuthError::PasswordTasksUnavailable)
}

#[tracing::instrument(
    name = "Get stored owner credentials",
    skip_all,
    fields(username = %username),
    err,
)]
async fn get_stored_credentials(
    username: &Username,
    pool: &DbPool,
) -> Result<Option<(OwnerId, PasswordHash)>, AuthError> {
    let row = sqlx::query!(
        r#"
        SELECT
            user_id,
            password_hash
        FROM
            users
        WHERE
            username = ?1
        "#,
        username.as_str()
    )
    .fetch_optional(pool)
    .await?;

    row.map(|row| {
        Ok((
            OwnerId::try_from(row.user_id.as_str())?,
            PasswordHash::try_from(row.password_hash)?,
        ))
    })
    .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        authentication::{compute_password_hash, test_support},
        db::test_support::migrated_pool,
    };
    use claims::{assert_err, assert_ok, assert_ok_eq};
    use password_auth::is_hash_obsolete;
    use secrecy::SecretString;
    /// A migrated in-memory database holding one user with a known password.
    async fn pool_with_user(username: &str, password: &str) -> (DbPool, OwnerId) {
        let pool = migrated_pool().await;
        let id = OwnerId::new();
        let session_version = crate::domain::SessionVersion::new().to_storage();
        let password = assert_ok!(Password::new(SecretString::from(password.to_owned())));
        let hash = compute_password_hash(&password);
        sqlx::query!(
            r#"
            INSERT INTO
                users (
                    user_id,
                    username,
                    password_hash,
                    session_version
                )
            VALUES
                (?1, ?2, ?3, ?4)
        "#,
            id.to_string(),
            username,
            hash.expose_secret(),
            session_version,
        )
        .execute(&pool)
        .await
        .expect("insert the user");
        (pool, id)
    }

    #[tokio::test]
    async fn the_right_password_returns_the_user_id() {
        let (pool, id) = pool_with_user("kristofers", "correct horse battery staple").await;
        assert_ok_eq!(
            validate_credentials(
                test_support::credentials("kristofers", "correct horse battery staple"),
                &pool
            )
            .await,
            id
        );
    }

    #[tokio::test]
    async fn a_wrong_password_is_rejected() {
        let (pool, _) = pool_with_user("kristofers", "correct horse battery staple").await;
        let result =
            validate_credentials(test_support::credentials("kristofers", "wrong"), &pool).await;
        assert!(matches!(result, Err(AuthError::InvalidCredentials)));
    }

    #[tokio::test]
    async fn an_unknown_username_is_rejected() {
        let (pool, _) = pool_with_user("kristofers", "correct horse battery staple").await;
        let result = validate_credentials(
            test_support::credentials("nobody", "correct horse battery staple"),
            &pool,
        )
        .await;
        assert!(matches!(result, Err(AuthError::InvalidCredentials)));
    }

    #[tokio::test]
    async fn a_successful_login_upgrades_an_old_hash() {
        let (pool, id) = pool_with_user("kristofers", "correct horse battery staple").await;
        let password = "password";
        let old_hash = test_support::obsolete_password_hash();
        let old_hash = old_hash.expose_secret().to_owned();
        sqlx::query!(
            r#"
            UPDATE
                users
            SET
                password_hash = ?1
            WHERE
                user_id = ?2
        "#,
            old_hash,
            id.to_string(),
        )
        .execute(&pool)
        .await
        .expect("install the old password hash");

        assert_ok!(
            validate_credentials(test_support::credentials("kristofers", password), &pool,).await
        );

        let upgraded = sqlx::query_scalar!(
            r#"
            SELECT
                password_hash
            FROM
                users
            WHERE
                user_id = ?1
        "#,
            id.to_string(),
        )
        .fetch_one(&pool)
        .await
        .expect("read the upgraded hash");
        assert_ok_eq!(is_hash_obsolete(&upgraded), false);
    }

    #[test]
    fn the_dummy_hash_uses_the_current_policy() {
        assert_ok_eq!(is_hash_obsolete(DUMMY_PASSWORD_HASH), false);
    }

    #[test]
    fn saturated_password_verification_is_rejected_without_queueing() {
        let semaphore = Semaphore::new(1);
        let _permit = assert_ok!(reserve_password_task(&semaphore));

        assert!(matches!(
            assert_err!(reserve_password_task(&semaphore)),
            AuthError::PasswordTasksUnavailable
        ));
    }
}
