use serde::Deserialize;
use std::env;

#[derive(Debug, thiserror::Error)]
pub enum ConfigurationError {
    #[error("missing environment variable {name}")]
    MissingEnvironmentVariable {
        name: &'static str,
        #[source]
        source: env::VarError,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
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
    /// Returns [`ConfigurationError`] when a required environment variable is
    /// missing or invalid.
    pub fn from_env() -> Result<Self, ConfigurationError> {
        let database = DatabaseSettings::from_env()?;
        Ok(Self { database })
    }
}

impl DatabaseSettings {
    fn from_env() -> Result<Self, ConfigurationError> {
        let url = env::var("DATABASE_URL").map_err(|source| {
            ConfigurationError::MissingEnvironmentVariable {
                name: "DATABASE_URL",
                source,
            }
        })?;
        Ok(Self { url })
    }
}
