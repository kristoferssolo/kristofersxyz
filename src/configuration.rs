use serde::Deserialize;
use std::env;

#[derive(Debug, thiserror::Error)]
pub enum ConfigurationError {
    #[error("environment variable {name} is not valid unicode")]
    InvalidEnvironmentVariable {
        name: &'static str,
        #[source]
        source: env::VarError,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Settings {
    /// `None` when `DATABASE_URL` is unset. The site renders static content, so
    /// booting without a database is a supported configuration rather than an
    /// error. The SQLite phase is what makes it required.
    pub database: Option<DatabaseSettings>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct DatabaseSettings {
    pub url: String,
}

impl Settings {
    /// Loads application settings from environment variables.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigurationError`] when an environment variable is set to a
    /// value that cannot be read.
    pub fn from_env() -> Result<Self, ConfigurationError> {
        let database = DatabaseSettings::from_env()?;
        Ok(Self { database })
    }
}

impl DatabaseSettings {
    /// `Ok(None)` when `DATABASE_URL` is unset. Only a value that is present
    /// and unreadable is an error: an operator who set the variable meant to
    /// use a database, so failing loudly beats silently serving without one.
    fn from_env() -> Result<Option<Self>, ConfigurationError> {
        match env::var("DATABASE_URL") {
            Ok(url) => Ok(Some(Self { url })),
            Err(env::VarError::NotPresent) => Ok(None),
            Err(source) => Err(ConfigurationError::InvalidEnvironmentVariable {
                name: "DATABASE_URL",
                source,
            }),
        }
    }
}
