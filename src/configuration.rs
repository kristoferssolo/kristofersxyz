use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use std::{env, fmt::Display, str::FromStr};
use thiserror::Error;

#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DatabaseSettings {
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub database_name: String,
    pub require_ssl: bool,
}

impl DatabaseSettings {
    /// Builds `PostgreSQL` connection options from the configured database settings.
    ///
    /// # Errors
    ///
    /// Returns an error when the database username or password is missing from
    /// configuration and environment overrides.
    pub fn connect_options(&self) -> Result<PgConnectOptions, DatabaseSettingsError> {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };

        let username = self
            .username
            .as_deref()
            .ok_or(DatabaseSettingsError::MissingUsername)?;
        let password = self
            .password
            .as_deref()
            .ok_or(DatabaseSettingsError::MissingPassword)?;

        Ok(PgConnectOptions::new()
            .host(&self.host)
            .port(self.port)
            .username(username)
            .password(password)
            .database(&self.database_name)
            .ssl_mode(ssl_mode))
    }
}

#[derive(Debug, Error)]
pub enum DatabaseSettingsError {
    #[error("database username must be provided through configuration or environment")]
    MissingUsername,
    #[error("database password must be provided through configuration or environment")]
    MissingPassword,
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

impl AsRef<str> for AppEnvironment {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Display for AppEnvironment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for AppEnvironment {
    type Err = ConfigurationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(ConfigurationError::InvalidEnvironment(other.to_owned())),
        }
    }
}

impl TryFrom<String> for AppEnvironment {
    type Error = ConfigurationError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
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
            config_directory.join(format!("{environment}.toml")),
        ))
        .add_source(Environment::with_prefix("APP").separator("__"))
        .build()?
        .try_deserialize()
        .map_err(ConfigurationError::Build)
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::assert_matches;
    use std::str::FromStr;

    #[test]
    fn parses_environment_names_case_insensitively() {
        assert_matches!(AppEnvironment::from_str("LoCaL"), Ok(AppEnvironment::Local));
        assert_matches!(
            AppEnvironment::from_str("PRODUCTION"),
            Ok(AppEnvironment::Production)
        );
    }
}
