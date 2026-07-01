use serde::Deserialize;
use std::env;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct DatabaseSettings {
    pub url: String,
}

#[derive(Debug, Error)]
pub enum ConfigurationError {
    #[error("missing environment variable {name}")]
    MissingEnvironmentVariable {
        name: &'static str,
        #[source]
        source: env::VarError,
    },
}

impl Settings {
    pub fn from_env() -> Result<Self, ConfigurationError> {
        let database = DatabaseSettings::from_env()?;
        Ok(Self { database })
    }
}

impl DatabaseSettings {
    fn from_env() -> Result<Self, ConfigurationError> {
        let url = env::var("DATABSE_URL").map_err(|source| {
            ConfigurationError::MissingEnvironmentVariable {
                name: "DATABSE_URL",
                source,
            }
        })?;
        Ok(Self { url })
    }
}
