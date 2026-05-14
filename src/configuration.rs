use std::{env, path::PathBuf};

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use thiserror::Error;

#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DatabaseSettings {
    pub path: PathBuf,
}

#[derive(Clone, Copy, Debug)]
enum AppEnvironment {
    Local,
    Production,
}

impl AppEnvironment {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Production => "production",
        }
    }
}

impl TryFrom<String> for AppEnvironment {
    type Error = ConfigurationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(ConfigurationError::InvalidEnvironment(other.to_owned())),
        }
    }
}

#[derive(Debug, Error)]
pub enum ConfigurationError {
    #[error("failed to determine the current working directory")]
    CurrentDirectory(#[source] std::io::Error),
    #[error("failed to build application configuration")]
    Build(#[from] ConfigError),
    #[error("unsupported APP_ENVIRONMENT value: {0}")]
    InvalidEnvironment(String),
}

/// Loads application settings from `config/base.toml`, the active environment file,
/// and `APP__...` environment variable overrides.
///
/// # Errors
///
/// Returns an error if the working directory cannot be resolved, the environment
/// name is unsupported, or the configuration files cannot be parsed.
pub fn get_settings() -> Result<Settings, ConfigurationError> {
    let config_directory = env::current_dir()
        .map_err(ConfigurationError::CurrentDirectory)?
        .join("config");

    let environment = env::var("APP_ENVIRONMENT").unwrap_or_else(|_| "local".to_owned());
    let environment = AppEnvironment::try_from(environment)?;

    Config::builder()
        .add_source(File::from(config_directory.join("base.toml")))
        .add_source(File::from(
            config_directory.join(format!("{}.toml", environment.as_str())),
        ))
        .add_source(Environment::with_prefix("APP").separator("__"))
        .build()?
        .try_deserialize()
        .map_err(ConfigurationError::Build)
}
