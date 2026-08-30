//! Command-line administration dispatched before server startup.
//!
//! No arguments starts the server. A subcommand runs its task and exits.

use crate::{
    authentication::{OwnerId, Password, PasswordError, compute_password_hash},
    configuration::Settings,
    db,
    domain::{SessionVersion, Username, UsernameError},
    security_events::SecurityEvent,
};
use sqlx::migrate::MigrateError;

#[derive(Debug, thiserror::Error)]
pub enum AdminCliError {
    #[error("usage: kristofersxyz set-password <username>")]
    Usage,
    #[error("unknown command '{0}'")]
    UnknownCommand(String),
    #[error("failed to read the password")]
    Io(#[from] std::io::Error),
    #[error("the passwords did not match")]
    Mismatch,
    #[error(transparent)]
    Username(#[from] UsernameError),
    #[error(transparent)]
    Password(#[from] PasswordError),
    #[error("failed to reach the database")]
    Database(#[from] sqlx::Error),
    #[error("failed to run migrations")]
    Migration(#[from] MigrateError),
}

/// Creates the user or replaces its password after running migrations.
///
/// # Errors
///
/// Returns an [`AdminCliError`] if the database is unreachable, a migration
/// fails, or the password cannot be stored.
#[tracing::instrument(
    name = "Set owner password",
    skip_all,
    fields(username = %username),
    err,
)]
pub async fn set_password(
    settings: &Settings,
    username: &Username,
    password: &Password,
) -> Result<(), AdminCliError> {
    let hash = compute_password_hash(password);
    let session_version = SessionVersion::new().to_storage();

    let pool = db::connect(&settings.database.url).await?;
    db::migrate(&pool).await?;

    let mut transaction = pool.begin().await?;
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
    (?1, ?2, ?3, ?4) ON CONFLICT(username) DO
UPDATE
SET
    password_hash = excluded.password_hash,
    session_version = excluded.session_version
    "#,
        OwnerId::new().to_string(),
        username.as_str(),
        hash.expose_secret(),
        session_version,
    )
    .execute(&mut *transaction)
    .await?;
    let revoked_sessions = sqlx::query!(
        r#"
DELETE FROM
    sessions
    "#
    )
    .execute(&mut *transaction)
    .await?;
    transaction.commit().await?;

    SecurityEvent::PasswordChanged {
        username,
        revoked_sessions: revoked_sessions.rows_affected(),
    }
    .record();

    Ok(())
}

/// Reads the same password twice without terminal echo and enforces the owner
/// strength policy.
///
/// # Errors
///
/// Returns [`AdminCliError::Io`] if the terminal cannot be read,
/// [`AdminCliError::Mismatch`] if the entries differ, or
/// [`AdminCliError::Password`] if the password is blank, too long, or too
/// short.
pub fn read_new_password() -> Result<Password, AdminCliError> {
    let password = rpassword::prompt_password("New password: ")?;
    let confirm = rpassword::prompt_password("Confirm password: ")?;
    if password != confirm {
        return Err(AdminCliError::Mismatch);
    }
    let password = Password::try_from(password)?;
    password.ensure_owner_strength()?;
    Ok(password)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        authentication::{Password, test_support, validate_credentials},
        configuration::{DeploymentMode, PublicOrigin},
        domain::Username,
    };
    use claims::{assert_err, assert_ok};
    use secrecy::SecretString;
    use tempfile::NamedTempFile;

    fn settings_for(database: &NamedTempFile) -> Settings {
        Settings::new(
            format!("sqlite://{}", database.path().display()),
            DeploymentMode::Local,
            "http://localhost:3000"
                .parse::<PublicOrigin>()
                .expect("the test origin is valid"),
        )
    }

    #[tokio::test]
    async fn set_password_creates_then_replaces_the_users_password() {
        let database = NamedTempFile::new().expect("create a temporary database");
        let settings = settings_for(&database);

        set_password(
            &settings,
            &assert_ok!(Username::new("owner".to_owned())),
            &assert_ok!(Password::new(SecretString::from("first pw".to_owned()))),
        )
        .await
        .expect("create the user");

        let pool = db::connect(&settings.database.url).await.expect("connect");
        assert_ok!(
            validate_credentials(test_support::credentials("owner", "first pw"), &pool,).await
        );

        set_password(
            &settings,
            &assert_ok!(Username::new("owner".to_owned())),
            &assert_ok!(Password::new(SecretString::from("second pw".to_owned()))),
        )
        .await
        .expect("replace the password");

        assert_err!(
            validate_credentials(test_support::credentials("owner", "first pw"), &pool,).await
        );
        assert_ok!(
            validate_credentials(test_support::credentials("owner", "second pw"), &pool,).await
        );
    }

    #[tokio::test]
    async fn changing_a_password_revokes_every_session() {
        let database = NamedTempFile::new().expect("create a temporary database");
        let settings = settings_for(&database);
        let username = assert_ok!(Username::new("owner".to_owned()));

        set_password(
            &settings,
            &username,
            &assert_ok!(Password::try_from("first password".to_owned())),
        )
        .await
        .expect("create the user");
        let pool = db::connect(&settings.database.url).await.expect("connect");
        sqlx::query!(
            r#"
INSERT INTO
    sessions (id, data, expiry_date)
VALUES
    ('active', '{}', 4102444800)
        "#
        )
        .execute(&pool)
        .await
        .expect("insert an active session");

        set_password(
            &settings,
            &username,
            &assert_ok!(Password::try_from("second password".to_owned())),
        )
        .await
        .expect("replace the password");

        let sessions = sqlx::query_scalar!(
            r#"
SELECT
    COUNT(*)
FROM
    sessions
        "#
        )
        .fetch_one(&pool)
        .await
        .expect("count sessions");
        assert_eq!(sessions, 0);
    }

    #[tokio::test]
    async fn failed_session_revocation_rolls_back_the_password() {
        let database = NamedTempFile::new().expect("create a temporary database");
        let settings = settings_for(&database);
        let username = assert_ok!(Username::new("owner".to_owned()));

        set_password(
            &settings,
            &username,
            &assert_ok!(Password::try_from("first password".to_owned())),
        )
        .await
        .expect("create the user");
        let pool = db::connect(&settings.database.url).await.expect("connect");
        sqlx::query!(
            r#"
INSERT INTO
    sessions (id, data, expiry_date)
VALUES
    ('active', '{}', 4102444800)
        "#
        )
        .execute(&pool)
        .await
        .expect("insert an active session");
        sqlx::query!(
            r#"
CREATE TRIGGER reject_session_deletion BEFORE DELETE ON sessions
BEGIN
    SELECT RAISE(ABORT, 'session deletion rejected');
END
        "#
        )
        .execute(&pool)
        .await
        .expect("install the failure trigger");

        assert_err!(
            set_password(
                &settings,
                &username,
                &assert_ok!(Password::try_from("second password".to_owned())),
            )
            .await
        );
        assert_ok!(
            validate_credentials(test_support::credentials("owner", "first password"), &pool,)
                .await
        );
        assert_err!(
            validate_credentials(test_support::credentials("owner", "second password"), &pool,)
                .await
        );
    }
}
