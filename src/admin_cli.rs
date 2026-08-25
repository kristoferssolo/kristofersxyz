//! Command-line administration dispatched before server startup.
//!
//! No arguments starts the server. A subcommand runs its task and exits.

use crate::{
    authentication::{AuthError, compute_password_hash},
    configuration::Settings,
    db,
};
use secrecy::{ExposeSecret, SecretString};
use sqlx::migrate::MigrateError;
use uuid::Uuid;

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
    #[error("the password must not be empty")]
    Empty,
    #[error("failed to reach the database")]
    Database(#[from] sqlx::Error),
    #[error("failed to run migrations")]
    Migration(#[from] MigrateError),
    #[error("failed to hash the password")]
    Hash(#[from] AuthError),
}

/// Creates the user or replaces its password after running migrations.
///
/// # Errors
///
/// Returns an [`AdminCliError`] if the database is unreachable, a migration
/// fails, or the password cannot be hashed.
pub async fn set_password(
    settings: &Settings,
    username: &str,
    password: &SecretString,
) -> Result<(), AdminCliError> {
    let hash = compute_password_hash(password)?;

    let pool = db::connect(&settings.database.url).await?;
    db::migrate(&pool).await?;

    sqlx::query(
        "INSERT INTO users (user_id, username, password_hash) VALUES (?1, ?2, ?3)
         ON CONFLICT(username) DO UPDATE SET password_hash = excluded.password_hash",
    )
    .bind(Uuid::new_v4().to_string())
    .bind(username)
    .bind(hash.expose_secret())
    .execute(&pool)
    .await?;

    Ok(())
}

/// Reads the same non-empty password twice without terminal echo.
///
/// # Errors
///
/// Returns [`AdminCliError::Io`] if the terminal cannot be read,
/// [`AdminCliError::Mismatch`] if the entries differ, or
/// [`AdminCliError::Empty`] if the password is blank.
pub fn read_new_password() -> Result<SecretString, AdminCliError> {
    let password = rpassword::prompt_password("New password: ")?;
    let confirm = rpassword::prompt_password("Confirm password: ")?;
    if password != confirm {
        return Err(AdminCliError::Mismatch);
    }
    if password.trim().is_empty() {
        return Err(AdminCliError::Empty);
    }
    Ok(SecretString::from(password))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        authentication::{Credentials, validate_credentials},
        configuration::{DatabaseSettings, SessionSettings},
    };
    use claims::{assert_err, assert_ok};
    use tempfile::NamedTempFile;

    fn settings_for(database: &NamedTempFile) -> Settings {
        Settings {
            database: DatabaseSettings {
                url: format!("sqlite://{}", database.path().display()),
            },
            session: SessionSettings {
                secure_cookie: false,
            },
        }
    }

    fn credentials(username: &str, password: &str) -> Credentials {
        Credentials {
            username: username.to_owned(),
            password: SecretString::from(password.to_owned()),
        }
    }

    #[tokio::test]
    async fn set_password_creates_then_replaces_the_users_password() {
        let database = NamedTempFile::new().expect("create a temporary database");
        let settings = settings_for(&database);

        set_password(
            &settings,
            "owner",
            &SecretString::from("first pw".to_owned()),
        )
        .await
        .expect("create the user");

        let pool = db::connect(&settings.database.url).await.expect("connect");
        assert_ok!(validate_credentials(credentials("owner", "first pw"), &pool).await);

        set_password(
            &settings,
            "owner",
            &SecretString::from("second pw".to_owned()),
        )
        .await
        .expect("replace the password");

        assert_err!(validate_credentials(credentials("owner", "first pw"), &pool).await);
        assert_ok!(validate_credentials(credentials("owner", "second pw"), &pool).await);
    }
}
